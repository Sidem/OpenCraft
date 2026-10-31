//! Belt corners. A flat belt turns a corner when items come in from one side (`Belt::curve_from`): when the player
//! bent it (`Belt::bend`, Shift+R, saved), or when the only belt feeding it joins from the side (derived in `relink`).
//! Its items ride a quarter circle about the cell corner between the entry and exit edges, and it is drawn as a quarter
//! ring of [`PIECES`] short belt pieces with an outer rail (six boxes, about one more than a straight belt);
//! presentation only, so the core never reads either.
//!
//! The bend is relative to the heading, so turning a belt with R keeps its bend. To change the model: `curve_model`.

use std::f32::consts::{FRAC_1_SQRT_2, FRAC_PI_2};

use crate::block::{tex, BlockId};
use crate::math::Vec3;

use super::belt::{Belt, BELT_HEIGHT, BELT_TIERS};
use super::belt_shape::Shape;
use super::render::push_box;
use super::{machine, Kind, DIRS};

/// Straight: items come in at the back. The other bends take them in from the belt's left or right side.
pub const BEND_STRAIGHT: u8 = 0;
pub const BEND_LEFT: u8 = 1;
pub const BEND_RIGHT: u8 = 2;
/// How many bends there are (Shift+R steps through them).
pub const BENDS: u8 = 3;

/// Belt pieces in a corner's quarter ring.
const PIECES: usize = 3;
/// The sine and cosine of each piece's middle angle round the corner: 15, 45 and 75 degrees.
const MIDS: [(f32, f32); PIECES] = [(0.258_819, 0.965_926), (FRAC_1_SQRT_2, FRAC_1_SQRT_2), (0.965_926, 0.258_819)];
/// Distance from the centreline to a rail, as on a straight belt.
const RAIL: f32 = 0.46;
/// A piece's length: the chord of its third of the ring at the outer rail (2 r sin 15 degrees), and a little overlap.
const PIECE: f32 = 2.0 * (0.5 + RAIL) * 0.258_819 + 0.02;

/// The placing `facing` byte of a belt heading `dir` with `bend`: `PlaceBlock`, ghosts and blueprints carry a belt's bend in its
/// upper bits, so none of them needed a new field.
pub fn belt_facing(dir: u8, bend: u8) -> u8 {
    (dir % 4) | (bend << 2)
}

/// The bend a placing `facing` byte carries (straight if it carries none that exists).
pub fn bend_of(facing: u8) -> u8 {
    Some(facing >> 2).filter(|&b| b < BENDS).unwrap_or(BEND_STRAIGHT)
}

/// `facing` made valid for placing `block`: a quarter turn, plus the bend for a flat belt.
pub fn clean_facing(block: BlockId, facing: u8) -> u8 {
    let flat_belt = machine(block).is_some_and(|m| m.kind == Kind::Belt) && Shape::of(block) == Shape::Flat;
    if flat_belt {
        belt_facing(facing, bend_of(facing))
    } else {
        facing % 4
    }
}

/// Readout words for a bend, after the heading.
pub fn bend_words(bend: u8) -> &'static str {
    match bend {
        BEND_LEFT => ", taking items in from its left",
        BEND_RIGHT => ", taking items in from its right",
        _ => "",
    }
}

impl Belt {
    /// The side items come in from, as a `DIRS` index from this cell: the back unless the player bent it.
    pub fn entry_side(&self) -> u8 {
        match self.bend {
            BEND_LEFT => (self.dir + 3) % 4,
            BEND_RIGHT => (self.dir + 1) % 4,
            _ => (self.dir + 2) % 4,
        }
    }

    /// Where an item at progress `p` rides on a corner taking items in from `side`, as an offset from the cell centre:
    /// on a quarter circle of radius one half about the cell corner between the entry and exit edges.
    pub(super) fn curve_offset(&self, side: u8, p: f32) -> (f32, f32) {
        let (x, z) = self.on_arc(side, 0.5, p.clamp(0.0, 1.0) * FRAC_PI_2);
        (x, z)
    }

    /// The corner's model: a quarter ring of belt pieces whose tops scroll along it, each with its stretch of the outer
    /// rail. The belts before and after it draw the cross bars at its edges. `base` is the cell's bottom centre.
    pub(super) fn curve_model(&self, out: &mut Vec<f32>, base: Vec3, scroll: f32, side: u8) {
        let top = [BELT_TIERS[self.tier as usize].top, tex::FRAME, tex::FRAME];
        let rail = [tex::FRAME, tex::stripe(self.tier), tex::FRAME];
        let at = |(x, z): (f32, f32), y: f32| base + Vec3::new(x as f64, y as f64, z as f64);
        for (k, &(s, c)) in MIDS.iter().enumerate() {
            let yaw = self.tangent_yaw(side, (k as f32 + 0.5) * FRAC_PI_2 / PIECES as f32);
            // Neighbouring pieces overlap near the inner corner: the middle one sits a hair higher so they never fight.
            let lift = (k % 2) as f32 * 0.004;
            // The top's texture moves on by the share of the corner already behind the piece, so it flows on round.
            let along = 0.5 + scroll - (k as f32 + 0.5) / PIECES as f32;
            let size = [0.84, BELT_HEIGHT + lift * 2.0, PIECE];
            push_box(out, at(self.arc_point(side, 0.5, s, c), BELT_HEIGHT * 0.5 + lift), yaw, size, along, top, true);
            push_box(out, at(self.arc_point(side, 0.5 + RAIL, s, c), 0.13), yaw, [0.08, 0.26, PIECE], 0.0, rail, true);
        }
    }

    /// The point at radius `r` and angle `phi` (0 at the entry edge, a quarter turn at the exit) about the corner pivot.
    fn on_arc(&self, side: u8, r: f32, phi: f32) -> (f32, f32) {
        let (s, c) = phi.sin_cos();
        self.arc_point(side, r, s, c)
    }

    /// `on_arc` from the angle's sine and cosine.
    fn arc_point(&self, side: u8, r: f32, s: f32, c: f32) -> (f32, f32) {
        let (u, v) = (DIRS[side as usize], DIRS[self.dir as usize]);
        let pivot = |a: i32, b: i32| 0.5 * (a + b) as f32;
        let x = pivot(u.x, v.x) - r * (c * v.x as f32 + s * u.x as f32);
        let z = pivot(u.z, v.z) - r * (c * v.z as f32 + s * u.z as f32);
        (x, z)
    }

    /// The yaw (`push_box`: local -z turned towards (sin, -cos)) of the way items move at angle `phi` round the corner:
    /// from the way they come in, a quarter turn off the heading, round to the heading.
    pub(super) fn tangent_yaw(&self, side: u8, phi: f32) -> f32 {
        let turn = if side == (self.dir + 3) % 4 { 1.0 } else { -1.0 };
        self.dir as f32 * FRAC_PI_2 + turn * (FRAC_PI_2 - phi)
    }
}
#[cfg(test)]
mod tests;
