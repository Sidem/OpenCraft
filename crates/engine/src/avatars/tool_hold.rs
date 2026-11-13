//! How tools with handles are held and used (`grip.rs` places them, `model.rs` moves the arm): where each is gripped,
//! which way its working end faces, and the stroke it makes while mining, by kind. A pick or axe chops: it winds up
//! slowly, strikes fast with its edge leading and settles back. A shovel digs: it stabs forward along its handle, prises
//! the blade up and draws back. A sickle sweeps across the front from right to left. The arm on an avatar follows the
//! same curve, so a strike lands when the tool's head is lowest. Presentation only; reads the motion's phase.
//! To add a tool with a handle: a `ToolKind` arm in [`hold`] (grip point in model units, its stroke).

use crate::item::ItemId;
use crate::tools::{self, ToolKind};
use std::f64::consts::{PI, TAU};

/// How a tool sits in the hand and moves while mining.
#[derive(Clone, Copy, PartialEq, Debug)]
pub(super) enum Stroke {
    /// Pick, axe: wind up, strike with the edge leading, settle.
    Chop,
    /// Shovel: stab along the handle, prise up, draw back.
    Dig,
    /// Sickle: sweep across, right to left.
    Sweep,
}

/// A tool with a handle: where its model is gripped and the stroke it makes.
pub(super) struct Hold {
    /// The grip, in model units (`item_models`): low on a pick's or axe's handle, at the top of a shovel's.
    pub grip: [f64; 3],
    pub stroke: Stroke,
}

/// The tool's pose at a moment of its stroke, on top of its rest pose: `swing` turns its head along its edge (positive
/// strikes), `lean` tips it forward about the view's x, `sweep` turns it about the vertical (positive to the right) and
/// moves the hand that way, `thrust` drives the hand forward (blocks on an avatar), `lift` raises the hand.
#[derive(Clone, Copy, Default, Debug)]
pub(super) struct Pose {
    pub swing: f64,
    pub lean: f64,
    pub sweep: f64,
    pub thrust: f64,
    pub lift: f64,
}

/// `held`'s grip and stroke, or `None` when it isn't held by a handle (a drone or a scanner sits on the hand).
pub(super) fn hold(held: ItemId) -> Option<Hold> {
    let (grip, stroke) = match tools::tool(held)?.kind {
        ToolKind::Pickaxe | ToolKind::Axe => ([0.0, -0.38, 0.0], Stroke::Chop),
        // Just below the T-handle.
        ToolKind::Shovel => ([0.0, 0.3, 0.0], Stroke::Dig),
        ToolKind::Sickle => ([0.0, -0.3, 0.0], Stroke::Sweep),
        _ => return None,
    };
    Some(Hold { grip, stroke })
}

/// The pose of a stroke at `phase` (0 to 2π, one stroke), scaled by how much the player is mining (`mining`, 0 to 1).
pub(super) fn pose(stroke: Stroke, phase: f64, mining: f64) -> Pose {
    let t = (phase / TAU).rem_euclid(1.0);
    let p = match stroke {
        // Wind up over most of the stroke, strike in a fifth of it, settle back.
        Stroke::Chop => {
            let swing = keyframes(t, &[(0.0, 0.0), (0.55, -CHOP_RAISE), (0.75, CHOP_STRIKE), (1.0, 0.0)]);
            Pose { swing, lift: -swing * CHOP_LIFT, ..Pose::default() }
        }
        // Stab forward, prise the blade up while it is in, draw back.
        Stroke::Dig => Pose {
            thrust: keyframes(t, &[(0.0, 0.0), (0.3, DIG_STAB), (0.65, DIG_STAB * 0.6), (1.0, 0.0)]),
            lean: keyframes(t, &[(0.0, 0.0), (0.3, 0.0), (0.65, DIG_PRISE), (1.0, 0.0)]),
            ..Pose::default()
        },
        // Draw out to the right, cut across to the left fast, come back.
        Stroke::Sweep => Pose {
            sweep: keyframes(t, &[(0.0, 0.0), (0.5, SWEEP_OUT), (0.72, -SWEEP_ACROSS), (1.0, 0.0)]),
            lean: keyframes(t, &[(0.0, 0.0), (0.5, -0.1), (0.72, 0.25), (1.0, 0.0)]),
            ..Pose::default()
        },
    };
    Pose {
        swing: p.swing * mining,
        lean: p.lean * mining,
        sweep: p.sweep * mining,
        thrust: p.thrust * mining,
        lift: p.lift * mining,
    }
}

/// How far the avatar's arm is through a stroke: -1 raised, 0 at rest, 1 struck (bare hands and other items swing
/// evenly, as before).
pub(super) fn arm(held: ItemId, phase: f64) -> f64 {
    match hold(held).map(|h| h.stroke) {
        Some(Stroke::Chop) => {
            let s = pose(Stroke::Chop, phase, 1.0).swing;
            if s < 0.0 {
                s / CHOP_RAISE
            } else {
                s / CHOP_STRIKE
            }
        }
        Some(Stroke::Dig) => pose(Stroke::Dig, phase, 1.0).thrust / DIG_STAB,
        Some(Stroke::Sweep) | None => -(phase + PI).cos(),
    }
}

/// How far a pick or axe winds back and strikes past rest, radians.
const CHOP_RAISE: f64 = 0.75;
const CHOP_STRIKE: f64 = 0.55;
/// How far the hand rises per radian of wind-up (and drops on the strike), blocks on an avatar: small, or a raised
/// head pokes above the body while looking up (`avatars/tests.rs`).
const CHOP_LIFT: f64 = 0.04;
/// How far a shovel stabs (blocks on an avatar) and prises its blade up (radians).
const DIG_STAB: f64 = 0.12;
const DIG_PRISE: f64 = 0.3;
/// How far a sickle draws out to the right and cuts across to the left, radians.
const SWEEP_OUT: f64 = 0.45;
const SWEEP_ACROSS: f64 = 0.6;

/// The value at `t` (0 to 1) of eased segments between `(time, value)` keys, the first at 0 and the last at 1.
fn keyframes(t: f64, keys: &[(f64, f64)]) -> f64 {
    for w in keys.windows(2) {
        let ((t0, a), (t1, b)) = (w[0], w[1]);
        if t <= t1 {
            let u = ((t - t0) / (t1 - t0)).clamp(0.0, 1.0);
            return a + (b - a) * u * u * (3.0 - 2.0 * u);
        }
    }
    keys.last().map_or(0.0, |k| k.1)
}

#[cfg(test)]
mod tests;
