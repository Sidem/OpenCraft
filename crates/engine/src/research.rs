//! Research: the tech tree ([`TECHS`], data) and the world's progress through it ([`Research`], core
//! state shared by every player; the factory owns it, saves and hashes it). Labs (`factory/lab.rs`)
//! do the work: each unit of a tech uses one of each of its science packs and `seconds` of lab time
//! at full power. A finished tech unlocks what it lists ([`Unlock`]: hand recipes, machine recipes);
//! anything no tech lists is available from the start.
//!
//! Invariants: progress never exceeds a tech's units; `current` is `None` or a tech that is available
//! (every prerequisite done) and not done.
//!
//! To add a tech: append a row to `TECHS` (saves store progress by index, so never reorder), naming
//! its prerequisites by index. A new science pack: an item, a hand recipe, and an entry in `PACKS`.
//! A new kind of unlock: an `Unlock` variant (features arrive with their first use), its arm in
//! `Unlock::item` and in the lint (`tests.rs`). A tier item's hand recipe is locked like its upgrade.

mod distance;
mod personal;
mod techs;

pub use techs::TECHS;

use crate::block::BlockId;
use crate::bytes::{ByteReader, ByteWriter};
use crate::factory::tiers;
use crate::item::{ItemId, BLUE_PACK, GREEN_PACK, RED_PACK, VIOLET_PACK};
use crate::recipes::MACHINE_RECIPES;
/// Something a finished tech makes possible.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Unlock {
    /// Crafting this item by hand.
    Recipe(ItemId),
    /// A machine recipe, by `MACHINE_RECIPES` index.
    MachineRecipe(u16),
    /// Upgrading a tiered family (its block) to a tier with kits, and crafting that tier's item.
    Upgrade(BlockId, u8),
}

impl Unlock {
    /// The item the research screen shows for it.
    pub fn item(self) -> ItemId {
        match self {
            Unlock::Recipe(item) => item,
            Unlock::MachineRecipe(i) => MACHINE_RECIPES.get(i as usize).map_or(ItemId::NONE, |r| r.main().0),
            Unlock::Upgrade(block, tier) => tiers::item_of(block, tier).unwrap_or(ItemId::NONE),
        }
    }
}

#[derive(Clone, Copy)]
pub struct Tech {
    pub name: &'static str,
    /// One line for the research screen.
    pub blurb: &'static str,
    /// Techs (by index) that must be done first.
    pub needs: &'static [u8],
    /// One of each is used per unit.
    pub packs: &'static [ItemId],
    pub units: u32,
    /// Lab time per unit at full power.
    pub seconds: f64,
    pub unlocks: &'static [Unlock],
}

/// Every science pack, in the order labs hold them (one buffer slot each).
pub const PACKS: [ItemId; 4] = [RED_PACK, GREEN_PACK, BLUE_PACK, VIOLET_PACK];

/// Where a tech stands, as the research screen shows it.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TechState {
    /// A prerequisite isn't done.
    Locked,
    Available,
    Done,
}

/// The world's research: the chosen tech and units done per tech.
#[derive(Clone, PartialEq, Debug)]
pub struct Research {
    pub current: Option<u8>,
    /// Units done, by tech index.
    progress: [u32; TECHS.len()],
}

impl Default for Research {
    fn default() -> Self {
        Research { current: None, progress: [0; TECHS.len()] }
    }
}

impl Research {
    pub fn progress(&self, tech: u8) -> u32 {
        self.progress.get(tech as usize).copied().unwrap_or(0)
    }

    pub fn state(&self, tech: u8) -> TechState {
        let Some(t) = TECHS.get(tech as usize) else { return TechState::Locked };
        if self.progress(tech) >= t.units {
            TechState::Done
        } else if t.needs.iter().all(|&n| self.state(n) == TechState::Done) {
            TechState::Available
        } else {
            TechState::Locked
        }
    }

    /// Chooses what labs work on (`None` to stop); ignored unless the tech is available.
    pub fn set_current(&mut self, tech: Option<u8>) {
        if tech.is_none_or(|t| self.state(t) == TechState::Available) {
            self.current = tech;
        }
    }

    /// The tech whose research unlocks `unlock`, while it isn't done.
    pub fn locked_by(&self, unlock: Unlock) -> Option<u8> {
        let unlock = match unlock {
            Unlock::Recipe(item) => {
                tiers::placed_by(item).filter(|t| t.1 > 0).map_or(unlock, |(b, t)| Unlock::Upgrade(b, t))
            }
            other => other,
        };
        let i = TECHS.iter().position(|t| t.unlocks.contains(&unlock))? as u8;
        (self.state(i) != TechState::Done).then_some(i)
    }

    /// Whether `unlock` is available: no tech lists it, or its tech is done.
    pub fn has(&self, unlock: Unlock) -> bool {
        self.locked_by(unlock).is_none()
    }

    /// Which machine recipes are unlocked, by `MACHINE_RECIPES` index (processors ask per item).
    pub fn machine_recipes_unlocked(&self) -> Vec<bool> {
        (0..MACHINE_RECIPES.len() as u16).map(|i| self.has(Unlock::MachineRecipe(i))).collect()
    }

    /// Records a finished unit of `tech`; when that finishes the tech, labs stop working on it.
    pub fn add_unit(&mut self, tech: u8) {
        let i = tech as usize;
        self.progress[i] = (self.progress[i] + 1).min(TECHS[i].units);
        if self.current == Some(tech) && self.state(tech) == TechState::Done {
            self.current = None;
        }
    }

    /// Current tech (`u8::MAX` for none), then units done for every tech.
    pub fn write_state(&self, w: &mut ByteWriter) {
        w.u8(self.current.unwrap_or(u8::MAX));
        w.count(TECHS.len());
        self.progress.iter().for_each(|&p| w.u32(p));
    }

    pub fn read_state(r: &mut ByteReader) -> Option<Research> {
        let current = r.u8()?;
        let n = r.count()?;
        if n > TECHS.len() {
            return None;
        }
        let mut res = Research::default();
        for (p, t) in res.progress.iter_mut().zip(&TECHS[..n]) {
            *p = r.u32()?.min(t.units);
        }
        if current != u8::MAX {
            res.set_current(Some(current));
        }
        Some(res)
    }
}

/// The science pack a lab keeps in buffer slot `i` holds.
pub fn pack_slot(item: ItemId) -> Option<usize> {
    PACKS.iter().position(|&p| p == item)
}

/// The hand recipe of the item that is `block`, to keep the table short.
const fn r(block: BlockId) -> Unlock {
    Unlock::Recipe(ItemId::block(block))
}

#[cfg(test)]
mod tests;
