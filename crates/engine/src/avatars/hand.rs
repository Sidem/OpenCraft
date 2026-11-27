//! Shared Kestrel forearm and articulated mechanical hand, used by both the 3rd-person avatar (`model::push_avatar`)
//! and the 1st-person POV arms (`view_hand.rs`) so the two match in silhouette, materials and colour accents.
//!
//! Hand-local axes (for `side = +1.0` right hand, mirrored by `side` for left):
//! `-z` points forward along the fingers, `+y` is the dorsal back of the hand, `-x * side` is the thumb/index edge
//! (which aligns with an upright tool handle's `+y` axis via `rz_m(-FRAC_PI_2)`). Every part is oriented through
//! `grip::M3` so the whole hand turns rigidly with the wrist and stays locked around a tool handle at any stroke angle.

use super::grip::{apply, euler, mul, rx_m, yaw_m, M3};
use crate::block::tex;
use crate::math::Vec3;

/// Center of the cylinder enclosed by the curled palm, fingers and thumb when gripping a tool handle (hand-local,
/// in avatar-scale blocks). Tool grips attach here so shafts pass right through the fist.
pub(super) const GRIP_POCKET: Vec3 = Vec3::new(0.0, -0.028, -0.062);

/// Finger and thumb curl state for the articulated hand.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum HandGrip {
    /// Relaxed open hand when empty-handed and idle.
    Open,
    /// Tightly clenched fist when mining/punching bare-handed.
    Fist,
    /// Fingers wrapped cleanly around a tool handle with the thumb braced over it.
    Tool,
    /// Gently cupped hand supporting a held block or non-tool item.
    Item,
}

/// One oriented box of a forearm or hand, in rig-local space (before rig yaw).
pub(super) struct HandPart {
    pub center: Vec3,
    pub size: [f32; 3],
    pub yaw: f64,
    pub pitch: f64,
    pub roll: f64,
    pub taper: f32,
    pub layer: u16,
}

/// Builds the Kestrel forearm from `elbow` to `wrist`: petrol-teal under-sleeve, tapered ivory ceramic gauntlet,
/// amber outer armor rib, sleek graphite wrist collar, wrist joint ball and gold telemetry pip.
pub(super) fn forearm_parts(elbow: Vec3, wrist: Vec3, dorsal_hint: Vec3, scale: f64) -> Vec<HandPart> {
    let delta = wrist - elbow;
    let len = delta.length().max(1e-5);
    let y_axis = delta * (1.0 / len); // +Y runs from elbow toward wrist.
    let proj = dorsal_hint - y_axis * dot(dorsal_hint, y_axis);
    let x_axis = if proj.length() > 1e-4 {
        proj * (1.0 / proj.length())
    } else {
        let fallback = if y_axis.x.abs() < 0.9 { Vec3::new(1.0, 0.0, 0.0) } else { Vec3::new(0.0, 0.0, 1.0) };
        let f = fallback - y_axis * dot(fallback, y_axis);
        f * (1.0 / f.length())
    };
    let z_axis = cross(x_axis, y_axis);
    let basis = cols(x_axis, y_axis, z_axis);
    let (yaw, roll, pitch) = euler(basis);
    let mut out = Vec::with_capacity(6);
    let s = scale;
    let mut add = |local: Vec3, size: [f64; 3], taper: f32, layer: u16| {
        out.push(HandPart {
            center: elbow + apply(basis, local),
            size: size.map(|v| v as f32),
            yaw,
            pitch,
            roll,
            taper,
            layer,
        });
    };
    // Petrol-teal inner sleeve connecting from upper arm into the wrist.
    add(Vec3::new(0.0, len * 0.48, 0.0), [0.098 * s, len * 0.94, 0.102 * s], 0.88, tex::AVATAR_SUIT);
    // Sculpted ivory ceramic gauntlet covering the forearm (matches torso & shin armor).
    add(Vec3::new(0.008 * s, len * 0.47, 0.0), [0.122 * s, len * 0.68, 0.128 * s], 0.83, tex::AVATAR_SKIN);
    // Amber outer gauntlet guard ridge along the dorsal face.
    add(Vec3::new(0.062 * s, len * 0.48, 0.0), [0.022 * s, len * 0.50, 0.054 * s], 0.88, tex::AVATAR_HELMET);
    // Sleek graphite wrist collar seated near the end of the gauntlet.
    add(Vec3::new(0.0, len * 0.88, 0.0), [0.106 * s, (0.034 * s).min(len * 0.20), 0.112 * s], 0.92, tex::AVATAR_JOINT);
    // Rounded graphite wrist joint bridging seamlessly into the heel of the palm at any wrist angle.
    add(Vec3::new(0.0, len * 0.98, 0.0), [0.084 * s, (0.062 * s).min(len * 0.25), 0.088 * s], 0.95, tex::AVATAR_JOINT);
    // Small glowing gold telemetry indicator on the outer wrist cuff.
    add(
        Vec3::new(0.056 * s, len * 0.86, 0.0),
        [0.016 * s, (0.022 * s).min(len * 0.14), 0.036 * s],
        1.0,
        tex::AVATAR_TRIM,
    );
    out
}

/// Builds the modelled Kestrel mechanical hand anchored at `wrist` with orientation `basis` (columns are hand local
/// `+X`, `+Y` dorsal, `+Z` toward wrist; fingers extend along `-Z`), scaled by `scale`, for `side` (`+1.0` right,
/// `-1.0` left) in pose `grip`.
pub(super) fn hand_parts(wrist: Vec3, basis: M3, scale: f64, side: f64, grip: HandGrip) -> Vec<HandPart> {
    let mut out = Vec::with_capacity(13);
    let s = scale;
    let push_local = |out: &mut Vec<HandPart>, pos: Vec3, local_m: M3, size: [f64; 3], taper: f32, layer: u16| {
        let (yaw, roll, pitch) = euler(mul(basis, local_m));
        out.push(HandPart {
            center: wrist + apply(basis, pos * s),
            size: size.map(|v| (v * s) as f32),
            yaw,
            pitch,
            roll,
            taper,
            layer,
        });
    };
    let id = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];
    // Mechanical graphite palm chassis, seated directly into the wrist ball at z = 0.
    push_local(&mut out, Vec3::new(0.0, -0.008, -0.042), id, [0.106, 0.046, 0.088], 0.92, tex::AVATAR_JOINT);
    // Sculpted ivory ceramic dorsal plate over the back of the hand.
    push_local(&mut out, Vec3::new(0.0, 0.018, -0.040), id, [0.110, 0.024, 0.080], 0.90, tex::AVATAR_SKIN);
    // Warm amber knuckle guard bar across the metacarpals.
    push_local(&mut out, Vec3::new(0.0, 0.015, -0.082), id, [0.114, 0.026, 0.022], 1.0, tex::AVATAR_HELMET);

    // Articulated 2-segment thumb on the radial (-x * side) edge.
    let (thumb_yaw, thumb_c1, thumb_c2) = match grip {
        HandGrip::Open => (0.46, 0.20, 0.26),
        HandGrip::Item => (0.52, 0.32, 0.34),
        HandGrip::Tool => (0.64, 0.56, 0.72),
        HandGrip::Fist => (0.98, 0.78, 0.88),
    };
    let thumb_base = Vec3::new(-side * 0.054, -0.006, -0.030);
    let m1 = mul(yaw_m(side * thumb_yaw), rx_m(-thumb_c1));
    let t_len1 = 0.040;
    let t_len2 = 0.034;
    let thumb_mid = thumb_base + apply(m1, Vec3::new(0.0, 0.0, -t_len1));
    push_local(
        &mut out,
        thumb_base + apply(m1, Vec3::new(0.0, 0.0, -t_len1 * 0.5)),
        m1,
        [0.027, 0.026, t_len1],
        0.92,
        tex::AVATAR_JOINT,
    );
    let m2 = mul(m1, rx_m(-thumb_c2));
    push_local(
        &mut out,
        thumb_mid + apply(m2, Vec3::new(0.0, 0.0, -t_len2 * 0.5)),
        m2,
        [0.024, 0.023, t_len2],
        0.88,
        tex::AVATAR_SKIN,
    );

    // Four articulated fingers (Index, Middle, Ring, Pinky), staggered in length and curl.
    let (base_c1, base_c2) = match grip {
        HandGrip::Open => (0.22, 0.30),
        HandGrip::Item => (0.42, 0.38),
        HandGrip::Tool => (1.08, 1.18),
        HandGrip::Fist => (1.42, 1.48),
    };
    const FINGERS: [(f64, f64, f64, f64); 4] = [
        (-0.040, 0.042, 0.035, 0.00), // Index
        (-0.013, 0.046, 0.038, 0.04), // Middle
        (0.014, 0.042, 0.035, 0.08),  // Ring
        (0.040, 0.034, 0.029, 0.14),  // Pinky
    ];
    for (fx, len1, len2, extra) in FINGERS {
        let knuckle = Vec3::new(side * fx, 0.004, -0.084);
        let f1 = rx_m(-(base_c1 + extra));
        let joint = knuckle + apply(f1, Vec3::new(0.0, 0.0, -len1));
        push_local(
            &mut out,
            knuckle + apply(f1, Vec3::new(0.0, 0.0, -len1 * 0.5)),
            f1,
            [0.022, 0.023, len1],
            0.94,
            tex::AVATAR_JOINT,
        );
        let f2 = rx_m(-(base_c1 + base_c2 + extra));
        push_local(
            &mut out,
            joint + apply(f2, Vec3::new(0.0, 0.0, -len2 * 0.5)),
            f2,
            [0.020, 0.020, len2],
            0.88,
            tex::AVATAR_SKIN,
        );
    }
    out
}

/// Builds an orthonormal right-handed rotation matrix from column vectors `(x, y, z)`.
pub(super) fn cols(x: Vec3, y: Vec3, z: Vec3) -> M3 {
    [[x.x, y.x, z.x], [x.y, y.y, z.y], [x.z, y.z, z.z]]
}

fn dot(a: Vec3, b: Vec3) -> f64 {
    a.x * b.x + a.y * b.y + a.z * b.z
}

fn cross(a: Vec3, b: Vec3) -> Vec3 {
    Vec3::new(a.y * b.z - a.z * b.y, a.z * b.x - a.x * b.z, a.x * b.y - a.y * b.x)
}
