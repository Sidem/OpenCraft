//! Saplings: leaves now and then leave one behind (broken by hand or decayed), and a sapling planted
//! on dirt or grass grows into a tree after a while, in the same shape as generated trees
//! (`worldgen::tree_blocks`).
//!
//! Invariants: growth is a block timer (`TimerKind::SaplingGrow`, started by `Sim::block_changed`), so
//! it is core state: its delay, the trunk height and the drop chance come from `Sim.rng`, and its edits
//! go through `set_block_anywhere`. A sapling grows only with dirt or grass under it and air where its
//! trunk goes; otherwise it tries again later. Its leaves only go into air. To tune: the constants.

use super::timers::TimerKind;
use crate::block::{AIR, DIRT, GRASS, SAPLING};
use crate::math::IVec3;
use crate::sim::{Sim, SimEvent};
use crate::worldgen::{tree_blocks, WORLD_HEIGHT};

/// Chance that a leaf leaves a sapling when it is broken or decays.
pub const SAPLING_CHANCE: f64 = 1.0 / 25.0;
/// Seconds before a planted sapling can grow, then the half-life of the rest of the wait.
const GROW_MIN: f64 = 60.0;
const GROW_HALF_LIFE: f64 = 90.0;
/// Trunk heights: `TRUNK_MIN` plus 0 to `TRUNK_SPREAD - 1` (as generated trees).
const TRUNK_MIN: i32 = 4;
const TRUNK_SPREAD: u32 = 3;

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
    pub(super) fn grow_sapling(&mut self, pos: IVec3) {
        if self.block(pos) != SAPLING {
            return;
        }
        let trunk = TRUNK_MIN + self.rng.below(TRUNK_SPREAD) as i32;
        let mut room = self.can_plant(pos) && pos.y + trunk + 1 < WORLD_HEIGHT;
        for dy in 1..trunk {
            room = room && self.block(pos + IVec3::new(0, dy, 0)) == AIR;
        }
        if !room {
            self.schedule_growth(pos);
            return;
        }
        let mut edits = Vec::new();
        tree_blocks(pos - IVec3::new(0, 1, 0), trunk, self.world.generator().seed(), |p, id, replace| {
            edits.push((p, id, replace))
        });
        for (p, id, replace) in edits {
            if replace || self.block(p) == AIR {
                self.world.set_block_anywhere(p, id);
            }
        }
        self.events.push(SimEvent::TreeGrew { pos });
    }
}

#[cfg(test)]
mod tests;
