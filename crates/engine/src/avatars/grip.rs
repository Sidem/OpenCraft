//! How an item sits in a hand (`model.rs` draws it, on an avatar and in first person): every item at one size, its
//! largest side [`HELD_SIZE`] (a block's cube is that size; a tool with a handle [`TOOL_SIZE`] times that), held by its
//! middle; a tool is held at its grip (`tool_hold.rs`), its working end leading its stroke. Each part's turn (its own yaw, lean or round axis) is
//! composed with the wrist's exactly and split back into the yaw, roll and pitch the box shader applies
//! (`render/box-glsl.ts`: yaw · roll · pitch), so a held item keeps its shape at any wrist angle: in first person it stays
//! still on screen while the camera looks up and down.
//! Invariant: positions are in the rig's frame before its yaw (x right, y up, -z ahead), sizes in blocks.

use super::tool_hold::{self, Pose, Stroke};
use crate::factory::{Axis, Form};
use crate::item::{self, ItemId};
use crate::item_models;
use crate::math::Vec3;
use std::f64::consts::{FRAC_PI_2, PI};

/// The largest side of a held item on an avatar, in blocks (first person scales it with the view hand). A block's
/// cube was 0.21; every item is now that size and 30% more.
pub const HELD_SIZE: f64 = 0.273;
/// A tool with a handle is this much longer than other items are large: a long, thin thing at a block's size looks
/// like a toy, and its handle should reach past the fist.
pub const TOOL_SIZE: f64 = 1.3;
/// Where a tool's grip sits from the hand's centre: in the fist, a little low and ahead. In blocks on an avatar.
const TOOL_AT: [f64; 3] = [0.0, -0.03, -0.03];
/// Where the middle of anything else sits from the hand's centre: about as far ahead as the old block cube's middle,
/// and high enough that a thin item (a drone) shows over the first-person glove while a block's top stays below the
/// crosshair. It turns about there.
const HOLD_AT: [f64; 3] = [0.0, 0.088, -0.1];
/// First person draws held items this much larger than the hand's scale (as games draw view models), so a block shows
/// about 30% wider on screen than its old cube (measured: 192 px before, 206 px at 1.0 on an 800 px view) while every
/// item stays inside the body (`avatars/tests.rs`).
const VIEW_GROW: f64 = 1.15;

/// One box of a held item, ready for the rig.
pub(super) struct HeldPart {
    pub center: Vec3,
    pub size: [f32; 3],
    /// Added to the rig's yaw.
    pub yaw: f64,
    pub pitch: f64,
    pub roll: f64,
    pub tex: [u16; 3],
    /// A round body, with its far end's width share.
    pub taper: Option<f32>,
}

/// How a tool sits before its stroke: its turn about its handle (from the model's: a pick's or axe's edge along +x, a
/// shovel's blade face towards +z, a sickle's hook towards +x) and its lean forward about x, on an avatar (the wrist
/// leans it) and in first person.
fn rest(stroke: Stroke, view: bool) -> (M3, f64) {
    match (stroke, view) {
        // Edge ahead, in the arm's swing plane.
        (Stroke::Chop, false) => (yaw_m(-FRAC_PI_2), 0.0),
        // Edge ahead and to the left, towards the crosshair, so the head shows from the side and leads the strike.
        (Stroke::Chop, true) => (yaw_m(-2.2), -0.55),
        (Stroke::Dig, false) => (IDENTITY, 0.0),
        // The blade up and ahead towards the crosshair, its face a little towards the eye: the hand is low in the view, so
        // a blade hanging down would be out of sight.
        (Stroke::Dig, true) => (yaw_m(-0.4), PI - 0.55),
        (Stroke::Sweep, false) => (yaw_m(-FRAC_PI_2), 0.0),
        // The hook towards the crosshair.
        (Stroke::Sweep, true) => (yaw_m(PI), -0.3),
    }
}

/// `held` in the hand at `hand`, the wrist turned by `pitch` about x, at `scale` times its size on an avatar. Anything
/// but a tool turns by `tip` about x through its middle; a tool is held at its grip and posed by `pose` (its stroke,
/// `tool_hold.rs`). First person (`view`) holds tools so their working end shows.
#[allow(clippy::too_many_arguments)]
pub(super) fn held_parts(
    held: ItemId,
    hand: Vec3,
    pitch: f64,
    tip: f64,
    scale: f64,
    view: bool,
    pose: &Pose,
) -> Vec<HeldPart> {
    let parts = model(held);
    if parts.is_empty() {
        return Vec::new();
    }
    // The model's bounds, each part's box turned as it is drawn.
    let (mut lo, mut hi) = ([f64::MAX; 3], [f64::MIN; 3]);
    for p in &parts {
        let half = p.size.map(|s| f64::from(s) * 0.5);
        for a in 0..3 {
            let reach: f64 = (0..3).map(|j| p.basis[a][j].abs() * half[j]).sum();
            lo[a] = lo[a].min(p.center[a] - reach);
            hi[a] = hi[a].max(p.center[a] + reach);
        }
    }
    let largest = (0..3).map(|a| hi[a] - lo[a]).fold(1e-6, f64::max);
    let tool = tool_hold::hold(held);
    let size = if tool.is_some() { TOOL_SIZE } else { 1.0 } * if view { VIEW_GROW } else { 1.0 };
    let k = HELD_SIZE * size * scale / largest;
    let v = |a: [f64; 3]| Vec3::new(a[0], a[1], a[2]);
    // The item's own turn about its pivot, in the hand's frame. A tool swings along its edge (about its model z), then
    // leans and sweeps with the wrist.
    let (pivot, anchor, own) = match tool {
        Some(t) => {
            let (turn, lean) = rest(t.stroke, view);
            let own = mul(yaw_m(pose.sweep), mul(rx_m(lean + pose.lean), mul(turn, rz_m(-pose.swing))));
            (v(t.grip), v(TOOL_AT) * scale, own)
        }
        None => {
            let mid = Vec3::new((lo[0] + hi[0]) * 0.5, (lo[1] + hi[1]) * 0.5, (lo[2] + hi[2]) * 0.5);
            (mid, v(HOLD_AT) * scale, rx_m(tip))
        }
    };
    let tilt = rx_m(pitch);
    let whole = mul(tilt, own);
    parts
        .into_iter()
        .map(|p| {
            let local = (Vec3::new(p.center[0], p.center[1], p.center[2]) - pivot) * k;
            let (yaw, roll, pitch) = euler(mul(whole, p.basis));
            HeldPart {
                center: hand + apply(tilt, anchor + apply(own, local)),
                size: p.size.map(|s| (f64::from(s) * k) as f32),
                yaw,
                pitch,
                roll,
                tex: p.tex,
                taper: p.taper,
            }
        })
        .collect()
}

/// A part of a held item in model space: its box's size as drawn and the turn from that box to the model.
struct ModelPart {
    center: [f64; 3],
    size: [f32; 3],
    basis: M3,
    tex: [u16; 3],
    taper: Option<f32>,
}

/// The parts of `held`'s model, or a block's cube (none for nothing or an unknown item).
fn model(held: ItemId) -> Vec<ModelPart> {
    let parts = item_models::parts(held);
    if parts.is_empty() {
        return item::def(held)
            .filter(|_| held != ItemId::NONE)
            .map(|def| ModelPart { center: [0.0; 3], size: [1.0; 3], basis: IDENTITY, tex: def.tex, taper: None })
            .into_iter()
            .collect();
    }
    parts
        .iter()
        .map(|p| {
            let [x, y, z] = p.size;
            let own = yaw_m(f64::from(p.yaw));
            // As `make_round` lays a round body and `push_shape` leans a box.
            let (size, basis, taper) = match p.form {
                Form::Box => (p.size, own, None),
                Form::Lean(lean) => (p.size, mul(own, rx_m(f64::from(lean))), None),
                Form::Round(Axis::Up, t) => (p.size, own, Some(t)),
                Form::Round(Axis::Z, t) => ([x, z, y], mul(own, rx_m(FRAC_PI_2)), Some(t)),
                Form::Round(Axis::X, t) => ([z, x, y], mul(own, mul(yaw_m(-FRAC_PI_2), rx_m(FRAC_PI_2))), Some(t)),
            };
            ModelPart { center: p.center.map(f64::from), size, basis, tex: p.tex, taper }
        })
        .collect()
}

/// A rotation, rows of a 3×3 matrix.
type M3 = [[f64; 3]; 3];

const IDENTITY: M3 = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];

/// The shader's yaw: x towards (cos, 0, sin), -z towards (sin, 0, -cos).
fn yaw_m(a: f64) -> M3 {
    let (s, c) = a.sin_cos();
    [[c, 0.0, -s], [0.0, 1.0, 0.0], [s, 0.0, c]]
}

/// The shader's pitch: y towards (0, cos, sin).
fn rx_m(a: f64) -> M3 {
    let (s, c) = a.sin_cos();
    [[1.0, 0.0, 0.0], [0.0, c, -s], [0.0, s, c]]
}

/// The shader's roll: x towards (cos, sin, 0).
fn rz_m(a: f64) -> M3 {
    let (s, c) = a.sin_cos();
    [[c, -s, 0.0], [s, c, 0.0], [0.0, 0.0, 1.0]]
}

fn mul(a: M3, b: M3) -> M3 {
    let mut m = [[0.0; 3]; 3];
    for (i, row) in m.iter_mut().enumerate() {
        for (j, v) in row.iter_mut().enumerate() {
            *v = (0..3).map(|k| a[i][k] * b[k][j]).sum();
        }
    }
    m
}

fn apply(m: M3, v: Vec3) -> Vec3 {
    let r = |i: usize| m[i][0] * v.x + m[i][1] * v.y + m[i][2] * v.z;
    Vec3::new(r(0), r(1), r(2))
}

/// `m` as the shader's yaw, roll and pitch (`m` = yaw · roll · pitch).
fn euler(m: M3) -> (f64, f64, f64) {
    let roll = m[1][0].clamp(-1.0, 1.0).asin();
    if roll.cos() < 1e-6 {
        // Rolled a quarter turn: pitch and yaw turn about one axis; all of it goes to yaw.
        return ((-m[0][2]).atan2(m[2][2]), roll, 0.0);
    }
    (m[2][0].atan2(m[0][0]), roll, (-m[1][2]).atan2(m[1][1]))
}

/// For tests: the shader's turn of a record with this yaw, roll and pitch.
#[cfg(test)]
fn compose(yaw: f64, roll: f64, pitch: f64) -> M3 {
    mul(yaw_m(yaw), mul(rz_m(roll), rx_m(pitch)))
}

#[cfg(test)]
mod tests;
