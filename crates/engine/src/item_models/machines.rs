//! Icons of machines whose placed model does not say what they are at hotbar size (`placed.rs` draws the rest), each
//! built after the real thing's silhouette: a cooling tower's waisted shell, a turbine's bladed rotor, a headframe's
//! wheel on its tower, a winch's drum and hook, an outlet pouring water, a water wheel's paddles, a coil of cable.
//! The plant machines (pumpjack, refinery, centrifuge, washer, constructor) are in `machines_plant.rs`.
//! Judged blind at 40 px (DEV_PLAN change log 2026-10-30). Parts back to front, floor at -0.5; add a const and a row.

use std::f32::consts::{FRAC_PI_2, FRAC_PI_8};

use super::shapes::{bar_x, cylinder, frustum};
use super::{part, turned, Axis, Form, Part};
use crate::block::{self, tex};
use crate::item::ItemId;

pub(super) fn parts(item: ItemId) -> &'static [Part] {
    // A tier's item (Constructor Mk3) places the same machine and shares its icon; the icon's tier chip tells them apart.
    let Some(block) = item.places() else { return &[] };
    match block {
        block::COOLING_TOWER => &COOLING_TOWER,
        block::TURBINE => &TURBINE,
        block::HOIST => &HEADFRAME,
        block::WINCH => &WINCH,
        block::OUTLET => &OUTLET,
        block::WATER_WHEEL => &WATER_WHEEL,
        block::CABLE => &CABLE,
        _ => super::machines_plant::parts(block),
    }
}

/// A box tilted by `pitch` about its own x after `yaw` turns it (`Form::Lean`).
pub(super) const fn lean(c: [f32; 3], size: [f32; 3], yaw: f32, pitch: f32, layer: u16) -> Part {
    Part { center: c, size, yaw, tex: [layer; 3], form: Form::Lean(pitch) }
}

/// A hyperboloid concrete shell, dark inside its mouth, steam rising from it.
const COOLING_TOWER: [Part; 6] = [
    cylinder([0.0, -0.475, 0.0], 0.84, 0.05, tex::FRAME),
    Part { form: Form::Round(Axis::Up, 0.66), tex: [tex::CONCRETE; 3], ..cylinder([0.0, -0.2, 0.0], 0.78, 0.6, 0) },
    Part {
        form: Form::Round(Axis::Up, 1.12),
        tex: [tex::SOOT, tex::CONCRETE, tex::CONCRETE],
        ..cylinder([0.0, 0.2, 0.0], 0.515, 0.2, 0)
    },
    part([-0.02, 0.36, -0.02], [0.4, 0.12, 0.4], tex::QUICKLIME),
    turned([0.04, 0.44, 0.0], [0.3, 0.12, 0.3], 0.5, tex::QUICKLIME),
    turned([0.08, 0.52, 0.04], [0.2, 0.1, 0.2], 0.2, tex::QUICKLIME),
];

/// One bladed wheel of a turbine rotor on the shaft at `x`, `d` across: a steel face, its rim the blades.
const fn blades(x: f32, d: f32) -> Part {
    Part { tex: [tex::MAT_METAL, tex::STEEL, tex::MAT_METAL], ..bar_x([x, -0.06, 0.0], 0.06, d, 0) }
}

/// A turbine's rotor lifted out over its open lower casing: bladed wheels stepping down from the exhaust end, on
/// bearings.
const TURBINE: [Part; 10] = [
    part([-0.04, -0.41, 0.0], [0.66, 0.18, 0.56], tex::TURBINE_SIDE),
    part([-0.4, -0.27, 0.0], [0.08, 0.46, 0.16], tex::FRAME),
    bar_x([0.0, -0.06, 0.0], 0.92, 0.06, tex::STEEL),
    blades(-0.29, 0.62),
    blades(-0.17, 0.55),
    blades(-0.06, 0.48),
    blades(0.04, 0.41),
    blades(0.13, 0.35),
    blades(0.21, 0.29),
    part([0.4, -0.27, 0.0], [0.08, 0.46, 0.16], tex::FRAME),
];

/// A mine headframe: four legs leaning in to a platform, the sheave wheel on top, its rope down into the shaft.
const HEADFRAME: [Part; 13] = {
    // A leg from a corner of the collar up to the platform: yaw aims its lean at the middle (`lean`).
    const L: [f32; 3] = [0.05, 0.78, 0.05];
    const P: f32 = 0.333;
    const Q: f32 = std::f32::consts::FRAC_PI_4;
    [
        part([0.0, -0.47, 0.0], [0.8, 0.06, 0.8], tex::CONCRETE),
        part([0.0, -0.435, 0.0], [0.36, 0.01, 0.36], tex::SOOT),
        lean([-0.21, -0.07, -0.21], L, -Q, P, tex::FRAME),
        part([0.0, 0.0, -0.193], [0.39, 0.03, 0.03], tex::FRAME),
        part([-0.193, 0.0, 0.0], [0.03, 0.03, 0.39], tex::FRAME),
        lean([0.21, -0.07, -0.21], L, Q, P, tex::FRAME),
        lean([-0.21, -0.07, 0.21], L, -3.0 * Q, P, tex::FRAME),
        part([0.0, 0.31, 0.0], [0.3, 0.03, 0.3], tex::FRAME),
        Part { tex: [tex::STEEL, tex::MAT_GUNMETAL, tex::STEEL], ..bar_x([0.0, 0.43, 0.0], 0.05, 0.36, 0) },
        bar_x([0.03, 0.43, 0.0], 0.06, 0.08, tex::FRAME),
        part([0.0, 0.0, 0.17], [0.015, 0.85, 0.015], tex::MAT_RUBBER),
        part([0.193, 0.0, 0.0], [0.03, 0.03, 0.39], tex::FRAME),
        lean([0.21, -0.07, 0.21], L, 3.0 * Q, P, tex::FRAME),
    ]
};

/// A cable drum between bright flanges, its motor at one end; the cable runs up over a pulley on a post and hangs
/// down to a hook.
const WINCH: [Part; 16] = [
    bar_z([-0.16, -0.3, -0.42], 0.16, 0.26, tex::MOTOR),
    part([-0.16, -0.38, -0.31], [0.16, 0.24, 0.06], tex::FRAME),
    bar_z([-0.16, -0.2, -0.27], 0.04, 0.6, tex::STEEL),
    bar_z([-0.16, -0.2, 0.0], 0.5, 0.44, tex::MAT_GUNMETAL),
    bar_z([-0.16, -0.2, -0.12], 0.025, 0.45, tex::MAT_METAL),
    bar_z([-0.16, -0.2, 0.0], 0.025, 0.45, tex::MAT_METAL),
    bar_z([-0.16, -0.2, 0.12], 0.025, 0.45, tex::MAT_METAL),
    bar_z([-0.16, -0.2, 0.27], 0.04, 0.6, tex::STEEL),
    part([-0.16, -0.38, 0.31], [0.16, 0.24, 0.06], tex::FRAME),
    part([0.28, -0.07, -0.08], [0.07, 0.86, 0.07], tex::STEEL),
    lean([0.06, 0.2, 0.0], [0.05, 0.54, 0.05], -FRAC_PI_2, 0.733, tex::MAT_METAL),
    bar_z([0.28, 0.36, 0.0], 0.06, 0.2, tex::MAT_BRASS),
    part([0.38, 0.16, 0.0], [0.05, 0.4, 0.05], tex::MAT_METAL),
    part([0.38, -0.1, 0.0], [0.18, 0.18, 0.18], tex::MAT_BRASS),
    part([0.38, -0.26, 0.06], [0.06, 0.16, 0.06], tex::MAT_BRASS),
    part([0.38, -0.32, -0.01], [0.06, 0.05, 0.16], tex::MAT_BRASS),
];

/// A round body of diameter `d` and length `len` lying along z.
const fn bar_z(c: [f32; 3], len: f32, d: f32, layer: u16) -> Part {
    Part { center: c, size: [d, d, len], yaw: 0.0, tex: [layer; 3], form: Form::Round(Axis::Z, 1.0) }
}

/// A water pipe on a post turning down to a nozzle, water falling from it into a pool.
const OUTLET: [Part; 7] = [
    part([-0.3, -0.17, 0.0], [0.06, 0.6, 0.06], tex::FRAME),
    cylinder([0.1, -0.475, 0.02], 0.62, 0.04, tex::WATER),
    bar_x([-0.2, 0.17, 0.0], 0.56, 0.2, tex::PIPE_WATER),
    bar_x([-0.47, 0.17, 0.0], 0.04, 0.28, tex::STEEL),
    cylinder([0.08, 0.08, 0.0], 0.2, 0.38, tex::PIPE_WATER),
    frustum([0.08, -0.14, 0.0], 0.24, 0.08, 0.7, tex::STEEL),
    cylinder([0.08, -0.31, 0.0], 0.13, 0.32, tex::WATER),
];

/// Cosine and sine of k eighths of a turn.
const EIGHTHS: [(f32, f32); 8] = {
    const H: f32 = std::f32::consts::FRAC_1_SQRT_2;
    [(1.0, 0.0), (H, H), (0.0, 1.0), (-H, H), (-1.0, 0.0), (-H, -H), (0.0, -1.0), (H, -H)]
};

/// A wooden wheel turning on its axle between two posts over a stream: spokes and rim boards on both faces, eight
/// paddles between them.
const WATER_WHEEL: [Part; 36] = {
    const R: f32 = 0.36;
    const Y: f32 = -0.04;
    let mut out = [super::shapes::EMPTY; 36];
    out[0] = part([0.0, -0.47, 0.0], [0.4, 0.06, 1.0], tex::WATER);
    out[1] = part([-0.24, -0.22, 0.0], [0.06, 0.56, 0.08], tex::HANDLE);
    let mut n = 2;
    // The back face, the paddles, then the front face.
    let mut side = 0;
    while side < 3 {
        let mut k = 0;
        while k < 8 {
            let (c, s) = EIGHTHS[k];
            let a = k as f32 * std::f32::consts::FRAC_PI_4;
            if side == 1 {
                out[n] = lean([0.0, Y + R * c, R * s], [0.3, 0.13, 0.03], 0.0, a, tex::PLANKS);
                n += 1;
            } else {
                let x = if side == 0 { -0.12 } else { 0.12 };
                // A rim board halfway between two paddles, and a spoke on every other one.
                let (c2, s2) = EIGHTHS[(k + 1) % 8];
                let (mc, ms) = ((c + c2) * 0.5, (s + s2) * 0.5);
                out[n] = lean(
                    [x, Y + 0.42 * mc / 0.924, 0.42 * ms / 0.924],
                    [0.03, 0.04, 0.3],
                    0.0,
                    a + FRAC_PI_8,
                    tex::HANDLE,
                );
                n += 1;
                if k % 2 == 0 {
                    out[n] = lean([x, Y, 0.0], [0.03, 0.8, 0.03], 0.0, a, tex::HANDLE);
                    n += 1;
                }
            }
            k += 1;
        }
        if side == 1 {
            out[n] = bar_x([0.0, Y, 0.0], 0.52, 0.07, tex::STEEL);
            n += 1;
        }
        side += 1;
    }
    out[n] = part([0.24, -0.22, 0.0], [0.06, 0.56, 0.08], tex::HANDLE);
    assert!(n + 1 == 36, "the water wheel's part count does not match its parts");
    out
};

/// A coil of orange power cable lying on the ground, three turns, its lead running out to a plug with brass pins.
const CABLE: [Part; 28] = {
    // The ring's eighths back to front for the viewer at +x and +z.
    const ORDER: [usize; 8] = [5, 4, 6, 3, 7, 2, 0, 1];
    const R: f32 = 0.26;
    let mut out = [super::shapes::EMPTY; 28];
    let mut n = 0;
    while n < 24 {
        let k = ORDER[n % 8];
        let (c, s) = EIGHTHS[k];
        let y = -0.45 + 0.08 * (n / 8) as f32;
        let yaw = k as f32 * std::f32::consts::FRAC_PI_4 + std::f32::consts::FRAC_PI_2;
        out[n] = Part { yaw, ..bar_x([R * c, y, R * s], 0.23, 0.09, tex::MAT_EMBER) };
        n += 1;
    }
    out[24] = bar_z([0.0, -0.29, 0.38], 0.24, 0.09, tex::MAT_EMBER);
    out[25] = part([0.0, -0.29, 0.53], [0.16, 0.14, 0.1], tex::MAT_RUBBER);
    out[26] = part([-0.04, -0.29, 0.62], [0.025, 0.025, 0.1], tex::MAT_BRASS);
    out[27] = part([0.04, -0.29, 0.62], [0.025, 0.025, 0.1], tex::MAT_BRASS);
    out
};
