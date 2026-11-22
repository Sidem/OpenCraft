//! How tools and hands move while mining or using (`grip.rs` places tools, `hand.rs` poses the hand, `model.rs`
//! drives the arm): each tool kind has its own grip and animation curve.
//! - Pickaxe (`Stroke::Pick`): high overhead wind-up, fast downward-forward strike with rock impact recoil.
//! - Axe (`Stroke::Axe`): wide draw-back to the right with blade rolled diagonally, cleaving right-to-left chop and bite recoil.
//! - Shovel (`Stroke::Dig`): low cock-back, forceful forward-down stab, upward scoop/lever with a slight toss.
//! - Sickle (`Stroke::Sweep`): wide right-to-left horizontal reaping slash.
//! - Bare hand / other item (`Stroke::Punch`): cock back, snap strike forward-inward with recoil.
//!
//! Presentation only; reads the motion's phase. To add a tool with a handle: a `ToolKind` arm in [`hold`].

use crate::item::ItemId;
use crate::tools::{self, ToolKind};
use std::f64::consts::TAU;

/// How a tool or hand moves while mining.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum Stroke {
    /// Pickaxe: overhead wind-up, downward-forward strike, crisp recoil.
    Pick,
    /// Axe: wide outward wind-up, diagonal-horizontal cleaving chop across the trunk, bite recoil.
    Axe,
    /// Shovel: stab forward-down along the handle, scoop and lever up, toss aside.
    Dig,
    /// Sickle: wide horizontal sweep right to left.
    Sweep,
    /// Flamethrower: held by its rear grip, barrel aimed straight ahead.
    Aim,
    /// Bare fist or non-tool item: cock back, active forward strike, impact recoil.
    Punch,
}

/// A tool with a handle: where its model is gripped and the stroke it makes.
pub(super) struct Hold {
    /// The grip, in model units (`item_models`): low on a pick's or axe's handle, at the top of a shovel's.
    pub grip: [f64; 3],
    pub stroke: Stroke,
}

/// The pose at a moment of a stroke, on top of rest: `swing` turns a tool's head along its edge (positive strikes),
/// `lean` pitches forward about the view's x (positive tips top away/back), `sweep` turns horizontally (positive right)
/// and shifts the hand laterally, `roll` rolls the wrist/blade about the view's -z (negative tilts top right),
/// `thrust` drives the hand forward (blocks on an avatar), `lift` raises the hand.
#[derive(Clone, Copy, Default, Debug)]
pub(super) struct Pose {
    pub swing: f64,
    pub lean: f64,
    pub sweep: f64,
    pub roll: f64,
    pub thrust: f64,
    pub lift: f64,
}

/// `held`'s handle grip and stroke, or `None` when it isn't held by a handle (bare hand, blocks, scanner, drone).
pub(super) fn hold(held: ItemId) -> Option<Hold> {
    let (grip, stroke) = match tools::tool(held)?.kind {
        ToolKind::Pickaxe => ([0.0, -0.38, 0.0], Stroke::Pick),
        ToolKind::Axe => ([0.0, -0.38, 0.0], Stroke::Axe),
        // Just below the T-handle.
        ToolKind::Shovel => ([0.0, 0.3, 0.0], Stroke::Dig),
        ToolKind::Sickle => ([0.0, -0.3, 0.0], Stroke::Sweep),
        ToolKind::Flamethrower => ([-0.24, 0.04, 0.0], Stroke::Aim),
        ToolKind::PaintSprayer => ([-0.24, -0.06, 0.0], Stroke::Aim),
        _ => return None,
    };
    Some(Hold { grip, stroke })
}

/// The active mining stroke for any held item (tools use their own stroke; bare hands and items punch).
pub(super) fn stroke_of(held: ItemId) -> Stroke {
    hold(held).map_or(Stroke::Punch, |h| h.stroke)
}

/// The pose of a stroke at `phase` (0 to 2π, one stroke), scaled by `mining` (0 to 1).
pub(super) fn pose(stroke: Stroke, phase: f64, mining: f64) -> Pose {
    let t = (phase / TAU).rem_euclid(1.0);
    let p = match stroke {
        // Pickaxe: deliberate overhead wind-up, fast accelerating downstroke, brief stone impact recoil, recovery.
        Stroke::Pick => Pose {
            swing: keyframes(
                t,
                &[(0.0, 0.0), (0.52, -PICK_RAISE), (0.72, PICK_STRIKE), (0.83, PICK_RECOIL), (1.0, 0.0)],
            ),
            lift: keyframes(
                t,
                &[(0.0, 0.0), (0.52, PICK_LIFT_UP), (0.72, -PICK_LIFT_DOWN), (0.83, -0.025), (1.0, 0.0)],
            ),
            thrust: keyframes(t, &[(0.0, 0.0), (0.52, -0.045), (0.72, PICK_THRUST), (0.83, 0.045), (1.0, 0.0)]),
            sweep: keyframes(t, &[(0.0, 0.0), (0.52, 0.09), (0.72, -0.08), (0.83, -0.04), (1.0, 0.0)]),
            roll: keyframes(t, &[(0.0, 0.0), (0.52, -0.10), (0.72, 0.08), (0.83, 0.04), (1.0, 0.0)]),
            lean: keyframes(t, &[(0.0, 0.0), (0.52, 0.14), (0.72, -0.12), (0.83, -0.05), (1.0, 0.0)]),
        },
        // Axe: draws wide right with blade rolled diagonally, whips right-to-left across the trunk, wood-bite recoil.
        Stroke::Axe => Pose {
            swing: keyframes(t, &[(0.0, 0.0), (0.50, -AXE_RAISE), (0.70, AXE_STRIKE), (0.82, AXE_RECOIL), (1.0, 0.0)]),
            sweep: keyframes(t, &[(0.0, 0.0), (0.50, AXE_SWEEP_OUT), (0.70, -AXE_SWEEP_IN), (0.82, -0.26), (1.0, 0.0)]),
            roll: keyframes(t, &[(0.0, 0.0), (0.50, -AXE_ROLL), (0.70, -AXE_ROLL * 0.85), (0.82, -0.28), (1.0, 0.0)]),
            lift: keyframes(t, &[(0.0, 0.0), (0.50, 0.05), (0.70, -0.035), (0.82, -0.02), (1.0, 0.0)]),
            thrust: keyframes(t, &[(0.0, 0.0), (0.50, -0.04), (0.70, 0.095), (0.82, 0.05), (1.0, 0.0)]),
            lean: keyframes(t, &[(0.0, 0.0), (0.50, 0.08), (0.70, -0.08), (0.82, -0.04), (1.0, 0.0)]),
        },
        // Shovel: cock low, stab forward-down along shaft, lever blade up to scoop & flick earth aside, draw back.
        Stroke::Dig => Pose {
            thrust: keyframes(t, &[(0.0, 0.0), (0.16, -0.04), (0.36, DIG_STAB), (0.66, DIG_STAB * 0.65), (1.0, 0.0)]),
            lift: keyframes(t, &[(0.0, 0.0), (0.16, -0.02), (0.36, -0.05), (0.66, 0.045), (1.0, 0.0)]),
            lean: keyframes(t, &[(0.0, 0.0), (0.16, -0.14), (0.36, -0.06), (0.66, DIG_PRISE), (1.0, 0.0)]),
            sweep: keyframes(t, &[(0.0, 0.0), (0.36, -0.06), (0.66, 0.16), (1.0, 0.0)]),
            roll: keyframes(t, &[(0.0, 0.0), (0.36, -0.05), (0.66, 0.28), (1.0, 0.0)]),
            swing: 0.0,
        },
        // Sickle: draw wide right, fast slicing horizontal sweep right-to-left across plants, snappy recovery.
        Stroke::Sweep => Pose {
            sweep: keyframes(t, &[(0.0, 0.0), (0.46, SWEEP_OUT), (0.68, -SWEEP_ACROSS), (0.82, -0.35), (1.0, 0.0)]),
            lean: keyframes(t, &[(0.0, 0.0), (0.46, -0.14), (0.68, 0.30), (1.0, 0.0)]),
            roll: keyframes(t, &[(0.0, 0.0), (0.46, -0.24), (0.68, 0.28), (1.0, 0.0)]),
            thrust: keyframes(t, &[(0.0, 0.0), (0.46, -0.03), (0.68, 0.08), (1.0, 0.0)]),
            lift: keyframes(t, &[(0.0, 0.0), (0.46, 0.02), (0.68, -0.025), (1.0, 0.0)]),
            swing: 0.0,
        },
        // Flamethrower: stays aimed forward with a subtle forward nudge on left-click.
        Stroke::Aim => Pose { thrust: keyframes(t, &[(0.0, 0.0), (0.4, 0.04), (1.0, 0.0)]), ..Pose::default() },
        // Bare hand / non-tool: cock fist back & out, punch forward-inward with wrist snap, crisp impact recoil.
        Stroke::Punch => Pose {
            thrust: keyframes(t, &[(0.0, 0.0), (0.36, -0.055), (0.58, PUNCH_THRUST), (0.72, 0.065), (1.0, 0.0)]),
            sweep: keyframes(t, &[(0.0, 0.0), (0.36, 0.14), (0.58, -0.24), (0.72, -0.12), (1.0, 0.0)]),
            lift: keyframes(t, &[(0.0, 0.0), (0.36, -0.028), (0.58, 0.024), (0.72, 0.005), (1.0, 0.0)]),
            lean: keyframes(t, &[(0.0, 0.0), (0.36, 0.24), (0.58, -0.38), (0.72, -0.18), (1.0, 0.0)]),
            roll: keyframes(t, &[(0.0, 0.0), (0.36, -0.18), (0.58, 0.32), (0.72, 0.16), (1.0, 0.0)]),
            swing: 0.0,
        },
    };
    Pose {
        swing: p.swing * mining,
        lean: p.lean * mining,
        sweep: p.sweep * mining,
        roll: p.roll * mining,
        thrust: p.thrust * mining,
        lift: p.lift * mining,
    }
}

/// How far the 3rd-person avatar's working arm is through a stroke: -1 raised/wound up, 0 rest, 1 struck.
pub(super) fn arm(held: ItemId, phase: f64) -> f64 {
    match stroke_of(held) {
        Stroke::Pick => {
            let s = pose(Stroke::Pick, phase, 1.0).swing;
            (if s < 0.0 { s / PICK_RAISE } else { s / PICK_STRIKE }).clamp(-1.0, 1.0)
        }
        Stroke::Axe => {
            let s = pose(Stroke::Axe, phase, 1.0).swing;
            (if s < 0.0 { s / AXE_RAISE } else { s / AXE_STRIKE }).clamp(-1.0, 1.0)
        }
        Stroke::Dig => (pose(Stroke::Dig, phase, 1.0).thrust / DIG_STAB).clamp(-1.0, 1.0),
        Stroke::Sweep => (-pose(Stroke::Sweep, phase, 1.0).sweep / SWEEP_ACROSS).clamp(-1.0, 1.0),
        Stroke::Aim => (pose(Stroke::Aim, phase, 1.0).thrust / 0.04).clamp(-1.0, 1.0),
        Stroke::Punch => (pose(Stroke::Punch, phase, 1.0).thrust / PUNCH_THRUST).clamp(-1.0, 1.0),
    }
}

/// Pickaxe swing angles (radians) and hand motion (blocks on an avatar).
const PICK_RAISE: f64 = 0.88;
const PICK_STRIKE: f64 = 0.72;
const PICK_RECOIL: f64 = 0.54;
const PICK_LIFT_UP: f64 = 0.058;
const PICK_LIFT_DOWN: f64 = 0.052;
const PICK_THRUST: f64 = 0.095;

/// Axe diagonal chop angles (radians): wider lateral sweep and diagonal blade roll.
const AXE_RAISE: f64 = 0.74;
const AXE_STRIKE: f64 = 0.58;
const AXE_RECOIL: f64 = 0.44;
const AXE_SWEEP_OUT: f64 = 0.34;
const AXE_SWEEP_IN: f64 = 0.42;
const AXE_ROLL: f64 = 0.56;

/// Shovel stab distance (blocks on an avatar) and scoop-lever angle (radians).
const DIG_STAB: f64 = 0.14;
const DIG_PRISE: f64 = 0.44;

/// Sickle lateral draw and slicing cut (radians).
const SWEEP_OUT: f64 = 0.48;
const SWEEP_ACROSS: f64 = 0.64;

/// Bare-hand / non-tool strike forward drive (blocks on an avatar).
const PUNCH_THRUST: f64 = 0.15;

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
