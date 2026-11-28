//! Sod (core state): a turf face an edit uncovers starts bare, showing soil as the wall of a freshly dug hole does,
//! and grass grows back over a block's bare faces one at a time. Faces open when generated are grown; dirt that turns
//! to grass greens one open face and grows over the rest the same way. Which grown faces draw green is the mesher's
//! (lit ones: `mesher/face_layer.rs`); the bare faces live in the chunks (`world/sod.rs`).
//!
//! Invariants: bare bits change only here, from block changes (`Sim::block_changed`) and `SodGrow` timers, so every
//! peer agrees; a covered face forgets it was bare; a turf cell with bare faces has a `SodGrow` timer, or gets one on
//! its next check (`check_grass`) if the timer list was full. To tune: `SOD_MIN`, `SOD_HALF_LIFE`.

use crate::block::{is_turf, BlockId, SOLID};
use crate::math::IVec3;
use crate::sim::Sim;

use super::timers::{TimerKind, FACES};

/// Seconds a freshly uncovered face stays bare at least, so a hole just dug never greens while you look at it.
pub(super) const SOD_MIN: f64 = 90.0;
/// Half-life in seconds of a bare face past that, or from when the one before it grew.
pub(super) const SOD_HALF_LIFE: f64 = 60.0;

impl Sim {
    /// The block at `pos` went from `old` to what is there now: turf faces it uncovers turn bare, the ones it covers
    /// forget they were.
    pub(super) fn sod_changed(&mut self, pos: IVec3, old: BlockId) {
        let now = SOLID[self.block(pos) as usize];
        if now == SOLID[old as usize] {
            return;
        }
        for (f, &off) in FACES.iter().enumerate() {
            let turf = pos + off;
            if !is_turf(self.block(turf)) {
                continue;
            }
            let (bit, mask) = (1u8 << (f ^ 1), self.world.bare_anywhere(turf));
            let next = if now { mask & !bit } else { mask | bit };
            self.world.set_bare_anywhere(turf, next);
            if next & !mask != 0 {
                self.schedule_after(turf, TimerKind::SodGrow, SOD_MIN, SOD_HALF_LIFE);
            }
        }
    }

    /// Dirt at `pos` just turned to grass: it shows green on one open face, picked at random, and grows over the rest
    /// in turn, so grass creeps across a wall rather than flicking every face green at once.
    pub(super) fn sod_sprouted(&mut self, pos: IVec3) {
        let mut open = 0;
        for (f, &off) in FACES.iter().enumerate() {
            if !SOLID[self.block(pos + off) as usize] {
                open |= 1 << f;
            }
        }
        self.grow_one(pos, open);
    }

    /// A `SodGrow` timer fired at `pos`: grass grows over one of its bare faces, picked at random; the next waits.
    pub(super) fn grow_sod(&mut self, pos: IVec3) {
        let mask = self.world.bare_anywhere(pos);
        self.grow_one(pos, mask);
    }

    /// Leaves `mask` less one random bit as the bare faces at `pos`, and a timer for the next if any remain.
    fn grow_one(&mut self, pos: IVec3, mask: u8) {
        if mask == 0 {
            return;
        }
        let nth = self.rng.next_u32() % mask.count_ones();
        let bit = (0..6).map(|f| 1u8 << f).filter(|&b| mask & b != 0).nth(nth as usize).unwrap_or(0);
        let rest = mask & !bit;
        self.world.grow_bare_anywhere(pos, rest);
        if rest != 0 {
            self.schedule(pos, TimerKind::SodGrow, SOD_HALF_LIFE);
        }
    }
}

#[cfg(test)]
mod tests;
