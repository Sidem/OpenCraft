//! Block timers (core state): single-block changes due at a later tick. Leaves decay once no log
//! holds them up; grass spreads onto bare dirt next to it and turns back to dirt under a solid block;
//! saplings grow into trees (`sim/saplings.rs`).
//!
//! Invariants: timers start only from block changes (`Sim::block_changed`, called by the actions that
//! break and place blocks and by the timers themselves), never from scanning chunks, and they read and
//! edit through the `*_anywhere` accessors, so they run the same in unloaded chunks and on every peer.
//! `BlockTimers` keeps its entries sorted by (due tick, position, kind), at most one per position and
//! kind and at most `MAX_TIMERS` (new ones beyond that are dropped). `Sim::step` runs the due ones after
//! the tick's actions; a timer checks its condition again when it fires. Delays are exponential draws
//! from `Sim.rng` with a half-life per kind.
//!
//! To add a timed rule: a `TimerKind` (append: saves store it as a byte), its check in `block_changed`
//! and its effect in `fire`.

use crate::block::{BlockId, AIR, DIRT, GRASS, LEAVES, LOG, SAPLING, SOLID};
use crate::bytes::{ByteReader, ByteWriter};
use crate::math::{IVec3, Vec3};
use crate::sim::{Sim, SimEvent};
use crate::TICK_RATE;

/// Most timers pending at once.
pub const MAX_TIMERS: usize = 4096;
/// Half-lives in seconds.
const LEAF_HALF_LIFE: f64 = 5.0;
const GRASS_GROW_HALF_LIFE: f64 = 30.0;
const GRASS_DIE_HALF_LIFE: f64 = 15.0;
/// Removing a log checks the leaves this far away (in each axis).
const LEAF_CHECK_RADIUS: i32 = 6;
/// A leaf stays while a log is at most this many face steps away through leaves.
const LEAF_SUPPORT_STEPS: i32 = 4;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TimerKind {
    LeafDecay = 0,
    GrassGrow = 1,
    GrassDie = 2,
    SaplingGrow = 3,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Timer {
    pub due: u64,
    pub pos: IVec3,
    pub kind: TimerKind,
}

#[derive(Default)]
pub struct BlockTimers {
    list: Vec<Timer>,
}

impl BlockTimers {
    #[cfg(test)]
    pub fn len(&self) -> usize {
        self.list.len()
    }

    /// Whether a timer of `kind` is pending at `pos`.
    pub fn pending(&self, pos: IVec3, kind: TimerKind) -> bool {
        self.list.iter().any(|t| t.pos == pos && t.kind == kind)
    }

    pub fn write_state(&self, w: &mut ByteWriter) {
        w.count(self.list.len());
        for t in &self.list {
            w.u64(t.due);
            w.ivec3(t.pos);
            w.u8(t.kind as u8);
        }
    }

    pub fn read_state(r: &mut ByteReader) -> Option<BlockTimers> {
        let mut timers = BlockTimers::default();
        for _ in 0..r.count()? {
            let (due, pos) = (r.u64()?, r.ivec3()?);
            let kind = match r.u8()? {
                0 => TimerKind::LeafDecay,
                1 => TimerKind::GrassGrow,
                2 => TimerKind::GrassDie,
                3 => TimerKind::SaplingGrow,
                _ => return None,
            };
            timers.schedule(Timer { due, pos, kind });
        }
        Some(timers)
    }
}

impl Sim {
    /// Starts the timers a block change at `pos` (from `old` to what is there now) calls for: leaves
    /// near a removed log, dirt that can grow grass and grass that is now covered, around `pos`, and a
    /// sapling planted at `pos`; a torch on `pos` losing its support (torches.rs); then the water checks
    /// (water.rs).
    pub(crate) fn block_changed(&mut self, pos: IVec3, old: BlockId) {
        if self.block(pos) == SAPLING {
            self.schedule_growth(pos);
        }
        if old == LOG {
            let r = LEAF_CHECK_RADIUS;
            for p in cube(pos, r) {
                if self.block(p) == LEAVES && !self.leaf_supported(p) {
                    self.schedule(p, TimerKind::LeafDecay, LEAF_HALF_LIFE);
                }
            }
        }
        for p in cube(pos, 1) {
            match self.block(p) {
                DIRT if self.grass_can_grow(p) => self.schedule(p, TimerKind::GrassGrow, GRASS_GROW_HALF_LIFE),
                GRASS if self.covered(p) => self.schedule(p, TimerKind::GrassDie, GRASS_DIE_HALF_LIFE),
                _ => {}
            }
        }
        self.torch_support_changed(pos);
        self.water_changed(pos, old);
    }

    /// Runs the timers due by this tick, in order. Called by `step` after the tick's actions.
    pub(crate) fn run_timers(&mut self) {
        let due = self.timers.list.iter().take_while(|t| t.due <= self.tick).count();
        let fired: Vec<Timer> = self.timers.list.drain(..due).collect();
        for t in fired {
            self.fire(t);
        }
    }
}

impl BlockTimers {
    /// Adds a timer in order, unless one of its kind is already pending there or the list is full.
    fn schedule(&mut self, t: Timer) {
        if self.list.len() >= MAX_TIMERS || self.pending(t.pos, t.kind) {
            return;
        }
        let key = |t: &Timer| (t.due, t.pos.x, t.pos.y, t.pos.z, t.kind as u8);
        let at = self.list.iter().position(|o| key(o) > key(&t)).unwrap_or(self.list.len());
        self.list.insert(at, t);
    }
}

impl Sim {
    /// Makes a timer's change if its block and condition still hold.
    fn fire(&mut self, t: Timer) {
        let (block, new) = match t.kind {
            TimerKind::LeafDecay => (LEAVES, AIR),
            TimerKind::GrassGrow => (DIRT, GRASS),
            TimerKind::GrassDie => (GRASS, DIRT),
            TimerKind::SaplingGrow => return self.grow_sapling(t.pos),
        };
        if self.block(t.pos) != block {
            return;
        }
        let holds = match t.kind {
            TimerKind::LeafDecay => !self.leaf_supported(t.pos),
            TimerKind::GrassGrow => self.grass_can_grow(t.pos),
            TimerKind::GrassDie => self.covered(t.pos),
            TimerKind::SaplingGrow => false, // returned above
        };
        if !holds || !self.world.set_block_anywhere(t.pos, new) {
            return;
        }
        if t.kind == TimerKind::LeafDecay {
            self.events.push(SimEvent::LeafDecayed { pos: t.pos });
            if self.leaf_drops_sapling() {
                let (pos, vel) = (t.pos.as_vec3() + Vec3::new(0.5, 0.5, 0.5), Vec3::new(0.0, 1.0, 0.0));
                self.events.push(SimEvent::Dropped { pos, vel, item: SAPLING.into(), count: 1 });
            }
        }
        self.block_changed(t.pos, block);
    }

    /// Schedules `kind` at `pos` after an exponential delay with the given half-life (at least a tick).
    fn schedule(&mut self, pos: IVec3, kind: TimerKind, half_life: f64) {
        self.schedule_after(pos, kind, 0.0, half_life);
    }

    /// Schedules `kind` at `pos` after `min` seconds plus an exponential delay with the given half-life.
    pub(super) fn schedule_after(&mut self, pos: IVec3, kind: TimerKind, min: f64, half_life: f64) {
        let u = 1.0 - self.rng.next_f64(); // (0, 1]
        let ticks = ((min - u.ln() * half_life / std::f64::consts::LN_2) * TICK_RATE as f64) as u64;
        self.timers.schedule(Timer { due: self.tick + ticks.max(1), pos, kind });
    }

    pub(super) fn block(&mut self, p: IVec3) -> BlockId {
        self.world.block_anywhere_or_generate(p)
    }

    /// Whether a log is within `LEAF_SUPPORT_STEPS` face steps of `leaf` through leaves.
    fn leaf_supported(&mut self, leaf: IVec3) -> bool {
        const SPAN: i32 = 2 * LEAF_SUPPORT_STEPS + 1;
        let mut seen = [false; (SPAN * SPAN * SPAN) as usize];
        let index = |p: IVec3| {
            let d = p - leaf + IVec3::new(LEAF_SUPPORT_STEPS, LEAF_SUPPORT_STEPS, LEAF_SUPPORT_STEPS);
            ((d.y * SPAN + d.z) * SPAN + d.x) as usize
        };
        seen[index(leaf)] = true;
        let mut frontier = vec![leaf];
        for _ in 0..LEAF_SUPPORT_STEPS {
            let mut next = Vec::new();
            for p in frontier {
                for n in FACES.map(|f| p + f) {
                    if std::mem::replace(&mut seen[index(n)], true) {
                        continue;
                    }
                    match self.block(n) {
                        LOG => return true,
                        LEAVES => next.push(n),
                        _ => {}
                    }
                }
            }
            frontier = next;
        }
        false
    }

    /// Dirt with nothing solid on top and grass among its 8 horizontal neighbours, one up or down.
    fn grass_can_grow(&mut self, p: IVec3) -> bool {
        if self.covered(p) {
            return false;
        }
        for dy in -1..=1 {
            for (dx, dz) in RING {
                if self.block(p + IVec3::new(dx, dy, dz)) == GRASS {
                    return true;
                }
            }
        }
        false
    }

    /// Whether a solid block sits on top of `p`.
    fn covered(&mut self, p: IVec3) -> bool {
        SOLID[self.block(p + IVec3::new(0, 1, 0)) as usize]
    }
}

/// The six face neighbours' offsets.
pub(super) const FACES: [IVec3; 6] = [
    IVec3::new(1, 0, 0),
    IVec3::new(-1, 0, 0),
    IVec3::new(0, 1, 0),
    IVec3::new(0, -1, 0),
    IVec3::new(0, 0, 1),
    IVec3::new(0, 0, -1),
];

const RING: [(i32, i32); 8] = [(-1, -1), (0, -1), (1, -1), (-1, 0), (1, 0), (-1, 1), (0, 1), (1, 1)];

/// The cube of positions within `r` of `c` in each axis, in (y, z, x) order.
fn cube(c: IVec3, r: i32) -> impl Iterator<Item = IVec3> {
    (-r..=r).flat_map(move |y| (-r..=r).flat_map(move |z| (-r..=r).map(move |x| c + IVec3::new(x, y, z))))
}

#[cfg(test)]
mod tests;
