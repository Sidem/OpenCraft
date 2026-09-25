//! What a player does to a machine by hand, and what its panel shows: `panel` (a read-only view of
//! a smelter or constructor), `set_recipe`, `insert` (put items in from the inventory) and
//! `take_contents` (right-click on a box or miner, the panel's take button). The actions that call
//! these live in `action.rs`; the host draws the panel (`web/src/ui/machine.ts`).
//!
//! To give a machine a panel: `panel: true` in its `MACHINES` row, a `panel()` method on it, and its
//! arms here.

use crate::block::BlockId;
use crate::inventory::Stack;
use crate::item::ItemId;
use crate::math::IVec3;

use super::links::Slot;
use super::Factory;

/// Buffer roles in `Panel::slots`.
pub const ROLE_INPUT: u8 = 0;
pub const ROLE_FUEL: u8 = 1;
pub const ROLE_OUTPUT: u8 = 2;

/// A machine's panel, as the host shows it.
pub struct Panel {
    pub block: BlockId,
    /// The chosen recipe (constructor) or the batch in progress (smelter), as a `MACHINE_RECIPES` index.
    pub recipe: Option<u16>,
    /// Whether the player chooses the recipe (constructor) or what it's given does (smelter).
    pub choosable: bool,
    /// Progress of the current batch, in thousandths.
    pub progress: u32,
    /// Seconds of fire left (0 for machines without fuel).
    pub fire: u32,
    /// Each buffer slot with its role (`ROLE_*`).
    pub slots: Vec<(u8, Stack)>,
    /// One line: what it's doing or what it needs.
    pub status: String,
}

impl Factory {
    /// The panel of the machine at `pos`, if it has one.
    pub fn panel(&self, pos: IVec3) -> Option<Panel> {
        match *self.at.get(&pos)? {
            Slot::Smelter(i) => Some(self.smelters[i as usize].panel()),
            Slot::Constructor(i) => Some(self.constructors[i as usize].panel()),
            Slot::Belt(_) | Slot::Miner(_) | Slot::Storage(_) => None,
        }
    }

    /// Sets the recipe of the machine at `pos` if `recipe` is one it makes (or `None`), returning the
    /// inputs it held. `None` when nothing changed.
    pub fn set_recipe(&mut self, pos: IVec3, recipe: Option<u16>) -> Option<Vec<Stack>> {
        let Some(&Slot::Constructor(i)) = self.at.get(&pos) else { return None };
        let c = &mut self.constructors[i as usize];
        let valid = recipe.is_none_or(|r| crate::recipes::machine_recipe(crate::block::CONSTRUCTOR, r).is_some());
        (valid && c.recipe != recipe).then(|| c.set_recipe(recipe))
    }

    /// Puts up to `n` of `item` into the machine at `pos`, where it belongs (ore or fuel, a recipe's
    /// input). Returns how many went in.
    pub fn insert(&mut self, pos: IVec3, item: ItemId, n: u32) -> u32 {
        let (room, buf) = match self.at.get(&pos) {
            Some(Slot::Smelter(i)) => {
                let s = &mut self.smelters[*i as usize];
                (s.room_for(item), s.buffer_for(item))
            }
            Some(Slot::Constructor(i)) => {
                let c = &mut self.constructors[*i as usize];
                (c.room_for(item), Some(&mut c.input))
            }
            _ => return 0,
        };
        let Some(buf) = buf else { return 0 };
        let put = n.min(room);
        buf.add(item, put);
        put
    }

    /// Whether `insert` would take any `item` at `pos` now.
    pub fn wants(&self, pos: IVec3, item: ItemId) -> bool {
        match self.at.get(&pos) {
            Some(Slot::Smelter(i)) => self.smelters[*i as usize].room_for(item) > 0,
            Some(Slot::Constructor(i)) => self.constructors[*i as usize].room_for(item) > 0,
            _ => false,
        }
    }

    /// Takes a machine's output (everything, for a box): offers it to `take(item, count)`, which
    /// returns how many it accepted. Returns false if there is no such machine at `pos`.
    pub fn take_contents(&mut self, pos: IVec3, take: impl FnMut(ItemId, u32) -> u32) -> bool {
        let buf = match self.at.get(&pos) {
            Some(Slot::Miner(i)) => &mut self.miners[*i as usize].out,
            Some(Slot::Storage(i)) => &mut self.storages[*i as usize].buf,
            Some(Slot::Smelter(i)) => &mut self.smelters[*i as usize].out,
            Some(Slot::Constructor(i)) => &mut self.constructors[*i as usize].out,
            Some(Slot::Belt(_)) | None => return false,
        };
        buf.drain(take);
        true
    }
}
