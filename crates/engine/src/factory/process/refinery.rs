//! The refinery, the cracker, the chemical plant and the electrolyser (Milestone 10): chemistry processors, ordinary recipe machines
//! (`Pick::ByInput`, categories `Distilling` and `Cracking`; the plant is `Pick::Chosen`, `Chemistry`) that also take
//! **water** from a pipe: a blue inlet on the left side (`Role::Water`; `steam.rs` links it and `draw_water` fills
//! the machine's small tank, a few units) and a recipe's `water_use` units are spent when a batch starts. With the
//! tank empty a machine reports `NoWater` and waits.
//!
//! - **Shells go through:** Distil turns 3 crude oil canisters into a naphtha, a diesel and a heavy oil canister
//!   and a sulfur; Crack turns 2 heavy oil canisters into a naphtha and a diesel canister. The first output leaves
//!   the front, the others the side hatches on the right (a mixed belt: the player sorts it with filters). A full
//!   stream stops the machine (`Status::OutputFull`, with the stream named), so using or storing every one is the
//!   game. Numbers: docs/TECH_ERAS.md section 6. The plant (plastic, acid, lubricant) hands the empty canisters
//!   back through its side hatch.
//!
//! To add a chemistry machine: a spec row here, a `Category`, a block and a `MACHINES` row.

use crate::block::{tex, CHEMICAL_PLANT, CRACKER, ELECTROLYSER, REFINERY};

use super::super::footprint::{Footprint, Port, Role, Side, Which};
use super::model::{part, Look, Part};
use super::{Energy, Pick, ProcessSpec, ProcessTier};
use crate::recipes::Category;

#[cfg(test)]
mod tests;

const IN_BACK: Port = Port { side: Side::Back, role: Role::In, cell: Which::All };
const OUT_FRONT: Port = Port { side: Side::Front, role: Role::Out, cell: Which::All };
const SIDE_RIGHT: Port = Port { side: Side::Right, role: Role::Side, cell: Which::All };

pub const REFINERY_SPEC: ProcessSpec = ProcessSpec {
    block: REFINERY,
    categories: &[Category::Distilling],
    pick: Pick::ByInput,
    buffers: [1, 0, 1],
    side: 3,
    tiers: &[ProcessTier { energy: Energy::Electric, speed: 1000, fuel: 0, power: 150 }],
    footprint: Footprint {
        size: [3, 3, 4],
        ports: &[IN_BACK, OUT_FRONT, SIDE_RIGHT, Port { side: Side::Left, role: Role::Water, cell: Which::Nth(1) }],
    },
    verb: "Distilling",
    products: "naphtha",
    waiting: "Waiting for crude oil canisters",
    map_colour: 0x6a7a8e,
    parts: &REFINERY_PARTS,
};

pub const CRACKER_SPEC: ProcessSpec = ProcessSpec {
    block: CRACKER,
    categories: &[Category::Cracking],
    pick: Pick::ByInput,
    buffers: [1, 0, 1],
    side: 1,
    tiers: &[ProcessTier { energy: Energy::Electric, speed: 1000, fuel: 0, power: 90 }],
    footprint: Footprint {
        size: [2, 2, 3],
        ports: &[IN_BACK, OUT_FRONT, SIDE_RIGHT, Port { side: Side::Left, role: Role::Water, cell: Which::Nth(0) }],
    },
    verb: "Cracking",
    products: "naphtha",
    waiting: "Waiting for heavy oil canisters",
    map_colour: 0x96705f,
    parts: &CRACKER_PARTS,
};

/// The chemical plant: the player chooses plastic, acid or lubricant in its panel (`Pick::Chosen`). Belts bring the
/// inputs in at the back, the product leaves the front, and plastic's two spare shells the side hatches.
pub const PLANT_SPEC: ProcessSpec = ProcessSpec {
    block: CHEMICAL_PLANT,
    categories: &[Category::Chemistry],
    pick: Pick::Chosen,
    buffers: [3, 0, 1],
    side: 1,
    tiers: &[ProcessTier { energy: Energy::Electric, speed: 1000, fuel: 0, power: 120 }],
    footprint: Footprint {
        size: [3, 2, 3],
        ports: &[IN_BACK, OUT_FRONT, SIDE_RIGHT, Port { side: Side::Left, role: Role::Water, cell: Which::Nth(0) }],
    },
    verb: "Making",
    products: "plastic, acid or lubricant",
    waiting: "",
    map_colour: 0x5fa88f,
    parts: &PLANT_PARTS,
};

/// The electrolyser splits water into hydrogen and oxygen canisters (`Splitting`): empties in at the back, hydrogen out
/// of the front and oxygen from the right-hand hatch.
pub const ELECTROLYSER_SPEC: ProcessSpec = ProcessSpec {
    block: ELECTROLYSER,
    categories: &[Category::Splitting],
    pick: Pick::ByInput,
    buffers: [1, 0, 1],
    side: 1,
    tiers: &[ProcessTier { energy: Energy::Electric, speed: 1000, fuel: 0, power: 500 }],
    footprint: Footprint {
        size: [2, 2, 2],
        ports: &[IN_BACK, OUT_FRONT, SIDE_RIGHT, Port { side: Side::Left, role: Role::Water, cell: Which::Nth(0) }],
    },
    verb: "Splitting",
    products: "hydrogen",
    waiting: "Waiting for empty canisters",
    map_colour: 0x4f86a8,
    parts: &ELECTROLYSER_PARTS,
};
const PLANT_BODY: [u16; 3] = [tex::CHEM_TOP, tex::CHEM_SIDE, tex::FRAME];

/// Across 3, deep 2, 3 high (±1.5, ±1, ±1.5): a banded plinth, a low house across the front, a tall and a short
/// storage tank behind it with a pipe bridge between their tops, and a status lamp.
const PLANT_PARTS: [Part; 8] = [
    part([0.0, -1.4, 0.0], [2.96, 0.2, 1.96], Look::Band(tex::FRAME)),
    part([0.0, -0.8, 0.35], [2.8, 1.1, 1.2], Look::Tex(PLANT_BODY)),
    part([-0.8, 0.3, -0.4], [1.0, 2.0, 1.0], Look::Tex(PLANT_BODY)),
    part([-0.8, 1.35, -0.4], [1.1, 0.1, 1.1], FRAME),
    part([0.7, -0.1, -0.4], [1.0, 1.4, 1.0], Look::Tex(PLANT_BODY)),
    part([0.7, 0.65, -0.4], [1.1, 0.1, 1.1], FRAME),
    part([0.0, 1.1, -0.4], [1.6, 0.12, 0.12], STEEL),
    part([1.2, -0.25, 0.98], [0.14, 0.1, 0.14], Look::Lamp),
];

const CELL_BODY: [u16; 3] = [tex::ELECTROLYSER_TOP, tex::ELECTROLYSER_SIDE, tex::FRAME];

/// Across and deep 2, 2 high (±1): a banded plinth, a low house across the front, two tall cell stacks behind it
/// joined by a copper bus bar, and a status lamp.
const ELECTROLYSER_PARTS: [Part; 6] = [
    part([0.0, -0.9, 0.0], [1.96, 0.2, 1.96], Look::Band(tex::FRAME)),
    part([0.0, -0.4, 0.45], [1.8, 0.8, 1.0], Look::Tex(CELL_BODY)),
    part([-0.5, 0.2, -0.4], [0.7, 1.6, 0.8], Look::Tex(CELL_BODY)),
    part([0.5, 0.2, -0.4], [0.7, 1.6, 0.8], Look::Tex(CELL_BODY)),
    part([0.0, 0.95, -0.4], [1.4, 0.12, 0.2], Look::Tex([tex::COPPER_INGOT; 3])),
    part([0.78, 0.0, 0.92], [0.14, 0.1, 0.14], Look::Lamp),
];
const REFINERY_BODY: [u16; 3] = [tex::REFINERY_TOP, tex::REFINERY_SIDE, tex::FRAME];
const CRACKER_BODY: [u16; 3] = [tex::CRACKER_TOP, tex::CRACKER_SIDE, tex::FRAME];
const STEEL: Look = Look::Tex([tex::STEEL; 3]);
const FRAME: Look = Look::Tex([tex::FRAME; 3]);

/// Parts are relative to the footprint's centre (3×3 across, 4 high: ±1.5 and ±2). A banded plinth, a low furnace
/// house across the front, a tall distillation column and a slimmer one behind it joined by a pipe bridge, a flare
/// that burns while it works and a status lamp.
const REFINERY_PARTS: [Part; 10] = [
    part([0.0, -1.9, 0.0], [2.96, 0.2, 2.96], Look::Band(tex::FRAME)),
    part([0.0, -1.2, 0.65], [2.5, 1.3, 1.5], Look::Tex(REFINERY_BODY)),
    part([0.0, -0.5, 0.65], [2.6, 0.1, 1.6], FRAME),
    part([-0.55, 0.2, -0.55], [1.1, 3.4, 1.1], Look::Tex(REFINERY_BODY)),
    part([-0.55, 1.95, -0.55], [1.2, 0.12, 1.2], FRAME),
    part([0.75, -0.1, -0.7], [0.7, 2.4, 0.7], Look::Tex(REFINERY_BODY)),
    part([0.75, 1.15, -0.7], [0.8, 0.1, 0.8], FRAME),
    part([0.1, 1.3, -0.62], [1.4, 0.14, 0.14], STEEL),
    part([-0.55, 2.2, -0.55], [0.2, 0.4, 0.2], Look::Fire(tex::GENERATOR_SIDE)),
    part([1.2, -0.45, 1.42], [0.14, 0.1, 0.14], Look::Lamp),
];

/// Across 2 and 3 high (±1 and ±1.5): a banded plinth, a house, one tall reactor column with a cap, a pipe stub
/// and a status lamp.
const CRACKER_PARTS: [Part; 6] = [
    part([0.0, -1.4, 0.0], [1.96, 0.2, 1.96], Look::Band(tex::FRAME)),
    part([0.0, -0.75, 0.3], [1.8, 1.1, 1.3], Look::Tex(CRACKER_BODY)),
    part([-0.2, 0.3, -0.35], [0.9, 2.2, 0.9], Look::Tex(CRACKER_BODY)),
    part([-0.2, 1.45, -0.35], [1.0, 0.1, 1.0], FRAME),
    part([0.45, 0.4, -0.35], [0.14, 0.14, 0.8], STEEL),
    part([0.75, -0.35, 0.98], [0.12, 0.08, 0.12], Look::Lamp),
];
