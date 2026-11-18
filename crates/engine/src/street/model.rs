//! The street lamp's model: a slim round steel post on a concrete footing and a banded base with a door plate, its top
//! bending over in a gooseneck (round segments along a half ellipse) down into a lantern: a shallow hood, a glowing warm
//! glass and a bottom plate with a finial. One instanced model per lamp, drawn every frame near the camera, so it stays
//! small ([`street_model`] pushes about twenty parts).
//!
//! Invariants: everything stays inside the 1×1×4 column (x and z within ±0.5 of `c`, y from -2, the ground, to 2;
//! `tests.rs` checks every dir); the glass (`tex::LAMP`) hangs at [`LANTERN_OUT`] along `DIRS[dir]` and [`LANTERN_Y`]
//! up, where the halo is; round things are round bodies (`push_round`, `push_cone`), never turned boxes.
//!
//! To change the look: the constants below (heights, widths, the arc), or a part in `street_model`. Points are given as
//! `(u, y)`: `u` along the arm's way from the column's middle, `y` up from it.

use std::f64::consts::PI;

use crate::block::tex;
use crate::factory::{mark_beacon_tinted, push_box, push_cone, push_round, Axis, DIRS};
use crate::math::Vec3;

use super::{LANTERN_OUT, LANTERN_Y};

/// Where the post stands: this far behind the column's middle (away from the arm), so the gooseneck has room to bend.
const POST_U: f64 = -0.1;
/// The top of the concrete footing, of the banded base and of the post (where the gooseneck starts).
const FOOTING_TOP: f64 = -1.86;
const BASE_TOP: f64 = -1.3;
const POST_TOP: f64 = 1.6;
/// The post's width at its foot and its top, and the gooseneck's at its start and its end.
const POST_WIDE: (f32, f32) = (0.15, 0.085);
const ARM_WIDE: (f32, f32) = (0.078, 0.06);
/// Segments of the gooseneck's half ellipse, and how far each runs past its ends so the bends close.
const ARC_STEPS: usize = 6;
const ARC_LAP: f64 = 0.012;
/// The gooseneck's height over the post's top (its half width is half the way from the post to the lantern).
const ARC_RISE: f64 = 0.2;
/// The lantern's hood: its foot and top.
const HOOD: (f64, f64) = (1.53, 1.62);
/// The glass's warm white, as a tinted beacon (it shows through the fog, as a lit street does at night).
const WARM: u32 = 0xffd9a0;

/// Draws the lamp whose column has its middle at `c` and whose arm leans toward `DIRS[dir]`.
pub fn street_model(out: &mut Vec<f32>, c: Vec3, dir: u8) {
    let way = DIRS[dir as usize % 4].as_vec3();
    let at = |u: f64, y: f64| c + way * u + Vec3::new(0.0, y, 0.0);
    let dark = [tex::MAT_GUNMETAL; 3];
    let steel = [tex::MAT_METAL; 3];
    let round = |out: &mut Vec<f32>, u: f64, y: f64, d: f32, h: f32, tex: [u16; 3]| {
        push_round(out, at(u, y), 0.0, Axis::Up, [d, h, d], 1.0, tex, false);
    };

    // The footing, a hair above the ground so its foot never shares the ground's plane.
    push_box(out, at(POST_U, -1.9305), 0.0, [0.42, 0.138, 0.42], 0.0, [tex::CONCRETE; 3], false);
    let base_len = BASE_TOP - FOOTING_TOP + 0.005;
    let base_mid = (BASE_TOP + FOOTING_TOP - 0.005) / 2.0;
    push_cone(out, at(POST_U, base_mid), 0.0, 0.0, base_len as f32, 0.2, 0.25, dark);
    round(out, POST_U, FOOTING_TOP + 0.022, 0.28, 0.044, steel);
    round(out, POST_U, BASE_TOP, 0.225, 0.036, steel);
    // The door plate on the base's back.
    let door_yaw = f32::from(dir % 4) * std::f32::consts::FRAC_PI_2;
    push_box(out, at(POST_U - 0.105, -1.6), door_yaw, [0.085, 0.2, 0.03], 0.0, steel, false);

    let post_len = POST_TOP - BASE_TOP;
    push_cone(out, at(POST_U, (POST_TOP + BASE_TOP) / 2.0), 0.0, 0.0, post_len as f32, POST_WIDE.1, POST_WIDE.0, dark);
    // Two bands up the post: one a little under the bend, a collar where the bend starts.
    let post_at = |y: f64| POST_WIDE.0 + (POST_WIDE.1 - POST_WIDE.0) * ((y - BASE_TOP) / post_len) as f32;
    round(out, POST_U, 1.1, post_at(1.1) + 0.024, 0.04, steel);
    round(out, POST_U, POST_TOP - 0.04, post_at(POST_TOP) + 0.022, 0.06, dark);

    // The gooseneck: a half ellipse from the post's top over to the lantern's hood, in round segments.
    let (mid, half) = ((POST_U + LANTERN_OUT) / 2.0, (LANTERN_OUT - POST_U) / 2.0);
    let arc = |t: f64| {
        let a = PI * (1.0 - t);
        (mid + half * a.cos(), POST_TOP + ARC_RISE * a.sin())
    };
    for i in 0..ARC_STEPS {
        let (t0, t1) = (i as f64 / ARC_STEPS as f64, (i + 1) as f64 / ARC_STEPS as f64);
        let (a, b) = (arc(t0), arc(t1));
        let (a, b) = (at(a.0, a.1), at(b.0, b.1));
        let d = b - a;
        let run = (d.x * d.x + d.z * d.z).sqrt();
        let len = (run * run + d.y * d.y).sqrt();
        let yaw = (-d.x).atan2(d.z) as f32;
        let wide = |t: f64| ARM_WIDE.0 + (ARM_WIDE.1 - ARM_WIDE.0) * t as f32;
        let lean = run.atan2(d.y) as f32;
        push_cone(out, (a + b) * 0.5, yaw, lean, (len + 2.0 * ARC_LAP) as f32, wide(t1), wide(t0), dark);
    }

    // The lantern: hood, its rim, the glowing glass, a bottom plate and a finial.
    let hood_len = (HOOD.1 - HOOD.0) as f32;
    push_cone(out, at(LANTERN_OUT, (HOOD.0 + HOOD.1) / 2.0), 0.0, 0.0, hood_len, 0.1, 0.34, dark);
    round(out, LANTERN_OUT, HOOD.0 - 0.006, 0.35, 0.022, steel);
    // The glass is centred on the lantern's place and runs up into the hood.
    let glass_len = 2.0 * (HOOD.0 + 0.004 - LANTERN_Y);
    let glass_foot = LANTERN_Y - glass_len / 2.0;
    push_cone(out, at(LANTERN_OUT, LANTERN_Y), 0.0, 0.0, glass_len as f32, 0.25, 0.19, [tex::LAMP; 3]);
    mark_beacon_tinted(out, WARM);
    round(out, LANTERN_OUT, glass_foot - 0.008, 0.2, 0.024, dark);
    push_cone(out, at(LANTERN_OUT, glass_foot - 0.04), 0.0, 0.0, 0.05, 0.06, 0.02, dark);
}
