//! The railway's docks as spec rows: the loading dock (belts fill it, a train stopped beside it takes the items) and
//! the unloading dock (a train gives it its cargo, belts empty it). Both hold items like a silo (`Pick::Load`,
//! `Pick::Unload`, `Pick::stores`); the trade with a train is `factory/trains/docks.rs`. A train stops at a dock
//! whose cell is within `STOP_REACH` blocks of a rail node it reaches.
//! To add another dock kind: a spec row and parts here, a `Pick` and its arm in `trade`.

use crate::block::{tex, LOADING_DOCK, UNLOADING_DOCK};

use super::super::footprint::{Footprint, Port, Role, Side, Which};
use super::model::{part, Look, Part};
use super::specs::{Energy, Pick, ProcessSpec, ProcessTier};

/// Slots of a dock's store: two wagons' worth.
const SLOTS: usize = 48;

const TIER: [ProcessTier; 1] = [ProcessTier { energy: Energy::Recipe, speed: 1000, fuel: 0, power: 0 }];

const fn ports(role: Role) -> [Port; 4] {
    [
        Port { side: Side::Front, role, cell: Which::All },
        Port { side: Side::Back, role, cell: Which::All },
        Port { side: Side::Left, role, cell: Which::All },
        Port { side: Side::Right, role, cell: Which::All },
    ]
}

pub const LOADING_DOCK_SPEC: ProcessSpec = ProcessSpec {
    block: LOADING_DOCK,
    categories: &[],
    pick: Pick::Load,
    buffers: [0, 0, SLOTS],
    side: 0,
    tiers: &TIER,
    footprint: Footprint { size: [2, 2, 1], ports: &ports(Role::In) },
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
    footprint: Footprint { size: [2, 2, 1], ports: &ports(Role::Out) },
    verb: "Unloading",
    products: "items",
    waiting: "",
    map_colour: 0x30a0e0,
    compute: 0,
    parts: &UNLOADING_PARTS,
};

const FRAME: Look = Look::Tex([tex::FRAME; 3]);
const LOAD: [u16; 3] = [tex::DOCK_LOAD_TOP, tex::DOCK_SIDE, tex::FRAME];
const UNLOAD: [u16; 3] = [tex::DOCK_UNLOAD_TOP, tex::DOCK_SIDE, tex::FRAME];

/// The two docks are opposites. Loading: a low platform with a hopper raised on four legs above it and a spout
/// pointing down, so goods pour into the wagon from above.
const LOADING_PARTS: [Part; 9] = [
    part([0.0, -0.42, 0.0], [1.96, 0.16, 1.96], Look::Band(tex::FRAME)),
    part([0.0, -0.2, 0.0], [1.84, 0.28, 1.84], Look::Tex(LOAD)),
    part([-0.6, 0.3, -0.6], [0.1, 0.7, 0.1], FRAME),
    part([0.6, 0.3, -0.6], [0.1, 0.7, 0.1], FRAME),
    part([-0.6, 0.3, 0.6], [0.1, 0.7, 0.1], FRAME),
    part([0.6, 0.3, 0.6], [0.1, 0.7, 0.1], FRAME),
    part([0.0, 0.8, 0.0], [1.4, 0.4, 1.4], Look::Tex(LOAD)),
    part([0.0, 0.4, 0.0], [0.36, 0.4, 0.36], Look::Tex([tex::STEEL; 3])),
    part([0.85, -0.03, 0.85], [0.12, 0.08, 0.12], Look::Lamp),
];

/// Unloading: a sunken grate between raised curbs that the wagon tips into, and a chute leading away below it.
const UNLOADING_PARTS: [Part; 8] = [
    part([0.0, -0.42, 0.0], [1.96, 0.16, 1.96], Look::Band(tex::FRAME)),
    part([0.0, -0.22, 0.0], [1.84, 0.24, 1.84], Look::Tex(UNLOAD)),
    part([0.0, -0.02, 0.85], [1.8, 0.18, 0.12], FRAME),
    part([0.0, -0.02, -0.85], [1.8, 0.18, 0.12], FRAME),
    part([0.85, -0.02, 0.0], [0.12, 0.18, 1.56], FRAME),
    part([-0.85, -0.02, 0.0], [0.12, 0.18, 1.56], FRAME),
    part([0.0, -0.3, -1.04], [0.5, 0.24, 0.12], Look::Tex([tex::STEEL, tex::SOOT, tex::STEEL])),
    part([0.8, 0.11, 0.8], [0.1, 0.08, 0.1], Look::Lamp),
];
