//! Milestone 12's recipes (the Rocketry tech): the launch site's hand recipe (listed in `RECIPES`) and the machine rows
//! `MACHINE_RECIPES` appends after the main table (indices in `rows.rs`): four rocket parts from the assembler, rocket
//! fuel from the chemical plant, and the rocket itself, which the launch site (`factory/process/launch.rs`) assembles
//! from them and launches. Numbers: docs/TECH_ERAS.md section 8.

use crate::block::*;
use crate::item::*;

use super::{b, Category, Group, MachineRecipe, Recipe};

pub const LAUNCH_SITE_RECIPE: Recipe = Recipe {
    output: b(LAUNCH_SITE),
    group: Group::Production,
    count: 1,
    inputs: &[(b(CONCRETE), 200), (STEEL_BEAM, 60), (MOTOR, 20)],
    blurb: "A pad and gantry, 7×7 and 6 high (R turns it before you place it). Belts bring rocket parts, fuel and a \
            satellite in at the back hatches. With 4 light frames, 2 engines, 2 fuel tanks, an avionics, 8 rocket fuel \
            canisters and a survey satellite it builds a rocket in a minute and launches the satellite into orbit; the \
            empty canisters come out of the front. Needs 2 MW. A pad missing a part says which.",
};

/// The rows after the main table: light frame, engine, fuel tank, avionics, rocket fuel, the launch with a survey satellite,
/// and the survey satellite itself.
pub const ORBIT: &[MachineRecipe] = &[
    MachineRecipe {
        category: Category::Assembly,
        inputs: &[(ALUMINIUM_PLATE, 2), (STEEL_BEAM, 1), (PLASTIC, 1)],
        outputs: &[(LIGHT_FRAME, 1)],
        seconds: 6.0,
    },
    MachineRecipe {
        category: Category::Assembly,
        inputs: &[(STEEL_PLATE, 8), (ALUMINIUM_PLATE, 4), (MOTOR, 2)],
        outputs: &[(ROCKET_ENGINE, 1)],
        seconds: 20.0,
    },
    MachineRecipe {
        category: Category::Assembly,
        inputs: &[(ALUMINIUM_PLATE, 6), (STEEL_PLATE, 4), (PLASTIC, 2)],
        outputs: &[(FUEL_TANK, 1)],
        seconds: 12.0,
    },
    MachineRecipe {
        category: Category::Assembly,
        inputs: &[(AI_ACCELERATOR, 4), (PROCESSOR, 4), (CIRCUIT, 8)],
        outputs: &[(AVIONICS, 1)],
        seconds: 30.0,
    },
    // Three shells in, three out: the fuel canister and two empties.
    MachineRecipe {
        category: Category::Chemistry,
        inputs: &[(HYDROGEN_CANISTER, 2), (OXYGEN_CANISTER, 1)],
        outputs: &[(ROCKET_FUEL_CANISTER, 1), (EMPTY_CANISTER, 2)],
        seconds: 6.0,
    },
    // The launch: everything is used up, the fuel's eight empty canisters come back, and the satellite is in orbit
    // (`satellites.rs`). One row a kind of satellite; the payload is always the last input.
    MachineRecipe {
        category: Category::Launch,
        inputs: &[
            (LIGHT_FRAME, 4),
            (ROCKET_ENGINE, 2),
            (FUEL_TANK, 2),
            (AVIONICS, 1),
            (ROCKET_FUEL_CANISTER, 8),
            (SURVEY_SATELLITE, 1),
        ],
        outputs: &[(EMPTY_CANISTER, 8)],
        seconds: 60.0,
    },
    MachineRecipe {
        category: Category::Assembly,
        inputs: &[(LIGHT_FRAME, 2), (b(SOLAR_PANEL), 4), (AI_ACCELERATOR, 4)],
        outputs: &[(SURVEY_SATELLITE, 1)],
        seconds: 40.0,
    },
];
