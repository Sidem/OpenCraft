//! A processor's work loop: picking the next recipe, starting a batch when its inputs and room allow,
//! counting work, lighting a burner's fire and delivering outputs (`mod.rs` has the invariants).

use crate::inventory::Stack;
use crate::item::ItemId;
use crate::recipes::{burn_time, water_use, MachineRecipe, MACHINE_RECIPES};
use crate::sim::SimEvent;

use super::super::power::FULL_SPEED;
use super::super::ticks;
use super::{Energy, Pick, Processor, Status, SIDE_ROOM};

impl Processor {
    pub(super) fn work(&mut self, power: u32, unlocked: &[bool]) {
        if self.batch.is_none() {
            let i = match self.next(unlocked) {
                Ok(i) => i,
                Err(why) => {
                    self.status = why;
                    return;
                }
            };
            if let Some(why) = self.blocked(i).or((power == 0).then_some(Status::NoPower)) {
                self.status = why;
                return;
            }
            for &(item, n) in MACHINE_RECIPES[i as usize].inputs {
                self.input.remove(item, n);
            }
            self.steam.water -= water_use(i);
            (self.batch, self.progress) = (Some(i), 0);
        }
        let Some(r) = self.batch_recipe() else { return };
        let work = self.stats().speed * power / FULL_SPEED;
        if self.energy() == Energy::Burner {
            if self.burn == 0 {
                self.light();
            }
            if self.burn == 0 {
                self.status = Status::NoFuel;
                return;
            }
            self.burn = self.burn.saturating_sub(work * self.stats().fuel / 1000);
        } else if self.energy() == Energy::Electric && power == 0 {
            self.status = Status::NoPower;
            return;
        }
        self.status = Status::Working;
        self.progress += work;
        if self.progress >= ticks(r.seconds) * FULL_SPEED {
            for (k, &(item, n)) in r.outputs.iter().enumerate() {
                self.made.push(Stack { item, count: n });
                if self.to_side(k) {
                    self.side.add(item, n);
                } else {
                    self.out.add(item, n);
                }
            }
            (self.batch, self.progress) = (None, 0);
        }
    }

    /// Reports what this tick made (a finished batch, a filled canister) as `Produced` events.
    pub fn report_made(&mut self, events: &mut Vec<SimEvent>) {
        events.extend(self.made.drain(..).map(|s| SimEvent::Produced { item: s.item, count: s.count }));
    }

    /// The recipe the next batch would be (a `MACHINE_RECIPES` index), or why there is none.
    pub(super) fn next(&self, unlocked: &[bool]) -> Result<u16, Status> {
        match self.spec.pick {
            Pick::Chosen => self.recipe.filter(|&i| self.spec.recipe(i).is_some()).ok_or(Status::NoRecipe),
            Pick::ByInput => {
                let held = self.input.slots.iter().find(|s| !s.is_empty()).ok_or(Status::NoInput)?;
                self.spec.recipe_using(held.item, unlocked).ok_or(Status::NoInput)
            }
            Pick::Store | Pick::Hangar | Pick::Pump | Pick::Load | Pick::Unload | Pick::Research | Pick::Recycle => {
                Err(Status::NoInput)
            }
        }
    }

    /// Why a new batch of recipe `i` can't start, if it can't.
    pub(super) fn blocked(&self, i: u16) -> Option<Status> {
        let r = &MACHINE_RECIPES[i as usize];
        if r.inputs.iter().any(|&(item, n)| self.input.count(item) < n) {
            Some(Status::NoInput)
        } else if self.short_of_water(i) {
            Some(Status::NoWater)
        } else if self.full_output(r).is_some() {
            Some(Status::OutputFull)
        } else {
            None
        }
    }

    /// Whether output `k` of a recipe (0 is the main product) goes to the byproduct buffer.
    fn to_side(&self, k: usize) -> bool {
        k > 0 && !self.side.slots.is_empty()
    }

    /// The first output of `r` with no room left in its buffer, and whether it is a byproduct.
    pub(super) fn full_output(&self, r: &MachineRecipe) -> Option<(ItemId, bool)> {
        let full = |&(k, &(item, n)): &(usize, &(ItemId, u32))| {
            if self.to_side(k) {
                self.side.total() + n > SIDE_ROOM || self.side.space_for(item) < n
            } else {
                self.out.space_for(item) < n
            }
        };
        let (k, &(item, _)) = r.outputs.iter().enumerate().find(full)?;
        Some((item, self.to_side(k)))
    }

    /// Lights the first fuel item it holds.
    fn light(&mut self) {
        let Some(i) = self.fuel.slots.iter().position(|s| !s.is_empty()) else { return };
        if let Some(secs) = burn_time(self.fuel.slots[i].item) {
            self.fuel.take(i, 1);
            self.burn = ticks(secs) * FULL_SPEED;
        }
    }
}
