//! Laying belts in lines, part of the local player's hands (`interaction.rs`). With a belt selected,
//! holding the use button on a surface starts a line at the cell in front of that face; the pointer
//! (up to `LINE_REACH` away) sets the far end; releasing builds it, a few cells per tick, as ordinary
//! `PlaceBlock` actions, so the core, co-op and saves see plain placements. A left click cancels.
//!
//! The path ([`plan`]) runs along the longer horizontal axis first and turns once. It follows the
//! ground one block up or down at a time; the factory turns those steps into ramps by itself
//! (`factory::belt_shape::derive_slopes`). It stops before anything in the way, so a line ending at a
//! machine feeds it. Planning reads the loaded world (the render cache), never core state.
//!
//! To change the path: `plan`. The preview: ghost belts (`write_line_preview`) and the host's outline
//! boxes and label (`api/hud.rs` `line_cells`, `line_label`).

use crate::action::Action;
use crate::block::{self, BlockId, BELT, FAST_BELT, RAMP_DOWN, RAMP_UP, SOLID};
use crate::factory::{self, Shape};
use crate::math::{IVec3, Vec3};
use crate::raycast::raycast;
use crate::Game;

/// The longest line one drag builds, in cells.
pub const MAX_LINE: usize = 64;
/// How far away the pointer can set the line's end.
const LINE_REACH: f64 = 48.0;
/// Cells placed per tick while a line is built (fast, but the placing sounds don't all land at once).
const BUILD_PER_TICK: usize = 3;
const UP: IVec3 = IVec3::new(0, 1, 0);

/// One cell of a planned line: where, which way it runs, and how it will slope.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct LineCell {
    pub pos: IVec3,
    pub dir: u8,
    pub shape: Shape,
}

/// A line being dragged out, or being built.
#[derive(Default)]
pub struct BeltLine {
    /// The cell the drag started in, and the direction a single click lays a belt.
    start: Option<(IVec3, u8)>,
    pub cells: Vec<LineCell>,
    /// Placements still to send, and the hotbar slot and item they come from.
    building: Vec<IVec3>,
    build_dir: Vec<u8>,
    build_from: (u8, u16),
}

/// Whether block `b` is laid in lines.
pub fn is_belt(b: BlockId) -> bool {
    matches!(b, BELT | FAST_BELT | RAMP_UP | RAMP_DOWN)
}

/// The cells of a line from `start` towards the column of `end`, reading blocks through `block`
/// (`None`: not loaded). A line of one cell runs `facing`. Empty when `start` isn't free.
pub fn plan(block: impl Fn(IVec3) -> Option<BlockId>, start: IVec3, end: IVec3, facing: u8) -> Vec<LineCell> {
    let free = |p: IVec3| block(p).is_some_and(block::replaceable);
    let solid = |p: IVec3| block(p).is_some_and(|b| SOLID[b as usize] && factory::machine(b).is_none());
    // The columns, each with the way it runs; the cell where the path turns runs the new way.
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

    let mut cells: Vec<LineCell> = Vec::with_capacity(cols.len());
    let mut y = start.y;
    for (i, &((x, z), dir)) in cols.iter().enumerate() {
        let at = |y: i32| IVec3::new(x, y, z);
        if i > 0 {
            y = if free(at(y)) && solid(at(y - 1)) {
                y
            } else if solid(at(y)) && free(at(y + 1)) {
                y + 1 // a step up: the belt before it becomes an up ramp
            } else if free(at(y)) && free(at(y - 1)) && solid(at(y - 2)) {
                y - 1 // a step down: this belt becomes a down ramp
            } else if free(at(y)) {
                y // over a gap, level
            } else {
                break;
            };
        } else if !free(at(y)) {
            break;
        }
        cells.push(LineCell { pos: at(y), dir, shape: Shape::Flat });
    }
    // The slopes the factory will derive, for the preview.
    for i in 0..cells.len() {
        let y = cells[i].pos.y;
        if cells.get(i + 1).is_some_and(|n| n.pos.y == y + 1) {
            cells[i].shape = Shape::Up;
        } else if i > 0 && cells[i - 1].pos.y == y + 1 {
            cells[i].shape = Shape::Down;
        }
    }
    cells
}

impl Game {
    /// Runs the line tool for one tick. True while it owns the use button (a belt is selected and the
    /// button went down on a surface, or a line is being built).
    pub(crate) fn update_belt_line(&mut self) -> bool {
        if self.send_line_placements() {
            return true;
        }
        let stack = self.inventory().selected_stack();
        if !stack.item.places().is_some_and(is_belt) || stack.is_empty() {
            self.line.start = None;
            self.line.cells.clear();
            return false;
        }
        match (self.line.start, self.using) {
            (None, false) => false,
            (None, true) => self.start_line(),
            (Some(_), _) if self.mining => {
                self.cancel_belt_line();
                true
            }
            (Some((start, facing)), true) => {
                let end = self.line_end(start).unwrap_or_else(|| self.line.cells.last().map_or(start, |c| c.pos));
                let world = &self.sim.world;
                self.line.cells = plan(|p| world.get_block(p), start, end, facing);
                true
            }
            (Some(_), false) => {
                self.line.start = None;
                let n = self.line.cells.len().min(stack.count as usize);
                let cells = std::mem::take(&mut self.line.cells);
                self.line.building = cells[..n].iter().rev().map(|c| c.pos).collect();
                self.line.build_dir = cells[..n].iter().rev().map(|c| c.dir).collect();
                self.line.build_from = (self.inventory().selected as u8, stack.item.0);
                true
            }
        }
    }

    /// Drops the line being dragged (the pointer was freed, or a left click).
    pub(crate) fn cancel_belt_line(&mut self) {
        self.line.start = None;
        self.line.cells.clear();
    }

    /// The use button went down: start a line at the cell in front of the targeted face. Boxes and
    /// machine panels keep their right-click (unless crouching).
    fn start_line(&mut self) -> bool {
        let Some(hit) = self.target else { return false };
        let machine = factory::machine(hit.id).is_some_and(|m| m.panel || m.slots > 0);
        if (machine && !self.body().input.crouch) || hit.normal == IVec3::ZERO {
            return false;
        }
        let start = hit.block + hit.normal;
        if self.sim.world.get_block(start).is_some_and(block::replaceable) {
            let facing = factory::dir_from_yaw(self.body().yaw);
            self.line.start = Some((start, facing));
            self.line.cells = vec![LineCell { pos: start, dir: facing, shape: Shape::Flat }];
        }
        true
    }

    /// The cell the pointer picks as the line's end: in front of the face it points at, or where it
    /// crosses the start's level when it points at the sky.
    fn line_end(&self, start: IVec3) -> Option<IVec3> {
        let (eye, dir) = (self.body().eye(), self.body().look_dir());
        let world = &self.sim.world;
        if let Some(hit) = raycast(eye, dir, LINE_REACH, |p| world.get_block(p).filter(|&b| !block::replaceable(b))) {
            return (hit.normal != IVec3::ZERO).then_some(hit.block + hit.normal);
        }
        let t = (start.y as f64 + 0.5 - eye.y) / dir.y;
        (dir.y < -1e-3 && t < LINE_REACH).then(|| (eye + dir * t).floor())
    }

    /// Sends the next few placements of a line being built. False when there are none. Stops if the
    /// slot no longer holds the belts it started with.
    fn send_line_placements(&mut self) -> bool {
        if self.line.building.is_empty() {
            return false;
        }
        let (slot, item) = self.line.build_from;
        if self.inventory().slots[slot as usize].item.0 != item {
            self.line.building.clear();
            return false;
        }
        for _ in 0..BUILD_PER_TICK {
            let (Some(pos), Some(facing)) = (self.line.building.pop(), self.line.build_dir.pop()) else { break };
            self.act(Action::PlaceBlock { pos, slot, facing, against: pos - UP });
        }
        true
    }

    /// Ghost belts along the planned line, drawn with the machines.
    pub(crate) fn write_line_preview(&mut self, eye: Vec3, time: f64) {
        let fast = self.inventory().selected_stack().item.places() == Some(FAST_BELT);
        for c in &self.line.cells {
            let rel = c.pos.as_vec3() + Vec3::new(0.5, 0.5, 0.5) - eye;
            factory::belt_preview(&mut self.instances, c.pos, c.dir, c.shape, fast, rel, time);
        }
    }

    /// Cells of the planned line for the host's outlines: x, y, z and 1 if it will be built (0 past
    /// the belts in hand).
    pub(crate) fn planned_cells(&self) -> Vec<i32> {
        let have = self.inventory().selected_stack().count as usize;
        let cell = |(i, c): (usize, &LineCell)| [c.pos.x, c.pos.y, c.pos.z, i32::from(i < have)];
        self.line.cells.iter().enumerate().flat_map(cell).collect()
    }
}

#[cfg(test)]
mod tests;
