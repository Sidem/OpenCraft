//! What survey satellites give (step 12.3, presentation only): every vein and lode under a survey satellite's
//! footprint goes on the maps as a fact, and the AI survey drops its guesses near them. Never core state; the
//! satellites themselves are (`satellites.rs`).
//!
//! The sky repeats every `WRAP` blocks, so "everything under coverage" is endless ground: only a square of
//! [`SPAN`] chunk columns each way around the player is swept. The sweep walks that square [`PER_CALL`] columns
//! at a time (the maps call `refresh_survey` a few times a second); a column counts as covered when its middle is (a fuzzy edge of up to 16 blocks), and the deposits
//! seeded in it join the finds. Finds are kept for the session (not saved) until they fall out of the square
//! plus [`KEEP`] columns, so a pass of the footprint leaves a lasting map. Outcrops are left out: the explored
//! map already shows those.
//!
//! To give another kind a use that needs ground queries, add a module beside this one; `Seen` is only the survey kind's.

use rustc_hash::FxHashMap;

use crate::block::BlockId;
use crate::chunk::{CHUNK_SHIFT, CHUNK_SIZE};
use crate::deposits::{Deposit, DepositKey, Tier};
use crate::satellites::{Satellites, SURVEY};
use crate::worldgen::WorldGen;

/// Chunk columns swept each way from the player's (a square of 65 × 65 columns, 2080 blocks wide).
pub const SPAN: i32 = 32;
/// Columns checked a call (the maps ask a few times a second): a full sweep takes a few seconds.
const PER_CALL: i32 = 200;
/// Chunk columns past the swept square at which an old find is forgotten.
const KEEP: i32 = 8;

/// Deposits survey satellites have covered, and the sweep that finds more.
#[derive(Default)]
pub struct Seen {
    found: FxHashMap<DepositKey, Deposit>,
    /// The sweep in progress: the chunk column it centres on and the next column's index in the square.
    sweep: Option<(i32, i32, i32)>,
    /// Bumps whenever `found` changes, so the AI survey knows to guess again.
    pub changes: u32,
}

impl Seen {
    /// The deposits found.
    pub fn deposits(&self) -> impl Iterator<Item = &Deposit> {
        self.found.values()
    }

    /// Where the finds lie and their ore: x, z, block (the same shape as `Known::centres`).
    pub fn centres(&self) -> impl Iterator<Item = (i32, i32, BlockId)> + '_ {
        self.found.values().map(|d| (d.center.x, d.center.z, d.ore()))
    }

    /// Sweeps a few more columns around the player at `centre` (world column) at `tick`. Does nothing while no survey
    /// satellite is up.
    pub fn step(&mut self, generator: &WorldGen, sats: &Satellites, tick: u64, centre: (i32, i32)) {
        if sats.count(SURVEY) == 0 {
            self.sweep = None;
            return;
        }
        let side = 2 * SPAN + 1;
        let (cx, cz, mut next) = *self.sweep.get_or_insert((centre.0 >> CHUNK_SHIFT, centre.1 >> CHUNK_SHIFT, 0));
        let mut seeded = Vec::new();
        for _ in 0..PER_CALL {
            if next >= side * side {
                self.sweep = None;
                self.forget_far(cx, cz);
                return;
            }
            let (col_x, col_z) = (cx - SPAN + next % side, cz - SPAN + next / side);
            next += 1;
            let middle = (col_x * CHUNK_SIZE + CHUNK_SIZE / 2, col_z * CHUNK_SIZE + CHUNK_SIZE / 2);
            if sats.covering(SURVEY, middle.0, middle.1, tick) == 0 {
                continue;
            }
            seeded.clear();
            generator.seed_deposits(col_x, col_z, &mut seeded);
            for d in seeded.iter().filter(|d| d.tier() != Tier::Outcrop) {
                if self.found.insert(d.key, *d).is_none() {
                    self.changes += 1;
                }
            }
        }
        self.sweep = Some((cx, cz, next));
    }

    /// Forgets finds more than `SPAN + KEEP` columns from the chunk column `(cx, cz)`.
    fn forget_far(&mut self, cx: i32, cz: i32) {
        let before = self.found.len();
        let far = SPAN + KEEP;
        self.found.retain(|k, _| (k.cx - cx).abs() <= far && (k.cz - cz).abs() <= far);
        if self.found.len() != before {
            self.changes += 1;
        }
    }
}

#[cfg(test)]
mod tests;
