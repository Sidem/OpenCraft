//! What a log is part of, read from the blocks around it: a node of a tree's skeleton (one log, or a square of 2×2 or
//! 3×3 upright logs drawn as one thick trunk), how thick it is there, and which nodes it joins.
//! Invariants: every number here is read from the grid relative to the node's cells in a fixed order, so the two
//! chunks drawing the two halves of a join agree. Two nodes join when a cell of one touches a cell of the other on a
//! face, or on an edge or corner with no log between them (so a staircase branch is one line, not a zigzag).
//! A chunk works each log's node out once (`Nodes`): its neighbours all ask for it.

use rustc_hash::FxHashMap;

use super::super::tree_grid::TreeGrid;
use crate::block::{
    is_leaves, is_log, BlockId, ACACIA_LOG, BAMBOO, BAOBAB_LOG, BIRCH_LOG, CHERRY_LOG, EUCALYPTUS_LOG, PALM_LOG,
    PINE_LOG, REDWOOD_LOG, SNAG_LOG,
};

/// A node's radius per log in a square trunk: 2×2 is this much, 3×3 half again.
const SQUARE_RADIUS: f32 = 0.47;
/// The load (logs above and around, `thickness`) at which a log is as thick as its species' trunk.
const LOAD_FULL: f32 = 5.0;

#[derive(Clone, Copy)]
pub struct Node {
    /// The lowest corner cell (chunk-local) and the square's width (1 for a single log).
    pub corner: [i32; 3],
    pub width: i32,
    /// The middle of its layer, chunk-local blocks.
    pub centre: [f32; 3],
    pub radius: f32,
    pub block: BlockId,
}

impl Node {
    pub fn same(&self, o: &Node) -> bool {
        self.corner == o.corner && self.width == o.width
    }

    fn holds(&self, c: [i32; 3]) -> bool {
        c[1] == self.corner[1]
            && (self.corner[0]..self.corner[0] + self.width).contains(&c[0])
            && (self.corner[2]..self.corner[2] + self.width).contains(&c[2])
    }

    /// Its cells (one layer).
    pub fn cells(&self) -> impl Iterator<Item = [i32; 3]> + '_ {
        let w = self.width;
        (0..w * w).map(move |i| [self.corner[0] + i % w, self.corner[1], self.corner[2] + i / w])
    }
}

/// A species' thinnest twig and its trunk's radius, in blocks.
pub fn radii(b: BlockId) -> (f32, f32) {
    match b {
        BIRCH_LOG => (0.09, 0.3),
        SNAG_LOG => (0.12, 0.35),
        BAMBOO => (0.13, 0.13),
        PALM_LOG => (0.22, 0.34),
        EUCALYPTUS_LOG => (0.09, 0.36),
        CHERRY_LOG | ACACIA_LOG => (0.09, 0.38),
        PINE_LOG => (0.08, 0.36),
        REDWOOD_LOG | BAOBAB_LOG => (0.1, 0.42),
        _ => (0.1, 0.4),
    }
}

/// The nodes of one chunk's drawing by log cell, each worked out once.
#[derive(Default)]
pub struct Nodes(FxHashMap<[i32; 3], Node>);

impl Nodes {
    /// The node the log at `c` belongs to.
    pub fn of(&mut self, g: &TreeGrid, c: [i32; 3]) -> Node {
        *self.0.entry(c).or_insert_with(|| node_of(g, c))
    }
}

/// The node the log at `c` belongs to.
fn node_of(g: &TreeGrid, c: [i32; 3]) -> Node {
    let block = g.at(c[0], c[1], c[2]);
    if let Some((corner, width)) = square(g, c) {
        let half = width as f32 / 2.0;
        return Node {
            corner,
            width,
            centre: [corner[0] as f32 + half, c[1] as f32 + 0.5, corner[2] as f32 + half],
            radius: SQUARE_RADIUS * width as f32,
            block,
        };
    }
    let centre = c.map(|v| v as f32 + 0.5);
    Node { corner: c, width: 1, centre, radius: thickness(g, c, block), block }
}

/// The nodes `n` joins, each once, in a fixed order.
pub fn joins(g: &TreeGrid, n: &Node, nodes: &mut Nodes, out: &mut Vec<Node>) {
    out.clear();
    for a in n.cells() {
        for dy in -1..=1 {
            for dz in -1..=1 {
                for dx in -1..=1 {
                    let d = [dx, dy, dz];
                    let b = [a[0] + dx, a[1] + dy, a[2] + dz];
                    if d == [0; 3] || n.holds(b) || !log(g, b) || !direct(g, a, d) {
                        continue;
                    }
                    let other = nodes.of(g, b);
                    if !other.same(n) && !out.iter().any(|o| o.same(&other)) {
                        out.push(other);
                    }
                }
            }
        }
    }
}

fn log(g: &TreeGrid, c: [i32; 3]) -> bool {
    is_log(g.at(c[0], c[1], c[2]))
}

/// Whether `a` and `a + d` touch with no log on a shorter path between them (always for a face).
fn direct(g: &TreeGrid, a: [i32; 3], d: [i32; 3]) -> bool {
    // The cells between: `d` with some (not all, not none) of its steps taken.
    for mask in 1..7u32 {
        let step: [i32; 3] = std::array::from_fn(|k| if mask >> k & 1 == 1 { d[k] } else { 0 });
        if step == [0; 3] || step == d {
            continue;
        }
        if log(g, [a[0] + step[0], a[1] + step[1], a[2] + step[2]]) {
            return false;
        }
    }
    true
}

/// The square of upright logs `c` is part of (3×3 before 2×2), by its lowest corner, if any.
fn square(g: &TreeGrid, c: [i32; 3]) -> Option<([i32; 3], i32)> {
    for w in [3, 2] {
        for oz in (0..w).rev() {
            for ox in (0..w).rev() {
                let corner = [c[0] - ox, c[1], c[2] - oz];
                let all = (0..w * w).all(|i| log(g, [corner[0] + i % w, c[1], corner[2] + i / w]));
                let upright = (0..w * w).any(|i| {
                    let (x, z) = (corner[0] + i % w, corner[2] + i / w);
                    log(g, [x, c[1] + 1, z]) || log(g, [x, c[1] - 1, z])
                });
                if all && upright {
                    return Some((corner, w));
                }
            }
        }
    }
    None
}

/// A single log's radius: its species' twig, thickening towards its trunk's with the logs above and around it (the
/// wood it carries, as a real tree's pipes add up), so a trunk tapers to its top. A limb (a log with none straight
/// above or below it) tapers from where it leaves its trunk to a twig `LIMB_REACH` blocks out. Wood with no leaves
/// near (a bare trunk low down, a dead tree, logs someone built with) keeps its full thickness.
fn thickness(g: &TreeGrid, c: [i32; 3], b: BlockId) -> f32 {
    let (tip, base) = radii(b);
    let (mut load, mut leafy) = (0.0, false);
    for dy in 0..=4 {
        for dz in -2..=2i32 {
            for dx in -2..=2i32 {
                let at = g.at(c[0] + dx, c[1] + dy, c[2] + dz);
                if is_log(at) {
                    load += 1.0 / (1 + dx.abs() + dz.abs()) as f32;
                }
                leafy |= is_leaves(at);
            }
        }
    }
    if !leafy {
        return base;
    }
    if !is_log(g.at(c[0], c[1] + 1, c[2])) && !is_log(g.at(c[0], c[1] - 1, c[2])) {
        return limb(g, c, tip, base);
    }
    tip + (base - tip) * (load / LOAD_FULL).min(1.0)
}

/// How far out a limb thins to a twig, and its thickness where it leaves the trunk as a share of the trunk's.
const LIMB_REACH: f32 = 4.0;
const LIMB_SHARE: f32 = 0.55;

/// A limb log's radius, from how far it is (across) from the nearest upright log: a trunk, or a limb's rising part.
/// Rings outward, so a limb by its trunk stops at the first.
fn limb(g: &TreeGrid, c: [i32; 3], tip: f32, base: f32) -> f32 {
    let upright =
        |x: i32, y: i32, z: i32| is_log(g.at(x, y, z)) && (is_log(g.at(x, y + 1, z)) || is_log(g.at(x, y - 1, z)));
    for k in 1..=LIMB_REACH as i32 {
        for dy in -2..=2 {
            for dz in -k..=k {
                for dx in -k..=k {
                    if (dx.abs() == k || dz.abs() == k) && upright(c[0] + dx, c[1] + dy, c[2] + dz) {
                        let t = (k - 1) as f32 / LIMB_REACH;
                        return tip + (base * LIMB_SHARE - tip) * (1.0 - t);
                    }
                }
            }
        }
    }
    tip
}
