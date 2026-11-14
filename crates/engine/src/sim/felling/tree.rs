//! A grown tree as one thing: its trunk found from any of its low logs (`trunk`), and the whole tree taken out of
//! the world when it falls (`take`). Pure functions of the blocks they read, so the core and the hands agree.
//!
//! Invariants: a trunk is keyed by its foot, the lowest-first (z, then x) cell of its bottom layer, so every log of
//! it finds the same tree. Where crowns touch, a log belongs to the trunk standing nearest it (as the forester's
//! felling decides, `forester/fell.rs`), so a neighbour keeps its limbs; leaves another log still holds stay.

use crate::block::{is_leaves, is_log, is_tree_wood, BlockId, AIR};
use crate::math::{sort_small_by_key, IVec3};
use crate::sim::LEAF_SUPPORT_STEPS;
use crate::worldgen::REACH as CROWN_REACH;

/// How far round its foot a tree's parts may lie, and how high above it.
const ACROSS: i32 = CROWN_REACH + 2;
const HIGH: i32 = 60;
const UP: IVec3 = IVec3::new(0, 1, 0);
const E: IVec3 = IVec3::new(1, 0, 0);
const W: IVec3 = IVec3::new(-1, 0, 0);
const S: IVec3 = IVec3::new(0, 0, 1);
const N: IVec3 = IVec3::new(0, 0, -1);
const DOWN: IVec3 = IVec3::new(0, -1, 0);
/// The box `take` reads: the tree and the leaves a log beyond it could hold.
const SIDE: i32 = 2 * (ACROSS + LEAF_SUPPORT_STEPS) + 1;
const LAYERS: i32 = HIGH + 3;

/// A tree's trunk: where it stands and what felling it takes.
#[derive(Clone, Debug, PartialEq)]
pub struct Trunk {
    /// The key: the first cell of its bottom layer.
    pub foot: IVec3,
    /// Its bottom layer's cells and their middle (x, z).
    pub base: Vec<IVec3>,
    pub centre: [f32; 2],
    /// How far its bark stands out from the middle.
    pub radius: f32,
    /// Logs in the trunk (not its limbs): what felling it is worth.
    pub logs: u32,
    /// Its lowest layer out in the open (a wide bole sinks into the ground): swings count from here up.
    pub low: i32,
    /// The most logs in one layer of its lower part: as many as a layer of the trunk holds.
    pub width: usize,
}

/// How far across from the log swung at the trunk's lower part is looked for.
const NEAR: i32 = 3;

/// What came down: every block taken (logs, then leaves) with its old block, how many were trunk logs and limbs,
/// the trunk's height, and its wood.
pub struct Taken {
    pub blocks: Vec<(IVec3, BlockId)>,
    pub trunk_logs: u32,
    pub limb_logs: u32,
    pub leaves: u32,
    pub height: i32,
    pub log: BlockId,
}

/// The trunk `pos` (a tree's log) belongs to, read through `get`; `None` when `pos` is no tree wood.
pub fn trunk(mut get: impl FnMut(IVec3) -> BlockId, pos: IVec3) -> Option<Trunk> {
    if !is_tree_wood(get(pos)) {
        return None;
    }
    // The trunk's lower part: tree wood joined to `pos` across faces, not above it and not far across. Its lowest
    // layer is the bottom; the lowest with a side open to the air is where it shows.
    let mut low = vec![pos];
    let mut i = 0;
    while i < low.len() && low.len() < 1024 {
        let p = low[i];
        i += 1;
        for d in [E, W, S, N, DOWN] {
            let n = p + d;
            let near = (n.x - pos.x).abs() <= NEAR && (n.z - pos.z).abs() <= NEAR && n.y > pos.y - HIGH;
            if near && !low.contains(&n) && is_tree_wood(get(n)) {
                low.push(n);
            }
        }
    }
    let bottom = low.iter().map(|p| p.y).min().unwrap_or(pos.y);
    let mut base: Vec<IVec3> = low.iter().copied().filter(|p| p.y == bottom).collect();
    sort_small_by_key(&mut base, |p| (p.z, p.x));
    let open = |p: &IVec3, get: &mut dyn FnMut(IVec3) -> BlockId| {
        [E, W, S, N].iter().any(|&d| {
            let b = get(*p + d);
            !is_tree_wood(b) && !crate::block::def(b).solid
        })
    };
    let shows = low.iter().filter(|p| open(p, &mut get)).map(|p| p.y).min().unwrap_or(bottom);
    let width = (bottom..=pos.y).map(|y| low.iter().filter(|p| p.y == y).count()).max().unwrap_or(1);
    let (centre, radius) = middle(&base);
    let logs = climb(&mut get, &base, width).len() as u32;
    Some(Trunk { foot: base[0], base, centre, radius, logs, low: shows, width })
}

/// Takes the tree of `trunk` out of the world: its logs (those nearer it than any other trunk) and the leaves no log
/// holds once they are gone (the caller clears them). `get` reads.
pub fn take(mut get: impl FnMut(IVec3) -> BlockId, trunk: &Trunk) -> Taken {
    let origin = trunk.foot - IVec3::new(SIDE / 2, 2, SIDE / 2);
    let at = |p: IVec3| ((p.y * SIDE + p.z) * SIDE + p.x) as usize;
    let inside = |p: IVec3| (0..SIDE).contains(&p.x) && (0..LAYERS).contains(&p.y) && (0..SIDE).contains(&p.z);
    let mut grid = vec![AIR; (SIDE * SIDE * LAYERS) as usize];
    // Read up to the first layer above the foot with no wood and no leaves: nothing over it joins this tree (most of
    // the box is sky, and reading it was most of a fall's tick).
    for y in 0..LAYERS {
        let mut tree = false;
        for z in 0..SIDE {
            for x in 0..SIDE {
                let p = IVec3::new(x, y, z);
                let b = get(origin + p);
                tree |= is_tree_wood(b) || is_leaves(b);
                grid[at(p)] = b;
            }
        }
        if !tree && y > 2 {
            break;
        }
    }
    // Other trunks: tree wood standing on something else, not in this one's bottom layer.
    let ours = trunk.centre;
    let mut others: Vec<[f32; 2]> = Vec::new();
    for z in 0..SIDE {
        for x in 0..SIDE {
            for y in 1..LAYERS {
                let p = IVec3::new(x, y, z);
                let w = origin + p;
                if is_tree_wood(grid[at(p)]) && !is_tree_wood(grid[at(p - UP)]) && !trunk.base.contains(&w) {
                    if !is_leaves(grid[at(p - UP)]) && grid[at(p - UP)] != AIR {
                        others.push([w.x as f32 + 0.5, w.z as f32 + 0.5]);
                    }
                    break;
                }
            }
        }
    }
    let d2 = |c: [f32; 2], p: IVec3| (p.x as f32 + 0.5 - c[0]).powi(2) + (p.z as f32 + 0.5 - c[1]).powi(2);
    let mine = |p: IVec3| others.iter().all(|&o| d2(ours, p) <= d2(o, p));
    // The tree: tree wood joined to the bottom layer (faces, edges or corners), within reach, nearer this trunk.
    let mut logs: Vec<IVec3> = trunk.base.iter().map(|&b| b - origin).collect();
    let mut taken = vec![false; grid.len()];
    for &p in &logs {
        taken[at(p)] = true;
    }
    let mut i = 0;
    while i < logs.len() {
        let p = logs[i];
        i += 1;
        for k in 0..27 {
            let n = p + IVec3::new(k % 3 - 1, k / 3 % 3 - 1, k / 9 - 1);
            let near = (n.x - SIDE / 2).abs() <= ACROSS && (n.z - SIDE / 2).abs() <= ACROSS;
            if inside(n) && near && !taken[at(n)] && is_tree_wood(grid[at(n)]) && mine(origin + n) {
                taken[at(n)] = true;
                logs.push(n);
            }
        }
    }
    let trunk_cells = climb(
        |p| if inside(p - origin) && taken[at(p - origin)] { grid[at(p - origin)] } else { AIR },
        &trunk.base,
        trunk.width,
    );
    let height = trunk_cells.iter().map(|p| p.y).max().unwrap_or(trunk.foot.y) - trunk.foot.y + 1;
    let log = grid[at(trunk.foot - origin)];
    let mut blocks = Vec::with_capacity(logs.len() * 4);
    for &p in &logs {
        blocks.push((origin + p, grid[at(p)]));
        grid[at(p)] = AIR;
    }
    // Leaves still held: out from every log left, through leaves, as far as a log holds them (`sim/timers.rs`). Leaves
    // on the box's rim count as held, since a log beyond it may hold them.
    let mut held = vec![false; grid.len()];
    let mut frontier = Vec::new();
    for (i, &b) in grid.iter().enumerate() {
        let i = i as i32;
        let p = IVec3::new(i % SIDE, i / (SIDE * SIDE), i / SIDE % SIDE);
        let rim = p.x == 0 || p.z == 0 || p.x == SIDE - 1 || p.z == SIDE - 1 || p.y == LAYERS - 1;
        if is_log(b) || (rim && is_leaves(b)) {
            held[i as usize] = true;
            frontier.push(p);
        }
    }
    for _ in 0..LEAF_SUPPORT_STEPS {
        let mut next = Vec::new();
        for p in frontier {
            for d in [E, W, UP, DOWN, S, N] {
                let n = p + d;
                if inside(n) && !held[at(n)] && is_leaves(grid[at(n)]) {
                    held[at(n)] = true;
                    next.push(n);
                }
            }
        }
        frontier = next;
    }
    let mut leaves = 0;
    for (i, &b) in grid.iter().enumerate() {
        if is_leaves(b) && !held[i] {
            let i = i as i32;
            blocks.push((origin + IVec3::new(i % SIDE, i / (SIDE * SIDE), i / SIDE % SIDE), b));
            leaves += 1;
        }
    }
    let trunk_logs = trunk_cells.len() as u32;
    Taken { blocks, trunk_logs, limb_logs: logs.len() as u32 - trunk_logs, leaves, height, log }
}

/// The trunk's cells from its bottom layer up: each layer the tree wood next to the layer below (a lean follows),
/// at most `width`, nearest the layer below's middle (so limbs leaving it are not trunk).
fn climb(mut get: impl FnMut(IVec3) -> BlockId, base: &[IVec3], width: usize) -> Vec<IVec3> {
    let mut cells = base.to_vec();
    let mut layer = base.to_vec();
    while !layer.is_empty() && cells.len() < 4096 {
        let (c, _) = middle(&layer);
        let mut next: Vec<IVec3> = Vec::new();
        for p in &layer {
            for k in 0..9 {
                let n = *p + IVec3::new(k % 3 - 1, 1, k / 3 - 1);
                if !next.contains(&n) && is_tree_wood(get(n)) {
                    next.push(n);
                }
            }
        }
        let away = |p: &IVec3| (((p.x as f32 + 0.5 - c[0]).powi(2) + (p.z as f32 + 0.5 - c[1]).powi(2)) * 64.0) as u32;
        sort_small_by_key(&mut next, |p| (away(p), p.z, p.x));
        next.truncate(width);
        cells.extend_from_slice(&next);
        layer = next;
    }
    cells
}

/// The middle (x, z) of some cells and how far bark round them reaches from it.
fn middle(cells: &[IVec3]) -> ([f32; 2], f32) {
    let n = cells.len().max(1) as f32;
    let c = [
        cells.iter().map(|p| p.x as f32 + 0.5).sum::<f32>() / n,
        cells.iter().map(|p| p.z as f32 + 0.5).sum::<f32>() / n,
    ];
    let far =
        cells.iter().map(|p| (p.x as f32 + 0.5 - c[0]).abs().max((p.z as f32 + 0.5 - c[1]).abs())).fold(0.0, f32::max);
    (c, far + 0.4)
}
