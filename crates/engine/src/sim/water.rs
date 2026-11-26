//! Flowing water (core state): a Minecraft-like spread, event-driven and capped per tick. `WATER` is a
//! source; `FLOW_1`..`FLOW_7` are flowing water, the number its level (`block::flow`).
//!
//! Rules, checked one cell at a time (`water_rule`) for air, flowing water and `block::waterloggable`
//! blocks (every other block is a wall, and sources stay):
//! - At or below `SEA_LEVEL` (the sea's top water block) a cell next to a source, sideways or above,
//!   becomes a source. The sea is endless: it refills holes and trenches, floods what connects to it,
//!   and a pump can't lower it (nor the few cells of a deep pond that reach that low).
//! - A cell under water becomes falling water: `FLOW_7` under a source, the same level under flowing water, so a fall
//!   never gains strength (a water wheel fed from a pump can't power that pump: `factory/process/hydro.rs`).
//! - Otherwise its level is its best feed minus 1 (0 is air, so a flow without a feed dries up). A
//!   source feeds 8; flowing water feeds its level, or one more where falling water lands, but only while it
//!   rests on a block that holds it (neither air nor water): water with air under it falls instead of
//!   spreading, and water landing on water joins it.
//! - A waterloggable block (`world/waterlog.rs`) follows the same rules using its sparse wet level
//!   (`0` = air, `1..=7` = `FLOW_1..=FLOW_7`, `8` = `WATER`) and feeds neighbours like the water it holds.
//!
//! Invariants: checks start only from block changes (`Sim::water_changed`, from `block_changed`), never
//! from scanning chunks. A change near water schedules its cell and the cells whose rule it touches
//! `WATER_DELAY` ticks later; the queue runs first in, first out (due ticks never decrease), holds at
//! most one check per cell and `MAX_PENDING` in all, and `run_water` runs at most
//! `MAX_WATER_UPDATES` due checks a tick; the rest wait. Reads and writes use the `*_anywhere`
//! accessors; loaded chunks remesh later, in the streaming budget.

use std::collections::VecDeque;

use rustc_hash::FxHashSet;

use crate::block::{block_of_wet, flow, flow_level, waterloggable, wet_of, BlockId, AIR, FLOW_7, LIQUID, WATER};
use crate::bytes::{ByteReader, ByteWriter};
use crate::math::IVec3;
use crate::sim::timers::FACES;
use crate::sim::Sim;
use crate::worldgen::SEA_LEVEL;

/// Ticks from a change to the checks it schedules.
pub const WATER_DELAY: u64 = 5;
/// Most checks run in one tick.
pub const MAX_WATER_UPDATES: usize = 256;
/// Most checks pending at once (new ones beyond that are dropped).
const MAX_PENDING: usize = 1 << 16;

/// Pending water checks: (due tick, cell), in order.
#[derive(Default)]
pub struct WaterQueue {
    list: VecDeque<(u64, IVec3)>,
    /// The cells in `list`, for a quick duplicate check (never iterated).
    pending: FxHashSet<IVec3>,
    /// Checks run in the last tick (not state: for tests and measurements).
    pub last_run: usize,
}

impl WaterQueue {
    /// Checks pending.
    #[cfg(test)]
    pub fn pending_count(&self) -> usize {
        self.list.len()
    }

    pub fn write_state(&self, w: &mut ByteWriter) {
        w.count(self.list.len());
        for &(due, pos) in &self.list {
            w.u64(due);
            w.ivec3(pos);
        }
    }

    pub fn read_state(r: &mut ByteReader) -> Option<WaterQueue> {
        let mut q = WaterQueue::default();
        for _ in 0..r.count()? {
            let (due, pos) = (r.u64()?, r.ivec3()?);
            if q.list.back().is_some_and(|b| b.0 > due) || !q.pending.insert(pos) {
                return None;
            }
            q.list.push_back((due, pos));
        }
        Some(q)
    }

    fn schedule(&mut self, due: u64, pos: IVec3) {
        if self.list.len() < MAX_PENDING && self.pending.insert(pos) {
            self.list.push_back((due, pos));
        }
    }
}

impl Sim {
    /// Effective water state at `pos`: the held liquid block for liquid or wet waterloggable cells,
    /// `AIR` for dry waterloggable cells, or the block itself otherwise.
    pub(crate) fn water_at(&mut self, pos: IVec3) -> BlockId {
        let b = self.block(pos);
        if waterloggable(b) {
            block_of_wet(self.world.wet_anywhere(pos))
        } else {
            b
        }
    }

    /// Schedules the water checks a block change at `pos` (from `old`) calls for, if water is involved:
    /// the cell, its six neighbours, and the sideways neighbours of the cells above and below (whose
    /// feed depends on what they rest on and whether water falls onto them).
    pub(crate) fn water_changed(&mut self, pos: IVec3, old: BlockId) {
        let wet = LIQUID[old as usize]
            || LIQUID[self.water_at(pos) as usize]
            || FACES.iter().any(|&f| LIQUID[self.water_at(pos + f) as usize]);
        if !wet {
            return;
        }
        let due = self.tick + WATER_DELAY;
        self.water.schedule(due, pos);
        for f in FACES {
            self.water.schedule(due, pos + f);
        }
        for dy in [1, -1] {
            for s in SIDES {
                self.water.schedule(due, pos + IVec3::new(s.x, dy, s.z));
            }
        }
    }

    /// Runs the water checks due by this tick, up to `MAX_WATER_UPDATES`. Called by `step`.
    pub(crate) fn run_water(&mut self) {
        self.water.last_run = 0;
        while self.water.last_run < MAX_WATER_UPDATES {
            let Some(&(due, pos)) = self.water.list.front() else { return };
            if due > self.tick {
                return;
            }
            self.water.last_run += 1;
            self.water.list.pop_front();
            self.water.pending.remove(&pos);
            let old = self.block(pos);
            if waterloggable(old) {
                let old_wet = self.world.wet_anywhere(pos);
                let old_water = block_of_wet(old_wet);
                if let Some(new_water) = self.water_rule(pos, old_water) {
                    let new_wet = wet_of(new_water);
                    if new_wet != old_wet && self.world.set_wet_anywhere_later(pos, new_wet) {
                        self.water_changed(pos, old_water);
                    }
                }
            } else if let Some(new) = self.water_rule(pos, old) {
                if new != old && self.world.set_block_anywhere_later(pos, new) {
                    self.block_changed(pos, old);
                }
            }
        }
    }

    /// What the cell at `pos` (holding effective water state `b`) should hold now; `None` for walls and sources.
    fn water_rule(&mut self, pos: IVec3, b: BlockId) -> Option<BlockId> {
        if b != AIR && flow_level(b).is_none() {
            return None;
        }
        let above = self.water_at(pos + UP);
        let sides = SIDES.map(|s| pos + s);
        if pos.y <= SEA_LEVEL && (above == WATER || sides.iter().any(|&s| self.water_at(s) == WATER)) {
            return Some(WATER);
        }
        if let Some(level) = self.factory.outlet_stream_level_at(pos, b) {
            return Some(flow(level));
        }
        if LIQUID[above as usize] {
            return Some(match flow_level(above) {
                Some(lvl) => flow(lvl),
                None => FLOW_7,
            });
        }
        let feed = sides.iter().map(|&s| self.feed(s)).max().unwrap_or(0);
        Some(if feed > 1 { flow(feed - 1) } else { AIR })
    }

    /// How strongly the cell at `pos` feeds its sideways neighbours (see the module header).
    fn feed(&mut self, pos: IVec3) -> u8 {
        let b = self.water_at(pos);
        if b == WATER {
            return 8;
        }
        let Some(level) = flow_level(b) else { return 0 };
        let below = self.water_at(pos - UP);
        if below == AIR || LIQUID[below as usize] {
            return 0;
        }
        if LIQUID[self.water_at(pos + UP) as usize] {
            level + 1
        } else {
            level
        }
    }
}

const UP: IVec3 = IVec3::new(0, 1, 0);
const SIDES: [IVec3; 4] = [IVec3::new(1, 0, 0), IVec3::new(-1, 0, 0), IVec3::new(0, 0, 1), IVec3::new(0, 0, -1)];

#[cfg(test)]
pub(super) mod tests;
