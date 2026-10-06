//! The path of a dragged belt line (part of `belt_line.rs`): [`plan`] lays the columns from the start towards
//! the pointer and follows the ground; where something is in the way it asks `pass.rs` for an underpass pair.
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

/// The cells of a line from `start` towards the column of `end`, reading blocks through `block`
/// (`None`: not loaded), and where it was stopped by something too wide to go under (`pass.rs`: an
/// obstacle in the way becomes an underpass entry and exit). A line of one cell runs `facing`. Empty
/// when `start` isn't free.
pub fn plan(
    block: impl Fn(IVec3) -> Option<BlockId>,
    start: IVec3,
    end: IVec3,
    facing: u8,
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
        if i == 0 {
            if !free(IVec3::new(col.0, y, col.1)) {
                break;
            }
        } else if let Some(next) = level(col, y) {
            if after_exit && next != y {
                break; // an underpass exit hands items on level
            }
            y = next;
        } else if let Some(k) = pass::bridge(&cols, i, y, &cells, &free) {
            pass::dive(&mut cells, &cols, i, k, y);
            (i, after_exit) = (i + k + 1, true);
            continue;
        } else {
            blocked = Some(IVec3::new(col.0, y, col.1));
            break;
        }
        after_exit = false;
        cells.push(LineCell::belt(IVec3::new(col.0, y, col.1), dir, Shape::Flat));
        i += 1;
    }
    // The slopes the factory will derive, for the preview.
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
    (cells, blocked)
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
