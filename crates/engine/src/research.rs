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

use crate::block::{
    BlockId, BELT, CONSTRUCTOR, FILTER, LIFT, MINER, OUTLET, PIPE, PUMP, SMELTER, SPLITTER, UNDERPASS_IN, UNDERPASS_OUT,
};
use crate::bytes::{ByteReader, ByteWriter};
use crate::factory::tiers;
use crate::item::{ItemId, GREEN_KIT, GREEN_PACK, RED_PACK};
use crate::recipes::{BRICK_RECIPE, GEAR_RECIPE, MACHINE_RECIPES, QUICKLIME_RECIPE};

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
pub const PACKS: [ItemId; 2] = [RED_PACK, GREEN_PACK];

pub const TECHS: &[Tech] = &[
    Tech {
        name: "Belt Routing",
        blurb: "Splitters share items between belts; filters sort them.",
        needs: &[],
        packs: &[RED_PACK],
        units: 10,
        seconds: 5.0,
        unlocks: &[r(SPLITTER), r(FILTER)],
    },
    Tech {
        name: "Belt Lifts",
        blurb: "Lifts carry items straight up cliffs. (Belts climb and drop one-block steps without research.)",
        needs: &[0],
        packs: &[RED_PACK],
        units: 20,
        seconds: 5.0,
        unlocks: &[r(LIFT)],
    },
    Tech {
        name: "Green Science",
        blurb: "Green science packs, made from belts and screws, for the next techs.",
        needs: &[0],
        packs: &[RED_PACK],
        units: 30,
        seconds: 5.0,
        unlocks: &[Unlock::Recipe(GREEN_PACK)],
    },
    Tech {
        name: "Underpasses",
        blurb: "Belts that dive under a crossing belt and come back up.",
        needs: &[2],
        packs: &[RED_PACK, GREEN_PACK],
        units: 15,
        seconds: 10.0,
        unlocks: &[r(UNDERPASS_IN), r(UNDERPASS_OUT)],
    },
    // Was "Miner Mk2" (saves keep progress by index): kits replaced the separate Mk2 recipes.
    Tech {
        name: "Mechanics",
        blurb: "Gears and green kits. Hold 4 kits and right-click a miner, smelter or constructor to make it \
                Mk2: miners draw twice as fast and keep 75% instead of 60%, the others work twice as fast.",
        needs: &[2],
        packs: &[RED_PACK, GREEN_PACK],
        units: 30,
        seconds: 10.0,
        unlocks: &[
            Unlock::MachineRecipe(GEAR_RECIPE),
            Unlock::Recipe(GREEN_KIT),
            Unlock::Upgrade(MINER, 1),
            Unlock::Upgrade(SMELTER, 1),
            Unlock::Upgrade(CONSTRUCTOR, 1),
        ],
    },
    // Was "Fast Belts".
    Tech {
        name: "Belt Mk2",
        blurb: "Green kits upgrade belts to Mk2, twice as fast: hold kits and drag along a belt line.",
        needs: &[4],
        packs: &[RED_PACK, GREEN_PACK],
        units: 20,
        seconds: 10.0,
        unlocks: &[Unlock::Upgrade(BELT, 1)],
    },
    Tech {
        name: "Fluid Handling",
        blurb: "Pumps lift water out of ponds and pits; pipes carry it to outlets that pour it out elsewhere.",
        needs: &[0],
        packs: &[RED_PACK],
        units: 15,
        seconds: 5.0,
        unlocks: &[r(PUMP), r(PIPE), r(OUTLET)],
    },
    Tech {
        name: "Masonry",
        blurb: "Smelters fire stone into bricks and limestone into quicklime.",
        needs: &[],
        packs: &[RED_PACK],
        units: 15,
        seconds: 5.0,
        unlocks: &[Unlock::MachineRecipe(BRICK_RECIPE), Unlock::MachineRecipe(QUICKLIME_RECIPE)],
    },
];

/// Where a tech stands, as the research screen shows it.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TechState {
    /// A prerequisite isn't done.
    Locked,
    Available,
    Done,
}

/// The world's research: the chosen tech and units done per tech.
#[derive(Default, Clone, PartialEq, Debug)]
pub struct Research {
    pub current: Option<u8>,
    /// Units done, by tech index.
    progress: [u32; TECHS.len()],
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
