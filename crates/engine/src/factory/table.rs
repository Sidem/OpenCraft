//! The machine table: which blocks are machines, of which `Kind`, with how many buffer slots and whether
//! right-click opens a panel. One row per kind first, in `Kind` order, then further blocks of an existing
//! kind (a filter is a router; a constructor and an assembler are processors, whose details are spec
//! rows: `process/specs.rs`).
//!
//! To add a machine block: a row here (a new kind also needs its `Kind` variant, in row order).

use crate::block::{
    BlockId, ACCUMULATOR, ARC_FURNACE, ASSEMBLER, BELT, BLAST_FURNACE, BOILER, CABLE, CONSTRUCTOR, CRACKER, CRUSHER,
    DRONE_PORT, ELECTROLYTIC_CELL, FAST_BELT, FILTER, GENERATOR, LAB, LIFT, LOADING_DOCK, MINER, MINER_MK2, OUTLET,
    PIPE, POLE, PUMP, PUMPJACK, QUARRY, RAIL, RAMP_DOWN, RAMP_UP, REFINERY, SENSOR, SILO, SMELTER, SOLAR_PANEL,
    SPLITTER, STORAGE, TURBINE, UNDERPASS_IN, UNDERPASS_OUT, UNLOADING_DOCK,
};
use crate::research::PACKS;
/// Machine kinds, in `MACHINES` order.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Kind {
    Belt,
    Miner,
    Storage,
    Process,
    Router,
    Generator,
    Pole,
    Lab,
    Pipe,
    Quarry,
    Sensor,
    Rail,
}

pub struct MachineDef {
    /// The block that is this machine (its name, textures and breaking come from `block.rs`).
    pub block: BlockId,
    pub kind: Kind,
    /// Item stacks per buffer: a box's slots, a miner's output. Belts carry items instead, and
    /// processors take theirs from their spec (0).
    pub slots: usize,
    /// Right-click opens its panel (`panel.rs`) instead of taking what it holds.
    pub panel: bool,
}

/// The machine table: first one row per kind, in `Kind` order (`Kind::def`), then further blocks of
/// an existing kind.
pub const MACHINES: [MachineDef; 40] = [
    MachineDef { block: BELT, kind: Kind::Belt, slots: 0, panel: false },
    MachineDef { block: MINER, kind: Kind::Miner, slots: 1, panel: false },
    MachineDef { block: STORAGE, kind: Kind::Storage, slots: 24, panel: true },
    MachineDef { block: SMELTER, kind: Kind::Process, slots: 0, panel: true },
    MachineDef { block: SPLITTER, kind: Kind::Router, slots: 0, panel: false },
    MachineDef { block: GENERATOR, kind: Kind::Generator, slots: 1, panel: true },
    MachineDef { block: POLE, kind: Kind::Pole, slots: 0, panel: false },
    MachineDef { block: LAB, kind: Kind::Lab, slots: PACKS.len(), panel: true },
    MachineDef { block: PIPE, kind: Kind::Pipe, slots: 0, panel: false },
    MachineDef { block: QUARRY, kind: Kind::Quarry, slots: 4, panel: true },
    MachineDef { block: SENSOR, kind: Kind::Sensor, slots: 0, panel: true },
    MachineDef { block: RAIL, kind: Kind::Rail, slots: 0, panel: false },
    MachineDef { block: FILTER, kind: Kind::Router, slots: 0, panel: true },
    MachineDef { block: RAMP_UP, kind: Kind::Belt, slots: 0, panel: false },
    MachineDef { block: RAMP_DOWN, kind: Kind::Belt, slots: 0, panel: false },
    MachineDef { block: LIFT, kind: Kind::Belt, slots: 0, panel: false },
    MachineDef { block: UNDERPASS_IN, kind: Kind::Belt, slots: 0, panel: false },
    MachineDef { block: UNDERPASS_OUT, kind: Kind::Belt, slots: 0, panel: false },
    MachineDef { block: CONSTRUCTOR, kind: Kind::Process, slots: 0, panel: true },
    MachineDef { block: ASSEMBLER, kind: Kind::Process, slots: 0, panel: true },
    MachineDef { block: BLAST_FURNACE, kind: Kind::Process, slots: 0, panel: true },
    MachineDef { block: BOILER, kind: Kind::Process, slots: 0, panel: true },
    MachineDef { block: TURBINE, kind: Kind::Process, slots: 0, panel: false },
    MachineDef { block: CRUSHER, kind: Kind::Process, slots: 0, panel: true },
    MachineDef { block: SILO, kind: Kind::Process, slots: 0, panel: true },
    MachineDef { block: ARC_FURNACE, kind: Kind::Process, slots: 0, panel: true },
    MachineDef { block: DRONE_PORT, kind: Kind::Process, slots: 0, panel: true },
    MachineDef { block: CABLE, kind: Kind::Pole, slots: 0, panel: false },
    MachineDef { block: SOLAR_PANEL, kind: Kind::Process, slots: 0, panel: false },
    MachineDef { block: ACCUMULATOR, kind: Kind::Process, slots: 0, panel: false },
    MachineDef { block: ELECTROLYTIC_CELL, kind: Kind::Process, slots: 0, panel: true },
    MachineDef { block: LOADING_DOCK, kind: Kind::Process, slots: 0, panel: true },
    MachineDef { block: UNLOADING_DOCK, kind: Kind::Process, slots: 0, panel: true },
    MachineDef { block: PUMPJACK, kind: Kind::Process, slots: 0, panel: true },
    MachineDef { block: REFINERY, kind: Kind::Process, slots: 0, panel: true },
    MachineDef { block: CRACKER, kind: Kind::Process, slots: 0, panel: true },
    // Legacy blocks: worlds from before tiers (`tiers.rs`) still hold them; nothing places them now.
    MachineDef { block: MINER_MK2, kind: Kind::Miner, slots: 1, panel: false },
    MachineDef { block: FAST_BELT, kind: Kind::Belt, slots: 0, panel: false },
    MachineDef { block: PUMP, kind: Kind::Pipe, slots: 0, panel: false },
    MachineDef { block: OUTLET, kind: Kind::Pipe, slots: 0, panel: false },
];

impl Kind {
    pub fn def(self) -> &'static MachineDef {
        &MACHINES[self as usize]
    }
}

/// The machine `block` is, if it is one.
pub fn machine(block: BlockId) -> Option<&'static MachineDef> {
    MACHINES.iter().find(|m| m.block == block)
}
