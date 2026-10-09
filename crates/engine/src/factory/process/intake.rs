//! What a processor takes in: how much of an item it has room and a use for (by what it makes, burns or stores),
//! and putting items where they belong (fuel, input or output buffer).

use crate::item::{stack_size, ItemId, EMPTY_CANISTER};
use crate::recipes::recycling::millicoins;
use crate::recipes::{MachineRecipe, MACHINE_RECIPES};

use super::{Energy, Pick, Processor};

impl Processor {
    /// How many of `item` it would take now, up to the room in its buffer. `unlocked`: which machine
    /// recipes research allows, by index.
    pub fn room_for(&self, item: ItemId, unlocked: &[bool]) -> u32 {
        if self.is_fuel(item) {
            return self.fuel.space_for(item);
        }
        if self.is_center() {
            return self.center_room(item);
        }
        if matches!(self.energy(), Energy::Diesel | Energy::Reactor) {
            return if item == self.generator_fuel() { self.input.space_for(item) } else { 0 };
        }
        if self.spec.pick.stores() {
            return self.out.space_for(item);
        }
        if self.spec.pick == Pick::Hangar {
            return self.hangar_room(item).min(self.input.space_for(item));
        }
        if self.spec.pick == Pick::Recycle {
            return if millicoins(item) > 0 { self.input.space_for(item) } else { 0 };
        }
        let wanted = match self.spec.pick {
            Pick::Chosen => self.chosen().is_some_and(|r| r.inputs.iter().any(|x| x.0 == item)),
            Pick::ByInput => self.spec.recipe_using(item, unlocked).is_some(),
            Pick::Pump => item == EMPTY_CANISTER,
            Pick::Store | Pick::Hangar | Pick::Load | Pick::Unload | Pick::Research | Pick::Recycle => false,
        };
        let cap = match self.spec.pick {
            Pick::Chosen => stack_size(item).saturating_sub(self.input.count(item)),
            Pick::ByInput => {
                (self.input_slots_for(item) as u32 * stack_size(item)).saturating_sub(self.input.count(item))
            }
            _ => u32::MAX,
        };
        if wanted {
            self.input.space_for(item).min(cap)
        } else {
            0
        }
    }

    /// How many input slots one item may fill in a `Pick::ByInput` machine: the buffer's slots less one for every other
    /// input of the widest recipe that uses the item, and at least one. Without it a belt of quicklime fills all three
    /// slots of a blast furnace and the crushed iron and coal that go with it can never get in, so it stalls.
    fn input_slots_for(&self, item: ItemId) -> usize {
        let uses =
            |r: &&MachineRecipe| self.spec.categories.contains(&r.category) && r.inputs.iter().any(|x| x.0 == item);
        let widest = MACHINE_RECIPES.iter().filter(uses).map(|r| r.inputs.len()).max().unwrap_or(1);
        self.input.slots.len().saturating_sub(widest - 1).max(1)
    }

    /// Puts up to `n` of `item` where it belongs; returns how many went in.
    pub fn insert(&mut self, item: ItemId, n: u32, unlocked: &[bool]) -> u32 {
        let put = n.min(self.room_for(item, unlocked));
        let buf = match () {
            _ if self.is_fuel(item) => &mut self.fuel,
            _ if self.spec.pick.stores() => &mut self.out,
            _ => &mut self.input,
        };
        buf.add(item, put);
        put
    }
}
