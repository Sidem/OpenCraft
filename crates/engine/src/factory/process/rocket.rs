//! The rocket on a launch site (presentation only; `launch.rs` owns the batch): a two-stage stainless stack in the manner
//! of a modern super-heavy launcher. A booster with a ring of engine bells under a tiled skirt, four grid fins and a
//! vented staging ring, then the ship with its ogive nose and two pairs of black tiled flaps. `model.rs` places it on the
//! mount (a `Look::Rocket` part) and says how far it has climbed and how hard its engines burn; the plume and the cloud
//! of exhaust boiling off the pad come from `plume.rs` and `smoke.rs`.
//!
//! To change the shape: the constants (blocks; heights from the mount's top) and the rows of [`draw`].

use std::f32::consts::{FRAC_PI_2, FRAC_PI_4, TAU};

use crate::block::tex;
use crate::math::Vec3;

use super::super::plume::push_plume;
use super::super::render::{push_box, push_cone};
use super::super::smoke;
use super::model::local;

/// The stack's width, the engine plane's height over the mount, and the two stages' lengths.
const WIDE: f32 = 1.3;
const ENGINES: f32 = 0.32;
const BOOSTER: f32 = 7.0;
const RING: f32 = 0.3;
const SHIP: f32 = 4.6;
/// The ogive nose: (length, width at its top) per section, from the ship's top up.
const NOSE: [(f32, f32); 4] = [(0.7, 1.14), (0.7, 0.84), (0.6, 0.4), (0.35, 0.06)];
/// The plume at full thrust: its width at the nozzles, and its length off the pad and once climbing.
const PLUME_WIDE: f32 = 1.5;
const PLUME_PAD: f32 = 6.0;
const PLUME_CLIMB: f32 = 15.0;
/// Blocks of climb over which the cloud of exhaust round the pad thins away.
const CLOUD_CLIMB: f64 = 40.0;

/// Draws the rocket standing on camera-relative `base` (the mount's top centre), turned `yaw`, `climb` blocks up, its
/// engines at `thrust` (0 off to 1 full).
pub(super) fn draw(out: &mut Vec<f32>, base: Vec3, yaw: f32, climb: f64, thrust: f32, time: f64) {
    let at = |x: f32, y: f32, z: f32| local(base, yaw, [f64::from(x), f64::from(y) + climb, f64::from(z)]);
    let steel = [tex::ROCKET_STEEL; 3];
    let tiles = [tex::ROCKET_TILE; 3];
    let round = |out: &mut Vec<f32>, y: f32, len: f32, top: f32, bottom: f32, look: [u16; 3]| {
        push_cone(out, at(0.0, y + len / 2.0, 0.0), yaw, 0.0, len, top, bottom, look);
    };
    // Engine bells: one in the middle and a ring of six, under the booster's tiled skirt.
    for k in 0..7 {
        let a = k as f32 / 6.0 * TAU;
        let r = if k == 6 { 0.0 } else { 0.4 };
        let bell = at(r * a.cos(), ENGINES - 0.15, r * a.sin());
        push_cone(out, bell, yaw, 0.0, 0.3, 0.16, 0.3, [tex::FRAME, tex::ROCKET_ENGINE, tex::FRAME]);
    }
    round(out, ENGINES, 0.4, WIDE + 0.02, WIDE + 0.02, tiles);
    round(out, ENGINES + 0.4, BOOSTER - 0.4, WIDE, WIDE, steel);
    // Four grid fins near the booster's top at the diagonals: flat lattice panels, square in plan, standing out of the hull.
    for k in 0..4 {
        let a = k as f32 * FRAC_PI_2 + FRAC_PI_4;
        let r = WIDE / 2.0 + 0.16;
        let fin = at(r * a.cos(), ENGINES + BOOSTER - 0.7, r * a.sin());
        push_box(out, fin, yaw, [0.36, 0.1, 0.36], 0.0, [tex::FRAME; 3], false);
    }
    let ship = ENGINES + BOOSTER + RING;
    round(out, ENGINES + BOOSTER, RING, WIDE - 0.04, WIDE - 0.04, tiles);
    round(out, ship, SHIP, WIDE, WIDE, steel);
    let mut y = ship + SHIP;
    let mut width = WIDE;
    for (len, top) in NOSE {
        round(out, y, len, top, width, steel);
        (y, width) = (y + len, top);
    }
    // Flaps: a big pair at the ship's tail and a small pair under the nose, black tiles, on the stack's x axis.
    for side in [-1.0, 1.0] {
        let aft = at(side * (WIDE / 2.0 + 0.28), ship + 0.85, 0.0);
        push_box(out, aft, yaw, [0.56, 1.5, 0.09], 0.0, tiles, false);
        let fore = at(side * (WIDE / 2.0 + 0.18), ship + SHIP + 0.45, 0.0);
        push_box(out, fore, yaw, [0.36, 0.95, 0.07], 0.0, tiles, false);
    }
    if thrust <= 0.0 {
        return;
    }
    let length = thrust * (PLUME_PAD + (PLUME_CLIMB - PLUME_PAD) * (climb / 6.0).min(1.0) as f32);
    push_plume(out, at(0.0, ENGINES - 0.3, 0.0), yaw, 0.0, PLUME_WIDE * thrust.sqrt(), length, time, 0.3);
    // Exhaust boiling out from under the mount, thinning as the rocket climbs away.
    let cloud = (1.0 - climb / CLOUD_CLIMB) as f32 * thrust;
    if cloud > 0.05 {
        for k in 0..6 {
            let a = k as f32 / 6.0 * TAU;
            let puff = local(base, yaw, [f64::from(2.2 * a.cos()), -0.2, f64::from(2.2 * a.sin())]);
            smoke::puffs(out, puff, time, k as f64 / 6.0, 2.6 * cloud);
        }
    }
}
