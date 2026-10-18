//! The shape and the look of water for the mesher (presentation only: nothing here is core state).
//!
//! A water block's surface is not a flat plane at its level: each top corner gets a height in sixteenths of a
//! block (`corner_drop` reads the four cells around the corner, like Minecraft), so a spreading flow thins out
//! smoothly and neighbouring cells share their edge heights. That is also why a face between two liquid cells
//! is never needed: the surface is continuous, and nothing can be seen through a step.
//!
//! A cell's own height (`height`): a source 15/16, flowing water 13/16 for level 7 down to 1/16 for level 1,
//! and a full block where more water lies above (falling water, the depths of a pool).
//!
//! Every liquid vertex carries, instead of AO and a texture layer (all water uses one tile, `u_waterLayer`):
//! bits 21..22 the low two bits of its `drop` (sixteenths below the top of its block, 0..15),
//! bits 23..24 the `kind` of water, bits 25..27 its flow direction (`dir`, eighths of a turn from +X towards
//! +Z), bits 28..29 the high two bits of the drop. The shader (`render/shaders.ts`, `WATER`) reads them.
//!
//! Invariants: only reads the padded cells around `p` (one cell of border), so a chunk meshes alone.
//! To add a kind of water look: a `KIND_*` below, its rule in `look`, its case in the shader.

use super::{P, P2};
use crate::block::{flow_level, BlockId, LIQUID, OPAQUE};

/// Still water: no slope, nothing falls in.
pub const KIND_STILL: u32 = 0;
/// Water running down a gentle slope: it drifts along `dir`.
pub const KIND_LAMINAR: u32 = 1;
/// Rapids and splashes: a steep drop, or beside a waterfall's foot. White water.
pub const KIND_TURBULENT: u32 = 2;
/// A waterfall: flowing water with water above it.
pub const KIND_FALLING: u32 = 3;

/// A block in sixteenths.
const FULL: i32 = 16;
/// Heights from this weigh more in a corner's average, so a pool's surface stays level out to its edge.
const DEEP: i32 = 13;
const DEEP_WEIGHT: i32 = 8;
/// A drop (the steepest slope across a cell) of at least this many sixteenths is rapids.
const STEEP: i32 = 9;

/// The surface height of liquid block `b` in sixteenths of a block when open to the air above it.
pub fn height(b: BlockId) -> u32 {
    match flow_level(b) {
        None => 15,
        Some(level) => 2 * u32::from(level) - 1,
    }
}

/// How far below the top of its block the surface corner (`hx`, `hz` each ±1, the corner's side of cell `p`) sits,
/// in sixteenths (0..=15): the average height of the cells sharing the corner, deep water and falling water
/// pulling it up and air pulling it down, with walls leaving it alone.
pub(super) fn corner_drop(pad: &[BlockId], p: usize, hx: isize, hz: isize) -> u32 {
    let at = |dx: isize, dz: isize| (p as isize + dx + dz * P as isize) as usize;
    let (mut sum, mut weight) = (0, 0);
    for c in [p, at(hx, 0), at(0, hz), at(hx, hz)] {
        let b = pad[c];
        if OPAQUE[b as usize] {
            continue;
        }
        if !LIQUID[b as usize] {
            weight += 1;
        } else if LIQUID[pad[c + P2] as usize] {
            return 0;
        } else {
            let h = height(b) as i32;
            let w = if h >= DEEP { DEEP_WEIGHT } else { 1 };
            sum += h * w;
            weight += w;
        }
    }
    (FULL - ((sum + weight / 2) / weight).max(1)) as u32
}

/// The tile data of the liquid cell `p`: its kind and flow direction, packed as `kind | dir << 2`.
pub(super) fn look(pad: &[BlockId], p: usize) -> u32 {
    if falls(pad, p) {
        return KIND_FALLING;
    }
    let sides = [1, -1, P as isize, -(P as isize)].map(|o| (p as isize + o) as usize);
    let [east, west, south, north] = sides.map(|s| cell_height(pad, s, p));
    let (gx, gz) = (west - east, north - south); // towards the lower side
    let splash = sides.iter().any(|&s| falls(pad, s));
    let kind = match (gx.abs() + gz.abs(), splash) {
        (0, false) => KIND_STILL,
        (m, s) if s || m >= STEEP => KIND_TURBULENT,
        _ => KIND_LAMINAR,
    };
    kind | direction(gx, gz) << 2
}

/// The height of cell `c` as a slope sees it from cell `from`: water its height (a full block if water lies above),
/// a wall or air resting on something the same as `from` (no slope), air over a drop nothing.
fn cell_height(pad: &[BlockId], c: usize, from: usize) -> i32 {
    let b = pad[c];
    if LIQUID[b as usize] {
        return if LIQUID[pad[c + P2] as usize] { FULL } else { height(b) as i32 };
    }
    if OPAQUE[b as usize] || OPAQUE[pad[c - P2] as usize] {
        return cell_height(pad, from, from);
    }
    0
}

/// Flowing water with water above it: one cell of a waterfall.
fn falls(pad: &[BlockId], p: usize) -> bool {
    let b = pad[p];
    LIQUID[b as usize] && flow_level(b).is_some() && LIQUID[pad[p + P2] as usize]
}

/// The direction of a slope as one of eight turns (0 +X, 2 +Z, 4 -X, 6 -Z, odd ones between); 0 when flat.
fn direction(gx: i32, gz: i32) -> u32 {
    let (ax, az) = (gx.abs(), gz.abs());
    match (gx > 0, gz > 0) {
        _ if ax == 0 && az == 0 => 0,
        _ if 2 * az <= ax => {
            if gx > 0 {
                0
            } else {
                4
            }
        }
        _ if 2 * ax <= az => {
            if gz > 0 {
                2
            } else {
                6
            }
        }
        (true, true) => 1,
        (false, true) => 3,
        (false, false) => 5,
        (true, false) => 7,
    }
}

#[cfg(test)]
mod tests;
