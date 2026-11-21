//! Burning grown trees (`Blaze`): one record per tree, keyed by `felling::Trunk::foot`.
//!
//! Two stages: `Crown` burns for `CROWN_TICKS` then removes this tree's leaves (leaving neighbouring trees' supported
//! leaves intact); `Trunk` smoulders for `TRUNK_TICKS` then removes its logs and drops `charcoal_for(logs)` at its
//! foot. Every `SPREAD_STEP`, if `heat > 0`, samples `SPREAD_SAMPLES` points via `hash3(foot, tick)` (never `Sim.rng`)
//! and lights adjacent trees with `heat - 1`. Water at the foot extinguishes a blaze; water between trees blocks spread.

use std::collections::HashSet;

use super::{
    CROWN_TICKS, FIRE_STEP, MAX_BLAZES, MAX_TREE_CLEARS, SPREAD_RADIUS, SPREAD_SAMPLES, SPREAD_STEP, TRUNK_TICKS,
};
use crate::block::{is_leaves, is_log, is_sapling, is_tree_wood, AIR, LIQUID};
use crate::bytes::{ByteReader, ByteWriter};
use crate::item::CHARCOAL;
use crate::math::{hash3, IVec3, Vec3};
use crate::sim::{trunk, Sim, SimEvent, LEAF_SUPPORT_STEPS};
use crate::worldgen::REACH as CROWN_REACH;

/// How far horizontally a tree's branches can reach from its foot (`felling/tree.rs`).
const ACROSS: i32 = CROWN_REACH + 2;
const HIGH: i32 = 60;
const FACES: [IVec3; 6] = [
    IVec3::new(1, 0, 0),
    IVec3::new(-1, 0, 0),
    IVec3::new(0, 1, 0),
    IVec3::new(0, -1, 0),
    IVec3::new(0, 0, 1),
    IVec3::new(0, 0, -1),
];

/// Stage of a burning tree.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Stage {
    Crown = 0,
    Trunk = 1,
}

/// One burning grown tree.
#[derive(Clone, Debug, PartialEq)]
pub struct Blaze {
    pub foot: IVec3,
    pub centre: [f32; 2],
    /// Measured top height of this tree above `foot.y` (logs + crown).
    pub height: u16,
    /// Lowest crown/leaf height above `foot.y` (so flames hug the real canopy).
    pub crown_y: u8,
    /// Horizontal crown radius in blocks (1..=8), matching narrow conifers vs broad oaks/cherries.
    pub radius: u8,
    pub logs: u32,
    pub heat: u8,
    pub stage: Stage,
    pub due: u64,
    pub stage_at: u64,
    pub spread_at: u64,
}

/// Charcoal pieces left at a tree's foot when `logs` logs burn down (at least 1).
pub fn charcoal_for(logs: u32) -> u32 {
    (logs / 2).max(1)
}

impl Blaze {
    pub(super) fn write_state(&self, w: &mut ByteWriter) {
        w.ivec3(self.foot);
        w.f32(self.centre[0]);
        w.f32(self.centre[1]);
        w.u16(self.height);
        w.u8(self.crown_y);
        w.u8(self.radius);
        w.u32(self.logs);
        w.u8(self.heat);
        w.u8(self.stage as u8);
        w.u64(self.due);
        w.u64(self.stage_at);
        w.u64(self.spread_at);
    }

    pub(super) fn read_state(r: &mut ByteReader) -> Option<Blaze> {
        let foot = r.ivec3()?;
        let (cx, cz) = (r.f32()?, r.f32()?);
        if !cx.is_finite() || !cz.is_finite() {
            return None;
        }
        let height = r.u16()?;
        let crown_y = r.u8()?;
        let radius = r.u8()?;
        let logs = r.u32()?;
        let heat = r.u8()?;
        let stage = match r.u8()? {
            0 => Stage::Crown,
            1 => Stage::Trunk,
            _ => return None,
        };
        Some(Blaze {
            foot,
            centre: [cx, cz],
            height,
            crown_y,
            radius,
            logs,
            heat,
            stage,
            due: r.u64()?,
            stage_at: r.u64()?,
            spread_at: r.u64()?,
        })
    }
}

impl Sim {
    /// Sets the grown tree containing `pos` ablaze with `heat`, unless already burning, capped, or wet.
    pub(crate) fn ignite_tree(&mut self, pos: IVec3, heat: u8) -> bool {
        if self.fire.blazes.len() + usize::from(self.fire.stepping_blaze) >= MAX_BLAZES
            || !is_tree_wood(self.block(pos))
        {
            return false;
        }
        let Some(t) = trunk(|p| self.world.block_anywhere_or_generate(p), pos) else { return false };
        if self.fire.has_blaze(t.foot) || self.water_near_foot(t.foot) {
            return false;
        }
        let seed = self.world.generator().seed();
        let phase = hash3(seed ^ 0xF1AE, t.foot.x, t.foot.y, t.foot.z) as u64 % FIRE_STEP;
        let (height, crown_y, radius) = self.measure_tree(t.foot, t.centre, t.logs, t.width);
        let b = Blaze {
            foot: t.foot,
            centre: t.centre,
            height,
            crown_y,
            radius,
            logs: t.logs.max(1),
            heat,
            stage: Stage::Crown,
            due: self.tick + 1 + phase,
            stage_at: self.tick + CROWN_TICKS + phase * 3,
            spread_at: self.tick + SPREAD_STEP + phase,
        };
        self.fire.insert_blaze(b);
        self.events.push(SimEvent::FireIgnited { pos: t.foot });
        true
    }

    /// Probes the tree's real height, crown base and horizontal crown radius in a few dozen block reads so fire
    /// emitters hug narrow conifers, broad oaks/cherries and short birches closely.
    fn measure_tree(&mut self, foot: IVec3, centre: [f32; 2], logs: u32, width: usize) -> (u16, u8, u8) {
        let cx = centre[0].floor() as i32;
        let cz = centre[1].floor() as i32;
        let mut top_wood = 0i32;
        let mut top_leaf = 0i32;
        let mut min_leaf = i32::MAX;
        let mut max_r = 1i32;
        for dy in 1..=36 {
            let y = foot.y + dy;
            let mut found = false;
            for (dx, dz) in [(0, 0), (1, 0), (-1, 0), (0, 1), (0, -1)] {
                let b = self.block(IVec3::new(cx + dx, y, cz + dz));
                if is_tree_wood(b) {
                    top_wood = top_wood.max(dy);
                    found = true;
                } else if is_leaves(b) {
                    top_leaf = top_leaf.max(dy);
                    min_leaf = min_leaf.min(dy);
                    found = true;
                }
            }
            if !found && dy > top_wood.max(top_leaf) + 4 {
                break;
            }
        }
        let est_h = (logs / width.max(1) as u32).clamp(4, 36) as i32;
        let height = top_leaf.max(top_wood).max(est_h).clamp(4, 48);
        let crown_y = if min_leaf < i32::MAX { min_leaf.clamp(1, height - 1) } else { (height / 2).max(1) };
        let mid_y = foot.y + (crown_y + height) / 2;
        for (dx, dz) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
            for r in 1..=8 {
                let b = self.block(IVec3::new(cx + dx * r, mid_y, cz + dz * r));
                let b_lo = self.block(IVec3::new(cx + dx * r, foot.y + crown_y + 1, cz + dz * r));
                if is_leaves(b) || is_tree_wood(b) || is_leaves(b_lo) || is_tree_wood(b_lo) {
                    max_r = max_r.max(r);
                } else {
                    break;
                }
            }
        }
        (height as u16, crown_y as u8, max_r.clamp(1, 8) as u8)
    }

    /// Steps one due blaze. Returns true if a heavy tree clear was deferred to the next tick due to `MAX_TREE_CLEARS`.
    pub(super) fn step_blaze(&mut self, mut b: Blaze, clears: &mut usize) -> bool {
        // Extinguish quietly if the tree was chopped or water reached its base.
        if !is_tree_wood(self.block(b.foot)) || self.water_near_foot(b.foot) {
            return false;
        }
        if self.tick >= b.stage_at {
            if *clears >= MAX_TREE_CLEARS {
                b.due = self.tick + 1;
                self.fire.insert_blaze(b);
                return true;
            }
            *clears += 1;
            match b.stage {
                Stage::Crown => {
                    let total_logs = self.burn_crown(b.foot, b.centre);
                    b.logs = b.logs.max(total_logs);
                    b.stage = Stage::Trunk;
                    b.stage_at = self.tick + TRUNK_TICKS;
                    b.due = self.tick + FIRE_STEP;
                    self.fire.insert_blaze(b);
                }
                Stage::Trunk => {
                    let removed = self.burn_trunk(b.foot, b.centre);
                    let count = charcoal_for(b.logs.max(removed));
                    let drop_pos = Vec3::new(b.centre[0] as f64, b.foot.y as f64 + 0.5, b.centre[1] as f64);
                    self.events.push(SimEvent::Dropped {
                        pos: drop_pos,
                        vel: Vec3::new(0.0, 1.2, 0.0),
                        item: CHARCOAL,
                        count,
                    });
                    self.events.push(SimEvent::TreeBurned { pos: b.foot });
                }
            }
            return false;
        }
        if b.heat > 0 && self.tick >= b.spread_at {
            b.spread_at = self.tick + SPREAD_STEP;
            self.spread_from_blaze(&b);
        }
        b.due = self.tick + FIRE_STEP;
        self.fire.insert_blaze(b);
        false
    }

    /// Samples `SPREAD_SAMPLES` points within `SPREAD_RADIUS` of `b` using `hash3` (never `Sim.rng`).
    fn spread_from_blaze(&mut self, b: &Blaze) {
        let seed = self.world.generator().seed() ^ (self.tick as u32).wrapping_mul(0x9E37_79B9);
        let span = (2 * SPREAD_RADIUS + 1) as u32;
        let mid_y = b.foot.y + (b.height as i32 / 2).max(2);
        for k in 0..SPREAD_SAMPLES {
            let h = hash3(seed, b.foot.x.wrapping_add(k as i32 * 131), b.foot.y, b.foot.z.wrapping_sub(k as i32 * 257));
            let dx = (h % span) as i32 - SPREAD_RADIUS;
            let dz = ((h >> 10) % span) as i32 - SPREAD_RADIUS;
            if dx * dx + dz * dz < 4 || dx * dx + dz * dz > SPREAD_RADIUS * SPREAD_RADIUS {
                continue;
            }
            let target_x = b.foot.x + dx;
            let target_z = b.foot.z + dz;
            if self.water_between(b.foot, IVec3::new(target_x, b.foot.y, target_z)) {
                continue;
            }
            for y in [b.foot.y + 1, b.foot.y, b.foot.y + 2, mid_y] {
                let p = IVec3::new(target_x, y, target_z);
                let blk = self.block(p);
                if is_tree_wood(blk) {
                    self.ignite_tree(p, b.heat - 1);
                    break;
                }
                if is_sapling(blk) {
                    self.burn_sapling(p);
                    break;
                }
            }
        }
    }

    /// Stage 1: removes leaves belonging only to this tree (leaves supported by another tree's log stay).
    /// Returns the number of logs in this tree.
    fn burn_crown(&mut self, foot: IVec3, centre: [f32; 2]) -> u32 {
        let logs = self.collect_tree_logs(foot, centre);
        let log_set: HashSet<IVec3> = logs.iter().copied().collect();
        // BFS through leaves up to `LEAF_SUPPORT_STEPS + 2` from our logs so outer canopy corners don't linger floating.
        let mut leaves: Vec<IVec3> = Vec::new();
        let mut leaf_set: HashSet<IVec3> = HashSet::new();
        let mut other_logs: Vec<IVec3> = Vec::new();
        let mut frontier: Vec<IVec3> = logs.clone();
        for _ in 0..(LEAF_SUPPORT_STEPS + 2) {
            let mut next = Vec::new();
            for p in frontier {
                for d in FACES {
                    let n = p + d;
                    if log_set.contains(&n) || leaf_set.contains(&n) {
                        continue;
                    }
                    let blk = self.block(n);
                    if is_leaves(blk) {
                        leaf_set.insert(n);
                        leaves.push(n);
                        next.push(n);
                    } else if is_log(blk) {
                        other_logs.push(n);
                    }
                }
            }
            frontier = next;
        }
        // Check a few steps beyond the leaf frontier so leaves held by a neighbouring tree's crown stay intact.
        let mut ext_seen: HashSet<IVec3> = leaf_set.clone();
        let mut ext_frontier = frontier;
        for _ in 0..LEAF_SUPPORT_STEPS {
            let mut next = Vec::new();
            for p in ext_frontier {
                for d in FACES {
                    let n = p + d;
                    if log_set.contains(&n) || ext_seen.contains(&n) {
                        continue;
                    }
                    let blk = self.block(n);
                    if is_log(blk) {
                        other_logs.push(n);
                    } else if is_leaves(blk) {
                        ext_seen.insert(n);
                        next.push(n);
                    }
                }
            }
            ext_frontier = next;
        }
        // Mark any of our leaves supported by `other_logs` within `LEAF_SUPPORT_STEPS`.
        let mut held: HashSet<IVec3> = HashSet::new();
        let mut hfront = other_logs;
        for _ in 0..LEAF_SUPPORT_STEPS {
            let mut next = Vec::new();
            for p in hfront {
                for d in FACES {
                    let n = p + d;
                    if held.contains(&n) {
                        continue;
                    }
                    if leaf_set.contains(&n) || ext_seen.contains(&n) {
                        held.insert(n);
                        next.push(n);
                    }
                }
            }
            hfront = next;
        }
        for p in leaves {
            if !held.contains(&p) {
                let old = self.block(p);
                if self.world.set_block_anywhere_later(p, AIR) {
                    self.block_changed_but_leaves(p, old);
                }
            }
        }
        logs.len() as u32
    }

    /// Stage 2: removes all logs belonging to this tree via `set_block_anywhere_later`. Returns count removed.
    fn burn_trunk(&mut self, foot: IVec3, centre: [f32; 2]) -> u32 {
        let logs = self.collect_tree_logs(foot, centre);
        let mut removed = 0u32;
        for p in logs {
            let old = self.block(p);
            if is_tree_wood(old) && self.world.set_block_anywhere_later(p, AIR) {
                self.block_changed_but_leaves(p, old);
                removed += 1;
            }
        }
        removed
    }

    /// Collects the connected `is_tree_wood` cells of the tree rooted at `foot` without scanning empty air.
    /// Stops at any log that belongs to another trunk standing nearer it than `centre`.
    fn collect_tree_logs(&mut self, foot: IVec3, centre: [f32; 2]) -> Vec<IVec3> {
        let mut logs = vec![foot];
        let mut seen: HashSet<IVec3> = HashSet::new();
        seen.insert(foot);
        let mut other_trunks: Vec<[f32; 2]> = Vec::new();
        let d2 = |c: [f32; 2], p: IVec3| (p.x as f32 + 0.5 - c[0]).powi(2) + (p.z as f32 + 0.5 - c[1]).powi(2);

        let mut i = 0;
        while i < logs.len() && logs.len() < 512 {
            let p = logs[i];
            i += 1;
            for k in 0..27 {
                let n = p + IVec3::new(k % 3 - 1, k / 3 % 3 - 1, k / 9 - 1);
                if seen.contains(&n) {
                    continue;
                }
                if (n.x - foot.x).abs() > ACROSS || (n.z - foot.z).abs() > ACROSS || n.y < foot.y || n.y > foot.y + HIGH
                {
                    continue;
                }
                if !is_tree_wood(self.block(n)) {
                    continue;
                }
                // Detect a neighbouring trunk touching at the ground (more than 2 blocks away from our centre).
                if n.y <= foot.y + 3 && d2(centre, n) > 5.0 {
                    let below = self.block(n - IVec3::new(0, 1, 0));
                    if !is_tree_wood(below) && !is_leaves(below) && below != AIR {
                        other_trunks.push([n.x as f32 + 0.5, n.z as f32 + 0.5]);
                        seen.insert(n);
                        continue;
                    }
                }
                if other_trunks.iter().any(|&o| d2(o, n) < d2(centre, n)) {
                    seen.insert(n);
                    continue;
                }
                seen.insert(n);
                logs.push(n);
            }
        }
        if !other_trunks.is_empty() {
            logs.retain(|&p| other_trunks.iter().all(|&o| d2(centre, p) <= d2(o, p)));
        }
        logs
    }

    /// True if water (`LIQUID`) touches `foot` or its immediate neighbours.
    fn water_near_foot(&mut self, foot: IVec3) -> bool {
        let mut check = |p: IVec3| LIQUID[self.block(p) as usize];
        if check(foot) || check(foot + IVec3::new(0, 1, 0)) {
            return true;
        }
        for d in FACES {
            if check(foot + d) {
                return true;
            }
        }
        false
    }

    /// True if water sits at the surface midpoint between `a` and `b` (a stream or pond stops spread).
    fn water_between(&mut self, a: IVec3, b: IVec3) -> bool {
        let mid = IVec3::new((a.x + b.x) / 2, (a.y + b.y) / 2, (a.z + b.z) / 2);
        for dy in -1..=1 {
            if LIQUID[self.block(mid + IVec3::new(0, dy, 0)) as usize] {
                return true;
            }
        }
        false
    }
}
