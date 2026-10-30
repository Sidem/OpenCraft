//! Icons of the plant machines, built after the real thing as `machines.rs` builds the rest: a pumpjack's nodding beam
//! and horsehead, a refinery's columns and flare, a centrifuge's rotor of tubes, an ore washer's drum over its trough,
//! a constructor's robot arm over its bench. Parts back to front (the viewer looks from +x and +z), floor at -0.5.

use std::f32::consts::{FRAC_PI_2, FRAC_PI_3, FRAC_PI_4, PI};

use super::machines::lean;
use super::shapes::{bar_x, cylinder, faced, frustum, Build};
use super::{part, turned, Axis, Form, Part};
use crate::block::{self, tex, BlockId};

/// The icon of the machine `block` places, if it is one of these.
pub(super) fn parts(block: BlockId) -> &'static [Part] {
    match block {
        block::PUMPJACK => &PUMPJACK,
        block::REFINERY => &REFINERY,
        block::CENTRIFUGE => &CENTRIFUGE,
        block::WASHER => &WASHER,
        block::CONSTRUCTOR => &CONSTRUCTOR,
        block::PUMP => &PUMP,
        _ => &[],
    }
}

/// A walking beam on its post, the horsehead at its front over the well, cranks and weights at its back.
const PUMPJACK: [Part; 14] = [
    part([0.0, -0.47, 0.0], [0.96, 0.06, 0.4], tex::CONCRETE),
    part([-0.3, -0.24, -0.12], [0.26, 0.24, 0.04], tex::MAT_GUNMETAL),
    part([-0.3, -0.37, 0.0], [0.18, 0.14, 0.18], tex::MOTOR),
    part([-0.33, -0.03, -0.1], [0.03, 0.4, 0.03], tex::STEEL),
    lean([0.0, -0.14, -0.1], [0.05, 0.62, 0.05], 0.0, 0.18, tex::FRAME),
    lean([-0.12, -0.16, 0.0], [0.05, 0.6, 0.05], -FRAC_PI_2, 0.38, tex::FRAME),
    lean([0.0, -0.14, 0.1], [0.05, 0.62, 0.05], PI, 0.18, tex::FRAME),
    part([0.0, 0.2, 0.0], [0.74, 0.08, 0.08], tex::MAT_BRASS),
    part([0.4, 0.17, 0.0], [0.09, 0.22, 0.1], tex::MAT_BRASS),
    lean([0.37, 0.02, 0.0], [0.1, 0.16, 0.08], FRAC_PI_2, -0.6, tex::MAT_BRASS),
    part([-0.33, -0.03, 0.1], [0.03, 0.4, 0.03], tex::STEEL),
    part([-0.3, -0.24, 0.12], [0.26, 0.24, 0.04], tex::MAT_GUNMETAL),
    part([0.43, -0.17, 0.0], [0.015, 0.36, 0.015], tex::MAT_RUBBER),
    cylinder([0.43, -0.39, 0.0], 0.1, 0.1, tex::STEEL),
];

/// Three distillation columns of falling height side by side, a platform ring on each, the flare burning behind.
const REFINERY: [Part; 12] = [
    part([0.0, -0.47, 0.0], [0.9, 0.06, 0.9], tex::CONCRETE),
    cylinder([0.32, -0.04, -0.36], 0.05, 0.8, tex::STEEL),
    frustum([0.32, 0.41, -0.36], 0.14, 0.16, 0.2, tex::LAMP_YELLOW),
    cylinder([-0.06, -0.01, -0.06], 0.24, 0.86, tex::STEEL),
    frustum([-0.06, 0.45, -0.06], 0.24, 0.06, 0.4, tex::STEEL),
    cylinder([-0.06, 0.16, -0.06], 0.3, 0.02, tex::FRAME),
    cylinder([0.25, -0.15, 0.0], 0.22, 0.58, tex::STEEL),
    frustum([0.25, 0.16, 0.0], 0.22, 0.05, 0.4, tex::STEEL),
    cylinder([0.25, 0.0, 0.0], 0.28, 0.02, tex::FRAME),
    cylinder([-0.22, -0.24, 0.26], 0.28, 0.4, tex::STEEL),
    frustum([-0.22, -0.02, 0.26], 0.28, 0.05, 0.4, tex::STEEL),
    cylinder([-0.22, -0.14, 0.26], 0.34, 0.02, tex::FRAME),
];

/// One of the rotor's tubes `k` sixths of a turn round, leaning out from it, its cap coloured.
const fn tube(k: usize) -> Part {
    const TILT: f32 = 0.5;
    let a = k as f32 * FRAC_PI_3;
    // cos and sin of a sixth turn, exactly enough for the six spots (const fns have no trig).
    const C: [f32; 6] = [1.0, 0.5, -0.5, -1.0, -0.5, 0.5];
    const S: [f32; 6] = [0.0, 0.866, 0.866, 0.0, -0.866, -0.866];
    let r = 0.21;
    Part {
        center: [r * C[k], 0.12, r * S[k]],
        size: [0.13, 0.38, 0.13],
        yaw: a - FRAC_PI_2,
        tex: [tex::LAMP_RED, tex::LAMP_BLUE, tex::LAMP_BLUE],
        form: Form::Lean(TILT),
    }
}

/// A white benchtop centrifuge open, tubes leaning out of its rotor, the panel at the front.
const CENTRIFUGE: [Part; 9] = Build::new()
    .with(Part { tex: [tex::SOOT, tex::MAT_METAL, tex::MAT_METAL], ..cylinder([0.0, -0.3, 0.0], 0.9, 0.4, 0) })
    // Back to front: the tubes' order by x + z, the hub among them.
    .with(tube(4))
    .with(tube(3))
    .with(tube(5))
    .with(frustum([0.0, -0.06, 0.0], 0.3, 0.08, 0.6, tex::MAT_METAL))
    .with(tube(2))
    .with(tube(0))
    .with(tube(1))
    .with(faced([0.36, -0.36, 0.3], [0.14, 0.1, 0.14], tex::LAMP_GREEN, tex::MAT_GUNMETAL))
    .done();

/// A washing tank: ore lying in water under a spray bar on two posts, the water falling on it.
const WASHER: [Part; 14] = [
    part([-0.33, -0.08, 0.33], [0.05, 0.56, 0.05], tex::FRAME),
    faced([0.0, -0.36, 0.0], [0.9, 0.28, 0.8], tex::WATER, tex::STEEL),
    turned([-0.2, -0.2, -0.16], [0.18, 0.12, 0.16], 0.3, tex::COPPER_ORE),
    turned([0.12, -0.2, -0.18], [0.16, 0.1, 0.18], 1.1, tex::COPPER_ORE),
    turned([-0.24, -0.2, 0.14], [0.16, 0.12, 0.16], 0.8, tex::COPPER_ORE),
    turned([0.0, -0.18, 0.0], [0.2, 0.14, 0.2], 0.5, tex::COPPER_ORE),
    turned([0.22, -0.2, 0.16], [0.16, 0.1, 0.16], 1.3, tex::COPPER_ORE),
    Part { yaw: -FRAC_PI_4, ..bar_x([0.0, 0.18, 0.0], 0.96, 0.08, tex::PIPE_WATER) },
    part([0.2, 0.0, -0.2], [0.04, 0.3, 0.04], tex::PIPE_WATER),
    part([0.0, 0.0, 0.0], [0.04, 0.3, 0.04], tex::PIPE_WATER),
    part([-0.2, 0.0, 0.2], [0.04, 0.3, 0.04], tex::PIPE_WATER),
    part([0.33, -0.08, -0.33], [0.05, 0.56, 0.05], tex::FRAME),
    cylinder([-0.32, -0.21, -0.26], 0.12, 0.02, tex::MAT_METAL),
    cylinder([0.3, -0.21, 0.24], 0.12, 0.02, tex::MAT_METAL),
];

/// A robot arm on a workbench, its gripper over the plate it is making.
const CONSTRUCTOR: [Part; 10] = [
    faced([0.0, -0.45, 0.0], [0.7, 0.1, 0.5], tex::CONSTRUCTOR_TOP, tex::CONSTRUCTOR_SIDE),
    part([0.2, -0.34, 0.0], [0.18, 0.12, 0.18], tex::STEEL),
    cylinder([-0.22, -0.34, 0.0], 0.24, 0.12, tex::MOTOR),
    lean([-0.184, 0.018, 0.0], [0.1, 0.6, 0.1], -FRAC_PI_2, 0.12, tex::MAT_AETHER),
    Part {
        center: [-0.148, 0.316, 0.0],
        size: [0.16, 0.16, 0.14],
        yaw: 0.0,
        tex: [tex::MOTOR; 3],
        form: Form::Round(Axis::Z, 1.0),
    },
    lean([0.026, 0.268, 0.0], [0.09, 0.361, 0.09], -FRAC_PI_2, -1.302, tex::MAT_AETHER),
    part([0.2, 0.07, 0.0], [0.07, 0.3, 0.07], tex::MAT_AETHER),
    part([0.2, -0.11, 0.0], [0.12, 0.07, 0.12], tex::MOTOR),
    part([0.2, -0.19, -0.05], [0.025, 0.1, 0.025], tex::STEEL),
    part([0.2, -0.19, 0.05], [0.025, 0.1, 0.025], tex::STEEL),
];

/// A centrifugal pump: its motor driving the blue volute, water in through the front flange and out of the top.
const PUMP: [Part; 9] = [
    part([0.0, -0.46, 0.0], [0.9, 0.08, 0.5], tex::FRAME),
    bar_x([-0.22, -0.22, 0.0], 0.44, 0.32, tex::MOTOR),
    bar_x([0.02, -0.22, 0.0], 0.06, 0.12, tex::STEEL),
    bar_x([0.18, -0.18, 0.0], 0.2, 0.52, tex::PIPE_WATER),
    bar_x([0.29, -0.18, 0.0], 0.03, 0.53, tex::STEEL),
    cylinder([0.18, 0.2, 0.1], 0.18, 0.36, tex::PIPE_WATER),
    cylinder([0.18, 0.39, 0.1], 0.28, 0.04, tex::STEEL),
    bar_x([0.37, -0.18, 0.0], 0.18, 0.18, tex::PIPE_WATER),
    bar_x([0.47, -0.18, 0.0], 0.04, 0.28, tex::STEEL),
];
