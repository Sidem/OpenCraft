//! The railway's docks as spec rows: the loading dock (belts fill it, a train stopped on it takes the items) and the
//! unloading dock (a train gives it its cargo, belts empty it). Both hold items like a silo (`Pick::Load`,
//! `Pick::Unload`, `Pick::stores`); the trade with a train is `factory/trains/docks.rs`. A dock is a piece of rail:
//! one cell standing on a straight rail node and taking its place (`factory/rail/pieces.rs`), its track along the
//! model's z. Belts feed it and empty it from the sides; the cars run through the middle, so every part keeps out of
//! the space a car needs (`CAR_HALF` either side, below `CAR_TOP`).
//! To add another dock kind: a spec row and parts here, a `Pick` and its arm in `trade`.

use crate::block::{tex, LOADING_DOCK, UNLOADING_DOCK};

use super::super::footprint::Footprint;
use super::model::{part, round, Look, Part};
use super::specs::{Energy, Pick, ProcessSpec, ProcessTier};

/// Slots of a dock's store: two wagons' worth.
const SLOTS: usize = 48;

const TIER: [ProcessTier; 1] = [ProcessTier { energy: Energy::Recipe, speed: 1000, fuel: 0, power: 0 }];

pub const LOADING_DOCK_SPEC: ProcessSpec = ProcessSpec {
    block: LOADING_DOCK,
    categories: &[],
    pick: Pick::Load,
    buffers: [0, 0, SLOTS],
    side: 0,
    tiers: &TIER,
    footprint: Footprint { size: [1, 1, 1], ports: &[] },
    verb: "Loading",
    products: "items",
    waiting: "",
    map_colour: 0xe0a030,
    compute: 0,
    parts: &LOADING_PARTS,
};

pub const UNLOADING_DOCK_SPEC: ProcessSpec = ProcessSpec {
    block: UNLOADING_DOCK,
    categories: &[],
    pick: Pick::Unload,
    buffers: [0, 0, SLOTS],
    side: 0,
    tiers: &TIER,
    footprint: Footprint { size: [1, 1, 1], ports: &[] },
    verb: "Unloading",
    products: "items",
    waiting: "",
    map_colour: 0x30a0e0,
    compute: 0,
    parts: &UNLOADING_PARTS,
};

/// The space a car runs through, from the cell's centre: half its width, and its roof.
pub const CAR_HALF: f32 = 0.56;
pub const CAR_TOP: f32 = 1.12;
/// The wagon's rim (`trains/model.rs`), where a loading dock's goods land; the intakes and outlets just outside the cars.
const WAGON_TOP: f32 = 0.7;
const SIDE_X: f32 = CAR_HALF + 0.1;

const FRAME: Look = Look::Tex([tex::FRAME; 3]);
const STEEL: Look = Look::Tex([tex::STEEL; 3]);
const DECK: Look = Look::Tex([tex::CONCRETE, tex::CONCRETE, tex::FRAME]);
const LOAD: Look = Look::Tex([tex::DOCK_LOAD_TOP, tex::DOCK_SIDE, tex::FRAME]);
const UNLOAD: Look = Look::Tex([tex::DOCK_UNLOAD_TOP, tex::DOCK_SIDE, tex::FRAME]);
/// The rails through the dock, as the track's (`rail/curve.rs`): gauge, height, thickness.
const RAIL_X: f32 = 0.18;
const RAIL_Y: f32 = -0.38;

/// The loading dock: a gantry over the track. Belts deliver to an intake at either side; a bucket elevator lifts the
/// goods into the round bin on top, and a chute hangs from it to just over the wagon.
const LOADING_PARTS: [Part; 22] = [
    part([0.0, -0.475, 0.0], [1.2, 0.05, 1.0], DECK),
    part([-RAIL_X, RAIL_Y, 0.0], [0.06, 0.08, 1.0], STEEL),
    part([RAIL_X, RAIL_Y, 0.0], [0.06, 0.08, 1.0], STEEL),
    // Four legs and the frame on them.
    part([-0.66, 0.5, -0.42], [0.08, 1.9, 0.08], FRAME),
    part([0.66, 0.5, -0.42], [0.08, 1.9, 0.08], FRAME),
    part([-0.66, 0.5, 0.42], [0.08, 1.9, 0.08], FRAME),
    part([0.66, 0.5, 0.42], [0.08, 1.9, 0.08], FRAME),
    part([0.0, 1.3, -0.42], [1.5, 0.1, 0.1], Look::Band(tex::FRAME)),
    part([0.0, 1.3, 0.42], [1.5, 0.1, 0.1], Look::Band(tex::FRAME)),
    part([-0.66, 1.3, 0.0], [0.1, 0.08, 0.84], FRAME),
    part([0.66, 1.3, 0.0], [0.1, 0.08, 0.84], FRAME),
    // The bin, its cone and the chute.
    round([0.0, 1.66, 0.0], [1.0, 0.56, 1.0], LOAD),
    round([0.0, 1.32, 0.0], [0.6, 0.14, 0.6], STEEL),
    round([0.0, CAR_TOP + 0.1, 0.0], [0.26, 0.16, 0.26], Look::Tex([tex::SOOT; 3])),
    part([0.0, (CAR_TOP + WAGON_TOP) / 2.0 + 0.01, 0.0], [0.16, CAR_TOP + 0.02 - WAGON_TOP, 0.16], Look::Pour),
    // Intakes where the belts come in, and the elevators up to the bin.
    part([-SIDE_X, -0.3, 0.0], [0.2, 0.36, 0.5], LOAD),
    part([SIDE_X, -0.3, 0.0], [0.2, 0.36, 0.5], LOAD),
    part([-0.66, 0.61, 0.0], [0.16, 1.52, 0.22], STEEL),
    part([0.66, 0.61, 0.0], [0.16, 1.52, 0.22], STEEL),
    part([-0.42, 1.62, 0.0], [0.4, 0.14, 0.18], STEEL),
    part([0.42, 1.62, 0.0], [0.4, 0.14, 0.18], STEEL),
    part([0.0, 2.0, 0.0], [0.14, 0.1, 0.14], Look::Lamp),
];

/// The unloading dock: a shed over a grate between the rails that the wagons empty through. Outlets at either side
/// hand the goods to belts; the roof is blue, the loading dock's opposite.
const UNLOADING_PARTS: [Part; 20] = [
    part([0.0, -0.475, 0.0], [1.2, 0.05, 1.0], DECK),
    part([0.0, -0.44, 0.0], [0.3, 0.02, 0.9], UNLOAD),
    part([-RAIL_X, RAIL_Y, 0.0], [0.06, 0.08, 1.0], STEEL),
    part([RAIL_X, RAIL_Y, 0.0], [0.06, 0.08, 1.0], STEEL),
    // Two portal frames and the roof on them.
    part([-0.66, 0.42, -0.4], [0.1, 1.74, 0.1], FRAME),
    part([0.66, 0.42, -0.4], [0.1, 1.74, 0.1], FRAME),
    part([-0.66, 0.42, 0.4], [0.1, 1.74, 0.1], FRAME),
    part([0.66, 0.42, 0.4], [0.1, 1.74, 0.1], FRAME),
    part([0.0, 1.32, -0.4], [1.5, 0.12, 0.12], Look::Band(tex::FRAME)),
    part([0.0, 1.32, 0.4], [1.5, 0.12, 0.12], Look::Band(tex::FRAME)),
    part([0.0, 1.43, 0.0], [1.6, 0.08, 1.1], UNLOAD),
    part([0.0, 1.5, 0.0], [0.5, 0.06, 0.9], STEEL),
    // Outlets to the belts at either side, fed by the hoppers under the grate.
    part([-SIDE_X, -0.3, 0.0], [0.2, 0.36, 0.5], UNLOAD),
    part([SIDE_X, -0.3, 0.0], [0.2, 0.36, 0.5], UNLOAD),
    part([0.0, -0.27, 0.0], [0.14, 0.32, 0.14], Look::Pour),
    part([-0.45, -0.43, 0.0], [0.26, 0.06, 0.4], Look::Tex([tex::SOOT; 3])),
    part([0.45, -0.43, 0.0], [0.26, 0.06, 0.4], Look::Tex([tex::SOOT; 3])),
    // Shakers on the posts that loosen the load.
    part([-0.66, 0.2, -0.26], [0.16, 0.2, 0.14], STEEL),
    part([0.66, 0.2, 0.26], [0.16, 0.2, 0.14], STEEL),
    part([0.66, 1.62, 0.42], [0.12, 0.08, 0.12], Look::Lamp),
];
