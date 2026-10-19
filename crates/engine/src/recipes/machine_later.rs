//! The machine rows of Chemistry and Compute (Milestones 10 and 11), from canisters to AI accelerators: the second
//! table `MACHINE_RECIPES` (`machine.rs`) joins, straight after the main one, so their indices are unchanged.
//! Saved machines hold row indices: never reorder; new rows go in the last table `MACHINE_RECIPES` joins.

use crate::block::*;
use crate::item::*;

use super::b;
use super::Category::{Assembly, Blasting, Chemistry, Cracking, Distilling, Electrolysis, Enrichment, Fabrication};
use super::Category::{Crushing, Pressing, Smelting, Splitting, Washing};
use super::MachineRecipe;

pub(super) const LATER: &[MachineRecipe] = &[
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
    // Compute (Wafers): ultra-pure water is filled in the chemical plant (two units of piped water and a sand to
    // filter them), the chip fab etches a wafer from silicon, acid and pure water and gives both shells back, then
    // builds an AI accelerator from wafers, processors and plastic.
    MachineRecipe {
        category: Chemistry,
        inputs: &[(EMPTY_CANISTER, 1), (b(SAND), 1)],
        outputs: &[(PURE_WATER_CANISTER, 1)],
        seconds: 3.0,
    },
    MachineRecipe {
        category: Fabrication,
        inputs: &[(SILICON, 2), (ACID_CANISTER, 1), (PURE_WATER_CANISTER, 1)],
        outputs: &[(WAFER, 1), (EMPTY_CANISTER, 2)],
        seconds: 8.0,
    },
    MachineRecipe {
        category: Fabrication,
        inputs: &[(WAFER, 2), (PROCESSOR, 2), (PLASTIC, 1)],
        outputs: &[(AI_ACCELERATOR, 1)],
        seconds: 20.0,
    },
];
