//! What machines make ([`MACHINE_RECIPES`]), grouped into categories that processors take whole
//! (their spec rows: `factory/process/specs.rs`), and what burns as fuel ([`FUELS`]). A better machine
//! of a category reuses the same recipes (docs/TECH_TREE.md section 8).
//!
//! Invariants: machines save the index of their recipe, so rows are appended, never reordered. A
//! recipe's first output is its main product (readouts name it); the rest are byproducts, which a
//! machine with a byproduct port keeps apart, and all must fit its buffers (`recipes/tests.rs` checks
//! this and more).
//!
//! To add a recipe: append a row. A category: a `Category` variant (and its entry in the lint's list) and
//! the processor specs that take it.

use super::automate::AUTOMATE;
use super::decor::DECOR;
use super::machine_later::LATER;
use super::orbit::ORBIT;
use super::refit::REFIT;
use super::rows::*;
use super::scrap::SCRAP;
use crate::block::*;
use crate::item::{
    ItemId, COPPER_INGOT, COPPER_WIRE, GEAR, GREEN_KIT, GREEN_PACK, IRON_INGOT, IRON_PLATE, IRON_ROD, MOTOR, QUICKLIME,
    RED_PACK, SCREW, STEEL_BEAM, STEEL_INGOT, STEEL_PLATE,
};
use crate::item::{ACTUATOR, DRONE, DRONE_CELL, GUIDANCE_MODULE, PROCESSOR, SERVO};
use crate::item::{ALUMINIUM_INGOT, ALUMINIUM_PLATE, BATTERY, CARGO_DRONE, CRUSHED_BAUXITE};
use crate::item::{BLUE_KIT, BLUE_PACK, CIRCUIT, CRUSHED_COPPER, CRUSHED_IRON, SILICON, VIOLET_KIT, VIOLET_PACK};
use crate::join::join;

/// A kind of machine work.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Category {
    /// Ore, sand and stone into ingots, glass, bricks and quicklime, with fuel.
    Smelting,
    /// Ingots into plates, rods, screws and wire.
    Pressing,
    /// Several parts into one: motors, concrete, packs and kits (the assembler).
    Assembly,
    /// Ore, coal and flux into steel and slag (the blast furnace).
    Blasting,
    /// Ore into crushed ore, slag into sand (the crusher).
    Crushing,
    /// Quartz and coal into silicon (the arc furnace).
    Arc,
    /// Crushed bauxite and quicklime into aluminium and slag (the electrolytic cell).
    Electrolysis,
    /// Crude oil canisters and water into naphtha, diesel and heavy oil canisters and sulfur (the refinery).
    Distilling,
    /// Heavy oil and water into naphtha and diesel (the cracker).
    Cracking,
    /// Plastic, acid and lubricant (the chemical plant).
    Chemistry,
    /// Empty canisters and water into hydrogen and oxygen canisters (the electrolyser).
    Splitting,
    /// Crushed ore and water into washed ore and tailings (the washer).
    Washing,
    /// Uranium ore and a steel casing into a fuel cell (the centrifuge).
    Enrichment,
    /// Silicon, acid and pure water into wafers, wafers into AI accelerators (the chip fab).
    Fabrication,
    /// Rocket parts and fuel into a rocket in orbit (the launch site); its outputs are the empties that come back.
    Launch,
    /// A Mk1 item and the kits of each step into a higher tier's item (the upgrader; rows are `refit.rs`'s).
    Refit,
}

/// Something a machine makes: `inputs` are used up when a batch starts, `outputs` appear after
/// `seconds` of work.
#[derive(Clone, Copy)]
pub struct MachineRecipe {
    pub category: Category,
    pub inputs: &'static [(ItemId, u32)],
    /// The main product first, then any byproducts.
    pub outputs: &'static [(ItemId, u32)],
    pub seconds: f64,
}

use Category::{Arc, Assembly, Blasting, Crushing, Electrolysis, Pressing, Smelting};

/// Every machine recipe: the main table (here, then Milestones 10 and 11's in `machine_later.rs`), then Milestone 12's
/// (`orbit.rs`), the automated buildings' (`automate.rs`) and the upgrader's, one per tier (`refit.rs`). Saved machines
/// hold row indices, so rows only ever go on the end: a new table joins after the last one (a tier or family added to
/// `refit.rs` later goes in a table of its own too, as the upgrader's rows are derived from `FAMILIES`, and a test asks
/// that every tier has one). The ruins' salvage row (`scrap.rs`) follows; the decor set's rows (`decor.rs`) are last.
pub const MACHINE_RECIPES: &[MachineRecipe] = &join::<
    MachineRecipe,
    { MAIN.len() + LATER.len() + ORBIT.len() + AUTOMATE.len() + REFIT.len() + SCRAP.len() + DECOR.len() },
>(&[MAIN, LATER, ORBIT, AUTOMATE, REFIT, SCRAP, DECOR]);

const MAIN: &[MachineRecipe] = &[
    MachineRecipe { category: Smelting, inputs: &[(b(IRON_ORE), 1)], outputs: &[(IRON_INGOT, 1)], seconds: 1.5 },
    MachineRecipe { category: Smelting, inputs: &[(b(COPPER_ORE), 1)], outputs: &[(COPPER_INGOT, 1)], seconds: 1.5 },
    MachineRecipe { category: Pressing, inputs: &[(IRON_INGOT, 2)], outputs: &[(IRON_PLATE, 1)], seconds: 2.0 },
    MachineRecipe { category: Pressing, inputs: &[(IRON_INGOT, 1)], outputs: &[(IRON_ROD, 1)], seconds: 2.0 },
    MachineRecipe { category: Pressing, inputs: &[(IRON_ROD, 1)], outputs: &[(SCREW, 4)], seconds: 3.0 },
    MachineRecipe { category: Pressing, inputs: &[(COPPER_INGOT, 1)], outputs: &[(COPPER_WIRE, 2)], seconds: 2.0 },
    MachineRecipe { category: Smelting, inputs: &[(b(QUARTZ_ORE), 1)], outputs: &[(b(GLASS), 2)], seconds: 2.0 },
    MachineRecipe { category: Smelting, inputs: &[(b(SAND), 1)], outputs: &[(b(GLASS), 1)], seconds: 2.0 },
    MachineRecipe { category: Pressing, inputs: &[(IRON_PLATE, 1)], outputs: &[(GEAR, 1)], seconds: 2.0 },
    MachineRecipe { category: Smelting, inputs: &[(b(STONE), 2)], outputs: &[(b(STONE_BRICKS), 1)], seconds: 3.0 },
    MachineRecipe { category: Smelting, inputs: &[(b(LIMESTONE), 1)], outputs: &[(QUICKLIME, 1)], seconds: 2.0 },
    MachineRecipe {
        category: Assembly,
        inputs: &[(IRON_ROD, 1), (GEAR, 2), (COPPER_WIRE, 4)],
        outputs: &[(MOTOR, 1)],
        seconds: 5.0,
    },
    MachineRecipe {
        category: Assembly,
        inputs: &[(QUICKLIME, 1), (b(SAND), 2), (b(STONE), 2)],
        outputs: &[(b(CONCRETE), 4)],
        seconds: 4.0,
    },
    MachineRecipe {
        category: Assembly,
        inputs: &[(IRON_PLATE, 1), (COPPER_WIRE, 2)],
        outputs: &[(RED_PACK, 1)],
        seconds: 5.0,
    },
    MachineRecipe {
        category: Assembly,
        inputs: &[(b(BELT), 2), (SCREW, 4)],
        outputs: &[(GREEN_PACK, 1)],
        seconds: 6.0,
    },
    MachineRecipe {
        category: Assembly,
        inputs: &[(GEAR, 2), (SCREW, 4), (COPPER_WIRE, 2)],
        outputs: &[(GREEN_KIT, 4)],
        seconds: 5.0,
    },
    MachineRecipe {
        category: Blasting,
        inputs: &[(b(IRON_ORE), 2), (b(COAL_ORE), 1), (QUICKLIME, 1)],
        outputs: &[(STEEL_INGOT, 1), (b(SLAG), 1)],
        seconds: 4.0,
    },
    MachineRecipe { category: Pressing, inputs: &[(STEEL_INGOT, 1)], outputs: &[(STEEL_PLATE, 1)], seconds: 3.0 },
    MachineRecipe { category: Pressing, inputs: &[(STEEL_INGOT, 2)], outputs: &[(STEEL_BEAM, 1)], seconds: 4.0 },
    MachineRecipe {
        category: Assembly,
        inputs: &[(MOTOR, 1), (STEEL_PLATE, 1), (b(CONCRETE), 1)],
        outputs: &[(BLUE_PACK, 2)],
        seconds: 12.0,
    },
    MachineRecipe {
        category: Assembly,
        inputs: &[(MOTOR, 1), (STEEL_PLATE, 2), (SCREW, 4)],
        outputs: &[(BLUE_KIT, 4)],
        seconds: 5.0,
    },
    MachineRecipe { category: Crushing, inputs: &[(b(IRON_ORE), 2)], outputs: &[(CRUSHED_IRON, 3)], seconds: 2.0 },
    MachineRecipe { category: Crushing, inputs: &[(b(COPPER_ORE), 2)], outputs: &[(CRUSHED_COPPER, 3)], seconds: 2.0 },
    MachineRecipe { category: Crushing, inputs: &[(b(SLAG), 1)], outputs: &[(b(SAND), 1)], seconds: 1.0 },
    MachineRecipe { category: Smelting, inputs: &[(CRUSHED_IRON, 1)], outputs: &[(IRON_INGOT, 1)], seconds: 1.5 },
    MachineRecipe { category: Smelting, inputs: &[(CRUSHED_COPPER, 1)], outputs: &[(COPPER_INGOT, 1)], seconds: 1.5 },
    MachineRecipe {
        category: Arc,
        inputs: &[(b(QUARTZ_ORE), 1), (b(COAL_ORE), 1)],
        outputs: &[(SILICON, 1)],
        seconds: 4.0,
    },
    MachineRecipe {
        category: Assembly,
        inputs: &[(SILICON, 1), (COPPER_WIRE, 3), (IRON_PLATE, 1)],
        outputs: &[(CIRCUIT, 2)],
        seconds: 4.0,
    },
    MachineRecipe {
        category: Assembly,
        inputs: &[(CIRCUIT, 2), (STEEL_BEAM, 1), (MOTOR, 1)],
        outputs: &[(VIOLET_PACK, 2)],
        seconds: 15.0,
    },
    MachineRecipe {
        category: Assembly,
        inputs: &[(CIRCUIT, 2), (MOTOR, 1), (STEEL_PLATE, 2)],
        outputs: &[(VIOLET_KIT, 4)],
        seconds: 5.0,
    },
    MachineRecipe {
        category: Assembly,
        inputs: &[(IRON_PLATE, 1), (IRON_ROD, 1)],
        outputs: &[(b(BELT), 4)],
        seconds: 3.0,
    },
    MachineRecipe {
        category: Assembly,
        inputs: &[(CIRCUIT, 4), (SILICON, 1), (STEEL_PLATE, 1)],
        outputs: &[(PROCESSOR, 1)],
        seconds: 10.0,
    },
    MachineRecipe {
        category: Assembly,
        inputs: &[(MOTOR, 1), (CIRCUIT, 2), (GEAR, 2)],
        outputs: &[(SERVO, 1)],
        seconds: 8.0,
    },
    MachineRecipe {
        category: Assembly,
        inputs: &[(SERVO, 2), (STEEL_BEAM, 2), (SCREW, 4)],
        outputs: &[(ACTUATOR, 1)],
        seconds: 12.0,
    },
    MachineRecipe {
        category: Assembly,
        inputs: &[(MOTOR, 1), (CIRCUIT, 3), (STEEL_PLATE, 2)],
        outputs: &[(DRONE_CELL, 1)],
        seconds: 14.0,
    },
    MachineRecipe {
        category: Assembly,
        inputs: &[(PROCESSOR, 1), (CIRCUIT, 2), (COPPER_WIRE, 4)],
        outputs: &[(GUIDANCE_MODULE, 1)],
        seconds: 16.0,
    },
    MachineRecipe {
        category: Assembly,
        inputs: &[(ACTUATOR, 2), (DRONE_CELL, 1), (GUIDANCE_MODULE, 1)],
        outputs: &[(DRONE, 1)],
        seconds: 30.0,
    },
    MachineRecipe {
        category: Crushing,
        inputs: &[(b(BAUXITE_ORE), 2)],
        outputs: &[(CRUSHED_BAUXITE, 3)],
        seconds: 2.0,
    },
    MachineRecipe {
        category: Electrolysis,
        inputs: &[(CRUSHED_BAUXITE, 2), (QUICKLIME, 1)],
        outputs: &[(ALUMINIUM_INGOT, 1), (b(SLAG), 1)],
        seconds: 6.0,
    },
    MachineRecipe {
        category: Pressing,
        inputs: &[(ALUMINIUM_INGOT, 1)],
        outputs: &[(ALUMINIUM_PLATE, 1)],
        seconds: 3.0,
    },
    MachineRecipe {
        category: Assembly,
        inputs: &[(ALUMINIUM_PLATE, 1), (CIRCUIT, 1), (COPPER_WIRE, 4)],
        outputs: &[(BATTERY, 1)],
        seconds: 6.0,
    },
    MachineRecipe {
        category: Assembly,
        inputs: &[(DRONE, 1), (ALUMINIUM_PLATE, 4), (BATTERY, 2)],
        outputs: &[(CARGO_DRONE, 1)],
        seconds: 30.0,
    },
];

/// Units of water a batch of a recipe takes from the machine's water inlet, by recipe index (machines that take
/// water have a `Role::Water` port, `process/steam.rs`); recipes not listed use none.
const WATER_USE: &[(u16, u32)] = &[
    (DISTIL_RECIPE, 1),
    (CRACK_RECIPE, 1),
    (ACID_RECIPE, 1),
    (ELECTROLYSE_RECIPE, 2),
    (WASH_RECIPES[0], 1),
    (WASH_RECIPES[1], 1),
    (WASH_RECIPES[2], 1),
    (PURE_WATER_RECIPE, 2),
];

/// Water units one batch of machine recipe `i` uses.
pub fn water_use(i: u16) -> u32 {
    WATER_USE.iter().find(|w| w.0 == i).map_or(0, |w| w.1)
}

impl MachineRecipe {
    /// The main product and how many a batch makes.
    pub fn main(&self) -> (ItemId, u32) {
        self.outputs[0]
    }
}

/// Fuel: the seconds of smelting one item keeps a fire going, and the energy it gives a generator in
/// kJ (a coal runs a Mk1 miner long enough to mine about 32 coal).
pub const FUELS: &[(ItemId, f64, u32)] =
    &[(b(COAL_ORE), 8.0, 270), (b(LOG), 4.0, 135), (b(PLANKS), 1.0, 34), (b(OIL_SAND), 2.5, 90)];

/// Seconds of work one `item` fuels, if it burns.
pub fn burn_time(item: ItemId) -> Option<f64> {
    FUELS.iter().find(|f| f.0 == item).map(|f| f.1)
}

/// The energy one `item` gives a generator, in kJ, if it burns.
pub fn fuel_energy(item: ItemId) -> Option<u32> {
    FUELS.iter().find(|f| f.0 == item).map(|f| f.2)
}

const fn b(id: BlockId) -> ItemId {
    ItemId::block(id)
}
