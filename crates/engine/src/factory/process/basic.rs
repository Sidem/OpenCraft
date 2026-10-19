//! The spec rows of the first processors, smelter to electrolytic cell: machines with no logic of their own beyond
//! `work.rs`, listed in `SPECS` (`specs.rs`) like every other machine file's rows. To add a plain processor: its row
//! here and its name in `SPECS`; a machine with logic of its own gets a file and its row there.

use crate::block::{
    ARC_FURNACE, ASSEMBLER, BLAST_FURNACE, BOILER, CONSTRUCTOR, CRUSHER, ELECTROLYTIC_CELL, SILO, SMELTER, TURBINE,
};
use crate::recipes::Category;

use super::super::footprint::{Footprint, Port, Role, Side, Which, SINGLE};
use super::parts::*;
use super::specs::{Energy, Pick, ProcessSpec, ProcessTier};

pub(super) const SMELTER_SPEC: ProcessSpec = ProcessSpec {
    block: SMELTER,
    categories: &[Category::Smelting],
    pick: Pick::ByInput,
    buffers: [1, 1, 1],
    side: 0,
    tiers: &[
        ProcessTier { energy: Energy::Burner, speed: 1000, fuel: 1000, power: 0 },
        ProcessTier { energy: Energy::Burner, speed: 2000, fuel: 750, power: 0 },
        ProcessTier { energy: Energy::Electric, speed: 3000, fuel: 0, power: 40 },
        ProcessTier { energy: Energy::Electric, speed: 5000, fuel: 0, power: 80 },
        ProcessTier { energy: Energy::Electric, speed: 8000, fuel: 0, power: 130 },
    ],
    footprint: SINGLE,
    verb: "Smelting",
    products: "ingots",
    waiting: "Waiting for ore, sand or stone",
    map_colour: 0xe5533d,
    compute: 0,
    parts: &SMELTER_PARTS,
};

pub(super) const CONSTRUCTOR_SPEC: ProcessSpec = ProcessSpec {
    block: CONSTRUCTOR,
    categories: &[Category::Pressing],
    pick: Pick::Chosen,
    buffers: [1, 0, 1],
    side: 0,
    tiers: &[
        ProcessTier { energy: Energy::Electric, speed: 1000, fuel: 0, power: 15 },
        ProcessTier { energy: Energy::Electric, speed: 2000, fuel: 0, power: 30 },
        ProcessTier { energy: Energy::Electric, speed: 3000, fuel: 0, power: 45 },
        ProcessTier { energy: Energy::Electric, speed: 5000, fuel: 0, power: 75 },
        ProcessTier { energy: Energy::Electric, speed: 8000, fuel: 0, power: 120 },
    ],
    footprint: SINGLE,
    verb: "Making",
    products: "parts",
    waiting: "",
    map_colour: 0x4a90e2,
    compute: 0,
    parts: &CONSTRUCTOR_PARTS,
};

pub(super) const ASSEMBLER_SPEC: ProcessSpec = ProcessSpec {
    block: ASSEMBLER,
    categories: &[Category::Assembly],
    pick: Pick::Chosen,
    buffers: [3, 0, 1],
    side: 0,
    tiers: &[
        ProcessTier { energy: Energy::Electric, speed: 1000, fuel: 0, power: 20 },
        ProcessTier { energy: Energy::Electric, speed: 2000, fuel: 0, power: 40 },
        ProcessTier { energy: Energy::Electric, speed: 3000, fuel: 0, power: 60 },
        ProcessTier { energy: Energy::Electric, speed: 5000, fuel: 0, power: 100 },
        ProcessTier { energy: Energy::Electric, speed: 8000, fuel: 0, power: 160 },
    ],
    footprint: Footprint {
        size: [2, 2, 2],
        ports: &[inlet(Side::Back), inlet(Side::Left), inlet(Side::Right), OUT_FRONT],
    },
    verb: "Assembling",
    products: "parts",
    waiting: "",
    map_colour: 0x34a898,
    compute: 0,
    parts: &ASSEMBLER_PARTS,
};

pub(super) const BLAST_SPEC: ProcessSpec = ProcessSpec {
    block: BLAST_FURNACE,
    categories: &[Category::Blasting],
    pick: Pick::ByInput,
    buffers: [3, 0, 1],
    side: 1,
    tiers: &[
        ProcessTier { energy: Energy::Recipe, speed: 1000, fuel: 0, power: 0 },
        ProcessTier { energy: Energy::Recipe, speed: 2000, fuel: 0, power: 0 },
        ProcessTier { energy: Energy::Recipe, speed: 3000, fuel: 0, power: 0 },
        ProcessTier { energy: Energy::Recipe, speed: 5000, fuel: 0, power: 0 },
        ProcessTier { energy: Energy::Recipe, speed: 8000, fuel: 0, power: 0 },
    ],
    footprint: Footprint {
        size: [2, 2, 3],
        ports: &[
            inlet(Side::Back),
            inlet(Side::Left),
            OUT_FRONT,
            Port { side: Side::Right, role: Role::Side, cell: Which::All },
        ],
    },
    verb: "Blasting",
    products: "steel",
    waiting: "",
    map_colour: 0xb84a30,
    compute: 0,
    parts: &BLAST_PARTS,
};

pub(super) const BOILER_SPEC: ProcessSpec = ProcessSpec {
    block: BOILER,
    categories: &[],
    pick: Pick::ByInput,
    buffers: [0, 2, 0],
    side: 0,
    tiers: &[ProcessTier { energy: Energy::Boiler, speed: 1000, fuel: 0, power: 0 }],
    footprint: Footprint { size: [2, 2, 2], ports: &BOILER_PORTS },
    verb: "Boiling",
    products: "steam",
    waiting: "Idle",
    map_colour: 0x3a5f9a,
    compute: 0,
    parts: &BOILER_PARTS,
};

pub(super) const TURBINE_SPEC: ProcessSpec = ProcessSpec {
    block: TURBINE,
    categories: &[],
    pick: Pick::ByInput,
    buffers: [0, 0, 0],
    side: 0,
    tiers: &[ProcessTier { energy: Energy::Turbine, speed: 1000, fuel: 0, power: 0 }],
    footprint: Footprint { size: [3, 2, 2], ports: &TURBINE_PORTS },
    verb: "Turning",
    products: "power",
    waiting: "Idle",
    map_colour: 0xc8ced8,
    compute: 0,
    parts: &TURBINE_PARTS,
};

pub(super) const CRUSHER_SPEC: ProcessSpec = ProcessSpec {
    block: CRUSHER,
    categories: &[Category::Crushing],
    pick: Pick::ByInput,
    buffers: [1, 0, 1],
    side: 0,
    tiers: &[ProcessTier { energy: Energy::Electric, speed: 1000, fuel: 0, power: 30 }],
    footprint: SINGLE,
    verb: "Crushing",
    products: "crushed ore",
    waiting: "Waiting for iron, copper or bauxite ore, or slag",
    map_colour: 0xd6a930,
    compute: 0,
    parts: &CRUSHER_PARTS,
};

pub(super) const SILO_SPEC: ProcessSpec = ProcessSpec {
    block: SILO,
    categories: &[],
    pick: Pick::Store,
    buffers: [0, 0, 144],
    side: 0,
    tiers: &[ProcessTier { energy: Energy::Recipe, speed: 1000, fuel: 0, power: 0 }],
    footprint: Footprint {
        size: [2, 2, 3],
        ports: &[inlet(Side::Back), inlet(Side::Left), inlet(Side::Right), OUT_FRONT],
    },
    verb: "Storing",
    products: "items",
    waiting: "",
    map_colour: 0x9a9a92,
    compute: 0,
    parts: &SILO_PARTS,
};

pub(super) const ARC_SPEC: ProcessSpec = ProcessSpec {
    block: ARC_FURNACE,
    categories: &[Category::Arc],
    pick: Pick::Chosen,
    buffers: [2, 0, 1],
    side: 0,
    tiers: &[ProcessTier { energy: Energy::Electric, speed: 1000, fuel: 0, power: 120 }],
    footprint: Footprint {
        size: [2, 2, 2],
        ports: &[inlet(Side::Back), inlet(Side::Left), inlet(Side::Right), OUT_FRONT],
    },
    verb: "Arcing",
    products: "silicon",
    waiting: "",
    map_colour: 0x9a7ad8,
    compute: 0,
    parts: &ARC_PARTS,
};

pub(super) const CELL_SPEC: ProcessSpec = ProcessSpec {
    block: ELECTROLYTIC_CELL,
    categories: &[Category::Electrolysis],
    pick: Pick::Chosen,
    buffers: [2, 0, 1],
    side: 1,
    tiers: &[ProcessTier { energy: Energy::Electric, speed: 1000, fuel: 0, power: 300 }],
    footprint: Footprint {
        size: [3, 2, 2],
        ports: &[
            inlet(Side::Back),
            inlet(Side::Left),
            OUT_FRONT,
            Port { side: Side::Right, role: Role::Side, cell: Which::All },
        ],
    },
    verb: "Electrolysing",
    products: "aluminium",
    waiting: "",
    map_colour: 0xcfd8e3,
    compute: 0,
    parts: &CELL_PARTS,
};

const fn inlet(side: Side) -> Port {
    Port { side, role: Role::In, cell: Which::All }
}

const OUT_FRONT: Port = Port { side: Side::Front, role: Role::Out, cell: Which::All };

/// A boiler's ports: on each of the back, left and right a belt inlet for fuel and, on the other cell of the
/// side, a pipe inlet for water (alternating round the corners); two steam outlets on the front.
const BOILER_PORTS: [Port; 7] = [
    Port { side: Side::Back, role: Role::In, cell: Which::Nth(0) },
    Port { side: Side::Back, role: Role::Water, cell: Which::Nth(1) },
    Port { side: Side::Left, role: Role::In, cell: Which::Nth(0) },
    Port { side: Side::Left, role: Role::Water, cell: Which::Nth(1) },
    Port { side: Side::Right, role: Role::In, cell: Which::Nth(1) },
    Port { side: Side::Right, role: Role::Water, cell: Which::Nth(0) },
    Port { side: Side::Front, role: Role::Steam, cell: Which::All },
];

/// A turbine's steam inlets: both cells of its right end (the end opposite the generator).
const TURBINE_PORTS: [Port; 1] = [Port { side: Side::Right, role: Role::Steam, cell: Which::All }];
