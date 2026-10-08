//! Processing machines as data: one [`ProcessSpec`] row per block that turns inputs into outputs
//! (docs/TECH_TREE.md section 8): the recipe categories it takes, how it is driven, how it picks a
//! recipe, its buffers, its numbers per tier, its footprint and ports (`footprint/`) and its model parts.
//!
//! Invariants: a spec has a tier row for every tier of its family (`tiers.rs`; tested); a spec with no
//! burner tier has no fuel buffer; its block is a `Kind::Process` row in `MACHINES`.
//!
//! To add a processor: its block (`block/`), a `MACHINES` row of `Kind::Process`, a row here with its
//! parts, and its recipes' category (`recipes/machine.rs`).

use crate::block::{
    BlockId, ARC_FURNACE, ASSEMBLER, BLAST_FURNACE, BOILER, CONSTRUCTOR, CRUSHER, ELECTROLYTIC_CELL, SILO, SMELTER,
    TURBINE,
};
use crate::item::ItemId;
use crate::recipes::{Category, MachineRecipe, MACHINE_RECIPES};

use super::super::footprint::{Footprint, Port, Role, Side, Which, SINGLE};
use super::model::Part;
use super::parts::*;

/// How a processor is driven.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Energy {
    /// Burns fuel (`recipes::FUELS`) from its fuel buffer while a batch runs.
    Burner,
    /// Draws `ProcessTier::power` from a power grid (`power.rs`) while it works.
    Electric,
    /// Needs nothing beyond its recipe (a blast furnace's coal is an input): always at full speed.
    Recipe,
    /// A boiler: burns fuel from its fuel buffer into steam for the turbines piped to its outlets (steam.rs).
    Boiler,
    /// A steam turbine: a power source fed by the boilers piped to its steam inlet (steam.rs).
    Turbine,
    /// A solar panel: a power source that follows the sun (solar.rs).
    Solar,
    /// An accumulator: stores spare solar power and gives it back before any fuel burns (solar.rs).
    Accumulator,
    /// A diesel generator: a power source burning diesel canisters (diesel.rs).
    Diesel,
    /// A water wheel: a power source driven by the water that touches it (hydro.rs).
    Hydro,
    /// A hoist winch: a power sink that makes the shaft beside it fast (hoist.rs).
    Hoist,
    /// A nuclear reactor: a water-cooled power source burning fuel cells (nuclear.rs).
    Reactor,
    /// A datacenter: draws power at all times and gives the data grid compute, water-cooled (datacenter.rs).
    Compute,
    /// A cooling tower: a power sink that closes a datacenter's water loop (tower.rs).
    Cooling,
    /// An optimizer: a power sink and data consumer that speeds up the machines around it (optimizer.rs).
    Optimizer,
}

impl Energy {
    /// Whether it is a power source or store with a step of its own in `Power::balance` (nothing to do in `step`).
    pub fn is_source(self) -> bool {
        matches!(
            self,
            Energy::Turbine | Energy::Solar | Energy::Accumulator | Energy::Diesel | Energy::Hydro | Energy::Reactor
        )
    }
}

/// How a processor picks its recipe.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Pick {
    /// The player chooses in its panel (`SetRecipe`); it takes only that recipe's inputs.
    Chosen,
    /// What it holds decides: it takes anything an unlocked recipe of its categories uses.
    ByInput,
    /// Takes anything into its output buffer, which belts empty (a silo: steam.rs).
    Store,
    /// Takes only drones, up to its fleet, and keeps them for `drones/` to fly (hangar.rs).
    Hangar,
    /// Takes only empty canisters and fills them from the reservoir below (pump.rs).
    Pump,
    /// A loading dock: stores like a silo, and a train stopped beside it takes what it holds (docks.rs).
    Load,
    /// An unloading dock: stores like a silo, a train stopped beside it gives it its cargo (docks.rs).
    Unload,
    /// A research center: takes science packs and researches the chosen tech (center.rs).
    Research,
    /// A recycler: destroys any item it is given and pays coins (recycler.rs).
    Recycle,
}

impl Pick {
    /// Whether it holds anything it is given in its output buffer: silos and docks.
    pub fn stores(self) -> bool {
        matches!(self, Pick::Store | Pick::Load | Pick::Unload)
    }

    /// Whether trains stop at it.
    pub fn docks(self) -> bool {
        matches!(self, Pick::Load | Pick::Unload)
    }
}

/// One tier's numbers.
pub struct ProcessTier {
    /// How this tier is driven: a smelter is a burner at Mk1 and Mk2 and electric from Mk3.
    pub energy: Energy,
    /// Work per tick at full power, in thousandths of a Mk1 tick.
    pub speed: u32,
    /// Fuel per unit of work, in thousandths (burners: 750 burns a quarter less a batch).
    pub fuel: u32,
    /// kW drawn while working (electric).
    pub power: u32,
}

pub struct ProcessSpec {
    pub block: BlockId,
    pub categories: &'static [Category],
    pub pick: Pick,
    /// Slots of the input, fuel and output buffers.
    pub buffers: [usize; 3],
    /// Slots of the byproduct buffer (0: byproducts go to the output buffer). It is emptied by belts at the
    /// footprint's `Role::Side` ports.
    pub side: usize,
    /// Mk1 first.
    pub tiers: &'static [ProcessTier],
    /// Its cells and ports (`SINGLE`: one cell, every side).
    pub footprint: Footprint,
    /// Status words: what it does ("Smelting"), what it makes ("ingots"), its line while it holds
    /// nothing to work on (`Pick::ByInput`).
    pub verb: &'static str,
    pub products: &'static str,
    pub waiting: &'static str,
    /// Its mark on the maps, 0xRRGGBB.
    pub map_colour: i32,
    /// Data grid compute at full power, in TF (fibre.rs): positive makes it, negative uses it, 0 takes no part.
    pub compute: i32,
    /// Boxes relative to the footprint's centre, front towards +z (turned with the machine).
    pub parts: &'static [Part],
}

pub const SPECS: &[ProcessSpec] = &[
    super::solar::SOLAR_SPEC,
    super::solar::ACCUMULATOR_SPEC,
    super::hangar::HANGAR_SPEC,
    super::docks::LOADING_DOCK_SPEC,
    super::docks::UNLOADING_DOCK_SPEC,
    super::pump::PUMPJACK_SPEC,
    super::refinery::REFINERY_SPEC,
    super::refinery::CRACKER_SPEC,
    super::refinery::PLANT_SPEC,
    super::refinery::ELECTROLYSER_SPEC,
    super::diesel::DIESEL_SPEC,
    super::washer::WASHER_SPEC,
    super::center::CENTER_SPEC,
    super::hydro::WHEEL_SPEC,
    super::hoist::WINCH_SPEC,
    super::nuclear::CENTRIFUGE_SPEC,
    super::nuclear::REACTOR_SPEC,
    super::recycler::RECYCLER_SPEC,
    super::fab::FAB_SPEC,
    super::datacenter::DATACENTER_SPEC,
    super::tower::TOWER_SPEC,
    super::ailab::AI_LAB_SPEC,
    super::optimizer::OPTIMIZER_SPEC,
    ProcessSpec {
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
    },
    ProcessSpec {
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
    },
    ProcessSpec {
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
    },
    ProcessSpec {
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
    },
    ProcessSpec {
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
    },
    ProcessSpec {
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
    },
    ProcessSpec {
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
    },
    ProcessSpec {
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
    },
    ProcessSpec {
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
    },
    ProcessSpec {
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
    },
];

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

/// The spec of the processor `block` is, if it is one.
pub fn spec(block: BlockId) -> Option<&'static ProcessSpec> {
    SPECS.iter().find(|s| s.block == block)
}

/// Whether the machine `block` makes recipes of `category`.
pub fn makes(block: BlockId, category: Category) -> bool {
    spec(block).is_some_and(|s| s.categories.contains(&category))
}

impl ProcessSpec {
    /// Recipe `i` if this machine makes it.
    pub fn recipe(&self, i: u16) -> Option<&'static MachineRecipe> {
        MACHINE_RECIPES.get(i as usize).filter(|r| self.categories.contains(&r.category))
    }

    /// The first recipe it makes that uses `item` and `unlocked` allows (by recipe index).
    pub fn recipe_using(&self, item: ItemId, unlocked: &[bool]) -> Option<u16> {
        self.recipe_using_if(item, unlocked, |_| true)
    }

    /// Like `recipe_using`, among only the recipes `ok` accepts.
    pub fn recipe_using_if(&self, item: ItemId, unlocked: &[bool], ok: impl Fn(&MachineRecipe) -> bool) -> Option<u16> {
        let fits = |(i, r): &(usize, &MachineRecipe)| {
            self.categories.contains(&r.category)
                && unlocked.get(*i) == Some(&true)
                && r.inputs.iter().any(|x| x.0 == item)
                && ok(r)
        };
        MACHINE_RECIPES.iter().enumerate().find(fits).map(|(i, _)| i as u16)
    }
}
