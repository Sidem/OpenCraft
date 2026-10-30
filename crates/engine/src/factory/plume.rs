//! Rocket exhaust (presentation only): the flame under a lifting rocket (`process/rocket.rs`) and a drop pod's thrusters
//! (`capsule.rs`). A plume is a chain of [`CELLS`] cells, each flaring out of a pinch and narrowing into the next the way
//! an under-expanded jet does, with a pale shock diamond at every pinch (the Mach disks), a white-hot first cell, a cooling
//! tail that closes it, and a flicker that is a pure function of the time and a seed. Everything glows (`render.rs`
//! `GLOWING`), so it reads by day and lights nothing.
//!
//! To change the look: the share constants below (all relative to the plume's width and length).

use crate::block::tex;
use crate::math::Vec3;

use super::render::push_cone;

#[cfg(test)]
mod tests;

/// Cells in a plume, and the share of its length they fill (the tail takes the rest).
const CELLS: usize = 4;
const CELL_SHARE: f32 = 0.8;
/// Widths as shares of the nozzle's: the first pinch, the first flare, and how much narrower each next cell is.
const NECK: f32 = 0.56;
const FLARE: f32 = 1.0;
const SHRINK: f32 = 0.11;
/// A diamond's width as a share of its pinch, and its length as a share of a cell.
const DIAMOND_WIDE: f32 = 1.55;
const DIAMOND_LONG: f32 = 0.55;

/// Pushes a plume leaving a nozzle at camera-relative `nozzle`, pointing down the leaned axis (`push_cone`: `yaw`, then
/// `lean`), `width` across at the nozzle and `length` long; `seed` (0..1) staggers the flicker of neighbouring plumes.
#[allow(clippy::too_many_arguments)]
pub fn push_plume(
    out: &mut Vec<f32>,
    nozzle: Vec3,
    yaw: f32,
    lean: f32,
    width: f32,
    length: f32,
    time: f64,
    seed: f64,
) {
    if width <= 0.0 || length <= 0.0 {
        return;
    }
    let (sy, cy, sl, cl) = (yaw.sin(), yaw.cos(), lean.sin(), lean.cos());
    let down = Vec3::new(f64::from(sy * sl), -f64::from(cl), -f64::from(cy * sl));
    let at = |s: f32| nozzle + down * f64::from(s);
    let wobble = |k: f64, rate: f64| 1.0 + k as f32 * ((time * rate + seed * 37.0).sin() as f32);
    let cell = length * CELL_SHARE / CELLS as f32;
    let neck = |i: usize| width * (NECK - SHRINK * i as f32) * wobble(0.04, 41.0 + i as f64 * 7.0);
    let mut s = 0.0;
    for i in 0..CELLS {
        let len = cell * wobble(0.06, 29.0 + i as f64 * 5.0);
        let (from, flare, to) =
            (if i == 0 { width * 0.9 } else { neck(i) }, width * (FLARE - SHRINK * i as f32), neck(i + 1));
        let layer =
            [[tex::PLUME_CORE; 3], [tex::PLUME_FLAME; 3], [tex::PLUME_FLAME; 3], [tex::PLUME_TAIL; 3]][i.min(3)];
        push_cone(out, at(s + len / 4.0), yaw, lean, len / 2.0, from, flare, layer);
        push_cone(out, at(s + 3.0 * len / 4.0), yaw, lean, len / 2.0, flare, to, layer);
        s += len;
        // The diamond at the pinch closing this cell: two cones tip to tip, brighter and a little wider than the pinch.
        let (d, half) = (to * DIAMOND_WIDE, len * DIAMOND_LONG / 2.0);
        push_cone(out, at(s - half / 2.0), yaw, lean, half, d * 0.25, d, [tex::PLUME_DIAMOND; 3]);
        push_cone(out, at(s + half / 2.0), yaw, lean, half, d, d * 0.25, [tex::PLUME_DIAMOND; 3]);
    }
    let tail = (length - s).max(length * 0.12);
    push_cone(out, at(s + tail / 2.0), yaw, lean, tail, neck(CELLS), neck(CELLS) * 0.1, [tex::PLUME_TAIL; 3]);
}
