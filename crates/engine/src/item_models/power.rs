//! Item models of the power poles, so an icon reads as a pole and not a striped cube: a post with a crossarm and two
//! insulators (a tier wears its stripe near the foot, as the placed pole does: `factory/pole.rs`), and the
//! long-distance pole as a small lattice transmission tower. Add a const and a `parts` row.

use std::f32::consts::{FRAC_PI_2, FRAC_PI_4};

use super::shapes::cylinder;
use super::{part, Form, Part};
use crate::block::{tex, LONG_POLE, POLE};
use crate::item::{ItemId, POLE_MK2, POLE_MK3, POLE_MK4};

const ITEM_POLE: ItemId = ItemId::block(POLE);
const ITEM_LONG_POLE: ItemId = ItemId::block(LONG_POLE);

/// A post on a footing with a crossarm: back to front for the icon, the tier's band round the post's foot.
const fn pole(tier: u8) -> [Part; 8] {
    let frame = tex::FRAME;
    let band = if tier == 0 { frame } else { tex::stripe(tier) };
    [
        part([0.0, -0.46, 0.0], [0.5, 0.08, 0.5], frame),
        part([0.0, 0.0, 0.0], [0.14, 0.92, 0.14], frame),
        part([0.0, -0.3, 0.0], [0.2, 0.16, 0.2], band),
        part([0.0, 0.3, 0.0], [0.86, 0.08, 0.12], frame),
        part([-0.34, 0.41, 0.0], [0.12, 0.14, 0.12], tex::FLASK_GLASS),
        part([0.34, 0.41, 0.0], [0.12, 0.14, 0.12], tex::FLASK_GLASS),
        part([-0.34, 0.475, 0.0], [0.08, 0.04, 0.08], tex::COPPER_WIRE),
        part([0.34, 0.475, 0.0], [0.08, 0.04, 0.08], tex::COPPER_WIRE),
    ]
}

const POLE_MK1_MODEL: [Part; 8] = pole(0);
const POLE_MK2_MODEL: [Part; 8] = pole(1);
const POLE_MK3_MODEL: [Part; 8] = pole(2);
const POLE_MK4_MODEL: [Part; 8] = pole(3);

/// A lattice transmission tower, as `factory/pole_tower.rs` builds the placed one at a quarter of its size: legs on
/// footings leaning in to the waist, X-braced, two crossarms with insulators at their tips and an earth-wire peak.
/// Listed back to front for the icon (it has no depth test).
const LONG_MODEL: [Part; 34] = [
    part([-0.2009, -0.4728, -0.2009], [0.08, 0.035, 0.08], tex::CONCRETE),
    part([0.2009, -0.4728, -0.2009], [0.08, 0.035, 0.08], tex::CONCRETE),
    part([-0.2009, -0.4728, 0.2009], [0.08, 0.035, 0.08], tex::CONCRETE),
    lean([-0.1372, -0.1262, -0.1372], [0.05, 0.7427, 0.05], -FRAC_PI_4, 0.2631),
    lean([-0.0637, -0.1262, -0.1372], [0.025, 0.7591, 0.025], -1.1361, 0.4248),
    lean([-0.1372, -0.1262, -0.0637], [0.025, 0.7591, 0.025], -0.4347, 0.4248),
    lean([0.0637, -0.1262, -0.1372], [0.025, 0.7591, 0.025], 1.1361, 0.4248),
    lean([-0.1372, -0.1262, 0.0637], [0.025, 0.7591, 0.025], -2.7069, 0.4248),
    lean([0.1372, -0.1262, -0.1372], [0.05, 0.7427, 0.05], FRAC_PI_4, 0.2631),
    lean([-0.1372, -0.1262, 0.1372], [0.05, 0.7427, 0.05], -3.0 * FRAC_PI_4, 0.2631),
    part([0.2009, -0.4728, 0.2009], [0.08, 0.035, 0.08], tex::CONCRETE),
    cylinder([-0.2254, 0.1641, 0.0], 0.04, 0.07, tex::FLASK_GLASS),
    lean([-0.0637, -0.1262, 0.1372], [0.025, 0.7591, 0.025], -2.0055, 0.4248),
    lean([0.1372, -0.1262, -0.0637], [0.025, 0.7591, 0.025], 0.4347, 0.4248),
    lean([0.0637, -0.1262, 0.1372], [0.025, 0.7591, 0.025], 2.0055, 0.4248),
    lean([0.1372, -0.1262, 0.0637], [0.025, 0.7591, 0.025], 2.7069, 0.4248),
    lean([-0.1452, 0.2475, 0.0], [0.02, 0.1944, 0.02], FRAC_PI_2, 1.9751),
    cylinder([-0.1715, 0.3111, 0.0], 0.04, 0.07, tex::FLASK_GLASS),
    lean([0.1372, -0.1262, 0.1372], [0.05, 0.7427, 0.05], 3.0 * FRAC_PI_4, 0.2631),
    lean([-0.0612, 0.3148, -0.0612], [0.04, 0.2559, 0.04], -FRAC_PI_4, 0.1611),
    part([0.0, 0.2082, 0.0], [0.4508, 0.04, 0.04], tex::FRAME),
    lean([-0.1098, 0.3944, 0.0], [0.02, 0.1611, 0.02], FRAC_PI_2, 2.0784),
    lean([0.0612, 0.3148, -0.0612], [0.04, 0.2559, 0.04], FRAC_PI_4, 0.1611),
    lean([-0.0612, 0.3148, 0.0612], [0.04, 0.2559, 0.04], -3.0 * FRAC_PI_4, 0.1611),
    part([0.0, 0.3553, 0.0], [0.343, 0.04, 0.04], tex::FRAME),
    cylinder([0.2254, 0.1641, 0.0], 0.04, 0.07, tex::FLASK_GLASS),
    lean([0.1452, 0.2475, 0.0], [0.02, 0.1944, 0.02], -FRAC_PI_2, 1.9751),
    lean([-0.0245, 0.452, -0.0245], [0.02, 0.1125, 0.02], -FRAC_PI_4, 0.847),
    lean([0.0612, 0.3148, 0.0612], [0.04, 0.2559, 0.04], 3.0 * FRAC_PI_4, 0.1611),
    lean([0.0245, 0.452, -0.0245], [0.02, 0.1125, 0.02], FRAC_PI_4, 0.847),
    lean([-0.0245, 0.452, 0.0245], [0.02, 0.1125, 0.02], -3.0 * FRAC_PI_4, 0.847),
    cylinder([0.1715, 0.3111, 0.0], 0.04, 0.07, tex::FLASK_GLASS),
    lean([0.0245, 0.452, 0.0245], [0.02, 0.1125, 0.02], 3.0 * FRAC_PI_4, 0.847),
    lean([0.1098, 0.3944, 0.0], [0.02, 0.1611, 0.02], -FRAC_PI_2, 2.0784),
];

/// A steel bar leaned by `pitch` after `yaw` turns it (`Form::Lean`).
const fn lean(center: [f32; 3], size: [f32; 3], yaw: f32, pitch: f32) -> Part {
    Part { center, size, yaw, tex: [tex::FRAME; 3], form: Form::Lean(pitch) }
}

pub(super) fn parts(item: ItemId) -> &'static [Part] {
    match item {
        ITEM_POLE => &POLE_MK1_MODEL,
        POLE_MK2 => &POLE_MK2_MODEL,
        POLE_MK3 => &POLE_MK3_MODEL,
        POLE_MK4 => &POLE_MK4_MODEL,
        ITEM_LONG_POLE => &LONG_MODEL,
        _ => &[],
    }
}
