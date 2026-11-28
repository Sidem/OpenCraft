//! Mass-conserving cellular water (core state): event-driven, zero-extra-memory 8-level liquid transfer capped per
//! tick. Every cell above `SEA_LEVEL` holds an integer liquid mass `0..=8` (`AIR` = 0, `FLOW_1..=FLOW_7` = 1..=7,
//! `WATER` = 8, and sparse waterlogged `Chunk::wets` = `0..=8`), so water is never created from nothing and never lost.
//!
//! Rules (`step_cell`):
//! - **Natural springs (`SPRING`):** unbreakable aquifer rock at river headwaters and mountain springs emits finite
//!   water horizontally out of its 4 side faces at its own elevation, topping open side cells up to `WATER` (8).
//!   Placing solid blocks over all 4 sides caps the spring.
//! - **Ocean (`y <= SEA_LEVEL`):** endless ocean reservoir — cells at or below `SEA_LEVEL` adjacent to or under
//!   `WATER` refill to `WATER` (8), and thin flowing water spilling onto the open ocean surface at `SEA_LEVEL + 1`
//!   merges cleanly into the sea.
//! - **Gravity (downward transfer):** any liquid cell with mass `m > 0` transfers `min(m, 8 - mb)` directly into the
//!   cell below when it can hold water (`mb < 8`), strictly conserving `m + mb`.
//! - **Sideways leveling (pressure equalization):** when resting on a solid block or full `8` water below, a cell
//!   transfers 1/8th units to eligible lower horizontal neighbours (`m >= nm + 2`, or `m == nm + 1` when the neighbour
//!   sits over a drop or has a strictly lower horizontal neighbour `m > nm > m2`), preferring drops and lowest levels
//!   with deterministic rotation tie-breaking. Strictly conserves total mass and settles without oscillation.
//!
//! Invariants: checks start only from block/wet changes (`Sim::water_changed`, from `block_changed`), never from
//! scanning chunks. Queue runs FIFO with at most one check per cell, `MAX_PENDING` total, and `MAX_WATER_UPDATES`
//! checks per tick. Reads and writes use `*_anywhere` accessors; loaded chunks remesh lazily.

use std::collections::VecDeque;

use rustc_hash::FxHashSet;

use crate::block::{block_of_wet, waterloggable, wet_of, BlockId, AIR, FLOW_7, LIQUID, SPRING, WATER};
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

/// Whether a raw block cell can hold liquid mass (air, liquid, or a waterloggable machine like a belt).
#[inline]
fn can_hold_water(b: BlockId) -> bool {
    b == AIR || LIQUID[b as usize] || waterloggable(b)
}

#[inline]
fn has_water_source(b: BlockId) -> bool {
    LIQUID[b as usize] || b == SPRING
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

    /// Liquid mass `0..=8` held at `pos` (reading `wet_anywhere` for waterloggable blocks).
    pub(crate) fn water_mass(&mut self, pos: IVec3) -> u8 {
        let b = self.block(pos);
        self.mass_of_cell(pos, b)
    }

    #[inline]
    fn mass_of_cell(&mut self, pos: IVec3, raw: BlockId) -> u8 {
        if waterloggable(raw) {
            self.world.wet_anywhere(pos)
        } else {
            wet_of(raw)
        }
    }

    /// Sets the liquid mass (`0..=8`) held at `pos` (updating either `wet` for waterloggable blocks or the block
    /// itself for air/liquid cells) and fires the appropriate change hook. Returns `true` if the cell changed.
    pub(crate) fn set_water_mass(&mut self, pos: IVec3, mass: u8) -> bool {
        let raw = self.block(pos);
        if waterloggable(raw) {
            let old_wet = self.world.wet_anywhere(pos);
            if mass != old_wet && self.world.set_wet_anywhere_later(pos, mass) {
                self.water_changed(pos, block_of_wet(old_wet));
                return true;
            }
            false
        } else if raw == AIR || LIQUID[raw as usize] {
            let new = block_of_wet(mass);
            if new != raw && self.world.set_block_anywhere_later(pos, new) {
                self.block_changed(pos, raw);
                return true;
            }
            false
        } else {
            false
        }
    }

    /// Schedules the water checks a block change at `pos` (from `old`) calls for, if water or a spring is involved:
    /// the cell, its six neighbours, and the sideways neighbours of the cells above and below.
    pub(crate) fn water_changed(&mut self, pos: IVec3, old: BlockId) {
        let wet = has_water_source(old)
            || has_water_source(self.water_at(pos))
            || FACES.iter().any(|&f| has_water_source(self.water_at(pos + f)));
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
            self.step_cell(pos);
        }
    }

    /// Whether `pos` is a continuous steady-state waterfall column cell: `FLOW_7` landing on `FLOW_7`/`WATER` while
    /// actively fed by `FLOW_7`/`WATER` above or `WATER` beside it. Steady falls hold their column without triggering
    /// runaway draining across downstream rivers, and dry up cleanly as soon as their feed is dammed or drains away.
    fn is_steady_fall(&mut self, pos: IVec3) -> bool {
        if self.block(pos) != FLOW_7 || pos.y <= 0 {
            return false;
        }
        let below = self.water_at(pos - UP);
        if below != FLOW_7 && below != WATER {
            return false;
        }
        let above = self.water_at(pos + UP);
        if above == FLOW_7 || above == WATER {
            return true;
        }
        SIDES.iter().any(|&s| self.water_at(pos + s) == WATER)
    }

    /// Whether `pos` is directly fed horizontally by an active `SPRING` block above sea level.
    #[inline]
    fn touches_spring(&mut self, pos: IVec3) -> bool {
        pos.y > SEA_LEVEL && SIDES.iter().any(|&s| self.block(pos + s) == SPRING)
    }

    /// Runs one cellular water step for `pos`.
    fn step_cell(&mut self, pos: IVec3) {
        let raw = self.block(pos);

        // 1. Natural river spring: emits finite water sideways at its own elevation (pluggable with solid blocks).
        if raw == SPRING {
            if pos.y > SEA_LEVEL {
                for s in SIDES {
                    let n = pos + s;
                    let nb = self.block(n);
                    if can_hold_water(nb) && self.mass_of_cell(n, nb) < 8 {
                        self.set_water_mass(n, 8);
                    }
                }
            }
            return;
        }

        if !can_hold_water(raw) {
            return;
        }

        // 2. Ocean reservoir at or below SEA_LEVEL: refills from adjacent/above ocean water.
        let mut m = self.mass_of_cell(pos, raw);
        if pos.y <= SEA_LEVEL && m < 8 {
            let ocean_feed = self.water_at(pos + UP) == WATER || SIDES.iter().any(|&s| self.water_at(pos + s) == WATER);
            if ocean_feed {
                self.set_water_mass(pos, 8);
                return;
            }
        }

        if m == 0 {
            return;
        }

        // 2b. Steady-state natural waterfall column (`FLOW_7` actively fed and landing in `FLOW_7`/`WATER`).
        if self.is_steady_fall(pos) {
            return;
        }

        let spring_fed = self.touches_spring(pos);

        // 3. Gravity: fall directly into the cell below whenever it can hold more water.
        // When dropping into empty AIR with mass >= 2, split `m.div_ceil(2)` so continuous streams (like pumped
        // waterfalls and cliff cascades) form an unbroken falling column while still draining cleanly to AIR on `m == 1`.
        let below = pos - UP;
        if below.y >= 0 {
            let bb = self.block(below);
            if can_hold_water(bb) {
                let bm = self.mass_of_cell(below, bb);
                if bm < 8 {
                    let transfer = if bm == 0 && m > 1 { m.div_ceil(2) } else { m.min(8 - bm) };
                    m -= transfer;
                    if spring_fed {
                        m = 8;
                    }
                    self.set_water_mass(pos, m);
                    self.set_water_mass(below, bm + transfer);
                    return;
                }
                // Water spilling onto the open ocean surface at SEA_LEVEL + 1 drains into the infinite sea.
                if pos.y == SEA_LEVEL + 1
                    && m < 8
                    && SIDES.iter().any(|&s| {
                        let side = pos + s;
                        self.water_at(side) == AIR && self.water_at(side - UP) == WATER
                    })
                {
                    self.set_water_mass(pos, 0);
                    return;
                }
            }
        }

        // 4. Sideways pressure equalization (strictly mass-conserving; spring-fed cells replenish immediately).
        let mut hold = [false; 4];
        let mut nm = [0u8; 4];
        let mut drop = [false; 4];
        let mut lower = [false; 4];
        for (i, &s) in SIDES.iter().enumerate() {
            let n = pos + s;
            let nb = self.block(n);
            if !can_hold_water(nb) || self.is_steady_fall(n) {
                continue;
            }
            hold[i] = true;
            let n_mass = self.mass_of_cell(n, nb);
            nm[i] = n_mass;
            if n.y > 0 {
                let n_below = self.block(n - UP);
                if can_hold_water(n_below) && self.mass_of_cell(n - UP, n_below) < 8 {
                    drop[i] = true;
                }
            }
            if n_mass > 0 && n_mass + 1 == m {
                lower[i] = SIDES.iter().any(|&s2| {
                    let n2 = n + s2;
                    if n2 == pos {
                        return false;
                    }
                    let nb2 = self.block(n2);
                    can_hold_water(nb2) && !self.is_steady_fall(n2) && self.mass_of_cell(n2, nb2) < n_mass
                });
            }
        }

        let orig_m = m;
        let orig_nm = nm;
        let rot = (pos.x.wrapping_mul(3) ^ pos.z.wrapping_mul(5) ^ pos.y) as usize & 3;
        while m > 0 {
            let mut best: Option<usize> = None;
            for k in 0..4 {
                let i = (rot + k) & 3;
                if !hold[i] {
                    continue;
                }
                let can_give = m >= nm[i] + 2 || (m == nm[i] + 1 && (drop[i] || lower[i]));
                if !can_give {
                    continue;
                }
                let better = match best {
                    None => true,
                    Some(b) => (drop[i], u8::MAX - nm[i]) > (drop[b], u8::MAX - nm[b]),
                };
                if better {
                    best = Some(i);
                }
            }
            let Some(i) = best else { break };
            m -= 1;
            nm[i] += 1;
        }

        if m != orig_m {
            // Immediate unthrottled replenishment when touching a SPRING so downstream queues can never starve it.
            let final_m = if spring_fed { 8 } else { m };
            self.set_water_mass(pos, final_m);
            for (i, &s) in SIDES.iter().enumerate() {
                if nm[i] != orig_nm[i] {
                    self.set_water_mass(pos + s, nm[i]);
                }
            }
        } else {
            // 2-hop wake-up: if this cell sits on a 1-step slope (higher > self > lower), wake the higher neighbour
            // so wide ponds flatten and channels keep flowing without scanning.
            let has_lower = (0..4).any(|i| hold[i] && nm[i] < m);
            if has_lower {
                let due = self.tick + WATER_DELAY;
                for (i, &s) in SIDES.iter().enumerate() {
                    if hold[i] && nm[i] == m + 1 {
                        self.water.schedule(due, pos + s);
                    }
                }
            }
        }
    }
}

const UP: IVec3 = IVec3::new(0, 1, 0);
pub(crate) const SIDES: [IVec3; 4] =
    [IVec3::new(1, 0, 0), IVec3::new(-1, 0, 0), IVec3::new(0, 0, 1), IVec3::new(0, 0, -1)];

#[cfg(test)]
pub(super) mod tests;
