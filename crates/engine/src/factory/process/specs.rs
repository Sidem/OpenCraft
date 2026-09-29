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
    tex, BlockId, ARC_FURNACE, ASSEMBLER, BLAST_FURNACE, BOILER, CONSTRUCTOR, CRUSHER, SILO, SMELTER, TURBINE,
};
use crate::item::ItemId;
use crate::recipes::{Category, MachineRecipe, MACHINE_RECIPES};

use super::super::footprint::{Footprint, Port, Role, Side, SINGLE};
use super::model::{part, Look, Part};

/// How a processor is driven.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Energy {
    /// Burns fuel (`recipes::FUELS`) from its fuel buffer while a batch runs.
    Burner,
    /// Draws `ProcessTier::power` from a power grid (`power.rs`) while it works.
    Electric,
    /// Needs nothing beyond its recipe (a blast furnace's coal is an input): always at full speed.
    Recipe,
    /// A boiler: burns fuel from its fuel buffer into steam for the turbines against it (steam.rs).
    Boiler,
    /// A steam turbine: a power source fed by the boilers it touches (steam.rs).
    Turbine,
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
    /// Boxes relative to the footprint's centre, front towards +z (turned with the machine).
    pub parts: &'static [Part],
}

pub const SPECS: &[ProcessSpec] = &[
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
        ],
        footprint: SINGLE,
        verb: "Smelting",
        products: "ingots",
        waiting: "Waiting for ore, sand or stone",
        map_colour: 0xe5533d,
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
        ],
        footprint: SINGLE,
        verb: "Making",
        products: "parts",
        waiting: "",
        map_colour: 0x4a90e2,
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
        ],
        footprint: Footprint {
            size: [2, 2, 2],
            ports: &[inlet(Side::Back), inlet(Side::Left), inlet(Side::Right), OUT_FRONT],
        },
        verb: "Assembling",
        products: "parts",
        waiting: "",
        map_colour: 0x34a898,
        parts: &ASSEMBLER_PARTS,
    },
    ProcessSpec {
        block: BLAST_FURNACE,
        categories: &[Category::Blasting],
        pick: Pick::Chosen,
        buffers: [3, 0, 1],
        side: 1,
        tiers: &[
            ProcessTier { energy: Energy::Recipe, speed: 1000, fuel: 0, power: 0 },
            ProcessTier { energy: Energy::Recipe, speed: 2000, fuel: 0, power: 0 },
            ProcessTier { energy: Energy::Recipe, speed: 3000, fuel: 0, power: 0 },
            ProcessTier { energy: Energy::Recipe, speed: 5000, fuel: 0, power: 0 },
        ],
        footprint: Footprint {
            size: [2, 2, 3],
            ports: &[inlet(Side::Back), inlet(Side::Left), OUT_FRONT, Port { side: Side::Right, role: Role::Side }],
        },
        verb: "Blasting",
        products: "steel",
        waiting: "",
        map_colour: 0xb84a30,
        parts: &BLAST_PARTS,
    },
    ProcessSpec {
        block: BOILER,
        categories: &[],
        pick: Pick::ByInput,
        buffers: [0, 2, 0],
        side: 0,
        tiers: &[ProcessTier { energy: Energy::Boiler, speed: 1000, fuel: 0, power: 0 }],
        footprint: Footprint { size: [2, 2, 2], ports: &[inlet(Side::Back), inlet(Side::Left), inlet(Side::Right)] },
        verb: "Boiling",
        products: "steam",
        waiting: "Idle",
        map_colour: 0x3a5f9a,
        parts: &BOILER_PARTS,
    },
    ProcessSpec {
        block: TURBINE,
        categories: &[],
        pick: Pick::ByInput,
        buffers: [0, 0, 0],
        side: 0,
        tiers: &[ProcessTier { energy: Energy::Turbine, speed: 1000, fuel: 0, power: 0 }],
        footprint: Footprint { size: [3, 2, 2], ports: &[] },
        verb: "Turning",
        products: "power",
        waiting: "Idle",
        map_colour: 0xc8ced8,
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
        waiting: "Waiting for iron or copper ore, or slag",
        map_colour: 0xd6a930,
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
        parts: &ARC_PARTS,
    },
];

const fn inlet(side: Side) -> Port {
    Port { side, role: Role::In }
}

const OUT_FRONT: Port = Port { side: Side::Front, role: Role::Out };

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
        let fits = |(i, r): &(usize, &MachineRecipe)| {
            self.categories.contains(&r.category)
                && unlocked.get(*i) == Some(&true)
                && r.inputs.iter().any(|x| x.0 == item)
        };
        MACHINE_RECIPES.iter().enumerate().find(fits).map(|(i, _)| i as u16)
    }
}

const BRICK: [u16; 3] = [tex::SMELTER_TOP, tex::SMELTER_SIDE, tex::FRAME];
const IVORY: [u16; 3] = [tex::CONSTRUCTOR_TOP, tex::CONSTRUCTOR_SIDE, tex::FRAME];
const FRAME: Look = Look::Tex([tex::FRAME; 3]);
const SOOT: Look = Look::Tex([tex::GENERATOR_SIDE; 3]);

/// A brick furnace: a dark upper ore mouth over a glowing crucible, a tall chimney, a status lamp.
const SMELTER_PARTS: [Part; 9] = [
    part([0.0, -0.42, 0.0], [0.94, 0.16, 0.94], Look::Band(tex::FRAME)),
    part([0.0, -0.06, -0.1], [0.82, 0.67, 0.72], Look::Tex(BRICK)),
    part([0.0, 0.31, -0.1], [0.9, 0.1, 0.8], FRAME),
    part([0.0, 0.14, 0.32], [0.46, 0.23, 0.13], SOOT),
    part([0.0, -0.24, 0.34], [0.55, 0.2, 0.15], Look::Fire(tex::GENERATOR_SIDE)),
    part([0.0, -0.36, 0.46], [0.65, 0.07, 0.08], FRAME),
    part([-0.23, 0.48, -0.23], [0.27, 0.48, 0.27], Look::Tex(BRICK)),
    part([-0.23, 0.75, -0.23], [0.33, 0.07, 0.33], SOOT),
    part([0.33, 0.34, 0.32], [0.12, 0.08, 0.12], Look::Lamp),
];

/// An open press: tray, rear gantry, a ram and head that pump while working, a status lamp.
const CONSTRUCTOR_PARTS: [Part; 9] = [
    part([0.0, -0.4, 0.0], [0.94, 0.18, 0.94], Look::Band(tex::CONSTRUCTOR_TOP)),
    part([0.0, -0.25, 0.14], [0.67, 0.08, 0.57], FRAME),
    part([-0.35, 0.0, -0.32], [0.13, 0.67, 0.16], Look::Tex(IVORY)),
    part([0.35, 0.0, -0.32], [0.13, 0.67, 0.16], Look::Tex(IVORY)),
    part([0.0, 0.35, -0.32], [0.85, 0.13, 0.22], Look::Tex(IVORY)),
    part([0.0, 0.23, -0.1], [0.18, 0.34, 0.18], Look::Press(0.5, [tex::DRILL; 3])),
    part([0.0, 0.04, -0.1], [0.48, 0.08, 0.48], Look::Press(1.0, [tex::IRON_PLATE; 3])),
    part([0.0, -0.36, 0.49], [0.64, 0.09, 0.06], Look::Tex([tex::COPPER_INGOT; 3])),
    part([0.39, 0.37, -0.31], [0.12, 0.08, 0.12], Look::Lamp),
];

const TEAL: [u16; 3] = [tex::ASSEMBLER_TOP, tex::ASSEMBLER_SIDE, tex::FRAME];
const STEEL: Look = Look::Tex([tex::STEEL; 3]);

/// A teal housing on a banded plinth, a gantry on its roof whose head works while assembling, a motor
/// casing high on its flank (the port hatches below come from the footprint) and a status lamp.
const ASSEMBLER_PARTS: [Part; 9] = [
    part([0.0, -0.9, 0.0], [1.96, 0.2, 1.96], Look::Band(tex::FRAME)),
    part([0.0, -0.05, 0.0], [1.8, 1.5, 1.8], Look::Tex(TEAL)),
    part([0.0, 0.74, 0.0], [1.9, 0.1, 1.9], FRAME),
    part([-0.6, 1.0, -0.05], [0.12, 0.42, 0.12], STEEL),
    part([0.6, 1.0, -0.05], [0.12, 0.42, 0.12], STEEL),
    part([0.0, 1.2, -0.05], [1.32, 0.1, 0.14], STEEL),
    part([0.0, 0.98, -0.05], [0.28, 0.24, 0.28], Look::Press(1.0, [tex::IRON_PLATE; 3])),
    part([0.9, 0.3, -0.3], [0.2, 0.5, 0.7], Look::Tex([tex::FRAME, tex::MOTOR, tex::FRAME])),
    part([0.78, 0.86, 0.78], [0.14, 0.12, 0.14], Look::Lamp),
];

const FIREBRICK: [u16; 3] = [tex::BLAST_TOP, tex::BLAST_SIDE, tex::FRAME];

/// A firebrick stack on a banded plinth: a wide hearth under a narrower shaft and chimney, a tap hole
/// on the front that glows while it works, a hot-blast pipe up one flank and a status lamp.
const BLAST_PARTS: [Part; 9] = [
    part([0.0, -1.4, 0.0], [1.96, 0.2, 1.96], Look::Band(tex::FRAME)),
    part([0.0, -0.55, 0.0], [1.8, 1.5, 1.8], Look::Tex(FIREBRICK)),
    part([0.0, 0.26, 0.0], [1.92, 0.12, 1.92], FRAME),
    part([0.0, 0.8, 0.0], [1.3, 1.0, 1.3], Look::Tex(FIREBRICK)),
    part([0.0, 1.38, 0.0], [1.42, 0.12, 1.42], FRAME),
    part([0.0, 1.2, 0.0], [0.6, 0.5, 0.6], SOOT),
    part([0.0, -0.4, 0.92], [0.7, 0.5, 0.12], Look::Fire(tex::GENERATOR_SIDE)),
    part([-0.98, 0.0, -0.5], [0.16, 2.0, 0.16], STEEL),
    part([0.78, 0.36, 0.78], [0.14, 0.12, 0.14], Look::Lamp),
];

const BLUE_TANK: [u16; 3] = [tex::BOILER_TOP, tex::BOILER_SIDE, tex::FRAME];

/// A riveted tank on a banded plinth with a glowing firebox door on the front, a roof plate, a steam
/// stack and a status lamp.
const BOILER_PARTS: [Part; 6] = [
    part([0.0, -0.9, 0.0], [1.96, 0.2, 1.96], Look::Band(tex::FRAME)),
    part([0.0, 0.0, 0.0], [1.8, 1.6, 1.8], Look::Tex(BLUE_TANK)),
    part([0.0, 0.86, 0.0], [1.9, 0.1, 1.9], FRAME),
    part([0.0, -0.45, 0.92], [0.8, 0.55, 0.12], Look::Fire(tex::GENERATOR_SIDE)),
    part([-0.5, 1.15, -0.5], [0.3, 0.5, 0.3], STEEL),
    part([0.78, 0.86, 0.78], [0.14, 0.12, 0.14], Look::Lamp),
];

const CASING: [u16; 3] = [tex::TURBINE_TOP, tex::TURBINE_SIDE, tex::FRAME];

/// A steel casing on a banded plinth, a rotor drum on top, a generator block at one end and a status lamp.
const TURBINE_PARTS: [Part; 5] = [
    part([0.0, -0.9, 0.0], [2.96, 0.2, 1.96], Look::Band(tex::FRAME)),
    part([0.0, -0.1, 0.0], [2.8, 1.4, 1.8], Look::Tex(CASING)),
    part([0.4, 0.75, 0.0], [1.5, 0.5, 1.5], Look::Tex(CASING)),
    part([-1.15, 0.75, 0.0], [0.7, 0.6, 0.9], Look::Tex([tex::FRAME, tex::MOTOR, tex::FRAME])),
    part([1.3, 0.86, 0.78], [0.14, 0.12, 0.14], Look::Lamp),
];

const HAZARD: [u16; 3] = [tex::CRUSHER_TOP, tex::CRUSHER_SIDE, tex::FRAME];

/// A hazard-striped body on a base, a wide hopper on top whose steel jaw works while crushing, a lamp.
const CRUSHER_PARTS: [Part; 5] = [
    part([0.0, -0.4, 0.0], [0.94, 0.18, 0.94], Look::Band(tex::FRAME)),
    part([0.0, -0.05, 0.0], [0.86, 0.6, 0.86], Look::Tex(HAZARD)),
    part([0.0, 0.36, 0.0], [0.96, 0.26, 0.96], Look::Tex(HAZARD)),
    part([0.0, 0.22, 0.0], [0.5, 0.16, 0.5], Look::Press(1.0, [tex::STEEL; 3])),
    part([0.36, 0.32, 0.36], [0.12, 0.08, 0.12], Look::Lamp),
];

const GRAPHITE: [u16; 3] = [tex::ARC_TOP, tex::ARC_SIDE, tex::FRAME];

/// A graphite housing on a banded plinth with a glowing arc slit on the front, two electrode rods that
/// pump on the roof while working, a roof plate and a status lamp.
const ARC_PARTS: [Part; 7] = [
    part([0.0, -0.9, 0.0], [1.96, 0.2, 1.96], Look::Band(tex::FRAME)),
    part([0.0, -0.05, 0.0], [1.8, 1.5, 1.8], Look::Tex(GRAPHITE)),
    part([0.0, 0.74, 0.0], [1.9, 0.1, 1.9], FRAME),
    part([0.0, -0.1, 0.92], [1.1, 0.3, 0.12], Look::Fire(tex::ARC_SIDE)),
    part([-0.45, 1.0, 0.0], [0.24, 0.4, 0.24], Look::Press(0.6, [tex::STEEL; 3])),
    part([0.45, 1.0, 0.0], [0.24, 0.4, 0.24], Look::Press(0.6, [tex::STEEL; 3])),
    part([0.78, 0.86, 0.78], [0.14, 0.12, 0.14], Look::Lamp),
];

const CONCRETE_RINGS: [u16; 3] = [tex::SILO_TOP, tex::SILO_SIDE, tex::FRAME];

/// A tall concrete drum on a banded plinth with two steel rings, a roof and hatch, a status lamp.
const SILO_PARTS: [Part; 6] = [
    part([0.0, -1.4, 0.0], [1.96, 0.2, 1.96], Look::Band(tex::FRAME)),
    part([0.0, -0.05, 0.0], [1.8, 2.7, 1.8], Look::Tex(CONCRETE_RINGS)),
    part([0.0, 0.5, 0.0], [1.9, 0.1, 1.9], FRAME),
    part([0.0, -0.6, 0.0], [1.9, 0.1, 1.9], FRAME),
    part([0.0, 1.45, 0.0], [1.9, 0.12, 1.9], Look::Tex([tex::SILO_TOP, tex::FRAME, tex::FRAME])),
    part([0.78, 1.36, 0.78], [0.14, 0.12, 0.14], Look::Lamp),
];
