//! Belts that climb and cross: the belt [`Shape`]s beyond a flat belt, where items ride on each, and
//! their models. Every shape is a `Belt` (same items, spacing and `belt_step`); only where it hands
//! items on differs, which `relink` (`links.rs`) works out:
//! - Up ramp: takes items at its back at floor level and hands them on one cell ahead and one up.
//!   Down ramp: the reverse; a belt one level up behind it feeds its high end.
//! - Lift: items rise through the cell. A lift above facing the same way takes them on straight up;
//!   the top lift hands them on one ahead and one up, like an up ramp.
//! - Underpass entry: hands items to the nearest exit facing the same way up to `UNDERPASS_RANGE`
//!   cells ahead, under whatever lies between. The exit carries on like a flat belt.
//!
//! Invariants: the shape is core state (saved from version 6); `lift_below` / `lift_above` are
//! derived. To add a shape: a variant (append, it is saved by index), its block in `Shape::of`, its
//! path in `item_at`, its model, and its output in `relink`.

use crate::block::{tex, BlockId, LIFT, RAMP_DOWN, RAMP_UP, UNDERPASS_IN, UNDERPASS_OUT};
use crate::math::Vec3;

use super::belt::{Belt, BELT_HEIGHT};
use super::render::push_box;
use super::DIRS;

/// How far ahead an underpass entry looks for its exit, in cells.
pub const UNDERPASS_RANGE: i32 = 5;
/// Share of a lift's length spent moving on or off it horizontally.
const LIFT_EDGE: f32 = 0.2;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Shape {
    Flat,
    Up,
    Down,
    Lift,
    Entry,
    Exit,
}

const SHAPES: [Shape; 6] = [Shape::Flat, Shape::Up, Shape::Down, Shape::Lift, Shape::Entry, Shape::Exit];

impl Shape {
    /// The shape belt block `block` has.
    pub fn of(block: BlockId) -> Shape {
        match block {
            RAMP_UP => Shape::Up,
            RAMP_DOWN => Shape::Down,
            LIFT => Shape::Lift,
            UNDERPASS_IN => Shape::Entry,
            UNDERPASS_OUT => Shape::Exit,
            _ => Shape::Flat,
        }
    }

    pub fn from_u8(v: u8) -> Option<Shape> {
        SHAPES.get(v as usize).copied()
    }

    /// Hands items on straight ahead at its own level, like a flat belt.
    pub fn flat_out(self) -> bool {
        matches!(self, Shape::Flat | Shape::Down | Shape::Exit)
    }

    /// Takes items from the cell behind it at its own level (belts and machines feed it there).
    pub fn fed_from_back(self) -> bool {
        !matches!(self, Shape::Down | Shape::Exit)
    }

    /// Readout words after the heading.
    pub fn words(self) -> &'static str {
        match self {
            Shape::Flat => "",
            Shape::Up => ", climbing one level",
            Shape::Down => ", going down one level",
            Shape::Lift => ", lifting items up",
            Shape::Entry => ", taking items under to an exit ahead",
            Shape::Exit => ", bringing items back up",
        }
    }
}

impl Belt {
    /// Where an item at progress `p` rides: offset from the bottom centre of the cell (x, height, z).
    pub fn item_at(&self, p: f32) -> (f32, f32, f32) {
        let d = DIRS[self.dir as usize];
        let along = |t: f32, y: f32| (d.x as f32 * t, y, d.z as f32 * t);
        match self.shape {
            Shape::Up => along(p - 0.5, p),
            Shape::Down => along(p - 0.5, 1.0 - p),
            Shape::Lift => {
                let a = if self.lift_below { 0.0 } else { LIFT_EDGE };
                let b = if self.lift_above { 1.0 } else { 1.0 - LIFT_EDGE };
                if p < a {
                    along((p / a - 1.0) * 0.5, 0.0)
                } else if p > b {
                    along((p - b) / (1.0 - b) * 0.5, 1.0)
                } else {
                    along(0.0, (p - a) / (b - a))
                }
            }
            Shape::Flat | Shape::Entry | Shape::Exit => {
                let (x, z) = self.offset(p);
                (x, 0.0, z)
            }
        }
    }

    /// Whether an item at `p` is in sight (not under an underpass hood).
    pub fn shows(&self, p: f32) -> bool {
        match self.shape {
            Shape::Entry => p <= 0.5,
            Shape::Exit => p >= 0.5,
            _ => true,
        }
    }

    /// The parts of the model that differ from a flat belt. `at` maps local coordinates (forward is
    /// -z, y up from the cell bottom) to camera-relative ones.
    pub(super) fn shape_model(&self, out: &mut Vec<f32>, at: &dyn Fn(f32, f32, f32) -> Vec3, yaw: f32, scroll: f32) {
        let belt = [tex::BELT_TOP, tex::FRAME, tex::FRAME];
        match self.shape {
            Shape::Flat => {}
            Shape::Up | Shape::Down => {
                // Three steps, each topped with belt, rising towards the high end.
                for i in 0..3 {
                    let rise = if self.shape == Shape::Up { i as f32 } else { 2.0 - i as f32 };
                    let h = (rise + 0.5) / 3.0 + BELT_HEIGHT;
                    let z = (1.0 - i as f32) / 3.0;
                    push_box(out, at(0.0, h * 0.5, z), yaw, [0.9, h, 1.0 / 3.0], scroll, belt, true);
                }
            }
            Shape::Lift => {
                for (x, z) in [(-0.42, -0.42), (0.42, -0.42), (-0.42, 0.42), (0.42, 0.42)] {
                    push_box(out, at(x, 0.5, z), yaw, [0.08, 1.0, 0.08], 0.0, [tex::FRAME; 3], true);
                }
                if !self.lift_below {
                    push_box(out, at(0.0, BELT_HEIGHT * 0.5, 0.0), yaw, [0.84, BELT_HEIGHT, 1.0], scroll, belt, true);
                }
            }
            Shape::Entry | Shape::Exit => {
                let z = if self.shape == Shape::Entry { -0.25 } else { 0.25 };
                push_box(out, at(0.0, 0.32, z), yaw, [1.0, 0.64, 0.5], 0.0, [tex::FRAME; 3], true);
            }
        }
    }
}
