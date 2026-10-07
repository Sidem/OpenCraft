//! What a processor takes in: how much of an item it has room and a use for (by what it makes, burns or stores),
//! and putting items where they belong (fuel, input or output buffer).

use crate::item::{stack_size, ItemId, EMPTY_CANISTER};
use crate::recipes::recycling::millicoins;

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
        let cap = if self.spec.pick == Pick::Chosen {
            stack_size(item).saturating_sub(self.input.count(item))
        } else {
            u32::MAX
        };
        if wanted {
            self.input.space_for(item).min(cap)
        } else {
            0
        }
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
