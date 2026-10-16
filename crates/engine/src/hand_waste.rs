//! Ore the local player wasted by hand mining this session, so the HUD can teach it at the moment it happens:
//! a short notice for every ore block broken by hand, and from `WARN_FROM` blocks a warning tip with the totals
//! (`api/hand_waste.rs`, `web/src/ui/hints.ts`). Presentation only: counted from `SimEvent::HandMined` in
//! `events.rs`, never saved, so a reloaded page starts at zero.
//!
//! The lesson: by hand a deposit's block gives a handful of ore and the rest of its share is gone; a miner keeps
//! most of it (`deposits.rs`). To change the wording: `notice` and `tip`.

use crate::block::{self, BlockId};
use crate::factory::{fmt_int, MINER_TIERS};

/// Ore blocks broken by hand this session before the warning tip shows.
pub const WARN_FROM: u32 = 3;

#[derive(Default)]
pub struct HandWaste {
    /// Ore blocks broken by hand this session.
    pub blocks: u32,
    kept: u64,
    lost: u64,
    /// The latest block: its ore, the ore kept and the units lost.
    last: (BlockId, u32, u32),
}

impl HandWaste {
    pub fn record(&mut self, ore: BlockId, kept: u32, lost: u32) {
        self.blocks += 1;
        self.kept += kept as u64;
        self.lost += lost as u64;
        self.last = (ore, kept, lost);
    }

    /// One line about the latest block, e.g. "+3 Iron Ore · *97 lost* with the block".
    pub fn notice(&self) -> String {
        if self.blocks == 0 {
            return String::new();
        }
        let (ore, kept, lost) = self.last;
        format!("+{kept} {} · *{} lost* with the block", block::def(ore).name, fmt_int(lost as u64))
    }

    /// The warning tip (hint markup), or nothing before `WARN_FROM` blocks.
    pub fn tip(&self) -> String {
        if self.blocks < WARN_FROM {
            return String::new();
        }
        format!(
            "Ore going to waste: by hand you kept *{} ore* from {} blocks and *{} more was lost* for good. A *miner* \
             on the deposit keeps {}% of it, and it never tires. Hand-mine only what your first miner needs, then \
             build one ([E]).",
            fmt_int(self.kept),
            self.blocks,
            fmt_int(self.lost),
            (MINER_TIERS[0].recovery * 100.0).round() as u32
        )
    }
}

#[cfg(test)]
mod tests;
