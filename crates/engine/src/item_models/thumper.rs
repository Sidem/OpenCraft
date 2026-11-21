//! Icons of the Seismic Thumper tiers (F-SEIS, Mk1–Mk3): heavy ground-coupling anvil plate, tier collar, four corner
//! struts, ground-seated telemetry cabinet, central chevron-banded drop-hammer with forged impact shoe, upper bridge,
//! hydraulic turret, twin geophone masts and status lamp. Back to front, floor at -0.5, all within [-0.5, 0.5].

use super::shapes::cylinder;
use super::{part, Part};
use crate::block::tex;
use crate::item::{ItemId, THUMPER_ITEM, THUMPER_MK2, THUMPER_MK3};

pub(super) fn parts(item: ItemId) -> &'static [Part] {
    match item {
        THUMPER_ITEM => &THUMPER_1,
        THUMPER_MK2 => &THUMPER_2,
        THUMPER_MK3 => &THUMPER_3,
        _ => &[],
    }
}

const fn thumper(stripe: u16) -> [Part; 14] {
    [
        // Heavy chamfered baseplate and tier collar.
        part([0.0, -0.45, 0.0], [0.92, 0.10, 0.92], tex::MAT_GUNMETAL),
        part([0.0, -0.37, 0.0], [0.86, 0.06, 0.86], stripe),
        // Four corner hydraulic struts (back to front).
        part([-0.34, -0.06, -0.34], [0.11, 0.58, 0.11], tex::FRAME),
        part([0.34, -0.06, -0.34], [0.11, 0.58, 0.11], tex::FRAME),
        part([-0.34, -0.06, 0.34], [0.11, 0.58, 0.11], tex::FRAME),
        part([0.34, -0.06, 0.34], [0.11, 0.58, 0.11], tex::FRAME),
        // Forged impact shoe + central seismic drop-hammer piston.
        cylinder([0.0, -0.30, 0.0], 0.38, 0.08, tex::MAT_GUNMETAL),
        cylinder([0.0, -0.07, 0.0], 0.34, 0.40, tex::THUMPER_PISTON),
        // Ground-seated side telemetry cabinet with glowing cyan screen.
        part([0.38, -0.18, 0.0], [0.12, 0.34, 0.36], tex::THUMPER_SIDE),
        part([0.45, -0.14, 0.0], [0.02, 0.12, 0.20], tex::LAMP_BLUE),
        // Upper instrument bridge deck and hydraulic turret cap.
        part([0.0, 0.24, 0.0], [0.84, 0.13, 0.84], tex::THUMPER_SIDE),
        cylinder([0.0, 0.35, 0.0], 0.34, 0.12, tex::THUMPER_TOP),
        // Twin geophone telemetry antennae.
        cylinder([-0.26, 0.41, -0.24], 0.05, 0.18, tex::STEEL),
        // Status indicator beacon at front corner.
        part([0.28, 0.34, 0.28], [0.11, 0.08, 0.11], tex::LAMP_GREEN),
    ]
}

const THUMPER_1: [Part; 14] = thumper(tex::stripe(0));
const THUMPER_2: [Part; 14] = thumper(tex::stripe(3));
const THUMPER_3: [Part; 14] = thumper(tex::stripe(4));
