//! Belts that climb and cross: the belt [`Shape`]s beyond a flat belt, where items ride on each, and
//! their models. Every shape is a `Belt` (same items, spacing and `belt_step`); only where it hands
//! items on differs, which `relink` (`links.rs`) works out:
//! - Up ramp: takes items at its back at floor level and hands them on one cell ahead and one up.
//!   Down ramp: the reverse; a belt one level up behind it feeds its high end. Ramps aren't placed as
//!   such: [`derive_slopes`] turns a plain belt into one from its neighbours (a belt one step up
//!   ahead, or one step up behind), so a line laid over a terrain step slopes by itself.
//! - Lift: items rise through the cell. A lift above facing the same way takes them on straight up;
//!   the top lift hands them on one ahead and one up, like an up ramp.
//! - Drop: a lift column carrying items down, the same block and cells as a lift with the way reversed: the top
//!   takes items from a belt one behind and one up (as a down ramp's high end is fed), the bottom hands them on
//!   straight ahead at its own level. [`derive_lift_ways`] turns a column into drops by itself when such a belt
//!   feeds its top and nothing stands behind its bottom, so a line laid down a cliff descends by itself.
//! - Underpass entry: hands items to the exit its pair has (`underpass.rs` pairs the pieces up), under
//!   whatever lies between. The exit carries on like a flat belt.
//!
//! Invariants: the shape is core state (saved from version 6), but flat, ramp, lift-or-drop and underpass shapes
//! are rederived at every relink; `lift_below` / `lift_above` are derived. Rederiving twice gives the same shapes
//! (joins resync from a relinked save): lift ways read no ramp shape, ramps read lift ways. To add a shape: a
//! variant (append, it is saved by index), its block in `Shape::of`, its path in `item_at`, its model, and its
//! output in `relink`.

use std::f32::consts::FRAC_PI_2;

use rustc_hash::FxHashMap;

use crate::block::{tex, BlockId, LIFT, RAMP_DOWN, RAMP_UP, UNDERPASS_IN, UNDERPASS_OUT};
use crate::math::{IVec3, Vec3};

use super::belt::{Belt, BELT_HEIGHT, BELT_TIERS};
use super::belt_curve::BEND_STRAIGHT;
use super::links::Slot;
use super::render::{push_box, INSTANCE_FLOATS, PITCH};
use super::{tier_mark, DIRS};

const UP: IVec3 = IVec3::new(0, 1, 0);

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
    Drop,
}

const SHAPES: [Shape; 7] = [Shape::Flat, Shape::Up, Shape::Down, Shape::Lift, Shape::Entry, Shape::Exit, Shape::Drop];

impl Shape {
    /// The shape belt block `block` has.
    pub fn of(block: BlockId) -> Shape {
        match block {
            RAMP_UP => Shape::Up,
            RAMP_DOWN => Shape::Down,
            LIFT => Shape::Lift,
            UNDERPASS_IN | UNDERPASS_OUT => Shape::Entry,
            _ => Shape::Flat,
        }
    }

    pub fn from_u8(v: u8) -> Option<Shape> {
        SHAPES.get(v as usize).copied()
    }

    /// Hands items on straight ahead at its own level, like a flat belt (a drop: the bottom of its column).
    pub fn flat_out(self) -> bool {
        matches!(self, Shape::Flat | Shape::Down | Shape::Exit | Shape::Drop)
    }

    /// Takes items from the cell behind it at its own level (belts and machines feed it there).
    pub fn fed_from_back(self) -> bool {
        !matches!(self, Shape::Down | Shape::Exit | Shape::Drop)
    }

    /// A lift either way (the Belt Lift block): stacks into a column with the lifts above and below it.
    pub fn is_lift(self) -> bool {
        matches!(self, Shape::Lift | Shape::Drop)
    }

    /// An underpass piece (an entry or an exit; `underpass.rs` decides which).
    pub fn is_pass(self) -> bool {
        matches!(self, Shape::Entry | Shape::Exit)
    }

    /// Flat belts and ramps: placement decides which (`derive_slopes`), not the block.
    pub fn sloped_by_placement(self) -> bool {
        matches!(self, Shape::Flat | Shape::Up | Shape::Down)
    }

    /// Readout words after the heading.
    pub fn words(self) -> &'static str {
        match self {
            Shape::Flat => "",
            Shape::Up => ", climbing one level",
            Shape::Down => ", going down one level",
            Shape::Lift => ", lifting items up",
            Shape::Drop => ", lowering items down",
            Shape::Entry => ", taking items under to its exit ahead",
            Shape::Exit => ", bringing items back up",
        }
    }
}

/// Gives every flat belt or ramp the shape its neighbours call for: an up ramp when a belt facing the
/// same way waits one cell ahead and one up (and nothing is straight ahead), a down ramp when one
/// facing the same way stands one cell behind and one up (with nothing in front of it or behind this
/// one), otherwise flat; a bent belt keeps its shape. Depends only on positions, directions and bends, so the order
/// doesn't matter.
pub(super) fn derive_slopes(belts: &mut [Belt], at: &FxHashMap<IVec3, Slot>) {
    let shapes: Vec<Shape> = belts
        .iter()
        .map(|b| {
            // A belt the player bent stays flat: a ramp has no side to take items in from.
            if !b.shape.sloped_by_placement() || b.bend != BEND_STRAIGHT {
                return b.shape;
            }
            let d = DIRS[b.dir as usize];
            let belt = |q: IVec3, fits: fn(Shape) -> bool| match at.get(&q) {
                Some(Slot::Belt(j)) => belts[*j as usize].dir == b.dir && fits(belts[*j as usize].shape),
                _ => false,
            };
            let free = |q: IVec3| !at.contains_key(&q);
            if free(b.pos + d) && belt(b.pos + d + UP, |s| s.sloped_by_placement() || s.fed_from_back()) {
                Shape::Up
            } else if free(b.pos + UP)
                && free(b.pos - d)
                && belt(b.pos - d + UP, |s| s.sloped_by_placement() || s.flat_out())
            {
                Shape::Down
            } else {
                Shape::Flat
            }
        })
        .collect();
    for (b, s) in belts.iter_mut().zip(shapes) {
        b.shape = s;
    }
}

/// Turns every lift column the way its neighbours call for: drops ([`Shape::Drop`]) when a belt facing the same
/// way hands items in over its top (one cell behind and one up) and nothing stands behind its bottom to feed it
/// there, otherwise lifts. Every piece of a column turns together. Reads only positions, directions, which belts
/// are lifts and the non-lift shapes' kind (never a ramp's slope), so ramps can be derived after it.
pub(super) fn derive_lift_ways(belts: &mut [Belt], at: &FxHashMap<IVec3, Slot>) {
    let ways: Vec<Shape> = belts
        .iter()
        .map(|b| match b.shape.is_lift() {
            true if column_drops(belts, at, b.pos, b.dir) => Shape::Drop,
            true => Shape::Lift,
            false => b.shape,
        })
        .collect();
    for (b, s) in belts.iter_mut().zip(ways) {
        b.shape = s;
    }
}

/// Whether the lift column through `pos` facing `dir` carries items down (`derive_lift_ways`). A drop column's
/// bottom can feed the top of another below it, so this asks again of the feeding column, always a higher one.
fn column_drops(belts: &[Belt], at: &FxHashMap<IVec3, Slot>, pos: IVec3, dir: u8) -> bool {
    let d = DIRS[dir as usize];
    let lift = |q: IVec3| matches!(at.get(&q), Some(Slot::Belt(j)) if belts[*j as usize].shape.is_lift() && belts[*j as usize].dir == dir);
    let (mut top, mut bottom) = (pos, pos);
    while lift(top + UP) {
        top = top + UP;
    }
    while lift(bottom - UP) {
        bottom = bottom - UP;
    }
    let feeder = top - d + UP;
    let fed_over_top = match at.get(&feeder) {
        Some(Slot::Belt(j)) if belts[*j as usize].dir == dir => match belts[*j as usize].shape {
            s if s.is_lift() => !lift(feeder - UP) && column_drops(belts, at, feeder, dir),
            s => s.sloped_by_placement() || s.flat_out(),
        },
        _ => false,
    };
    fed_over_top && !at.contains_key(&(bottom - d))
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
            Shape::Drop => {
                // A lift run backwards: in over the top from behind, down the middle, out along the floor ahead.
                let a = if self.lift_above { 0.0 } else { LIFT_EDGE };
                let b = if self.lift_below { 1.0 } else { 1.0 - LIFT_EDGE };
                if p < a {
                    along((p / a - 1.0) * 0.5, 1.0)
                } else if p > b {
                    along((p - b) / (1.0 - b) * 0.5, 0.0)
                } else {
                    along(0.0, 1.0 - (p - a) / (b - a))
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
        let top = BELT_TIERS[self.tier as usize].top;
        let belt = [top, tex::FRAME, tex::FRAME];
        // Rails and frames wear the tier stripe on their sides.
        let rail = [tex::FRAME, tex::stripe(self.tier), tex::FRAME];
        match self.shape {
            Shape::Flat => {}
            Shape::Up | Shape::Down => {
                // Three steps, each topped with belt, rising towards the high end.
                for i in 0..3 {
                    let rise = if self.shape == Shape::Up { i as f32 } else { 2.0 - i as f32 };
                    let h = (rise + 0.5) / 3.0 + BELT_HEIGHT;
                    let z = (1.0 - i as f32) / 3.0;
                    push_box(out, at(0.0, h * 0.5, z), yaw, [0.9, h, 1.0 / 3.0], scroll, belt, true);
                    for x in [-0.46, 0.46] {
                        push_box(out, at(x, h - 0.06, z), yaw, [0.07, 0.14, 1.0 / 3.0], 0.0, rail, false);
                    }
                }
            }
            Shape::Lift => self.lift_model(out, at, yaw, scroll, belt, rail),
            // A drop stands where a lift facing the other way would (its parts mirrored), but its boxes keep the
            // drop's own heading so the belts' arrows and scroll point the way items go: on, then down, then out.
            Shape::Drop => {
                let back = |x: f32, y: f32, z: f32| at(-x, y, -z);
                self.lift_model(out, &back, yaw, scroll, belt, rail);
            }
            Shape::Entry | Shape::Exit => {
                let z = if self.shape == Shape::Entry { -0.25 } else { 0.25 };
                for x in [-0.42, 0.42] {
                    push_box(out, at(x, 0.32, z), yaw, [0.14, 0.64, 0.5], 0.0, rail, false);
                }
                push_box(out, at(0.0, 0.62, z), yaw, [0.96, 0.1, 0.5], 0.0, [tex::COPPER_INGOT; 3], false);
                // Every pass piece shows its Mk: studs across its hood.
                if tier_mark::near(at(0.0, 0.0, 0.0)) {
                    let (paint, d) = ([tex::paint(self.tier); 3], tier_mark::DEPTH);
                    let stud = [tier_mark::STUD, d, tier_mark::STUD];
                    for o in tier_mark::offsets(self.tier) {
                        push_box(out, at(o, 0.67 + d / 2.0, z), yaw, stud, 0.0, paint, false);
                    }
                }
            }
        }
    }

    /// A lift: one belt that comes in along the floor from the back (the bottom lift), climbs the front spine
    /// (every lift, a full cell high so a stack reads as one belt) and turns forward over the top (the top lift),
    /// level with the belt one ahead and one up, which it overhangs into the cell above by `BELT_HEIGHT`.
    /// Open between stacked lifts: only the top one has a frame ring.
    fn lift_model(
        &self,
        out: &mut Vec<f32>,
        at: &dyn Fn(f32, f32, f32) -> Vec3,
        yaw: f32,
        scroll: f32,
        belt: [u16; 3],
        rail: [u16; 3],
    ) {
        const FACE: f32 = -0.18; // the climbing belt's face (items rise at the centre, just behind it)
        const BELT_T: f32 = 0.06;
        const SPINE_FRONT: f32 = -0.44;
        let frame = [tex::FRAME; 3];
        // The spine and the belt climbing it. The belt is a box stood on end (pitched so its top faces back and
        // its forward scroll runs upwards): every piece is a full cell high, so stacked pieces scroll in phase.
        let spine_back = FACE - BELT_T;
        let spine = [0.92, 1.0, spine_back - SPINE_FRONT];
        push_box(out, at(0.0, 0.5, (spine_back + SPINE_FRONT) * 0.5), yaw, spine, 0.0, rail, true);
        push_box(out, at(0.0, 0.5, FACE - BELT_T * 0.5), yaw, [0.84, BELT_T, 1.0], scroll, belt, true);
        let rec = out.len() - INSTANCE_FLOATS;
        // Stood on end: a lift's arrows run up its face, a drop's down (tipped the other way).
        out[rec + PITCH] = if self.shape == Shape::Drop { -FRAC_PI_2 } else { FRAC_PI_2 };
        for x in [-0.42, 0.42] {
            push_box(out, at(x, 0.5, 0.42), yaw, [0.08, 1.0, 0.08], 0.0, frame, true);
        }
        // A flat belt span with its rails at height `y`, from `back` to `front` (local z).
        let span = |out: &mut Vec<f32>, y: f32, back: f32, front: f32| {
            let (z, len) = ((back + front) * 0.5, back - front);
            push_box(out, at(0.0, y + BELT_HEIGHT * 0.5, z), yaw, [0.84, BELT_HEIGHT, len], scroll, belt, true);
            for x in [-0.46, 0.46] {
                push_box(out, at(x, y + 0.13, z), yaw, [0.08, 0.26, len], 0.0, rail, true);
            }
        };
        if !self.lift_below {
            // The way in: a belt from the back edge into the foot of the climb, with a flat belt's back bar unless a
            // belt feeds it.
            span(out, 0.0, 0.5, FACE - 0.02);
            // (A drop's floor belt is its way out: no end bar.)
            if !self.fed_by_belt && self.shape == Shape::Lift {
                push_box(out, at(0.0, 0.2, 0.45), yaw, [0.83, 0.07, 0.08], 0.0, [tex::DRILL; 3], false);
            }
        }
        if !self.lift_above {
            // The way out: over the top of the spine to the front edge, where the next belt starts.
            span(out, 1.0, FACE + 0.01, -0.5);
            push_box(out, at(0.0, 0.96, 0.42), yaw, [0.84, 0.08, 0.08], 0.0, frame, true);
            let (z, len) = ((0.42 + spine_back) * 0.5, 0.42 - spine_back);
            for x in [-0.42, 0.42] {
                push_box(out, at(x, 0.96, z), yaw, [0.08, 0.08, len], 0.0, frame, true);
            }
        }
    }
}
