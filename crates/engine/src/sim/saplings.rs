//! Saplings: leaves now and then leave one behind (broken by hand or decayed), and a sapling planted
//! on dirt or grass grows into a tree after a while, grown like generated trees but young (`worldgen::grow_young`:
//! a trunk one block wide, at most `YOUNG_MAX` tall), so it fits a forester's patch.
//!
//! Invariants: growth is a block timer (`TimerKind::SaplingGrow`, started by `Sim::block_changed`), so
//! it is core state: its delay, the tree's height and the drop chance come from `Sim.rng`, and its edits
//! go through `set_block_anywhere_later` (a tree is hundreds of blocks). A sapling grows only with dirt or grass under it and every log of its tree
//! going into air, leaves or wood (never through a build); otherwise it tries again later. Its leaves only go into
//! air. To tune: the constants.

use super::timers::TimerKind;
use crate::block::{is_leaves, is_log, is_sapling, AIR, DIRT, GRASS};
use crate::math::IVec3;
use crate::sim::{Sim, SimEvent};
use crate::worldgen::{grow_young, kind_of_sapling, WORLD_HEIGHT};

/// Chance that a leaf leaves a sapling when it is broken or decays.
pub const SAPLING_CHANCE: f64 = 1.0 / 25.0;
/// Seconds before a planted sapling can grow, then the half-life of the rest of the wait.
const GROW_MIN: f64 = 60.0;
const GROW_HALF_LIFE: f64 = 90.0;

impl Sim {
    /// Whether a leaf that just went away leaves a sapling behind (draws from `rng`).
    pub(crate) fn leaf_drops_sapling(&mut self) -> bool {
        self.rng.next_f64() < SAPLING_CHANCE
    }

    /// Whether a sapling can be planted at `pos`: on dirt or grass.
    pub(crate) fn can_plant(&mut self, pos: IVec3) -> bool {
        matches!(self.block(pos - IVec3::new(0, 1, 0)), DIRT | GRASS)
    }

    pub(super) fn schedule_growth(&mut self, pos: IVec3) {
        self.schedule_after(pos, TimerKind::SaplingGrow, GROW_MIN, GROW_HALF_LIFE);
    }

    /// Grows the sapling at `pos` into a tree if it has room, else tries again later.
    pub(crate) fn grow_sapling(&mut self, pos: IVec3) {
        let sapling = self.block(pos);
        if !is_sapling(sapling) {
            return;
        }
        let ground = pos - IVec3::new(0, 1, 0);
        let mut edits = Vec::new();
        let roll = self.rng.next_u32();
        grow_young(kind_of_sapling(sapling), ground, roll, self.world.generator().seed(), |p, id, replace| {
            edits.push((p, id, replace))
        });
        let mut room = self.can_plant(pos);
        for &(p, _, replace) in &edits {
            let b = if replace && p != pos && p != ground { self.block(p) } else { AIR };
            room = room && p.y < WORLD_HEIGHT && (b == AIR || is_leaves(b) || is_log(b));
        }
        if !room {
            self.schedule_growth(pos);
            return;
        }
        for (p, id, replace) in edits {
            if replace || self.block(p) == AIR {
                // Remeshed with the streaming budget: one remesh per block was a whole-second freeze.
                self.world.set_block_anywhere_later(p, id);
            }
        }
        self.events.push(SimEvent::TreeGrew { pos });
    }
}

#[cfg(test)]
mod tests;
