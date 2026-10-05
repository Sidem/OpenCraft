//! Version 7 rivers and lakes (Milestone 10), carved into the height map before ponds.
//!
//! - A river network from nodes: one jittered node per `CELL`² cell, its ground height from the unshaped terrain.
//!   Every node's *downstream* is its lowest lower neighbour of the 8 around it, so a chain only ever descends and ends
//!   in the sea or at a local low point. A node is *wet* by a hash weighted by the region's moisture (dry regions have
//!   few rivers, wet ones many); a wet node's river runs to its downstream as a segment, so rivers join where chains
//!   meet and start wherever a wet node has no wet upstream.
//! - Lakes: where a river ends inland (a wet node's downstream is dry, or it has none), and by chance in wet regions.
//! - A segment's water surface is a staircase of pools about `POOL_LENGTH` long, each at an integer level that never
//!   rises downstream and equals the nodes' levels at both ends, so segments agree where they meet. Between pools a
//!   *weir* (a strip of ground at the upper pool's level) keeps still water from spilling over the step. Levels follow
//!   the ground, snapped to the sea when near it.
//! - Shape: the channel is carved to a bed under the pool; banks are raised to the water around it; beyond them the
//!   ground blends to the old height over a ramp that grows with the height difference (gorges through mountains).
//!
//! Invariants: a pure function of the seed and the column (nodes and segments are cached, which never changes
//! results). Nothing within `SPAWN_DRY` of spawn. A column asks the segments reaching its cell (`REACH`); the
//! caches fill lazily. To tune: the constants below. Version 7 only.

use std::cell::RefCell;
use std::rc::Rc;

use rustc_hash::FxHashMap;

use crate::math::{hash2, smoothstep, unit};

use super::{WorldGen, SEA_LEVEL};

/// Blocks between nodes (one per cell of this size) and how far a node keeps from its cell's edge.
const CELL: i32 = 256;
const NODE_MARGIN: i32 = 48;
/// A river's surface at a node lies this far under the ground there.
const NODE_DROP: i32 = 6;
/// Pool length along a river, and a weir's width.
const POOL_LENGTH: f64 = 12.0;
const WEIR_WIDTH: f64 = 1.6;
/// Raised bank beside the water, the longest valley ramp beyond it, and so how far a segment shapes the ground.
const BANK: f64 = 3.0;
const MAX_RAMP: f64 = 70.0;
const REACH: f64 = 110.0;
/// Channel depth under the pool in the middle, for rivers and lakes.
const RIVER_DEPTH: f64 = 3.0;
const LAKE_DEPTH: f64 = 5.0;
/// Lake radius range, and the chance of a standalone lake at a node in wet country (moisture above `WET_LAKE`).
const LAKE_RADIUS: (u32, u32) = (16, 28);
const LAKE_CHANCE: f64 = 0.45;
const WET_LAKE: f64 = 0.12;
/// No river or lake within this many blocks of spawn (their water included).
const SPAWN_DRY: f64 = 100.0;

#[derive(Clone, Copy)]
pub(super) struct Node {
    gx: i32,
    gz: i32,
    x: i32,
    z: i32,
    /// Ground height here, before rivers.
    h: i32,
    /// The river's water surface here (a block y).
    level: i32,
    width: f64,
    wet: bool,
    /// Under or at the sea (rivers end here, never start).
    ocean: bool,
    /// Wet country chose a lake here whatever flows in.
    lakey: bool,
}

impl Node {
    /// Orders nodes by ground height, ties by cell, so "lower" is total.
    fn key(&self) -> (i32, i32, i32) {
        (self.h, self.gx, self.gz)
    }
}

/// A river reach (a node to its downstream) or, with both ends equal, a lake.
pub(super) struct Seg {
    a: (f64, f64),
    b: (f64, f64),
    len: f64,
    wa: f64,
    wb: f64,
    depth: f64,
    /// Water level of each pool from the upstream end; the first and last equal the nodes' levels.
    levels: Vec<i32>,
    pool_len: f64,
}

/// What a segment says about one column.
#[derive(Clone, Copy)]
pub(super) struct Sample {
    /// Distance from the water's edge (negative inside), the water's half-width there, the distance to its axis and
    /// the channel's depth in the middle.
    sd: f64,
    width: f64,
    axis: f64,
    depth: f64,
    /// The pool's water level, and the weir's top when on one (`i32::MIN` if not).
    level: i32,
    weir: i32,
}

type Segs = Rc<Vec<Rc<Seg>>>;

/// The lazily filled caches (all pure functions of the seed).
#[derive(Default)]
pub(super) struct RiverCache {
    nodes: RefCell<FxHashMap<(i32, i32), Node>>,
    segs: RefCell<FxHashMap<(i32, i32), Segs>>,
    cells: RefCell<FxHashMap<(i32, i32), Segs>>,
    last: RefCell<Option<((i32, i32), Segs)>>,
}

/// Levels this close above the sea are the sea (so no river hangs a block over it).
fn snap(level: i32) -> i32 {
    if level < SEA_LEVEL + 2 {
        SEA_LEVEL
    } else {
        level
    }
}

impl Seg {
    fn new(a: &Node, b: Option<&Node>, levels: Vec<i32>) -> Seg {
        let (ax, az) = (a.x as f64, a.z as f64);
        let (bx, bz) = b.map_or((ax, az), |b| (b.x as f64, b.z as f64));
        let len = ((bx - ax).powi(2) + (bz - az).powi(2)).sqrt();
        let pool_len = len / levels.len() as f64;
        let (wa, wb, depth) = match b {
            Some(b) => (a.width, b.width, RIVER_DEPTH),
            None => (a.width, a.width, LAKE_DEPTH),
        };
        Seg { a: (ax, az), b: (bx, bz), len, wa, wb, depth, levels, pool_len }
    }

    /// Whether the segment can shape any column of the rectangle (`REACH` beyond its ends).
    fn reaches(&self, min: (f64, f64), max: (f64, f64)) -> bool {
        self.a.0.min(self.b.0) - REACH < max.0
            && self.a.0.max(self.b.0) + REACH > min.0
            && self.a.1.min(self.b.1) - REACH < max.1
            && self.a.1.max(self.b.1) + REACH > min.1
    }

    fn sample(&self, px: f64, pz: f64) -> Sample {
        let (dx, dz) = (self.b.0 - self.a.0, self.b.1 - self.a.1);
        let t = if self.len > 0.0 {
            (((px - self.a.0) * dx + (pz - self.a.1) * dz) / (self.len * self.len)).clamp(0.0, 1.0)
        } else {
            0.0
        };
        let (cx, cz) = (self.a.0 + dx * t, self.a.1 + dz * t);
        let axis = ((px - cx).powi(2) + (pz - cz).powi(2)).sqrt();
        let width = self.wa + (self.wb - self.wa) * t;
        let s = t * self.len;
        let i = ((s / self.pool_len) as usize).min(self.levels.len() - 1);
        let on_weir = i > 0 && self.levels[i - 1] != self.levels[i] && s - i as f64 * self.pool_len < WEIR_WIDTH;
        let weir = if on_weir { self.levels[i - 1] } else { i32::MIN };
        Sample { sd: axis - width, width, axis, depth: self.depth, level: self.levels[i], weir }
    }
}

impl WorldGen {
    /// Version 7's ground height `h` after the nearest river or lake reshapes it.
    pub(super) fn shape_rivers(&self, x: i32, z: i32, h: i32) -> i32 {
        let Some(s) = self.river_sample(x, z) else { return h };
        if s.sd < 0.0 {
            if s.weir != i32::MIN {
                return s.weir;
            }
            let u = s.axis / s.width;
            let bed = s.level - 1 - (s.depth * (1.0 - u * u)).round() as i32;
            return if s.level <= SEA_LEVEL { bed.min(h) } else { bed };
        }
        let target = s.level + 1;
        let excess = (h - target) as f64;
        if excess <= 0.0 && s.level <= SEA_LEVEL {
            return h;
        }
        let ramp = (8.0 + 0.6 * excess.abs()).min(MAX_RAMP);
        if s.sd >= BANK + ramp {
            return h;
        }
        (target as f64 + excess * smoothstep(BANK, BANK + ramp, s.sd)).round() as i32
    }

    /// The top water block a river or lake holds at (x, z) whose (shaped) ground is `h`, if any.
    pub(super) fn river_water(&self, x: i32, z: i32, h: i32) -> Option<i32> {
        let s = self.river_sample(x, z)?;
        (s.sd < 0.0 && s.weir == i32::MIN && h < s.level).then_some(s.level)
    }

    /// Whether any river or lake shapes the ground at (x, z) (ponds keep off).
    pub(super) fn river_near(&self, x: i32, z: i32) -> bool {
        self.river_sample(x, z).is_some()
    }

    /// The segment whose water is nearest (x, z), as it sees the column.
    fn river_sample(&self, x: i32, z: i32) -> Option<Sample> {
        let segs = self.river_segs_near(x.div_euclid(CELL), z.div_euclid(CELL));
        let (px, pz) = (x as f64, z as f64);
        let mut best: Option<Sample> = None;
        for seg in segs.iter() {
            let s = seg.sample(px, pz);
            if best.is_none_or(|b| s.sd < b.sd) {
                best = Some(s);
            }
        }
        best.filter(|b| b.sd < BANK + MAX_RAMP)
    }

    /// The segments that can reach cell (cx, cz): those of the nodes up to two cells away, filtered (cached).
    fn river_segs_near(&self, cx: i32, cz: i32) -> Segs {
        if let Some((cell, segs)) = &*self.rivers.last.borrow() {
            if *cell == (cx, cz) {
                return segs.clone();
            }
        }
        let cached = self.rivers.cells.borrow().get(&(cx, cz)).cloned();
        let segs = cached.unwrap_or_else(|| {
            let min = ((cx * CELL) as f64, (cz * CELL) as f64);
            let max = (min.0 + CELL as f64, min.1 + CELL as f64);
            let mut out = Vec::new();
            for gz in cz - 2..=cz + 2 {
                for gx in cx - 2..=cx + 2 {
                    out.extend(self.node_segs(gx, gz).iter().filter(|s| s.reaches(min, max)).cloned());
                }
            }
            let segs = Rc::new(out);
            self.rivers.cells.borrow_mut().insert((cx, cz), segs.clone());
            segs
        });
        *self.rivers.last.borrow_mut() = Some(((cx, cz), segs.clone()));
        segs
    }

    /// The river reach and lake of node (gx, gz), if it has them (cached).
    fn node_segs(&self, gx: i32, gz: i32) -> Segs {
        if let Some(s) = self.rivers.segs.borrow().get(&(gx, gz)) {
            return s.clone();
        }
        let n = self.river_node(gx, gz);
        let mut out = Vec::new();
        let down = self.river_down(&n);
        if n.wet {
            if let Some(d) = &down {
                out.push(Rc::new(Seg::new(&n, Some(d), self.river_levels(&n, d))));
            }
        }
        if self.has_lake(&n, down.is_some()) {
            out.push(Rc::new(Seg::new(&Node { width: self.lake_radius(&n), ..n }, None, vec![n.level])));
        }
        out.retain(|s| {
            let (ax, az, bx, bz) = (s.a.0, s.a.1, s.b.0, s.b.1);
            // The nearest point of the segment to spawn.
            let (dx, dz) = (bx - ax, bz - az);
            let t = if s.len > 0.0 { (-(ax * dx + az * dz) / (s.len * s.len)).clamp(0.0, 1.0) } else { 0.0 };
            let d = ((ax + dx * t).powi(2) + (az + dz * t).powi(2)).sqrt();
            d > SPAWN_DRY + s.wa.max(s.wb)
        });
        let out = Rc::new(out);
        self.rivers.segs.borrow_mut().insert((gx, gz), out.clone());
        out
    }

    /// Node (gx, gz): its place, ground, level and whether it holds water (cached).
    fn river_node(&self, gx: i32, gz: i32) -> Node {
        if let Some(n) = self.rivers.nodes.borrow().get(&(gx, gz)) {
            return *n;
        }
        let j = hash2(self.seed ^ 0x2155, gx, gz);
        let span = (CELL - 2 * NODE_MARGIN) as u32;
        let x = gx * CELL + NODE_MARGIN + (j % span) as i32;
        let z = gz * CELL + NODE_MARGIN + ((j >> 16) % span) as i32;
        let h = self.base_height(x, z);
        let m = self.moisture.fbm2(x as f64 / 900.0, z as f64 / 900.0, 3);
        let ocean = h <= SEA_LEVEL + 2;
        let wet = !ocean && unit(hash2(self.seed ^ 0x2156, gx, gz)) < (0.55 + 1.3 * m).clamp(0.1, 0.9);
        let lakey = !ocean && m > WET_LAKE && unit(hash2(self.seed ^ 0x2157, gx, gz)) < LAKE_CHANCE;
        // Wider towards the sea.
        let low = 1.0 - ((h - SEA_LEVEL) as f64 / 60.0).clamp(0.0, 1.0);
        let width = 2.5 + 4.5 * low + 1.5 * unit(hash2(self.seed ^ 0x2158, gx, gz));
        let n = Node { gx, gz, x, z, h, level: snap(h - NODE_DROP), width, wet, ocean, lakey };
        self.rivers.nodes.borrow_mut().insert((gx, gz), n);
        n
    }

    /// The lowest of the 8 neighbours that is lower than `n`.
    fn river_down(&self, n: &Node) -> Option<Node> {
        let mut best: Option<Node> = None;
        for dz in -1..=1 {
            for dx in -1..=1 {
                let m = self.river_node(n.gx + dx, n.gz + dz);
                if (dx, dz) != (0, 0) && m.key() < n.key() && best.is_none_or(|b| m.key() < b.key()) {
                    best = Some(m);
                }
            }
        }
        best
    }

    /// Whether node `n` holds a lake: a river ends here, or wet country chose one.
    fn has_lake(&self, n: &Node, has_down: bool) -> bool {
        if n.ocean {
            return false;
        }
        let dead_end = !n.wet || !has_down;
        let fed = dead_end
            && (-1..=1).any(|dz| {
                (-1..=1).any(|dx| {
                    let m = self.river_node(n.gx + dx, n.gz + dz);
                    (dx, dz) != (0, 0) && m.wet && self.river_down(&m).is_some_and(|d| (d.gx, d.gz) == (n.gx, n.gz))
                })
            });
        fed || n.lakey
    }

    fn lake_radius(&self, n: &Node) -> f64 {
        (LAKE_RADIUS.0 + hash2(self.seed ^ 0x2159, n.gx, n.gz) % (LAKE_RADIUS.1 - LAKE_RADIUS.0 + 1)) as f64
    }

    /// The pool levels of the reach from `a` to `b`: a line between their levels that never rises and dips with the
    /// ground at each pool's middle, the ends exactly the nodes' levels.
    fn river_levels(&self, a: &Node, b: &Node) -> Vec<i32> {
        let len = (((b.x - a.x) as f64).powi(2) + ((b.z - a.z) as f64).powi(2)).sqrt();
        let pools = ((len / POOL_LENGTH).round() as usize).max(2);
        let mut levels = vec![a.level];
        for i in 1..pools - 1 {
            let t = (i as f64 + 0.5) / pools as f64;
            let (x, z) = (a.x + ((b.x - a.x) as f64 * t) as i32, a.z + ((b.z - a.z) as f64 * t) as i32);
            let line = (a.level as f64 + (b.level - a.level) as f64 * t).round() as i32;
            let level = snap(line.min(self.base_height(x, z) - 2)).clamp(b.level, levels[i - 1]);
            levels.push(level);
        }
        levels.push(b.level);
        levels
    }
}

#[cfg(test)]
mod tests;
