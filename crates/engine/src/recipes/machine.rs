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

use super::rows::*;
use crate::block::*;
use crate::item::{
    ItemId, COPPER_INGOT, COPPER_WIRE, GEAR, GREEN_KIT, GREEN_PACK, IRON_INGOT, IRON_PLATE, IRON_ROD, MOTOR, QUICKLIME,
    RED_PACK, SCREW, STEEL_BEAM, STEEL_INGOT, STEEL_PLATE,
};
use crate::item::{ACID_CANISTER, HYDROGEN_CANISTER, LUBRICANT_CANISTER, OXYGEN_CANISTER, PLASTIC};
use crate::item::{ACTUATOR, DRONE, DRONE_CELL, GUIDANCE_MODULE, PROCESSOR, SERVO};
use crate::item::{ALUMINIUM_INGOT, ALUMINIUM_PLATE, BATTERY, CARGO_DRONE, CRUSHED_BAUXITE, EMPTY_CANISTER};
use crate::item::{BLUE_KIT, BLUE_PACK, CIRCUIT, CRUSHED_COPPER, CRUSHED_IRON, SILICON, VIOLET_KIT, VIOLET_PACK};
use crate::item::{CRUDE_CANISTER, DIESEL_CANISTER, HEAVY_OIL_CANISTER, NAPHTHA_CANISTER, SULFUR};
use crate::item::{FUEL_CELL, GOLD_KIT, GOLD_PACK, WASHED_BAUXITE, WASHED_COPPER, WASHED_IRON};

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
}

/// Something a machine makes: `inputs` are used up when a batch starts, `outputs` appear after
/// `seconds` of work.
pub struct MachineRecipe {
    pub category: Category,
    pub inputs: &'static [(ItemId, u32)],
    /// The main product first, then any byproducts.
    pub outputs: &'static [(ItemId, u32)],
    pub seconds: f64,
}

use Category::{
    Arc, Assembly, Blasting, Chemistry, Cracking, Crushing, Distilling, Electrolysis, Enrichment, Pressing, Smelting,
    Splitting, Washing,
};

pub const MACHINE_RECIPES: &[MachineRecipe] = &[
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
    MachineRecipe { category: Pressing, inputs: &[(STEEL_PLATE, 1)], outputs: &[(EMPTY_CANISTER, 2)], seconds: 1.5 },
    // Distil and Crack pour a canister's oil into another, so the shells go through: 3 in, 3 out, and 2 in, 2 out.
    MachineRecipe {
        category: Distilling,
        inputs: &[(CRUDE_CANISTER, 3)],
        outputs: &[(NAPHTHA_CANISTER, 1), (DIESEL_CANISTER, 1), (HEAVY_OIL_CANISTER, 1), (SULFUR, 1)],
        seconds: 6.0,
    },
    MachineRecipe {
        category: Cracking,
        inputs: &[(HEAVY_OIL_CANISTER, 2)],
        outputs: &[(NAPHTHA_CANISTER, 1), (DIESEL_CANISTER, 1)],
        seconds: 4.0,
    },
    // The chemical plant: plastic gives its two naphtha canisters' shells back (byproduct), acid and lubricant fill one.
    MachineRecipe {
        category: Chemistry,
        inputs: &[(NAPHTHA_CANISTER, 2), (b(COAL_ORE), 1)],
        outputs: &[(PLASTIC, 4), (EMPTY_CANISTER, 2)],
        seconds: 4.0,
    },
    MachineRecipe {
        category: Chemistry,
        inputs: &[(SULFUR, 1), (EMPTY_CANISTER, 1)],
        outputs: &[(ACID_CANISTER, 1)],
        seconds: 3.0,
    },
    MachineRecipe {
        category: Chemistry,
        inputs: &[(HEAVY_OIL_CANISTER, 1), (EMPTY_CANISTER, 1)],
        outputs: &[(LUBRICANT_CANISTER, 2)],
        seconds: 3.0,
    },
    // The electrolyser splits two units of water into three canisters of gas (three empties in, three out).
    MachineRecipe {
        category: Splitting,
        inputs: &[(EMPTY_CANISTER, 3)],
        outputs: &[(HYDROGEN_CANISTER, 2), (OXYGEN_CANISTER, 1)],
        seconds: 6.0,
    },
    // The washer takes only crushed ore: 3 crushed + 1 water → 4 washed + 1 tailings (the crusher stays the first step).
    MachineRecipe {
        category: Washing,
        inputs: &[(CRUSHED_IRON, 3)],
        outputs: &[(WASHED_IRON, 4), (b(TAILINGS), 1)],
        seconds: 3.0,
    },
    MachineRecipe {
        category: Washing,
        inputs: &[(CRUSHED_COPPER, 3)],
        outputs: &[(WASHED_COPPER, 4), (b(TAILINGS), 1)],
        seconds: 3.0,
    },
    MachineRecipe {
        category: Washing,
        inputs: &[(CRUSHED_BAUXITE, 3)],
        outputs: &[(WASHED_BAUXITE, 4), (b(TAILINGS), 1)],
        seconds: 3.0,
    },
    MachineRecipe { category: Smelting, inputs: &[(WASHED_IRON, 1)], outputs: &[(IRON_INGOT, 1)], seconds: 1.5 },
    MachineRecipe { category: Smelting, inputs: &[(WASHED_COPPER, 1)], outputs: &[(COPPER_INGOT, 1)], seconds: 1.5 },
    MachineRecipe {
        category: Electrolysis,
        inputs: &[(WASHED_BAUXITE, 2), (QUICKLIME, 1)],
        outputs: &[(ALUMINIUM_INGOT, 1), (b(SLAG), 1)],
        seconds: 6.0,
    },
    MachineRecipe { category: Crushing, inputs: &[(b(TAILINGS), 1)], outputs: &[(b(SAND), 1)], seconds: 1.0 },
    // The centrifuge: four uranium ore and a steel casing make a fuel cell (2 MW for 150 s in a reactor).
    MachineRecipe {
        category: Enrichment,
        inputs: &[(b(URANIUM_ORE), 4), (STEEL_PLATE, 1)],
        outputs: &[(FUEL_CELL, 1)],
        seconds: 10.0,
    },
    // Gold science (assembler only): the pack of the chemical era, and the kit that makes Mk5.
    MachineRecipe {
        category: Assembly,
        inputs: &[(PLASTIC, 1), (BATTERY, 1), (PROCESSOR, 1)],
        outputs: &[(GOLD_PACK, 2)],
        seconds: 20.0,
    },
    MachineRecipe {
        category: Assembly,
        inputs: &[(PROCESSOR, 1), (ALUMINIUM_PLATE, 2), (PLASTIC, 1)],
        outputs: &[(GOLD_KIT, 4)],
        seconds: 5.0,
    },
    // The blast furnace on crushed ore (Ore Crushing): one crushed ore for one, like the smelter, so an ore gives steel
    // at 1.5 times the raw rate.
    MachineRecipe {
        category: Blasting,
        inputs: &[(CRUSHED_IRON, 2), (b(COAL_ORE), 1), (QUICKLIME, 1)],
        outputs: &[(STEEL_INGOT, 1), (b(SLAG), 1)],
        seconds: 4.0,
    },
    // Washed iron, the same two for one (Ore Washing).
    MachineRecipe {
        category: Blasting,
        inputs: &[(WASHED_IRON, 2), (b(COAL_ORE), 1), (QUICKLIME, 1)],
        outputs: &[(STEEL_INGOT, 1), (b(SLAG), 1)],
        seconds: 4.0,
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
