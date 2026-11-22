//! Item models for L-COLOUR: the Glass Kiln (`GLASS_KILN_ITEM`) and the handheld Paint Sprayer (`PAINT_SPRAYER`).

use crate::block::tex;
use crate::item::{ItemId, GLASS_KILN_ITEM, PAINT_SPRAYER};

use super::shapes::{bar_x, cylinder, frustum, Build};
use super::{part, Part};

pub(super) fn parts(item: ItemId) -> &'static [Part] {
    match item {
        GLASS_KILN_ITEM => &GLASS_KILN_MODEL,
        PAINT_SPRAYER => &PAINT_SPRAYER_MODEL,
        _ => &[],
    }
}

/// Compact firebrick dome kiln with cast-iron hearth plinth, glowing molten-glass arch window and chimney stack
/// (sits on y = -0.5 for a full-cube wide-machine item).
const GLASS_KILN_MODEL: [Part; 7] = Build::new()
    // Cast-iron base plinth (bottom at -0.50).
    .with(part([0.0, -0.42, 0.0], [0.92, 0.16, 0.92], tex::MAT_GUNMETAL))
    // Firebrick kiln chamber body.
    .with(part([0.0, -0.08, 0.0], [0.84, 0.52, 0.84], tex::KILN_SIDE))
    // Upper refractory collar & dome cap.
    .with(frustum([0.0, 0.25, 0.0], 0.78, 0.16, 0.72, tex::KILN_TOP))
    // Cast-iron chimney flue rising to +0.48.
    .with(cylinder([0.0, 0.40, -0.10], 0.24, 0.18, tex::MAT_GUNMETAL))
    // Front iron arch frame around the crucible viewing window.
    .with(part([0.0, -0.08, 0.42], [0.46, 0.34, 0.05], tex::MAT_GUNMETAL))
    // Glowing amber molten-glass crucible window.
    .with(part([0.0, -0.10, 0.445], [0.34, 0.24, 0.02], tex::KILN_GLOW))
    // Brass air-damper knob on the side.
    .with(part([0.43, -0.05, 0.08], [0.05, 0.10, 0.10], tex::MAT_BRASS))
    .done();

/// Handheld HVLP paint spray gun pointing along +x, resting on -0.22 (matches `PAINT_SPRAYER` `size[1] = 0.44`):
/// dark rubber pistol grip, brushed steel air body, brass air cap & spray nozzle, and a cylindrical underslung
/// paint canister cup with brass collar.
const PAINT_SPRAYER_MODEL: [Part; 10] = Build::new()
    // Pistol grip resting on y = -0.22 (-0.22 .. +0.06).
    .with(part([-0.24, -0.08, 0.0], [0.12, 0.28, 0.10], tex::MAT_RUBBER))
    // Trigger lever in front of grip.
    .with(part([-0.14, 0.01, 0.0], [0.04, 0.12, 0.04], tex::MAT_BRASS))
    // Lower air-hose fitting under grip.
    .with(cylinder([-0.24, -0.20, 0.0], 0.07, 0.04, tex::MAT_BRASS))
    // Main gun body barrel along +x (y = 0.06 .. 0.19).
    .with(part([-0.04, 0.125, 0.0], [0.46, 0.13, 0.11], tex::MAT_METAL))
    // Rear pressure regulator dial.
    .with(bar_x([-0.29, 0.125, 0.0], 0.05, 0.09, tex::MAT_GUNMETAL))
    // Underslung cylindrical paint canister cup (bottom at -0.21, top at +0.04).
    .with(cylinder([0.08, -0.085, 0.0], 0.22, 0.25, tex::MAT_AETHER))
    // Brass lid & siphon neck joining canister to gun body.
    .with(cylinder([0.08, 0.055, 0.0], 0.23, 0.035, tex::MAT_BRASS))
    .with(cylinder([0.08, 0.08, 0.0], 0.07, 0.04, tex::MAT_GUNMETAL))
    // Front brass air cap & precision spray nozzle tip.
    .with(bar_x([0.23, 0.125, 0.0], 0.09, 0.11, tex::MAT_BRASS))
    .with(bar_x([0.30, 0.125, 0.0], 0.06, 0.055, tex::MAT_GUNMETAL))
    .done();
