//! Rails snap to straight (user, 2026-10-31: docks only go on north-south or east-west track, so straight track
//! should be the usual result of aiming). Pure helpers for the track hand (`rail_tools.rs`):
//!
//! - `first_heading`: the first node of a line faces the view, but an axis when the view is within `AXIS_STEPS` of
//!   one, and otherwise the nearest of the eight compass points, so a line starts on a direction.
//! - `snap_chord`: the next node, aimed within `SNAP_STEPS` (about 13 degrees) of a grid axis or of the selected
//!   node's own heading, moves onto that line. Onto an axis (which wins over a nearer heading line) the node
//!   takes the axis as its heading, so a line leaving a curve ends squarely on the grid and docks fit it; onto the
//!   heading's line it continues straight. Aim further off to bend.
//!
//! Headings are bytes, 256 steps round the compass (`factory/rail/curve.rs`).

use crate::factory::{dir_of, yaw_of};
use crate::math::IVec3;

/// How far a chord may be from a line to snap to it, in heading steps (9 steps: 12.7 degrees).
const SNAP_STEPS: i32 = 9;
/// How far a first node's heading may be from an axis to take it (18 steps: 25 degrees).
const AXIS_STEPS: i32 = 18;
/// Heading steps between grid axes and between compass points.
const AXIS: i32 = 64;
const COMPASS: i32 = 32;
/// Chords shorter than this (blocks on the longer axis) are too short to read a direction from.
const MIN_CHORD: i32 = 2;

/// A node moved onto a line: where it goes, and the heading it must take if the line is a grid axis.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Snapped {
    pub pos: IVec3,
    pub yaw: Option<u8>,
}

/// The heading of the first node of a line, from the view's heading.
pub fn first_heading(yaw: u8) -> u8 {
    let yaw = i32::from(yaw);
    let step = if turn(yaw, nearest(yaw, AXIS)) <= AXIS_STEPS { AXIS } else { COMPASS };
    nearest(yaw, step) as u8
}

/// The cell for a node aimed at `aim`, after the node at `a` heading `ya`: on the nearest grid axis or on the
/// heading's line through `a` if the aim lies within `SNAP_STEPS` of it (the same height; the caller settles it on
/// the ground), `None` if it lies in no line's reach.
pub fn snap_chord(a: IVec3, ya: u8, aim: IVec3) -> Option<Snapped> {
    let (cx, cz) = (aim.x - a.x, aim.z - a.z);
    if cx.abs().max(cz.abs()) < MIN_CHORD {
        return None;
    }
    let chord = i32::from(yaw_of(f64::from(cx), f64::from(cz)));
    // An axis within reach always wins, even over a nearer heading line: straight track on an axis takes docks.
    let axis = nearest(chord, AXIS);
    let line = [i32::from(ya), i32::from(ya) + 128].into_iter().min_by_key(|&h| turn(chord, h)).unwrap_or(axis);
    let (target, on_axis) = if turn(chord, axis) <= SNAP_STEPS { (axis, true) } else { (line % 256, false) };
    if turn(chord, target) > SNAP_STEPS {
        return None;
    }
    let (dx, dz) = dir_of(target as u8);
    let at =
        |n: i32| IVec3::new(a.x + (dx * f64::from(n)).round() as i32, aim.y, a.z + (dz * f64::from(n)).round() as i32);
    let reach = |p: IVec3| (p.x - a.x).pow(2) + (p.z - a.z).pow(2);
    // As far along the line as the aim was, never further (a snapped node at full reach must still be in reach).
    let mut along = (f64::from(cx) * dx + f64::from(cz) * dz).round() as i32;
    while along > 1 && reach(at(along)) > reach(aim) {
        along -= 1;
    }
    Some(Snapped { pos: at(along), yaw: on_axis.then_some(target as u8) })
}

/// The heading nearest `yaw` that is a multiple of `step` (0..256).
fn nearest(yaw: i32, step: i32) -> i32 {
    (yaw + step / 2).div_euclid(step) * step % 256
}

/// Steps between two headings, the short way round.
fn turn(a: i32, b: i32) -> i32 {
    let d = (a - b).rem_euclid(256);
    d.min(256 - d)
}

#[cfg(test)]
mod tests;
