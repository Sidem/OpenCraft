//! Glass Kiln (`block::GLASS_KILN`, L-COLOUR): a brick-and-iron burner furnace that melts glass, sand or quartz
//! together with any of the 16 pigments into stained glass (`recipes/glass.rs`).
//!
//! Uses `Pick::ByInput` with 2 input slots + 1 fuel slot + 1 output slot (`SINGLE` 1×1×1 footprint), so players can
//! drop or belt-feed any silica source (`GLASS`, `SAND` or `QUARTZ_ORE`) plus any pigment and coal/charcoal/wood fuel,
//! and the kiln automatically melts the matching stained glass colour.

use crate::block::{tex, GLASS_KILN};
use crate::recipes::Category;

use super::super::footprint::SINGLE;
use super::model::{part, round, Look, Part};
use super::specs::{Energy, Pick, ProcessSpec, ProcessTier};

pub(super) const GLASS_KILN_SPEC: ProcessSpec = ProcessSpec {
    block: GLASS_KILN,
    categories: &[Category::Glassmaking],
    pick: Pick::ByInput,
    buffers: [2, 1, 1],
    side: 0,
    tiers: &[ProcessTier { energy: Energy::Burner, speed: 1000, fuel: 1000, power: 0 }],
    footprint: SINGLE,
    verb: "Melting",
    products: "stained glass",
    waiting: "Waiting for glass, sand or quartz + pigment",
    map_colour: 0xe89b3c,
    compute: 0,
    parts: &KILN_PARTS,
};

const KILN_BRICK: [u16; 3] = [tex::KILN_TOP, tex::KILN_SIDE, tex::MAT_GUNMETAL];

/// Firebrick glassblowing kiln with cast-iron hearth, arched crucible window (`Look::Fire`), chimney with smoke
/// (`Look::Smoke`), product display shelf (`Look::Product`), brass damper knob and status lamp.
const KILN_PARTS: [Part; 10] = [
    // Cast-iron hearth plinth.
    part([0.0, -0.43, 0.0], [0.94, 0.14, 0.94], Look::Tex([tex::MAT_GUNMETAL; 3])),
    // Firebrick kiln body.
    part([0.0, -0.06, -0.04], [0.84, 0.60, 0.80], Look::Tex(KILN_BRICK)),
    // Upper tapered refractory crown (y = 0.24 .. 0.36, z = -0.40 .. +0.28).
    round([0.0, 0.30, -0.06], [0.68, 0.12, 0.68], Look::Tex(KILN_BRICK)),
    // Cast-iron chimney flue (y = 0.36 .. 0.48) & smoke vent.
    round([-0.20, 0.42, -0.22], [0.18, 0.12, 0.18], Look::Tex([tex::MAT_GUNMETAL; 3])),
    part([-0.20, 0.56, -0.22], [0.45, 0.0, 0.0], Look::Smoke),
    // Front iron arch frame around the molten-glass crucible window.
    part([0.0, -0.08, 0.37], [0.54, 0.36, 0.08], Look::Tex([tex::MAT_GUNMETAL; 3])),
    // Glowing crucible window (dark soot when idle, warm molten glow when working).
    part([0.0, -0.10, 0.415], [0.40, 0.24, 0.04], Look::Fire(tex::SOOT)),
    // Brass damper valve on the right side.
    part([0.44, -0.06, 0.04], [0.06, 0.12, 0.12], Look::Tex([tex::MAT_BRASS; 3])),
    // Product display standing flush on the crown (y = 0.36 .. 0.72), clear of the chimney from every angle.
    part([0.08, 0.54, 0.06], [0.36, 0.36, 0.36], Look::Product),
    // Status lamp on the front-right shoulder (y = 0.24 .. 0.32, z = 0.285 .. 0.355, clear of the crown).
    part([0.34, 0.28, 0.32], [0.10, 0.08, 0.07], Look::Lamp),
];
