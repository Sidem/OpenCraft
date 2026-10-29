//! The long-distance pole's model (`pole.rs` stands it on `LONG_FOOTPRINT`): a lattice transmission tower 2×2×4. Four
//! legs on concrete footings lean in to a waist and run up a narrower body, X-braced on all four faces; two levels of
//! crossarms (the lower one wider) carry a string of insulator discs at each tip, and an earth-wire peak crowns it,
//! where the wire hangs (`TOP`). Members are boxes run between two points (`strut`). The grid-coloured sleeves are drawn
//! apart (`long_accent`) because the grid's colour is the factory's, not the pole's. All points are relative to the
//! footprint's centre, y from -2 (the ground) to 2.

use crate::block::tex;
use crate::math::Vec3;

use super::render::{push_box, push_round, Axis};

/// How high the earth-wire peak, where the wire hangs, stands above the footprint's centre.
pub const TOP: f64 = 1.95;

/// A leg's half-width at its foot, its waist (where the lower arms are) and the top of the body, and their heights.
const FOOT: (f64, f64) = (0.82, -1.88);
const WAIST: (f64, f64) = (0.3, 0.85);
const HEAD: (f64, f64) = (0.2, 1.72);
/// The crossarms: their height and how far out their tips reach (the lower ones further).
const ARMS: [(f64, f64); 2] = [(WAIST.1, 0.92), (1.45, 0.7)];
/// Where the X-bracing panels of the lower legs meet, between the foot and the waist.
const PANEL: f64 = -0.7;

const LEG: f32 = 0.09;
const BRACE: f32 = 0.045;

/// Draws the tower; `c` is its footprint's centre relative to the camera.
pub fn long_model(out: &mut Vec<f32>, c: Vec3) {
    let frame = [tex::FRAME; 3];
    for (sx, sz) in CORNERS {
        let foot = Vec3::new(sx * FOOT.0, -1.92, sz * FOOT.0);
        push_box(out, c + foot, 0.0, [0.34, 0.16, 0.34], 0.0, [tex::CONCRETE; 3], false);
    }
    for (sx, sz) in CORNERS {
        let at = |y: f64| {
            let w = half_width(y);
            c + Vec3::new(sx * w, y, sz * w)
        };
        strut(out, at(FOOT.1), at(WAIST.1), LEG, frame);
        strut(out, at(WAIST.1), at(HEAD.1), LEG * 0.8, frame);
        // The earth-wire peak.
        strut(out, at(HEAD.1), c + Vec3::new(0.0, TOP + 0.02, 0.0), BRACE, frame);
    }
    for panel in [(FOOT.1, PANEL), (PANEL, WAIST.1), (WAIST.1, HEAD.1)] {
        braces(out, c, panel, frame);
    }
    for y in [PANEL, WAIST.1, HEAD.1] {
        let w = half_width(y);
        for s in [-w, w] {
            push_box(out, c + Vec3::new(0.0, y, s), 0.0, [2.0 * w as f32, 0.05, 0.05], 0.0, frame, false);
            push_box(out, c + Vec3::new(s, y, 0.0), 0.0, [0.045, 0.045, 2.0 * w as f32], 0.0, frame, false);
        }
    }
    for (y, reach) in ARMS {
        let w = half_width(y);
        let rise = half_width(y + 0.3);
        for s in [-1.0, 1.0] {
            let tip = c + Vec3::new(s * reach, y, 0.0);
            // Two lower chords from the body's corners and one upper chord meet at the tip: a triangular truss.
            strut(out, c + Vec3::new(s * w, y, -w), tip, 0.06, frame);
            strut(out, c + Vec3::new(s * w, y, w), tip, 0.06, frame);
            strut(out, c + Vec3::new(s * rise, y + 0.3, 0.0), tip + Vec3::new(0.0, 0.02, 0.0), 0.05, frame);
            insulators(out, tip);
        }
    }
    push_box(out, c + Vec3::new(0.0, TOP, 0.0), 0.0, [0.07, 0.06, 0.07], 0.0, [tex::COPPER_WIRE; 3], false);
}

/// The tower's grid-coloured sleeves round the foot of each leg and the tip of each crossarm; `c` is its footprint's
/// centre relative to the camera.
pub fn long_accent(out: &mut Vec<f32>, c: Vec3, layer: u16) {
    let tex = [layer; 3];
    for (sx, sz) in CORNERS {
        let at = |y: f64| {
            let w = half_width(y);
            c + Vec3::new(sx * w, y, sz * w)
        };
        strut(out, at(-1.55), at(-1.3), 0.14, tex);
    }
    for (y, reach) in ARMS {
        for s in [-1.0, 1.0] {
            push_box(out, c + Vec3::new(s * (reach - 0.1), y, 0.0), 0.0, [0.1, 0.1, 0.12], 0.0, tex, false);
        }
    }
}

const CORNERS: [(f64, f64); 4] = [(-1.0, -1.0), (1.0, -1.0), (-1.0, 1.0), (1.0, 1.0)];

/// A leg's distance from the middle (along x and z) at height `y`: straight from the foot to the waist, then to the top.
fn half_width(y: f64) -> f64 {
    let lerp = |(w0, y0): (f64, f64), (w1, y1): (f64, f64)| w0 + (w1 - w0) * (y - y0) / (y1 - y0);
    if y <= WAIST.1 {
        lerp(FOOT, WAIST)
    } else {
        lerp(WAIST, HEAD)
    }
}

/// The X-braces between heights `y0` and `y1` on all four faces; one brace of each X sits a little inside the other
/// so their faces never share a plane.
fn braces(out: &mut Vec<f32>, c: Vec3, (y0, y1): (f64, f64), tex: [u16; 3]) {
    let (w0, w1) = (half_width(y0), half_width(y1));
    for face in 0..4 {
        // A point on the face: `u` across it (-1 to 1 of its width), at height `y` and width `w`, `inset` inwards.
        let on = |u: f64, y: f64, w: f64, inset: f64| {
            let (a, n) = (u * w, w - inset);
            c + match face {
                0 => Vec3::new(a, y, n),
                1 => Vec3::new(a, y, -n),
                2 => Vec3::new(n, y, a),
                _ => Vec3::new(-n, y, a),
            }
        };
        strut(out, on(-1.0, y0, w0, 0.0), on(1.0, y1, w1, 0.0), BRACE, tex);
        strut(out, on(1.0, y0, w0, 0.015), on(-1.0, y1, w1, 0.015), BRACE, tex);
    }
}

/// A string of three insulator discs on a rod hanging from crossarm tip `tip`, a copper clamp at its foot.
fn insulators(out: &mut Vec<f32>, tip: Vec3) {
    let down = |d: f64| tip - Vec3::new(0.0, d, 0.0);
    push_round(out, down(0.15), 0.0, Axis::Up, [0.025, 0.3, 0.025], 1.0, [tex::FRAME; 3], false);
    for d in [0.08, 0.15, 0.22] {
        push_round(out, down(d), 0.0, Axis::Up, [0.12, 0.035, 0.12], 1.0, [tex::FLASK_GLASS; 3], false);
    }
    push_box(out, down(0.31), 0.0, [0.07, 0.05, 0.05], 0.0, [tex::COPPER_WIRE; 3], false);
}

/// A square bar `t` thick from `a` to `b` (camera-relative), a little longer so joints close: a box turned by its yaw
/// and leaned by its pitch (the shader turns local up to (-sin yaw sin pitch, cos pitch, cos yaw sin pitch)).
fn strut(out: &mut Vec<f32>, a: Vec3, b: Vec3, t: f32, tex: [u16; 3]) {
    let d = b - a;
    let run = (d.x * d.x + d.z * d.z).sqrt();
    let len = (run * run + d.y * d.y).sqrt();
    let yaw = if run > 1e-9 { (-d.x).atan2(d.z) } else { 0.0 };
    push_box(out, (a + b) * 0.5, yaw as f32, [t, len as f32 + t, t], 0.0, tex, false);
    let at = out.len() - 4;
    out[at] = run.atan2(d.y) as f32;
}
