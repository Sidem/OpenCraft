//! Research: the tech tree ([`TECHS`], data) and the world's progress through it ([`Research`], core
//! state shared by every player; the factory owns it, saves and hashes it). Labs (`factory/lab.rs`)
//! do the work: each unit of a tech uses one of each of its science packs and `seconds` of lab time
//! at full power. A finished tech unlocks what it lists ([`Unlock`]: hand recipes, machine recipes);
//! anything no tech lists is available from the start.
//!
//! Invariants: progress never exceeds a tech's units; `current` is `None` or a tech that is available
//! (every prerequisite done) and not done; the queue of techs after it is in `queue.rs`.
//!
//! To add a tech: append a row to `TECHS` (saves store progress by index, so never reorder), naming
//! its prerequisites by index. A new science pack: an item, a hand recipe, and an entry in `PACKS`.
//! A new kind of unlock: an `Unlock` variant (features arrive with their first use), its arm in
//! `Unlock::item` and in the lint (`tests.rs`). A tier item's hand recipe is locked like its upgrade.

mod ai;
mod bonus;
mod chemistry;
mod compute;
mod distance;
mod hands;
mod join;
mod personal;
mod queue;
mod recycling;
mod techs;

pub use bonus::{is_bonus, Bonus};
pub use compute::needs_ai_lab;
pub use queue::MAX_QUEUE;
pub use techs::TECHS;

use crate::block::BlockId;
use crate::bytes::{ByteReader, ByteWriter};
use crate::factory::tiers;
use crate::item::{ItemId, BLUE_PACK, GOLD_PACK, GREEN_PACK, RED_PACK, VIOLET_PACK};
use crate::perks::Stat;
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
    /// An endless bonus to one kind of work (`bonus.rs`): the tech is repeated, never done in play.
    Bonus(Bonus),
    /// A tool or view the player gets (`Feature`).
    Feature(Feature),
    /// One tier (1, 2, ...) of a player's own rate; `perks::player_bonus` adds up the finished tiers.
    Perk(Stat, u8),
}

/// What a `Unlock::Feature` switches on; the code that offers it asks `Research::has`.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Feature {
    /// Belt lines route themselves to a machine and plan as ghosts in ghost mode (`belt_line/route.rs`).
    AutoRoute,
    /// The maps guess where ore lies from the stained soil seen (`survey.rs`).
    AiSurvey,
}

impl Unlock {
    /// The item the research screen shows for it.
    pub fn item(self) -> ItemId {
        match self {
            Unlock::Recipe(item) => item,
            Unlock::MachineRecipe(i) => MACHINE_RECIPES.get(i as usize).map_or(ItemId::NONE, |r| r.main().0),
            Unlock::Upgrade(block, tier) => tiers::item_of(block, tier).unwrap_or(ItemId::NONE),
            Unlock::Bonus(_) | Unlock::Feature(_) | Unlock::Perk(..) => ItemId::NONE,
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
pub const PACKS: [ItemId; 5] = [RED_PACK, GREEN_PACK, BLUE_PACK, VIOLET_PACK, GOLD_PACK];

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
    /// The techs after `current`, next first (`queue.rs`).
    queue: Vec<u8>,
    /// Units done, by tech index.
    progress: [u32; TECHS.len()],
    /// Which machine recipes are unlocked (derived from `progress` by `refresh`, never saved).
    unlocked: [bool; MACHINE_RECIPES.len()],
}

impl Default for Research {
    fn default() -> Self {
        let mut res = Research {
            current: None,
            queue: Vec::new(),
            progress: [0; TECHS.len()],
            unlocked: [false; MACHINE_RECIPES.len()],
        };
        res.refresh();
        res
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

    /// Chooses what labs work on now; ignored unless the tech is available. The tech it replaces waits at the
    /// front of the queue (an endless one is just dropped). `None` gives the current tech up and moves on to the
    /// next queued one.
    pub fn set_current(&mut self, tech: Option<u8>) {
        if tech.is_some_and(|t| self.state(t) != TechState::Available) {
            return;
        }
        let old = self.current.filter(|&c| tech != Some(c) && !is_bonus(c));
        self.current = tech;
        if let Some(old) = old.filter(|_| tech.is_some()) {
            self.queue.insert(0, old);
        }
        self.queue.retain(|&q| Some(q) != tech);
        self.prune();
        self.advance();
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

    /// Marks every tech done (creative worlds), so labs have nothing left to work on.
    pub fn complete_all(&mut self) {
        for (p, t) in self.progress.iter_mut().zip(TECHS.iter()) {
            *p = t.units;
        }
        self.current = None;
        self.queue.clear();
        self.refresh();
    }

    /// Which machine recipes are unlocked, by `MACHINE_RECIPES` index (processors ask per item). Cached (it
    /// changes only when a tech finishes) and returned as a copy, so a tick works from one snapshot even
    /// while labs finish techs.
    pub fn machine_recipes_unlocked(&self) -> [bool; MACHINE_RECIPES.len()] {
        self.unlocked
    }

    /// Recomputes `unlocked` after `progress` changed.
    fn refresh(&mut self) {
        for i in 0..MACHINE_RECIPES.len() {
            self.unlocked[i] = self.has(Unlock::MachineRecipe(i as u16));
        }
    }

    /// Records a finished unit of `tech`; when that finishes the tech, labs move on to the next queued one.
    pub fn add_unit(&mut self, tech: u8) {
        let i = tech as usize;
        self.progress[i] = (self.progress[i] + 1).min(TECHS[i].units);
        if self.state(tech) == TechState::Done {
            self.refresh();
            if self.current == Some(tech) {
                self.current = None;
            }
            self.prune();
            self.advance();
        }
    }

    /// Current tech (`u8::MAX` for none), units done for every tech, then the queue (since save 41).
    pub fn write_state(&self, w: &mut ByteWriter) {
        w.u8(self.current.unwrap_or(u8::MAX));
        w.count(TECHS.len());
        self.progress.iter().for_each(|&p| w.u32(p));
        self.write_queue(w);
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
        res.refresh();
        if current != u8::MAX {
            res.set_current(Some(current));
        }
        if r.version >= 41 {
            res.read_queue(r)?;
            res.advance();
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
