//! Research: the tech tree ([`TECHS`], data) and the world's progress through it ([`Research`], core
//! state shared by every player; the factory owns it, saves and hashes it). Labs (`factory/lab.rs`)
//! do the work: each unit of a tech uses one of each of its science packs and `seconds` of lab time
//! at full power. A finished tech unlocks the hand recipes whose outputs it lists; recipes no tech
//! lists are available from the start.
//!
//! Invariants: progress never exceeds a tech's units; `current` is `None` or a tech that is available
//! (every prerequisite done) and not done.
//!
//! To add a tech: append a row to `TECHS` (saves store progress by index, so never reorder), naming
//! its prerequisites by index. A new science pack: an item, a hand recipe, and an entry in `PACKS`.

use crate::block::{BlockId, FILTER, LIFT, RAMP_DOWN, RAMP_UP, SPLITTER, UNDERPASS_IN, UNDERPASS_OUT};
use crate::bytes::{ByteReader, ByteWriter};
use crate::item::{ItemId, GREEN_PACK, RED_PACK};

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
    /// Items whose hand recipes it unlocks.
    pub unlocks: &'static [ItemId],
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
        unlocks: &[b(SPLITTER), b(FILTER)],
    },
    Tech {
        name: "Belt Climbing",
        blurb: "Ramps and lifts carry items up and down hills.",
        needs: &[0],
        packs: &[RED_PACK],
        units: 20,
        seconds: 5.0,
        unlocks: &[b(RAMP_UP), b(RAMP_DOWN), b(LIFT)],
    },
    Tech {
        name: "Green Science",
        blurb: "Green science packs, made from belts and screws, for the next techs.",
        needs: &[0],
        packs: &[RED_PACK],
        units: 30,
        seconds: 5.0,
        unlocks: &[GREEN_PACK],
    },
    Tech {
        name: "Underpasses",
        blurb: "Belts that dive under a crossing belt and come back up.",
        needs: &[2],
        packs: &[RED_PACK, GREEN_PACK],
        units: 15,
        seconds: 10.0,
        unlocks: &[b(UNDERPASS_IN), b(UNDERPASS_OUT)],
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

    /// The tech whose research unlocks crafting `item`, while it isn't done.
    pub fn locked_by(&self, item: ItemId) -> Option<u8> {
        let i = TECHS.iter().position(|t| t.unlocks.contains(&item))? as u8;
        (self.state(i) != TechState::Done).then_some(i)
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

const fn b(block: BlockId) -> ItemId {
    ItemId::block(block)
}

#[cfg(test)]
mod tests;
