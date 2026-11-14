//! The path of a dragged belt line (part of `belt_line.rs`): [`plan`] lays the columns from the start towards
//! the pointer and follows the ground; where something is in the way it asks `pass.rs` for an underpass pair, and
//! a cliff more than a block high is climbed on lifts or gone down on drops (`belt_shape.rs` turns a column fed
//! over its top into drops by itself).
//! [`plan_upgrade`] is the same path over existing belts, for upgrade kits. Both read blocks or tiers through a
//! closure, never the core, so they are plain functions of their arguments (tests: `tests.rs`).

use crate::block::{self, BlockId, SOLID};
use crate::factory::{self, Shape};
use crate::math::IVec3;

use super::{pass, LineCell, Piece, MAX_LINE};

/// The columns (x, z) of a path from `start` towards the column of `end`, each with the way it runs:
/// along the longer axis first, turning once (the turning cell runs the new way); one column runs `facing`.
fn columns(start: IVec3, end: IVec3, facing: u8) -> Vec<((i32, i32), u8)> {
    let (dx, dz) = (end.x - start.x, end.z - start.z);
    let legs = if dx.abs() >= dz.abs() { [(dx, 0), (0, dz)] } else { [(0, dz), (dx, 0)] };
    let mut cols = vec![((start.x, start.z), facing)];
    for (lx, lz) in legs {
        let dir = match (lx.signum(), lz.signum()) {
            (0, 0) => continue,
            (0, -1) => 0,
            (1, 0) => 1,
            (0, 1) => 2,
            _ => 3,
        };
        cols.last_mut().expect("starts with one").1 = dir;
        let mut c = cols.last().expect("starts with one").0;
        for _ in 0..lx.abs().max(lz.abs()) {
            c = (c.0 + lx.signum(), c.1 + lz.signum());
            cols.push((c, dir));
        }
    }
    cols.truncate(MAX_LINE);
    cols
}

/// The tallest lift column a dragged line builds, in lifts.
pub const MAX_LIFT: i32 = 16;

/// The cells of a line from `start` towards the column of `end`, reading blocks through `block`
/// (`None`: not loaded), and where it was stopped by something too wide to go under (`pass.rs`: an
/// obstacle in the way becomes an underpass entry and exit). A line of one cell runs `facing`. Empty
/// when `start` isn't free. With `lifts` in hand the line climbs a cliff more than a block high on a lift column
/// (when it can't go under it), and goes down one on a column of drops when `end` lies below it (otherwise it
/// stays level over the gap); it never plans more lifts than `lifts`.
pub fn plan(
    block: impl Fn(IVec3) -> Option<BlockId>,
    start: IVec3,
    end: IVec3,
    facing: u8,
    mut lifts: usize,
) -> (Vec<LineCell>, Option<IVec3>) {
    let free = |p: IVec3| block(p).is_some_and(block::replaceable);
    let solid = |p: IVec3| block(p).is_some_and(|b| SOLID[b as usize] && factory::machine(b).is_none());
    // The level the belt takes in column (x, z) coming from level `y`; none if something is in the way.
    let level = |(x, z): (i32, i32), y: i32| {
        let at = |y: i32| IVec3::new(x, y, z);
        if free(at(y)) && solid(at(y - 1)) {
            Some(y)
        } else if solid(at(y)) && free(at(y + 1)) {
            Some(y + 1) // a step up: the belt before it becomes an up ramp
        } else if free(at(y)) && free(at(y - 1)) && solid(at(y - 2)) {
            Some(y - 1) // a step down: this belt becomes a down ramp
        } else if free(at(y)) {
            Some(y) // over a gap, level
        } else {
            None
        }
    };
    let cols = columns(start, end, facing);
    let mut cells: Vec<LineCell> = Vec::with_capacity(cols.len());
    let (mut y, mut blocked, mut i, mut after_exit) = (start.y, None, 0, false);
    while i < cols.len() {
        let (col, dir) = cols[i];
        let straight = i > 0 && cols[i - 1].1 == dir;
        if i == 0 {
            if !free(IVec3::new(col.0, y, col.1)) {
                break;
            }
        } else if let Some(k) =
            drop_depth(&free, &solid, col, y, end.y).filter(|&k| straight && !after_exit && k <= lifts)
        {
            // Down a cliff towards a lower end: a column of drops in this column, from just below the line to the
            // ground (or the end's level); the line goes on from its bottom.
            for j in 1..=k {
                cells.push(lift_cell(IVec3::new(col.0, y - j as i32, col.1), dir, Shape::Drop));
            }
            (y, lifts, i) = (y - k as i32, lifts - k, i + 1);
            continue;
        } else if let Some(next) = level(col, y) {
            if after_exit && next != y {
                break; // an underpass exit hands items on level
            }
            y = next;
        } else if let Some(k) = pass::bridge(&cols, i, y, &cells, &free) {
            pass::dive(&mut cells, &cols, i, k, y);
            (i, after_exit) = (i + k + 1, true);
            continue;
        } else if let Some(h) = climb_height(&free, &solid, &cells, col, y).filter(|&h| h <= lifts) {
            // Up a cliff: the belt before it becomes the foot of a lift column, whose top hands on onto the cliff.
            let foot = cells.last_mut().expect("climb_height needs a belt before the cliff");
            (foot.piece, foot.shape) = (Piece::Lift, Shape::Lift);
            let foot = *foot;
            for j in 1..h {
                cells.push(LineCell { pos: foot.pos + IVec3::new(0, j as i32, 0), ..foot });
            }
            (y, lifts) = (y + h as i32, lifts - h);
        } else {
            blocked = Some(IVec3::new(col.0, y, col.1));
            break;
        }
        after_exit = false;
        cells.push(LineCell::belt(IVec3::new(col.0, y, col.1), dir, Shape::Flat));
        i += 1;
    }
    slopes(&mut cells);
    (cells, blocked)
}

/// A lift (`Shape::Lift`) or drop (`Shape::Drop`) of a planned line.
fn lift_cell(pos: IVec3, dir: u8, shape: Shape) -> LineCell {
    LineCell { piece: Piece::Lift, ..LineCell::belt(pos, dir, shape) }
}

/// How many drops take a line at level `y` down into column `col`: from the cell below the line down to the ground,
/// or to `end_y` if that comes first. None for less than two (a one-block step is a ramp), when the end is not below,
/// or when no ground or end lies within `MAX_LIFT`.
fn drop_depth(
    free: &impl Fn(IVec3) -> bool,
    solid: &impl Fn(IVec3) -> bool,
    (x, z): (i32, i32),
    y: i32,
    end_y: i32,
) -> Option<usize> {
    let at = |y: i32| IVec3::new(x, y, z);
    if end_y > y - 2 || !free(at(y)) {
        return None;
    }
    let k = (1..=MAX_LIFT).take_while(|&j| free(at(y - j))).find(|&j| y - j == end_y || solid(at(y - j - 1)))?;
    (k >= 2).then_some(k as usize)
}

/// How many lifts take a line at level `y` up the cliff in column `col`: a column standing on the last cell, as tall
/// as the cliff is above it (the top hands on onto the ground there). None when the last cell can't be a lift's
/// foot (not a plain belt, or fed from above), when the column has no room, or the cliff is taller than `MAX_LIFT`.
fn climb_height(
    free: &impl Fn(IVec3) -> bool,
    solid: &impl Fn(IVec3) -> bool,
    cells: &[LineCell],
    (x, z): (i32, i32),
    y: i32,
) -> Option<usize> {
    let foot = cells.last().filter(|c| c.piece == Piece::Belt && c.pos.y == y)?;
    let behind = cells.len().checked_sub(2).map(|i| cells[i]);
    if behind.is_some_and(|b| b.pos.y > y || b.dir != foot.dir) {
        return None; // fed from above or from the side (a corner): a lift takes items only from straight behind
    }
    let at = |y: i32| IVec3::new(x, y, z);
    let h = (2..=MAX_LIFT).find(|&h| solid(at(y + h - 1)) && free(at(y + h)))?;
    (1..h).all(|j| free(foot.pos + IVec3::new(0, j, 0))).then_some(h as usize)
}

/// Marks the belts the factory will turn into ramps (it derives them from the heights), for the preview.
pub(super) fn slopes(cells: &mut [LineCell]) {
    for i in 0..cells.len() {
        let y = cells[i].pos.y;
        if cells[i].piece != Piece::Belt {
            continue;
        }
        if cells.get(i + 1).is_some_and(|n| n.pos.y == y + 1) {
            cells[i].shape = Shape::Up;
        } else if i > 0 && cells[i - 1].pos.y == y + 1 {
            cells[i].shape = Shape::Down;
        }
    }
}

/// The belts to upgrade from the belt at `start` towards the column of `end`: the path follows belts
/// (`tier_at`: the tier of the belt at a cell) one block up or down at a time and stops where none is;
/// the cells are those at tier `from`.
pub fn plan_upgrade(tier_at: impl Fn(IVec3) -> Option<u8>, start: IVec3, end: IVec3, from: u8) -> Vec<LineCell> {
    let mut cells = Vec::new();
    let mut y = start.y;
    for ((x, z), dir) in columns(start, end, 0) {
        let Some(pos) = [y, y + 1, y - 1].map(|y| IVec3::new(x, y, z)).into_iter().find(|&p| tier_at(p).is_some())
        else {
            break;
        };
        y = pos.y;
        if tier_at(pos) == Some(from) {
            cells.push(LineCell::belt(pos, dir, Shape::Flat));
        }
    }
    cells
}
