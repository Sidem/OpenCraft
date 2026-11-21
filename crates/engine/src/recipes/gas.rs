//! Natural gas hand recipes and machine recipes (CHEM-GAS): gas extraction (`GAS_WELL`, `WELLHEAD_SEPARATOR`),
//! sweetening & fractionation (`GAS_PLANT`), clean gas power (`GAS_TURBINE`), steam methane reforming (`STEAM_REFORMER`),
//! and steam-cracking ethane/LPG into ethylene for polyethylene in the chemical plant.
//!
//! Machine recipes append to `MACHINE_RECIPES` starting at `GAS_FIRST` (row 193); never reorder.

use crate::block::*;
use crate::item::*;

use super::{b, Category, Group, MachineRecipe, Recipe};

pub const GAS_WELL_RECIPE: Recipe = Recipe {
    output: b(GAS_WELL),
    group: Group::Production,
    count: 1,
    inputs: &[(STEEL_PLATE, 12), (STEEL_BEAM, 4), (MOTOR, 2), (CIRCUIT, 2)],
    blurb:
        "Stand it on the ground above a gas shale field (scan under plains, lowlands and deserts, 220+ blocks out): \
            it drills straight down and fills empty barrels with raw natural gas at the field's pace. Belts bring \
            empties in at the back and take raw gas from the front. Needs 90 kW.",
};

pub const WELLHEAD_SEPARATOR_RECIPE: Recipe = Recipe {
    output: b(WELLHEAD_SEPARATOR),
    group: Group::Production,
    count: 1,
    inputs: &[(STEEL_PLATE, 14), (STEEL_BEAM, 6), (MOTOR, 2), (CIRCUIT, 4), (LUBRICANT_BARREL, 2)],
    blurb: "Replaces a pumpjack over any oil sand deposit, 2×3×3 (R turns it): separates associated raw natural gas \
            alongside crude oil (1 gas barrel per 2 to 4 crude, depending on the field). Empties in at the back, crude \
            out the front, raw gas out the violet side hatches on the right. Needs 110 kW.",
};

pub const GAS_PLANT_RECIPE: Recipe = Recipe {
    output: b(GAS_PLANT),
    group: Group::Production,
    count: 1,
    inputs: &[(STEEL_PLATE, 16), (STEEL_BEAM, 8), (b(GLASS), 6), (MOTOR, 2), (CIRCUIT, 6)],
    blurb:
        "Sweetens and fractionates raw natural gas, 3×3×4 (R turns it): 4 raw gas barrels and a unit of water (blue \
            inlet on the left) make 2 methane barrels out the front and an ethane barrel, an LPG barrel and a sulfur \
            out the violet side hatches on the right in 6 seconds. Needs 140 kW.",
};

pub const GAS_TURBINE_RECIPE: Recipe = Recipe {
    output: b(GAS_TURBINE),
    group: Group::Power,
    count: 1,
    inputs: &[(STEEL_PLATE, 18), (STEEL_BEAM, 8), (MOTOR, 4), (CIRCUIT, 6), (LUBRICANT_BARREL, 2)],
    blurb: "Clean 600 kW turbine, 3×2×2 (R turns it): burns methane barrels (40 s at full load) or LPG barrels (60 s) \
            and returns the empty barrels from the front. Wire it to a pole like any power source; it burns only what \
            the grid asks for, ahead of diesel generators.",
};

pub const STEAM_REFORMER_RECIPE: Recipe = Recipe {
    output: b(STEAM_REFORMER),
    group: Group::Production,
    count: 1,
    inputs: &[(STEEL_PLATE, 12), (STEEL_BEAM, 6), (b(STONE_BRICKS), 10), (CIRCUIT, 4)],
    blurb: "Steam methane reformer, 2×2×3 (R turns it): 1 methane barrel, 2 empty barrels and 2 units of water (blue \
            inlet on the left) make 3 hydrogen barrels in 4 seconds — a fifth the power draw of an electrolyser. \
            Needs 100 kW.",
};

/// The five machine recipes of CHEM-GAS, appended after `STAINED_GLASS` (`GAS_FIRST` = 193..=197).
pub const GAS_MACHINES: &[MachineRecipe] = &[
    // 193: GAS_PROCESS_RECIPE — raw gas sweetening & NGL fractionation
    MachineRecipe {
        category: Category::GasProcessing,
        inputs: &[(RAW_GAS_BARREL, 4)],
        outputs: &[(METHANE_BARREL, 2), (ETHANE_BARREL, 1), (LPG_BARREL, 1), (SULFUR, 1)],
        seconds: 6.0,
    },
    // 194: REFORM_RECIPE — steam methane reforming for cheap industrial hydrogen
    MachineRecipe {
        category: Category::Reforming,
        inputs: &[(METHANE_BARREL, 1), (EMPTY_BARREL, 2)],
        outputs: &[(HYDROGEN_BARREL, 3)],
        seconds: 4.0,
    },
    // 195: ETHANE_CRACK_RECIPE — ethane steam cracking to ethylene + hydrogen
    MachineRecipe {
        category: Category::Cracking,
        inputs: &[(ETHANE_BARREL, 2)],
        outputs: &[(ETHYLENE_BARREL, 1), (HYDROGEN_BARREL, 1)],
        seconds: 4.0,
    },
    // 196: LPG_CRACK_RECIPE — LPG steam cracking to ethylene + methane
    MachineRecipe {
        category: Category::Cracking,
        inputs: &[(LPG_BARREL, 2)],
        outputs: &[(ETHYLENE_BARREL, 1), (METHANE_BARREL, 1)],
        seconds: 4.0,
    },
    // 197: POLYETHYLENE_RECIPE — ethylene polymerisation to plastic without naphtha
    MachineRecipe {
        category: Category::Chemistry,
        inputs: &[(ETHYLENE_BARREL, 2)],
        outputs: &[(PLASTIC, 6), (EMPTY_BARREL, 2)],
        seconds: 4.0,
    },
];
