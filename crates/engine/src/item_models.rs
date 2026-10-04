//! Presentation-only box assemblies shared by loose items and inventory icons.
//! Coordinates and dimensions are fractions of a full item cube (y up); parts are drawn in order, so
//! list them back to front (the icon looks from +x, +z, above). Add a static slice for a new model.
//! Tools stand upright in the x-y plane, head up; their heads use their tier's material.

use crate::block::tex;
use crate::item::{
    ItemId, BLUE_PACK, COPPER_INGOT, COPPER_WIRE, CORE_DRILL, GREEN_PACK, IRON_AXE, IRON_INGOT, IRON_PICKAXE,
    IRON_PLATE, IRON_ROD, IRON_SHOVEL, RED_PACK, SCANNER, SCANNER_MK2, SCREW, STEEL_AXE, STEEL_BEAM, STEEL_INGOT,
    STEEL_PICKAXE, STEEL_PLATE, STEEL_SHOVEL, STONE_AXE, STONE_PICKAXE, STONE_SHOVEL, VIOLET_PACK,
};

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
/// A spool on its side: copper windings between two wooden flanges, a loose end on the ground.
const WIRE: [Part; 4] = [
    part([-0.22, 0.0, 0.0], [0.06, 0.5, 0.5], tex::HANDLE),
    part([0.0, 0.0, 0.0], [0.38, 0.4, 0.4], tex::COPPER_WIRE),
    part([0.22, 0.0, 0.0], [0.06, 0.5, 0.5], tex::HANDLE),
    part([0.05, -0.22, 0.33], [0.3, 0.04, 0.05], tex::COPPER_WIRE),
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

const BLUE_FLASK: [Part; 4] = [
    part([0.0, -0.12, 0.0], [0.42, 0.42, 0.42], tex::BLUE_PACK),
    part([0.0, 0.11, 0.0], [0.32, 0.06, 0.32], tex::FLASK_GLASS),
    part([0.0, 0.23, 0.0], [0.17, 0.2, 0.17], tex::FLASK_GLASS),
    part([0.0, 0.35, 0.0], [0.22, 0.08, 0.22], tex::BOX_TOP),
];

const VIOLET_FLASK: [Part; 4] = [
    part([0.0, -0.12, 0.0], [0.42, 0.42, 0.42], tex::VIOLET_PACK),
    part([0.0, 0.11, 0.0], [0.32, 0.06, 0.32], tex::FLASK_GLASS),
    part([0.0, 0.23, 0.0], [0.17, 0.2, 0.17], tex::FLASK_GLASS),
    part([0.0, 0.35, 0.0], [0.22, 0.08, 0.22], tex::BOX_TOP),
];

const fn pickaxe(head: u16) -> [Part; 6] {
    [
        part([0.0, -0.08, 0.0], [0.08, 0.8, 0.08], tex::HANDLE),
        part([0.0, 0.3, 0.0], [0.14, 0.16, 0.14], head),
        part([0.0, 0.3, 0.0], [0.5, 0.1, 0.1], head),
        part([-0.3, 0.25, 0.0], [0.12, 0.1, 0.09], head),
        part([0.3, 0.25, 0.0], [0.12, 0.1, 0.09], head),
        part([0.39, 0.18, 0.0], [0.07, 0.08, 0.07], head),
    ]
}

const fn axe(head: u16) -> [Part; 4] {
    [
        part([0.0, -0.05, 0.0], [0.08, 0.84, 0.08], tex::HANDLE),
        part([-0.04, 0.27, 0.0], [0.14, 0.18, 0.12], head),
        part([0.14, 0.27, 0.0], [0.22, 0.2, 0.06], head),
        part([0.28, 0.27, 0.0], [0.08, 0.32, 0.05], head),
    ]
}

const fn shovel(head: u16) -> [Part; 5] {
    [
        part([0.0, 0.1, 0.0], [0.08, 0.66, 0.08], tex::HANDLE),
        part([0.0, 0.43, 0.0], [0.22, 0.06, 0.08], tex::HANDLE),
        part([0.0, -0.24, 0.0], [0.1, 0.08, 0.08], head),
        part([0.0, -0.36, 0.0], [0.3, 0.18, 0.05], head),
        part([0.0, -0.47, 0.0], [0.2, 0.05, 0.05], head),
    ]
}

/// A bar over a plate, in steel.
const STEEL_BAR: [Part; 2] = [
    part([0.0, -0.07, 0.0], [0.88, 0.29, 0.53], tex::STEEL_INGOT),
    part([0.0, 0.105, 0.0], [0.69, 0.07, 0.4], tex::STEEL_PLATE),
];
const STEEL_SHEET: [Part; 2] = [
    part([0.0, -0.07, 0.0], [0.84, 0.1, 0.84], tex::STEEL_PLATE),
    part([0.0, 0.005, 0.0], [0.66, 0.05, 0.66], tex::STEEL_PLATE),
];
/// An I-beam on its side: a web between two flanges.
const STEEL_GIRDER: [Part; 3] = [
    part([0.0, 0.0, 0.0], [0.9, 0.12, 0.12], tex::STEEL_BEAM),
    part([0.0, -0.09, 0.0], [0.9, 0.06, 0.34], tex::STEEL_BEAM),
    part([0.0, 0.09, 0.0], [0.9, 0.06, 0.34], tex::STEEL_BEAM),
];

const STONE_PICK: [Part; 6] = pickaxe(tex::STONE);
const IRON_PICK: [Part; 6] = pickaxe(tex::STEEL);
const STONE_HATCHET: [Part; 4] = axe(tex::STONE);
const IRON_HATCHET: [Part; 4] = axe(tex::STEEL);
const STONE_SPADE: [Part; 5] = shovel(tex::STONE);
const IRON_SPADE: [Part; 5] = shovel(tex::STEEL);
const STEEL_PICK: [Part; 6] = pickaxe(tex::STEEL_HEAD);
const STEEL_HATCHET: [Part; 4] = axe(tex::STEEL_HEAD);
const STEEL_SPADE: [Part; 5] = shovel(tex::STEEL_HEAD);

/// A handset: casing, a wooden grip, a steel antenna, and the screen on its front (+z).
const SCANNER_SET: [Part; 4] = [
    part([0.0, 0.0, 0.0], [0.44, 0.56, 0.16], tex::CORE_DRILL),
    part([0.0, -0.36, 0.0], [0.16, 0.18, 0.12], tex::HANDLE),
    part([0.14, 0.36, 0.0], [0.04, 0.18, 0.04], tex::STEEL),
    part([0.0, 0.07, 0.09], [0.34, 0.3, 0.02], tex::SCANNER),
];

/// The Mk2 handset: the same shape with a violet tier stripe across the casing and the Mk2 screen.
const SCANNER_MK2_SET: [Part; 5] = [
    part([0.0, 0.0, 0.0], [0.44, 0.56, 0.16], tex::CORE_DRILL),
    part([0.0, -0.36, 0.0], [0.16, 0.18, 0.12], tex::HANDLE),
    part([0.14, 0.36, 0.0], [0.04, 0.18, 0.04], tex::STEEL),
    part([0.0, 0.07, 0.09], [0.34, 0.3, 0.02], tex::SCANNER_MK2),
    part([0.0, -0.2, 0.0], [0.46, 0.06, 0.18], tex::stripe(3)),
];

/// A motor housing with a carrying bar, over a fluted bit with a steel point.
const DRILL_SET: [Part; 4] = [
    part([0.0, -0.16, 0.0], [0.09, 0.46, 0.09], tex::DRILL),
    part([0.0, -0.43, 0.0], [0.05, 0.08, 0.05], tex::STEEL),
    part([0.0, 0.2, 0.0], [0.4, 0.28, 0.3], tex::CORE_DRILL),
    part([0.0, 0.4, 0.0], [0.3, 0.06, 0.08], tex::HANDLE),
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
        BLUE_PACK => &BLUE_FLASK,
        VIOLET_PACK => &VIOLET_FLASK,
        STONE_PICKAXE => &STONE_PICK,
        IRON_PICKAXE => &IRON_PICK,
        STONE_AXE => &STONE_HATCHET,
        IRON_AXE => &IRON_HATCHET,
        STONE_SHOVEL => &STONE_SPADE,
        IRON_SHOVEL => &IRON_SPADE,
        STEEL_INGOT => &STEEL_BAR,
        STEEL_PLATE => &STEEL_SHEET,
        STEEL_BEAM => &STEEL_GIRDER,
        STEEL_PICKAXE => &STEEL_PICK,
        STEEL_AXE => &STEEL_HATCHET,
        STEEL_SHOVEL => &STEEL_SPADE,
        SCANNER => &SCANNER_SET,
        SCANNER_MK2 => &SCANNER_MK2_SET,
        CORE_DRILL => &DRILL_SET,
        _ => &[],
    }
}

#[cfg(test)]
mod tests;
