//! What a player does to a machine by hand, and what its panel shows: `panel` (a read-only view of
//! a processor, filter, generator or lab), `box_slots` (a box's screen), `set_recipe`, `set_filter`,
//! `rotate` (belts and routers, the R key), `set_quarry`, `insert` (put items in from the inventory) and `take_contents` (right-click on a miner, the take
//! buttons). The actions that call these live in `action.rs`; the host draws the panels
//! (`web/src/ui/machine.ts`, and `ui/inventory.ts` for a box).
//!
//! To give a machine a panel: `panel: true` in its `MACHINES` row, a `panel()` method on it, and its
//! arms here.

use crate::block::{BlockId, LASER_EMITTER, LASER_MIRROR};
use crate::inventory::Stack;
use crate::item::ItemId;
use crate::math::IVec3;

use super::links::Slot;
use super::process::Pick;
use super::quarry::{Quarry, DEPTHS, WIDTHS};
use super::Factory;

/// Buffer roles in `Panel::slots`.
pub const ROLE_INPUT: u8 = 0;
pub const ROLE_FUEL: u8 = 1;
pub const ROLE_OUTPUT: u8 = 2;

/// Whether R turns a placed `block` (belts, routers and sensors do too; so does a laser emitter or mirror).
pub fn turns(block: BlockId) -> bool {
    matches!(block, LASER_EMITTER | LASER_MIRROR)
}

/// A machine's panel, as the host shows it.
pub struct Panel {
    pub block: BlockId,
    /// The chosen recipe (a constructor) or the batch in progress (a smelter), as a `MACHINE_RECIPES` index.
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
    /// A filter's chosen item (`Some(NONE)`: none yet); `None` for machines that don't filter.
    pub filter: Option<ItemId>,
}

impl Factory {
    /// The panel of the machine at `pos`, if it has one.
    pub fn panel(&self, pos: IVec3) -> Option<Panel> {
        match *self.at.get(&pos)? {
            Slot::Process(i) => Some(self.processors[i as usize].panel()),
            Slot::Router(i) => Some(&self.routers[i as usize]).filter(|r| r.is_filter).map(|r| r.panel()),
            Slot::Generator(i) => {
                let g = &self.generators[i as usize];
                Some(g.panel(g.status_text(self)))
            }
            Slot::Lab(i) => Some(self.labs[i as usize].panel(&self.research)),
            Slot::Quarry(i) => Some(self.quarries[i as usize].panel(self)),
            Slot::Belt(_)
            | Slot::Miner(_)
            | Slot::Storage(_)
            | Slot::Pole(_)
            | Slot::Pipe(_)
            | Slot::Sensor(_)
            | Slot::Rail(_)
            | Slot::Node(_) => None,
        }
    }

    /// The slots of the box at `pos`, if there is one (its screen shows them, `ClickBox` edits them).
    pub fn box_slots(&self, pos: IVec3) -> Option<&[Stack]> {
        match self.at.get(&pos) {
            Some(Slot::Storage(i)) => Some(&self.storages[*i as usize].buf.slots),
            _ => None,
        }
    }

    pub(crate) fn box_slots_mut(&mut self, pos: IVec3) -> Option<&mut [Stack]> {
        match self.at.get(&pos) {
            Some(Slot::Storage(i)) => Some(&mut self.storages[*i as usize].buf.slots),
            _ => None,
        }
    }

    /// Turns the belt, router, sensor, laser emitter or mirror at `pos` a quarter turn clockwise, carrying its items
    /// along. False when nothing there turns.
    pub fn rotate(&mut self, pos: IVec3) -> bool {
        let dir = match self.at.get(&pos) {
            Some(&Slot::Belt(i)) => &mut self.belts[i as usize].dir,
            Some(&Slot::Router(i)) => &mut self.routers[i as usize].dir,
            Some(&Slot::Sensor(i)) => &mut self.sensors[i as usize].dir,
            Some(&Slot::Process(i)) if turns(self.processors[i as usize].spec.block) => {
                &mut self.processors[i as usize].dir
            }
            _ => return false,
        };
        *dir = (*dir + 1) % 4;
        self.dirty = true;
        true
    }

    /// Sets what the filter at `pos` sends straight on (`NONE` for nothing).
    pub fn set_filter(&mut self, pos: IVec3, item: ItemId) {
        if let Some(&Slot::Router(i)) = self.at.get(&pos) {
            let r = &mut self.routers[i as usize];
            if r.is_filter && (item == ItemId::NONE || item.is_valid()) {
                r.filter = item;
            }
        }
    }

    /// The quarry at `pos`, if there is one (its panel's box and choices).
    pub fn quarry(&self, pos: IVec3) -> Option<&Quarry> {
        match self.at.get(&pos) {
            Some(Slot::Quarry(i)) => Some(&self.quarries[*i as usize]),
            _ => None,
        }
    }

    /// Sets the quarry at `pos`'s box (`WIDTHS` and `DEPTHS` indices; a new box starts over) and pause.
    pub fn set_quarry(&mut self, pos: IVec3, width: u8, depth: u8, paused: bool) {
        if let Some(&Slot::Quarry(i)) = self.at.get(&pos) {
            if (width as usize) < WIDTHS.len() && (depth as usize) < DEPTHS.len() {
                self.quarries[i as usize].set(width, depth, paused);
            }
        }
    }

    /// Sets the recipe of the machine at `pos` if `recipe` is one it makes (or `None`), returning the
    /// inputs it held. `None` when nothing changed.
    pub fn set_recipe(&mut self, pos: IVec3, recipe: Option<u16>) -> Option<Vec<Stack>> {
        let Some(&Slot::Process(i)) = self.at.get(&pos) else { return None };
        let p = &mut self.processors[i as usize];
        let valid = p.spec.pick == Pick::Chosen && recipe.is_none_or(|r| p.spec.recipe(r).is_some());
        (valid && p.recipe != recipe).then(|| p.set_recipe(recipe))
    }

    /// Puts up to `n` of `item` into the machine at `pos`, where it belongs (ore or fuel, a recipe's
    /// input). Returns how many went in.
    pub fn insert(&mut self, pos: IVec3, item: ItemId, n: u32) -> u32 {
        match self.at.get(&pos) {
            Some(Slot::Process(i)) => {
                self.processors[*i as usize].insert(item, n, &self.research.machine_recipes_unlocked())
            }
            Some(Slot::Generator(i)) => {
                let g = &mut self.generators[*i as usize];
                let put = n.min(g.room_for(item));
                g.fuel.add(item, put);
                put
            }
            Some(Slot::Lab(i)) => n - self.labs[*i as usize].add(item, n),
            _ => 0,
        }
    }

    /// Whether `insert` would take any `item` at `pos` now.
    pub fn wants(&self, pos: IVec3, item: ItemId) -> bool {
        match self.at.get(&pos) {
            Some(Slot::Process(i)) => {
                self.processors[*i as usize].room_for(item, &self.research.machine_recipes_unlocked()) > 0
            }
            Some(Slot::Generator(i)) => self.generators[*i as usize].room_for(item) > 0,
            Some(Slot::Lab(i)) => self.labs[*i as usize].room_for(item) > 0,
            _ => false,
        }
    }

    /// Takes a machine's output (everything, for a box): offers it to `take(item, count)`, which
    /// returns how many it accepted. Returns false if there is no such machine at `pos`.
    pub fn take_contents(&mut self, pos: IVec3, take: impl FnMut(ItemId, u32) -> u32) -> bool {
        let mut take = take;
        let buf = match self.at.get(&pos) {
            Some(Slot::Miner(i)) => &mut self.miners[*i as usize].out,
            Some(Slot::Storage(i)) => &mut self.storages[*i as usize].buf,
            Some(Slot::Process(i)) => {
                let p = &mut self.processors[*i as usize];
                p.side.drain(&mut take);
                &mut p.out
            }
            Some(Slot::Quarry(i)) => &mut self.quarries[*i as usize].out,
            Some(
                Slot::Belt(_)
                | Slot::Router(_)
                | Slot::Generator(_)
                | Slot::Pole(_)
                | Slot::Lab(_)
                | Slot::Pipe(_)
                | Slot::Sensor(_)
                | Slot::Rail(_)
                | Slot::Node(_),
            )
            | None => return false,
        };
        buf.drain(take);
        true
    }
}
