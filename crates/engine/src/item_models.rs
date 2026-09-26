//! Presentation-only box assemblies shared by loose items and inventory icons.
//! Coordinates and dimensions are fractions of a full item cube; add a static slice for a new part.

use crate::block::tex;
use crate::item::{ItemId, COPPER_INGOT, COPPER_WIRE, GREEN_PACK, IRON_INGOT, IRON_PLATE, IRON_ROD, RED_PACK, SCREW};

pub struct Part {
    pub center: [f32; 3],
    pub size: [f32; 3],
    pub tex: [u16; 3],
}

const fn part(center: [f32; 3], size: [f32; 3], layer: u16) -> Part {
    Part { center, size, tex: [layer; 3] }
}

const IRON_BAR: [Part; 2] = [
    part([0.0, -0.07, 0.0], [0.88, 0.29, 0.53], tex::IRON_INGOT),
    part([0.0, 0.105, 0.0], [0.69, 0.07, 0.4], tex::IRON_PLATE),
];
const COPPER_BAR: [Part; 2] = [
    part([0.0, -0.07, 0.0], [0.88, 0.29, 0.53], tex::COPPER_INGOT),
    part([0.0, 0.105, 0.0], [0.69, 0.07, 0.4], tex::COPPER_INGOT),
];
const PLATE: [Part; 2] = [
    part([0.0, -0.07, 0.0], [0.84, 0.1, 0.84], tex::IRON_PLATE),
    part([0.0, 0.005, 0.0], [0.66, 0.05, 0.66], tex::IRON_PLATE),
];
const ROD: [Part; 3] = [
    part([0.0, 0.0, 0.0], [0.9, 0.13, 0.13], tex::IRON_ROD),
    part([-0.39, 0.0, 0.0], [0.13, 0.19, 0.19], tex::IRON_PLATE),
    part([0.39, 0.0, 0.0], [0.13, 0.19, 0.19], tex::IRON_PLATE),
];
const SCREWS: [Part; 3] = [
    part([-0.09, -0.06, -0.1], [0.62, 0.1, 0.1], tex::SCREW),
    part([-0.36, -0.06, -0.1], [0.13, 0.24, 0.24], tex::IRON_PLATE),
    part([0.11, 0.09, 0.17], [0.43, 0.09, 0.09], tex::SCREW),
];
const WIRE: [Part; 9] = [
    part([0.0, -0.06, -0.28], [0.58, 0.08, 0.08], tex::COPPER_WIRE),
    part([0.0, -0.06, 0.28], [0.58, 0.08, 0.08], tex::COPPER_WIRE),
    part([-0.28, -0.06, 0.0], [0.08, 0.08, 0.58], tex::COPPER_WIRE),
    part([0.28, -0.06, 0.0], [0.08, 0.08, 0.58], tex::COPPER_WIRE),
    part([0.35, -0.06, 0.27], [0.28, 0.08, 0.08], tex::COPPER_WIRE),
    part([0.0, 0.055, -0.22], [0.48, 0.07, 0.07], tex::COPPER_WIRE),
    part([0.0, 0.055, 0.22], [0.48, 0.07, 0.07], tex::COPPER_WIRE),
    part([-0.22, 0.055, 0.0], [0.07, 0.07, 0.48], tex::COPPER_WIRE),
    part([0.22, 0.055, 0.0], [0.07, 0.07, 0.48], tex::COPPER_WIRE),
];
const RED_FLASK: [Part; 4] = [
    part([0.0, -0.12, 0.0], [0.42, 0.42, 0.42], tex::RED_PACK),
    part([0.0, 0.11, 0.0], [0.32, 0.06, 0.32], tex::FLASK_GLASS),
    part([0.0, 0.23, 0.0], [0.17, 0.2, 0.17], tex::FLASK_GLASS),
    part([0.0, 0.35, 0.0], [0.22, 0.08, 0.22], tex::BOX_TOP),
];
const GREEN_FLASK: [Part; 4] = [
    part([0.0, -0.12, 0.0], [0.42, 0.42, 0.42], tex::GREEN_PACK),
    part([0.0, 0.11, 0.0], [0.32, 0.06, 0.32], tex::FLASK_GLASS),
    part([0.0, 0.23, 0.0], [0.17, 0.2, 0.17], tex::FLASK_GLASS),
    part([0.0, 0.35, 0.0], [0.22, 0.08, 0.22], tex::BOX_TOP),
];

pub fn parts(item: ItemId) -> &'static [Part] {
    match item {
        IRON_INGOT => &IRON_BAR,
        COPPER_INGOT => &COPPER_BAR,
        IRON_PLATE => &PLATE,
        IRON_ROD => &ROD,
        SCREW => &SCREWS,
        COPPER_WIRE => &WIRE,
        RED_PACK => &RED_FLASK,
        GREEN_PACK => &GREEN_FLASK,
        _ => &[],
    }
}

#[cfg(test)]
mod tests;
