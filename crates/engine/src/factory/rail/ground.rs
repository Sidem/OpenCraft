//! Where track may lie: on the ground, never in the air or through it (user, 2026-10-30: strict ground, no bridges).
//! A rail node needs a solid block under it (`node_stands`). A track is checked every `STEP` blocks along its curve
//! (`faults`): the ground's top must be within `MAX_DROP` below the bed (the height a level track's bed has on its
//! node's ground) and at most `MAX_SINK` above it, so a ramp may cross the voxel steps under it, and the cells a car
//! passes through (`CLEARANCE`) must be empty (air or a rail piece: a node, a dock). Pure helpers over a block query, so the
//! core (`Sim::connect`, with `block_anywhere_or_generate`) and the hand's preview (`rail_tools.rs`, loaded chunks)
//! share one rule. The renderer draws ballast from the bed down to `MAX_DROP` (`curve::write_track`), so every
//! stretch the rule allows reaches the ground. Older saves keep the track they have; the rule applies to new track.

use crate::block::{BlockId, AIR, RAIL, SOLID};
use crate::math::{IVec3, Vec3};

use super::curve::Curve;

/// The most the ground may lie below a track's bed, and the most it may rise into it (blocks).
pub const MAX_DROP: f64 = 1.0;
pub const MAX_SINK: f64 = 0.25;
/// Heights above the bed that a car passes through: its wheels, body and roof.
const CLEARANCE: [f64; 3] = [MAX_SINK, 1.0, 1.75];
/// Distance between the checked points along a track (blocks).
const STEP: f64 = 0.25;
/// How closely a column crossing is narrowed down, in curve parameter (a 32-block track: under a hundredth of a block).
const CROSSING: f64 = 1.0 / 4096.0;
const UP: IVec3 = IVec3::new(0, 1, 0);

/// What is wrong with a point of a track.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Fault {
    /// No ground within `MAX_DROP` under it.
    Floating,
    /// Ground or something else fills the space a car runs through.
    Buried,
}

/// Whether a rail node at `pos` stands on something solid.
pub fn node_stands(mut block: impl FnMut(IVec3) -> BlockId, pos: IVec3) -> bool {
    SOLID[block(pos - UP) as usize]
}

/// Every fault along `curve`, in order, each cell once: the cell in the way (`Buried`) or the cell to fill so the
/// ground carries the track (`Floating`). Points are checked every `STEP` and just either side of every place the
/// track crosses into another column, so no column it passes over is missed.
pub fn faults(curve: &Curve, mut block: impl FnMut(IVec3) -> BlockId) -> Vec<(IVec3, Fault)> {
    let n = ((curve.length() / STEP).ceil() as usize).max(1);
    let mut out: Vec<(IVec3, Fault)> = Vec::new();
    let mut note = |p: Vec3| {
        if let Some(f) = fault_at(p, &mut block).filter(|f| !out.contains(f)) {
            out.push(f);
        }
    };
    let column = |p: Vec3| (p.x.floor(), p.z.floor());
    let mut prev = 0.0;
    note(curve.at(0.0));
    for i in 1..=n {
        let t = i as f64 / n as f64;
        if column(curve.at(prev)) != column(curve.at(t)) {
            // Narrow down to the crossing (one at a time: a step crosses at most a column edge on each axis).
            let (mut lo, mut hi) = (prev, t);
            while column(curve.at(lo)) != column(curve.at(t)) && hi - lo > CROSSING {
                let (mut a, mut b) = (lo, hi);
                while b - a > CROSSING {
                    let mid = (a + b) / 2.0;
                    if column(curve.at(mid)) == column(curve.at(lo)) {
                        a = mid;
                    } else {
                        b = mid;
                    }
                }
                note(curve.at(a));
                note(curve.at(b));
                (lo, hi) = (b, t);
            }
        }
        note(curve.at(t));
        prev = t;
    }
    out
}

/// What is wrong at the point `p` of a track (its centreline, a node's cell centre high), if anything.
fn fault_at(p: Vec3, block: &mut impl FnMut(IVec3) -> BlockId) -> Option<(IVec3, Fault)> {
    // The bed: the bottom of the cell a level track's node stands in.
    let bed = p.y - 0.5;
    let (x, z) = (p.x.floor() as i32, p.z.floor() as i32);
    let cell = |y: f64| IVec3::new(x, y.floor() as i32, z);
    if let Some(c) = CLEARANCE.iter().map(|&h| cell(bed + h)).find(|&c| !passable(block(c))) {
        return Some((c, Fault::Buried));
    }
    // Ground tops are whole numbers: the highest one allowed, and every solid block down to `MAX_DROP`.
    let top = (bed + MAX_SINK).floor() as i32;
    let lowest = (bed - MAX_DROP).ceil() as i32;
    let grounded = (lowest..=top).rev().any(|t| SOLID[block(IVec3::new(x, t - 1, z)) as usize]);
    // The cell to fill: the highest whose top would carry the track.
    (!grounded).then_some((IVec3::new(x, top - 1, z), Fault::Floating))
}
/// Whether a car runs through a cell holding `b`: empty, or a rail piece (a node, a dock).
fn passable(b: BlockId) -> bool {
    b == AIR || b == RAIL || super::is_dock(b)
}

#[cfg(test)]
mod tests;
