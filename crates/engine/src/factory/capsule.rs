//! The drop pod in flight (presentation only; `pods.rs` owns where it is): a white crew-capsule shape, a frustum on a
//! charred heat shield with a short collar and a domed cap, four thruster pods round its waist firing canted plumes
//! (`plume.rs`) as it rises out of its launcher or brakes onto its pad, and four landing legs. `drone_view.rs` draws one
//! for every pod in sight.
//!
//! To change the shape: the constants (blocks, from the feet up) and the rows of [`push_capsule`].

use std::f32::consts::{FRAC_PI_2, FRAC_PI_4};

use crate::block::tex;
use crate::math::Vec3;

use super::plume::push_plume;
use super::render::{push_box, push_cone, push_shape, Form};

/// The heat shield's height over the feet and its width; the body's height and top width.
const SHIELD_AT: f32 = 0.18;
const SHIELD: f32 = 1.5;
const BODY: f32 = 1.0;
const BODY_TOP: f32 = 0.86;
/// The thrusters: how far out and up their nozzles sit, their cant outwards (radians), plume width and length.
const NOZZLE_OUT: f32 = 0.74;
const NOZZLE_UP: f32 = 0.36;
const CANT: f32 = 0.35;
const PLUME_WIDE: f32 = 0.24;
const PLUME_LONG: f32 = 2.2;

/// Pushes a capsule whose feet stand at camera-relative `at`, its thrusters at `thrust` (0 off to 1 full); `seed` (0..1)
/// staggers the flicker of pods side by side.
pub(crate) fn push_capsule(out: &mut Vec<f32>, at: Vec3, thrust: f32, time: f64, seed: f64) {
    let up = |y: f32| at + Vec3::new(0.0, f64::from(y), 0.0);
    let hull = [tex::CAPSULE_HULL; 3];
    let base = SHIELD_AT + 0.14;
    push_cone(out, up(SHIELD_AT + 0.07), 0.0, 0.0, 0.14, SHIELD, SHIELD, [tex::CAPSULE_SHIELD; 3]);
    push_cone(out, up(base + BODY / 2.0), 0.0, 0.0, BODY, BODY_TOP, SHIELD - 0.02, hull);
    push_cone(out, up(base + BODY + 0.09), 0.0, 0.0, 0.18, BODY_TOP - 0.04, BODY_TOP, hull);
    push_cone(out, up(base + BODY + 0.29), 0.0, 0.0, 0.22, 0.48, BODY_TOP - 0.04, hull);
    for k in 0..4 {
        // Thruster pods at the diagonals, legs square to the sides; each yaw turns a lean outwards along its own angle.
        let a = k as f32 * FRAC_PI_2 + FRAC_PI_4;
        let (c, s) = (a.cos(), a.sin());
        let outward = c.atan2(-s);
        let pod = at + Vec3::new(f64::from(c * 0.66), f64::from(base + 0.26), f64::from(s * 0.66));
        push_box(out, pod, a, [0.16, 0.42, 0.26], 0.0, hull, false);
        let nozzle = at + Vec3::new(f64::from(c * NOZZLE_OUT), f64::from(NOZZLE_UP), f64::from(s * NOZZLE_OUT));
        if thrust > 0.0 {
            let (wide, long) = (PLUME_WIDE * thrust.sqrt(), PLUME_LONG * thrust);
            push_plume(out, nozzle, outward, CANT, wide, long, time, seed + k as f64 * 0.25);
        }
        let b = a - FRAC_PI_4;
        let (c, s) = (b.cos(), b.sin());
        let leg = at + Vec3::new(f64::from(c * 0.76), 0.2, f64::from(s * 0.76));
        push_shape(out, Form::Lean(0.85), leg, c.atan2(-s), [0.07, 0.5, 0.07], [tex::FRAME; 3], false);
        let foot = at + Vec3::new(f64::from(c * 0.92), 0.03, f64::from(s * 0.92));
        push_box(out, foot, b, [0.2, 0.06, 0.2], 0.0, [tex::FRAME; 3], false);
    }
}
