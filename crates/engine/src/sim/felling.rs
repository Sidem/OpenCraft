//! Felling a grown tree whole (core state), the way the game Rust does it: a grown tree's wood (`block::is_tree_wood`)
//! is never mined a block at a time. Swings at its trunk near the ground (`Action::ChopTree`, sent by the hands every
//! `SWING_SECONDS`) add work; a mark on the bark (`mark_of`) counts whole and moves once struck, a swing elsewhere
//! counts half. When the work reaches what mining the trunk would take (`work_needed`) the tree comes down: its
//! blocks leave the world at once (`tree.rs`), the host draws it falling (`Sim::fallen`, not state), and after
//! `FALL_TICKS` the trunk's logs, its limbs as sticks and a sapling or two land where the trunk lies.
//!
//! Invariants: chops and falls change only through actions and ticks, and are saved (`write_state`); a chop is
//! keyed by its trunk's foot (`tree::Trunk`), so swings at any of its low logs add up. Old worlds' oaks and logs a
//! player placed are the oak's `LOG`, which still mines block by block. To tune: the constants below.

mod tree;

pub use tree::{trunk, Trunk};

use super::{PlayerId, Sim, SimEvent};
use crate::block::{self, BlockId};
use crate::bytes::{ByteReader, ByteWriter};
use crate::item::{ItemId, STICK};
use crate::math::{hash3, IVec3, Vec3};
use crate::tools;
use crate::wood::{self, Form};
use crate::TICK_RATE;

/// How often the hands swing at a tree while the button is held.
pub const SWING_SECONDS: f32 = 0.5;
/// A swing counts on the trunk's lowest this many layers (the cut is near the ground).
pub const LOW: i32 = 3;
/// How long a tree takes to fall before what it gives lands.
pub const FALL_TICKS: u64 = TICK_RATE as u64 * 8 / 5;
/// A swing off the mark does this share of one on it.
const OFF_MARK: f32 = 0.5;
/// How near the mark a swing must land: across the bark (radians round the trunk) and up it (blocks).
const MARK_ANGLE: f32 = 0.7;
const MARK_HEIGHT: f32 = 0.75;
/// Limb logs for each stick a felled tree gives, and leaves for each sapling past the first.
const LIMBS_PER_STICK: u32 = 2;
const LEAVES_PER_SAPLING: u32 = 120;

/// Trees being chopped and trees falling.
#[derive(Default, Clone, Debug, PartialEq)]
pub struct Felling {
    /// Sorted by foot.
    chops: Vec<Chop>,
    falls: Vec<Fall>,
}

#[derive(Clone, Debug, PartialEq)]
struct Chop {
    foot: IVec3,
    /// Work done, thousandths of a second of bare-handed mining.
    work: u32,
    /// Marks struck so far: which mark shows (`mark_of`).
    marks: u16,
}

#[derive(Clone, Debug, PartialEq)]
struct Fall {
    due: u64,
    drops: Vec<(Vec3, ItemId, u32)>,
}

/// A tree that just came down, for the host to draw falling: its blocks, the foot it turns about and the way it
/// falls (radians from +x towards +z).
pub struct Fallen {
    pub foot: Vec3,
    pub angle: f32,
    pub blocks: Vec<(IVec3, BlockId)>,
}

/// Work felling `trunk` takes: what mining its logs one by one would, in thousandths of a second bare-handed.
pub fn work_needed(trunk: &Trunk, log: BlockId) -> u32 {
    (trunk.logs as f32 * block::def(log).break_time * 1000.0) as u32
}

/// Where mark `k` of the tree standing on `trunk` sits on its bark: (x, y, z) in the world.
pub fn mark_of(seed: u32, trunk: &Trunk, k: u16) -> Vec3 {
    let h = hash3(seed ^ 0xFE11, trunk.foot.x.wrapping_add(k as i32 * 7919), trunk.foot.y, trunk.foot.z);
    let a = (h % 3600) as f32 / 3600.0 * std::f32::consts::TAU;
    let up = 0.5 + ((h >> 12) % 3) as f32 * 0.6;
    Vec3::new(
        (trunk.centre[0] + a.cos() * trunk.radius) as f64,
        trunk.low as f64 + up as f64,
        (trunk.centre[1] + a.sin() * trunk.radius) as f64,
    )
}

/// Whether a swing landing at `at` strikes `mark` on a trunk with its middle at `centre`.
fn on_mark(at: Vec3, mark: Vec3, centre: [f32; 2]) -> bool {
    let angle = |p: Vec3| (p.z as f32 - centre[1]).atan2(p.x as f32 - centre[0]);
    let mut d = (angle(at) - angle(mark)).abs();
    if d > std::f32::consts::PI {
        d = std::f32::consts::TAU - d;
    }
    d < MARK_ANGLE && ((at.y - mark.y) as f32).abs() < MARK_HEIGHT
}

impl Felling {
    /// The marks struck on the tree standing on `foot`, or `None` before its first swing.
    pub fn marks(&self, foot: IVec3) -> Option<u16> {
        self.chops.iter().find(|c| c.foot == foot).map(|c| c.marks)
    }

    /// Work done on the tree standing on `foot`.
    pub fn work(&self, foot: IVec3) -> u32 {
        self.chops.iter().find(|c| c.foot == foot).map_or(0, |c| c.work)
    }

    pub fn write_state(&self, w: &mut ByteWriter) {
        w.count(self.chops.len());
        for c in &self.chops {
            w.ivec3(c.foot);
            w.u32(c.work);
            w.u16(c.marks);
        }
        w.count(self.falls.len());
        for f in &self.falls {
            w.u64(f.due);
            w.count(f.drops.len());
            for &(p, item, n) in &f.drops {
                w.vec3(p);
                w.u16(item.0);
                w.u32(n);
            }
        }
    }

    pub fn read_state(r: &mut ByteReader) -> Option<Felling> {
        let mut f = Felling::default();
        for _ in 0..r.count()? {
            f.chops.push(Chop { foot: r.ivec3()?, work: r.u32()?, marks: r.u16()? });
        }
        for _ in 0..r.count()? {
            let due = r.u64()?;
            let mut drops = Vec::new();
            for _ in 0..r.count()? {
                drops.push((r.vec3()?, ItemId(r.u16()?), r.u32()?));
            }
            f.falls.push(Fall { due, drops });
        }
        Some(f)
    }
}

/// A save stream of `sim` as it was before save 73: without the felling at the end of the core (none under way).
#[cfg(test)]
pub(crate) fn before_felling(sim: &Sim, stream: &[u8]) -> Vec<u8> {
    assert_eq!(sim.felling, Felling::default(), "a save before 73 had no felling");
    let mut empty = ByteWriter::default();
    sim.felling.write_state(&mut empty);
    let mut core = ByteWriter::default();
    sim.write_state(&mut core);
    let at = stream.windows(core.bytes.len()).position(|s| s == core.bytes).expect("the core's bytes");
    let end = at + core.bytes.len();
    let mut out = stream.to_vec();
    out.drain(end - empty.bytes.len()..end);
    out
}

impl Sim {
    /// A swing by `player` at the tree log `pos`, landing at `at` from the side `from` (0–15, sixteenths of a turn
    /// from +x towards +z: where the player stands, so the tree falls the other way).
    pub(crate) fn chop_tree(&mut self, player: PlayerId, pos: IVec3, at: Vec3, from: u8) {
        let world = &mut self.world;
        let Some(trunk) = trunk(|p| world.block_anywhere_or_generate(p), pos) else { return };
        if pos.y >= trunk.low + LOW {
            return;
        }
        let log = self.world.block_anywhere_or_generate(trunk.foot);
        let Some(core) = self.player(player) else { return };
        let inv = &core.inventory;
        let speed = tools::break_speed(inv.in_hand(inv.tool_hand(log)).item, log) * inv.mining_speed();
        let seed = self.world.generator().seed();
        let at_ = self.chops_at(trunk.foot);
        let struck = match self.felling.chops.get(at_).filter(|c| c.foot == trunk.foot) {
            Some(c) => on_mark(at, mark_of(seed, &trunk, c.marks), trunk.centre),
            None => true,
        };
        let work = (SWING_SECONDS * speed * if struck { 1.0 } else { OFF_MARK } * 1000.0) as u32;
        if self.felling.chops.get(at_).is_none_or(|c| c.foot != trunk.foot) {
            self.felling.chops.insert(at_, Chop { foot: trunk.foot, work: 0, marks: 0 });
        }
        let chop = &mut self.felling.chops[at_];
        chop.work = chop.work.saturating_add(work);
        chop.marks += u16::from(struck);
        let done = chop.work >= work_needed(&trunk, log);
        self.wear_tool(player, log);
        self.events.push(SimEvent::Chopped { player, pos: at, struck, felled: done });
        if done {
            self.felling.chops.remove(at_);
            self.fell(&trunk, from);
        }
    }

    /// Where the chop of the tree on `foot` is, or would go, in `chops` (sorted by foot).
    fn chops_at(&self, foot: IVec3) -> usize {
        let key = |p: IVec3| (p.x, p.y, p.z);
        self.felling.chops.partition_point(|c| key(c.foot) < key(foot))
    }

    /// Brings the tree of `trunk` down away from the side `from`: out of the world now, what it gives later.
    fn fell(&mut self, trunk: &Trunk, from: u8) {
        let world = &mut self.world;
        let taken = tree::take(|p| world.block_anywhere_or_generate(p), trunk);
        // Remeshed with the streaming budget. The leaves its logs held are checked once for the whole tree (a check a log was most of a fall's tick).
        let (mut lo, mut hi) = (trunk.foot, trunk.foot);
        for &(p, old) in &taken.blocks {
            self.world.set_block_anywhere_later(p, block::AIR);
            self.block_changed_but_leaves(p, old);
            if block::is_log(old) {
                lo = IVec3::new(lo.x.min(p.x), lo.y.min(p.y), lo.z.min(p.z));
                hi = IVec3::new(hi.x.max(p.x), hi.y.max(p.y), hi.z.max(p.z));
            }
        }
        self.logs_removed(lo, hi);
        let angle = (from as f32 + 8.0) * std::f32::consts::TAU / 16.0;
        let (dx, dz) = (angle.cos(), angle.sin());
        let foot = Vec3::new(trunk.centre[0] as f64, trunk.foot.y as f64, trunk.centre[1] as f64);
        let w = wood::of_tree(taken.log).unwrap_or(0);
        let mut drops = Vec::new();
        // The trunk's logs along where it lies, a stack a block; limbs as sticks and saplings round its crown.
        let len = taken.height.max(1) as u32;
        let along = |d: f32| (foot.x as f32 + dx * d, foot.z as f32 + dz * d);
        for k in 0..len.min(taken.trunk_logs) {
            let n = taken.trunk_logs / len + u32::from(k < taken.trunk_logs % len);
            drops.push((along(k as f32 + 1.0), wood::item(w, Form::Log), n));
        }
        let crown = along(len as f32 * 0.85 + 1.0);
        let sticks = taken.limb_logs.div_ceil(LIMBS_PER_STICK);
        if sticks > 0 {
            drops.push((crown, STICK, sticks));
        }
        if taken.leaves > 0 {
            let sapling = block::sapling_of(
                taken.blocks.iter().map(|b| b.1).find(|&b| block::is_leaves(b)).unwrap_or(block::LEAVES),
            );
            drops.push((crown, ItemId::block(sapling), 1 + taken.leaves / LEAVES_PER_SAPLING));
        }
        let drops = drops
            .into_iter()
            .filter(|d| d.2 > 0)
            .map(|((x, z), item, n)| {
                (Vec3::new(x as f64, self.ground_under(x, z, trunk.foot.y) as f64 + 0.6, z as f64), item, n)
            })
            .collect();
        self.felling.falls.push(Fall { due: self.tick + FALL_TICKS - 1, drops });
        self.fallen.push(Fallen { foot, angle, blocks: taken.blocks });
    }

    /// The top of the first solid block under (`x`, `z`) from a little above `near` down.
    fn ground_under(&mut self, x: f32, z: f32, near: i32) -> i32 {
        let (x, z) = (x.floor() as i32, z.floor() as i32);
        for y in (near - 16..=near + 8).rev() {
            let b = self.world.block_anywhere_or_generate(IVec3::new(x, y, z));
            if block::def(b).solid && !block::is_leaves(b) {
                return y + 1;
            }
        }
        near
    }

    /// Lands what the trees that have finished falling give.
    pub(super) fn run_felling(&mut self) {
        let tick = self.tick;
        while let Some(i) = self.felling.falls.iter().position(|f| f.due <= tick) {
            for (pos, item, count) in self.felling.falls.remove(i).drops {
                self.events.push(SimEvent::Dropped { pos, vel: Vec3::new(0.0, 1.0, 0.0), item, count });
            }
        }
    }
}

#[cfg(test)]
mod tests;
