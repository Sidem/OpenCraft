//! Processors: every machine that turns inputs into outputs (the smelter, the constructor, and the
//! assembler, furnaces and crushers to come) is one `Processor` driven by its spec row (`specs.rs`):
//! which recipe categories it takes, burner or electric, how it picks a recipe, its buffers, its
//! numbers per tier, its model (`model.rs`) and what it says (`view.rs`). Saves from before version 18 held smelters and
//! constructors in lists of their own (`legacy.rs`).
//!
//! Belts, miners and the panel deliver into it; it sorts fuel into its fuel buffer and takes only
//! what a recipe it may make uses (`Pick`; a recipe research still locks counts as unknown). Like a
//! box, it pushes one item a tick into the next belt leading away.
//!
//! Invariants: a batch uses up its inputs when it starts and only starts when its outputs fit (and,
//! electric, while powered). Work is counted in thousandths of a Mk1 tick: a tier's `speed` times the
//! grid's power share, so batches finish exactly. A burner's fire holds work too, lights a new fuel
//! item only while a batch runs and burns `fuel` thousandths per unit of work. Changing the recipe hands
//! back the inputs, an unfinished batch's included.
//!
//! To add a processor: a spec row (`specs.rs`). New behaviour (a byproduct port, flows) goes here.

mod legacy;
mod model;
mod specs;
mod view;

pub use legacy::{read_constructor, read_smelter};
#[cfg(test)]
pub use specs::SPECS;
pub use specs::{makes, spec, Energy, Pick, ProcessSpec, ProcessTier};

use crate::block::BlockId;
use crate::bytes::{ByteReader, ByteWriter};
use crate::inventory::Stack;
use crate::item::ItemId;
use crate::math::{IVec3, Vec3};
use crate::recipes::{burn_time, MachineRecipe, MACHINE_RECIPES};

use super::belt::Belt;
use super::buffer::Buffer;
use super::power::FULL_SPEED;
use super::{ticks, Factory, Machine};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Status {
    NoRecipe,
    Working,
    NoInput,
    OutputFull,
    NoPower,
    NoFuel,
}

/// Every status, in declaration order: saves store `status as u8`.
const STATUSES: [Status; 6] =
    [Status::NoRecipe, Status::Working, Status::NoInput, Status::OutputFull, Status::NoPower, Status::NoFuel];

pub struct Processor {
    pub pos: IVec3,
    pub spec: &'static ProcessSpec,
    /// Index into `spec.tiers` (0 is Mk1).
    pub tier: u8,
    /// The chosen recipe (`Pick::Chosen`), a `MACHINE_RECIPES` index.
    pub recipe: Option<u16>,
    /// The batch in progress and the work it has had (thousandths of a Mk1 tick).
    pub batch: Option<u16>,
    pub progress: u32,
    /// Work the fire can still fuel (burners, same units).
    pub burn: u32,
    pub input: Buffer,
    pub fuel: Buffer,
    pub out: Buffer,
    /// Last tick's power share, in thousandths (derived, for the readout).
    pub speed: u32,
    /// Belt indices leading away from it.
    pub outs: Vec<u32>,
    pub next_out: usize,
    pub status: Status,
}

impl Processor {
    pub fn new(pos: IVec3, spec: &'static ProcessSpec, tier: u8) -> Processor {
        let [input, fuel, out] = spec.buffers.map(Buffer::new);
        let status = if spec.pick == Pick::Chosen { Status::NoRecipe } else { Status::NoInput };
        let tier = tier.min(spec.tiers.len() as u8 - 1);
        Processor {
            pos,
            spec,
            tier,
            recipe: None,
            batch: None,
            progress: 0,
            burn: 0,
            input,
            fuel,
            out,
            speed: 0,
            outs: Vec::new(),
            next_out: 0,
            status,
        }
    }

    /// This tier's numbers.
    pub fn stats(&self) -> &'static ProcessTier {
        &self.spec.tiers[self.tier as usize]
    }

    /// How many of `item` it would take now, up to the room in its buffer. `unlocked`: which machine
    /// recipes research allows, by index.
    pub fn room_for(&self, item: ItemId, unlocked: &[bool]) -> u32 {
        if self.is_fuel(item) {
            return self.fuel.space_for(item);
        }
        let wanted = match self.spec.pick {
            Pick::Chosen => self.chosen().is_some_and(|r| r.inputs.iter().any(|x| x.0 == item)),
            Pick::ByInput => self.spec.recipe_using(item, unlocked).is_some(),
        };
        if wanted {
            self.input.space_for(item)
        } else {
            0
        }
    }

    /// Takes one `item`; false if it doesn't want it or it doesn't fit.
    pub fn accept(&mut self, item: ItemId, unlocked: &[bool]) -> bool {
        self.insert(item, 1, unlocked) == 1
    }

    /// Puts up to `n` of `item` where it belongs; returns how many went in.
    pub fn insert(&mut self, item: ItemId, n: u32, unlocked: &[bool]) -> u32 {
        let put = n.min(self.room_for(item, unlocked));
        let buf = if self.is_fuel(item) { &mut self.fuel } else { &mut self.input };
        buf.add(item, put);
        put
    }

    /// Switches to `recipe` (one it makes, or `None`) and returns the inputs it held, an unfinished
    /// batch's included.
    pub fn set_recipe(&mut self, recipe: Option<u16>) -> Vec<Stack> {
        let mut back = self.input.contents();
        back.extend(self.batch_inputs());
        self.input = Buffer::new(self.input.slots.len());
        (self.recipe, self.batch, self.progress) = (recipe, None, 0);
        self.status = if recipe.is_some() { Status::NoInput } else { Status::NoRecipe };
        back
    }

    /// One tick at `power` (thousandths: its grid's share, or full for burners): work on the batch
    /// (starting one if it can), then push an item out.
    pub fn step(&mut self, belts: &mut [Belt], power: u32, unlocked: &[bool]) {
        self.speed = power;
        self.work(power, unlocked);
        self.out.feed(&self.outs, &mut self.next_out, belts);
    }

    /// Whether it would work this tick if powered (its grid counts it as demand).
    pub fn wants_power(&self, unlocked: &[bool]) -> bool {
        self.batch.is_some() || self.next(unlocked).is_ok_and(|i| self.blocked(i).is_none())
    }

    /// kW it draws while it works (0 for burners).
    pub fn power(&self) -> u32 {
        self.stats().power
    }

    fn work(&mut self, power: u32, unlocked: &[bool]) {
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
            (self.batch, self.progress) = (Some(i), 0);
        }
        let Some(r) = self.batch_recipe() else { return };
        let work = self.stats().speed * power / FULL_SPEED;
        if self.spec.energy == Energy::Burner {
            if self.burn == 0 {
                self.light();
            }
            if self.burn == 0 {
                self.status = Status::NoFuel;
                return;
            }
            self.burn = self.burn.saturating_sub(work * self.stats().fuel / 1000);
        } else if power == 0 {
            self.status = Status::NoPower;
            return;
        }
        self.status = Status::Working;
        self.progress += work;
        if self.progress >= ticks(r.seconds) * FULL_SPEED {
            for &(item, n) in r.outputs {
                self.out.add(item, n);
            }
            (self.batch, self.progress) = (None, 0);
        }
    }

    /// The recipe the next batch would be (a `MACHINE_RECIPES` index), or why there is none.
    fn next(&self, unlocked: &[bool]) -> Result<u16, Status> {
        match self.spec.pick {
            Pick::Chosen => self.recipe.filter(|&i| self.spec.recipe(i).is_some()).ok_or(Status::NoRecipe),
            Pick::ByInput => {
                let held = self.input.slots.iter().find(|s| !s.is_empty()).ok_or(Status::NoInput)?;
                self.spec.recipe_using(held.item, unlocked).ok_or(Status::NoInput)
            }
        }
    }

    /// Why a new batch of recipe `i` can't start, if it can't.
    fn blocked(&self, i: u16) -> Option<Status> {
        let r = &MACHINE_RECIPES[i as usize];
        if r.inputs.iter().any(|&(item, n)| self.input.count(item) < n) {
            Some(Status::NoInput)
        } else if r.outputs.iter().any(|&(item, n)| self.out.space_for(item) < n) {
            Some(Status::OutputFull)
        } else {
            None
        }
    }

    /// Lights the first fuel item it holds.
    fn light(&mut self) {
        let Some(i) = self.fuel.slots.iter().position(|s| !s.is_empty()) else { return };
        if let Some(secs) = burn_time(self.fuel.slots[i].item) {
            self.fuel.take(i, 1);
            self.burn = ticks(secs) * FULL_SPEED;
        }
    }

    fn is_fuel(&self, item: ItemId) -> bool {
        self.spec.energy == Energy::Burner && burn_time(item).is_some()
    }

    fn chosen(&self) -> Option<&'static MachineRecipe> {
        self.spec.recipe(self.recipe?)
    }

    fn batch_recipe(&self) -> Option<&'static MachineRecipe> {
        self.spec.recipe(self.batch?)
    }

    fn batch_inputs(&self) -> Vec<Stack> {
        let r = self.batch_recipe();
        r.map_or_else(Vec::new, |r| r.inputs.iter().map(|&(item, count)| Stack { item, count }).collect())
    }

    /// `None` if a saved recipe or batch isn't one it makes (damage).
    fn valid(self) -> Option<Processor> {
        let ok = |i: Option<u16>| i.is_none_or(|i| self.spec.recipe(i).is_some());
        (ok(self.recipe) && ok(self.batch)).then_some(self)
    }
}

impl Machine for Processor {
    fn pos(&self) -> IVec3 {
        self.pos
    }

    /// Core state: block and tier, recipe, batch, buffers, work, fire, round-robin position and status
    /// (`outs` comes from `relink`).
    fn write_state(&self, w: &mut ByteWriter) {
        w.ivec3(self.pos);
        w.u8(self.spec.block);
        w.u8(self.tier);
        for i in [self.recipe, self.batch] {
            w.bool(i.is_some());
            w.u16(i.unwrap_or(0));
        }
        self.input.write_state(w);
        self.fuel.write_state(w);
        self.out.write_state(w);
        w.u32(self.progress);
        w.u32(self.burn);
        w.u32(self.next_out as u32);
        w.u8(self.status as u8);
    }

    fn read_state(r: &mut ByteReader) -> Option<Processor> {
        let pos = r.ivec3()?;
        let spec = spec(r.u8()? as BlockId)?;
        let tier = r.u8()?;
        if tier as usize >= spec.tiers.len() {
            return None;
        }
        let mut p = Processor::new(pos, spec, tier);
        p.recipe = read_opt(r)?;
        p.batch = read_opt(r)?;
        let [input, fuel, out] = spec.buffers;
        p.input = Buffer::read_state(r, input)?;
        p.fuel = Buffer::read_state(r, fuel)?;
        p.out = Buffer::read_state(r, out)?;
        p.progress = r.u32()?;
        p.burn = r.u32()?;
        p.next_out = r.u32()? as usize;
        p.status = *STATUSES.get(r.u8()? as usize)?;
        p.valid()
    }

    /// Its buffers, plus the inputs of an unfinished batch (given back rather than lost).
    fn contents(&self) -> Vec<Stack> {
        let mut all = self.input.contents();
        all.extend(self.fuel.contents());
        all.extend(self.out.contents());
        all.extend(self.batch_inputs());
        all
    }

    fn describe(&self, _: &Factory) -> String {
        self.readout()
    }

    fn model(&self, out: &mut Vec<f32>, rel: Vec3, time: f64) {
        model::draw(self, out, rel, time);
    }
}

/// An optional recipe index as written: a flag, then the index (0 for none).
fn read_opt(r: &mut ByteReader) -> Option<Option<u16>> {
    let (some, i) = (r.bool()?, r.u16()?);
    Some(some.then_some(i))
}

#[cfg(test)]
mod tests;
