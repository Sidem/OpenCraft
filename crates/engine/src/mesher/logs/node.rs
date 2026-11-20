//! What a log is part of, read from the blocks around it: a node of a tree's skeleton (one log, or a square of 2×2 or
//! 3×3 upright logs drawn as one thick trunk), how thick it is there, and which nodes it joins.
//! Invariants: every number here is read from the grid relative to the node's cells in a fixed order, so the two
//! chunks drawing the two halves of a join agree. Two nodes join when a cell of one touches a cell of the other on a
//! face, or on an edge or corner with no log between them (so a staircase branch is one line, not a zigzag).
//! A chunk works each log's node out once (`Nodes`): its neighbours all ask for it. A single log in a bending run sits
//! a little off its cell (`bend`), read from its neighbours only, so a leaning stem curves.

use rustc_hash::FxHashMap;

use super::super::tree_grid::TreeGrid;
use crate::block::{
    is_leaves, is_log, BlockId, ACACIA_LOG, BAMBOO, BAOBAB_LOG, BIRCH_LOG, CHERRY_LOG, EUCALYPTUS_LOG, OPAQUE,
    PALM_LOG, PINE_LOG, REDWOOD_LOG, SNAG_LOG,
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
    /// Part of a living tree (leaves nearby): free ends taper cleanly instead of showing flat cut end grain.
    pub leafy: bool,
    /// Part of a vertical trunk column (not a lateral limb or stepping branch elbow).
    pub trunk: bool,
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
            leafy: is_leafy(g, corner),
            trunk: true,
        };
    }
    let centre = std::array::from_fn(|k| c[k] as f32 + 0.5 + bend(g, c)[k]);
    let (radius, leafy, trunk) = thickness(g, c, block);
    Node { corner: c, width: 1, centre, radius, block, leafy, trunk }
}

/// How far a single log in a run of single logs (joining exactly two) sits off its cell's middle: `SMOOTH` of the way
/// to the middle of its two neighbours, so a stem stepping over a block curves instead of turning at an elbow. A
/// straight run, level or slanting, stays on its cells.
fn bend(g: &TreeGrid, c: [i32; 3]) -> [f32; 3] {
    let mut sum = [0i32; 3];
    let mut n = 0;
    for i in 0..27 {
        let d = [i % 3 - 1, i / 3 % 3 - 1, i / 9 - 1];
        let b = [c[0] + d[0], c[1] + d[1], c[2] + d[2]];
        if d != [0; 3] && log(g, b) && linked(g, c, d) {
            if n == 2 || square(g, b).is_some() {
                return [0.0; 3];
            }
            sum = std::array::from_fn(|k| sum[k] + d[k]);
            n += 1;
        }
    }
    if n == 2 {
        sum.map(|v| v as f32 * 0.5 * SMOOTH)
    } else {
        [0.0; 3]
    }
}

/// How far a log in a bending run moves towards its neighbours' middle.
const SMOOTH: f32 = 0.5;

/// The nodes `n` joins, each once, in a fixed order.
pub fn joins(g: &TreeGrid, n: &Node, nodes: &mut Nodes, out: &mut Vec<Node>) {
    out.clear();
    for a in n.cells() {
        for dy in -1..=1 {
            for dz in -1..=1 {
                for dx in -1..=1 {
                    let d = [dx, dy, dz];
                    let b = [a[0] + dx, a[1] + dy, a[2] + dz];
                    if d == [0; 3] || n.holds(b) || !log(g, b) || !linked(g, a, d) {
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

/// Whether a non-trunk cell `c` has any direct non-trunk log neighbour besides `skip` (i.e. continues outward as its
/// own limb rather than being only the rising root step of `skip`).
fn has_outward_branch(g: &TreeGrid, c: [i32; 3], skip: [i32; 3]) -> bool {
    for dy in -1..=1 {
        for dz in -1..=1 {
            for dx in -1..=1 {
                let d = [dx, dy, dz];
                let o = [c[0] + dx, c[1] + dy, c[2] + dz];
                if d != [0; 3] && o != skip && log(g, o) && !is_trunk_col(g, o) && direct(g, c, d) {
                    return true;
                }
            }
        }
    }
    false
}

/// Whether `a` and `a + d` form a skeleton edge: direct neighbours, minus ladder rungs and D-handles between branches.
/// Symmetric (`linked(g, a, d) == linked(g, a + d, -d)`), so neighbouring chunks always agree across their border.
fn linked(g: &TreeGrid, a: [i32; 3], d: [i32; 3]) -> bool {
    if !direct(g, a, d) {
        return false;
    }
    let b = [a[0] + d[0], a[1] + d[1], a[2] + d[2]];
    let (ta, tb) = (is_trunk_col(g, a), is_trunk_col(g, b));
    // Rising limb beside a trunk: if `b` sits above a non-trunk step `b_lo` that has no outward continuation of its own
    // (just rising from `t_lo` to `b`), break the upper trunk shortcut `(t, b)` so the limb sweeps smoothly up from
    // `t_lo -> b_lo -> b` instead of leaving `b_lo` as a horizontal dead-end spike.
    if d[1] == 0 && (ta != tb) {
        let (t, br) = if ta { (a, b) } else { (b, a) };
        let (t_lo, b_lo) = ([t[0], t[1] - 1, t[2]], [br[0], br[1] - 1, br[2]]);
        if is_trunk_col(g, t_lo) && log(g, b_lo) && !is_trunk_col(g, b_lo) && !has_outward_branch(g, b_lo, br) {
            return false;
        }
    }
    // Break vertical rungs between parallel branches or beside a trunk (D-handles), except where `lo` is the rising
    // root step of `hi` (handled above by breaking the upper trunk shortcut instead).
    if d[0] == 0 && d[2] == 0 && d[1].abs() == 1 && !ta && !tb {
        let (lo, hi) = if a[1] < b[1] { (a, b) } else { (b, a) };
        let lo_has_out = has_outward_branch(g, lo, hi);
        let mut parallel = false;
        for (dx, dz) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
            let (lo_n, hi_n) = ([lo[0] + dx, lo[1], lo[2] + dz], [hi[0] + dx, hi[1], hi[2] + dz]);
            if log(g, lo_n) && log(g, hi_n) {
                if is_trunk_col(g, lo_n) && is_trunk_col(g, hi_n) && !lo_has_out {
                    return true; // keep `(lo, hi)` since `(hi_n, hi)` was dropped above
                }
                parallel = true;
            }
        }
        if parallel {
            return false;
        }
    }
    // Break exactly one outer cross-rung of any horizontal 2×2 branch square in living crowns so touching limbs stay
    // separate without ever disconnecting a corner node.
    if d[1] == 0 && d[0].abs() + d[2].abs() == 1 && !ta && !tb && is_leafy(g, a) && is_leafy(g, b) {
        for s in [-1, 1] {
            let (px, pz) = (-d[2] * s, d[0] * s);
            let (ap, bp) = ([a[0] + px, a[1], a[2] + pz], [b[0] + px, b[1], b[2] + pz]);
            if log(g, ap) && log(g, bp) && is_worst_square_edge(g, a, b, ap, bp) {
                return false;
            }
        }
    }
    true
}

/// Whether `(a, b)` is the unique highest-key (outermost) edge of the horizontal 2×2 square `{a, b, ap, bp}`.
/// Uses only coordinates relative to the square's lowest corner `(x0, z0)`, so neighbouring chunks always agree.
fn is_worst_square_edge(g: &TreeGrid, a: [i32; 3], b: [i32; 3], ap: [i32; 3], bp: [i32; 3]) -> bool {
    let x0 = a[0].min(b[0]).min(ap[0]).min(bp[0]);
    let z0 = a[2].min(b[2]).min(ap[2]).min(bp[2]);
    let key = |p: [i32; 3], q: [i32; 3]| {
        let (dp, dq) = (trunk_d2(g, p), trunk_d2(g, q));
        let z_edge = i32::from(p[0] == q[0]);
        let off = if z_edge == 1 { p[0] - x0 } else { p[2] - z0 };
        (dp.saturating_add(dq), dp.max(dq), z_edge, off)
    };
    let mine = key(a, b);
    mine > key(ap, bp) && mine > key(a, ap) && mine > key(b, bp)
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

/// The square of upright logs `c` is part of (3×3 before 2×2), by its lowest corner, if any. Requires a full matching
/// square directly above or directly below so a 1×1 trunk with branches beside it never swells into a 2×2 saucer.
fn square(g: &TreeGrid, c: [i32; 3]) -> Option<([i32; 3], i32)> {
    let full = |corner: [i32; 3], w: i32| (0..w * w).all(|i| log(g, [corner[0] + i % w, corner[1], corner[2] + i / w]));
    for w in [3, 2] {
        for oz in (0..w).rev() {
            for ox in (0..w).rev() {
                let corner = [c[0] - ox, c[1], c[2] - oz];
                if full(corner, w)
                    && (full([corner[0], c[1] + 1, corner[2]], w) || full([corner[0], c[1] - 1, corner[2]], w))
                {
                    return Some((corner, w));
                }
            }
        }
    }
    None
}

/// Whether `c` belongs to a vertical trunk column (a square trunk, a branchless stem like palm/bamboo, standing on
/// solid ground, or part of a vertical stack of at least 3 logs) rather than a lateral branch or stepping elbow.
fn is_trunk_col(g: &TreeGrid, c: [i32; 3]) -> bool {
    let b = g.at(c[0], c[1], c[2]);
    if !is_log(b) {
        return false;
    }
    if matches!(b, PALM_LOG | BAMBOO) || square(g, c).is_some() {
        return true;
    }
    let below = g.at(c[0], c[1] - 1, c[2]);
    if !is_log(below) && OPAQUE[below as usize] {
        return true;
    }
    let mut run = 1;
    for s in [-1, 1] {
        for step in 1..=2 {
            if log(g, [c[0], c[1] + s * step, c[2]]) {
                run += 1;
            } else {
                break;
            }
        }
    }
    run >= 3
}

/// Whether `c` sits near leaves (a living tree rather than bare/cut wood or a dead snag).
fn is_leafy(g: &TreeGrid, c: [i32; 3]) -> bool {
    for dy in -1..=4 {
        for dz in -3..=3 {
            for dx in -3..=3 {
                if is_leaves(g.at(c[0] + dx, c[1] + dy, c[2] + dz)) {
                    return true;
                }
            }
        }
    }
    false
}

/// Squared horizontal distance to the nearest trunk column cell (within 3 blocks), or `u32::MAX` if none.
fn trunk_d2(g: &TreeGrid, c: [i32; 3]) -> u32 {
    let mut best = u32::MAX;
    for dy in -2..=2 {
        for dz in -3..=3i32 {
            for dx in -3..=3i32 {
                let d2 = (dx * dx + dz * dz) as u32;
                if d2 < best && is_trunk_col(g, [c[0] + dx, c[1] + dy, c[2] + dz]) {
                    best = d2;
                }
            }
        }
    }
    best
}

/// A single log's `(radius, leafy, trunk)`: a trunk tapers upward with the wood load above it; any non-trunk branch
/// (including rising branch elbows) tapers smoothly outward from the nearest trunk column to its species' twig.
/// Wood with no leaves near (bare low trunk, dead tree, player-built logs) keeps its full thickness.
fn thickness(g: &TreeGrid, c: [i32; 3], b: BlockId) -> (f32, bool, bool) {
    let (tip, base) = radii(b);
    let trunk = is_trunk_col(g, c);
    let leafy = is_leafy(g, c);
    if !leafy {
        return (base, false, trunk);
    }
    if !trunk {
        let mut r = limb(g, c, tip, base);
        // Free living twig tips stay slender so they never look like thick wooden spears.
        let mut degree = 0;
        for i in 0..27 {
            let d = [i % 3 - 1, i / 3 % 3 - 1, i / 9 - 1];
            if d != [0; 3] && log(g, [c[0] + d[0], c[1] + d[1], c[2] + d[2]]) && linked(g, c, d) {
                degree += 1;
                if degree > 1 {
                    break;
                }
            }
        }
        if degree <= 1 {
            r = r.min(tip + (base * LIMB_SHARE - tip) * 0.25);
        }
        return (r, true, false);
    }
    let mut load = 0.0;
    for dy in 0..=4 {
        for dz in -2..=2i32 {
            for dx in -2..=2i32 {
                if is_log(g.at(c[0] + dx, c[1] + dy, c[2] + dz)) {
                    load += 1.0 / (1 + dx.abs() + dz.abs()) as f32;
                }
            }
        }
    }
    (tip + (base - tip) * (load / LOAD_FULL).min(1.0), true, true)
}

/// How far out a limb thins to a twig, and its thickness where it leaves the trunk as a share of the trunk's.
const LIMB_REACH: f32 = 4.0;
const LIMB_SHARE: f32 = 0.52;

/// A limb log's radius, smoothly tapering by Euclidean horizontal distance from the nearest true trunk column.
fn limb(g: &TreeGrid, c: [i32; 3], tip: f32, base: f32) -> f32 {
    let mut best_d2 = f32::MAX;
    for dy in -2..=2 {
        for dz in -4..=4i32 {
            for dx in -4..=4i32 {
                let d2 = (dx * dx + dz * dz) as f32;
                if d2 < best_d2 && is_trunk_col(g, [c[0] + dx, c[1] + dy, c[2] + dz]) {
                    best_d2 = d2;
                }
            }
        }
    }
    if best_d2 == f32::MAX {
        return tip;
    }
    let dist = (best_d2.sqrt() - 1.0).max(0.0);
    let t = (dist / LIMB_REACH).min(1.0);
    tip + (base * LIMB_SHARE - tip) * (1.0 - t)
}
