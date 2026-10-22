//! Milestone 12's recipes (the Rocketry tech): the launch site's hand recipe (listed in `RECIPES`) and the machine rows
//! `MACHINE_RECIPES` appends after the main table (indices in `rows.rs`): four rocket parts from the assembler, rocket
//! fuel from the chemical plant, and the rocket itself, which the launch site (`factory/process/launch.rs`) assembles
//! from them and launches. Numbers: docs/TECH_ERAS.md section 8.

use crate::block::*;
use crate::item::*;

use super::{b, Category, Group, MachineRecipe, Recipe};

pub const LAUNCH_SITE_RECIPE: Recipe = Recipe {
    output: b(LAUNCH_SITE),
    group: Group::Space,
    count: 1,
    inputs: &[(b(CONCRETE), 200), (STEEL_BEAM, 60), (MOTOR, 20)],
    blurb: "A pad and gantry, 7×7 and 6 high (R turns it before you place it). Belts bring rocket parts, fuel and a \
            satellite in at the back hatches. With 4 light frames, 2 engines, 2 fuel tanks, an avionics, 8 rocket fuel \
            canisters and a survey satellite it builds a rocket in a minute and launches the satellite into orbit; the \
            empty canisters come out of the front. Needs 2 MW. A pad missing a part says which.",
};

pub const RECTENNA_RECIPE: Recipe = Recipe {
    output: b(RECTENNA),
    group: Group::Power,
    count: 1,
    inputs: &[(ALUMINIUM_PLATE, 20), (COPPER_WIRE, 60), (CIRCUIT, 20), (b(CONCRETE), 20)],
    blurb:
        "A flat field of receiving antennas, 5×5. Every power satellite whose circle covers it beams down 200 kW, up \
            to 1 MW, day and night with no fuel, for as long as the satellite is overhead (a satellite flies on at 6 \
            blocks a second, so build a constellation). Hang it on a power pole.",
};

pub const GROUND_STATION_RECIPE: Recipe = Recipe {
    output: b(GROUND_STATION),
    group: Group::Space,
    count: 1,
    inputs: &[(STEEL_BEAM, 20), (ALUMINIUM_PLATE, 20), (CIRCUIT, 30), (PROCESSOR, 8)],
    blurb: "A dish, 3×3 and 2 high. Every science satellite whose circle covers it downlinks 40 TF to its data grid, \
            up to 100 TF, for as long as the satellite is overhead (build a constellation). Needs 150 kW on a power \
            pole and a fibre node within reach, like a datacenter's.",
};

pub const POD_LAUNCHER_RECIPE: Recipe = Recipe {
    output: b(POD_LAUNCHER),
    group: Group::Space,
    count: 1,
    inputs: &[(b(CONCRETE), 60), (STEEL_BEAM, 20), (MOTOR, 10), (CIRCUIT, 20)],
    blurb: "A launch tube, 3×3 and 3 high. Right-click it to pick its channel, a colour. Belts bring cargo in, up to 12 \
            stacks, and drop pods. Under a comms satellite, with a landing pad on its channel also under cover, it loads \
            for 15 seconds and sends a pod with up to 6 stacks of cargo, which lands 20 seconds later. Needs 500 kW.",
};

pub const LANDING_PAD_RECIPE: Recipe = Recipe {
    output: b(LANDING_PAD),
    group: Group::Space,
    count: 1,
    inputs: &[(b(CONCRETE), 40), (STEEL_BEAM, 10), (CIRCUIT, 20)],
    blurb:
        "A target pad, 3×3. Right-click it to pick its channel, a colour. Drop pods sent from a pod launcher on the \
            same channel come down on the nearest pad that comms cover, and belts leading away from its sides take the \
            cargo out. Needs no power.",
};

/// The rows after the main table: light frame, engine, fuel tank, avionics, rocket fuel, the launch with a survey satellite,
/// the survey satellite itself, then the comms, power and science satellites with their launches.
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
    // The comms satellite (tech Orbital Comms) and the launch that carries it.
    MachineRecipe {
        category: Category::Assembly,
        inputs: &[(LIGHT_FRAME, 2), (PROCESSOR, 8), (ALUMINIUM_PLATE, 8)],
        outputs: &[(COMMS_SATELLITE, 1)],
        seconds: 40.0,
    },
    MachineRecipe {
        category: Category::Launch,
        inputs: &[
            (LIGHT_FRAME, 4),
            (ROCKET_ENGINE, 2),
            (FUEL_TANK, 2),
            (AVIONICS, 1),
            (ROCKET_FUEL_CANISTER, 8),
            (COMMS_SATELLITE, 1),
        ],
        outputs: &[(EMPTY_CANISTER, 8)],
        seconds: 60.0,
    },
    // The power satellite (tech Orbital Power) and the launch that carries it.
    MachineRecipe {
        category: Category::Assembly,
        inputs: &[(LIGHT_FRAME, 4), (b(SOLAR_PANEL), 24), (CIRCUIT, 16)],
        outputs: &[(POWER_SATELLITE, 1)],
        seconds: 60.0,
    },
    MachineRecipe {
        category: Category::Launch,
        inputs: &[
            (LIGHT_FRAME, 4),
            (ROCKET_ENGINE, 2),
            (FUEL_TANK, 2),
            (AVIONICS, 1),
            (ROCKET_FUEL_CANISTER, 8),
            (POWER_SATELLITE, 1),
        ],
        outputs: &[(EMPTY_CANISTER, 8)],
        seconds: 60.0,
    },
    // The science satellite (tech Orbital Science) and the launch that carries it.
    MachineRecipe {
        category: Category::Assembly,
        inputs: &[(LIGHT_FRAME, 2), (AI_ACCELERATOR, 8), (PROCESSOR, 8)],
        outputs: &[(SCIENCE_SATELLITE, 1)],
        seconds: 50.0,
    },
    MachineRecipe {
        category: Category::Launch,
        inputs: &[
            (LIGHT_FRAME, 4),
            (ROCKET_ENGINE, 2),
            (FUEL_TANK, 2),
            (AVIONICS, 1),
            (ROCKET_FUEL_CANISTER, 8),
            (SCIENCE_SATELLITE, 1),
        ],
        outputs: &[(EMPTY_CANISTER, 8)],
        seconds: 60.0,
    },
    // The drop pod (tech Drop Pods): a pod launcher spends one for each load it sends.
    MachineRecipe {
        category: Category::Assembly,
        inputs: &[(LIGHT_FRAME, 1), (FUEL_TANK, 1), (CIRCUIT, 10)],
        outputs: &[(DROP_POD, 1)],
        seconds: 20.0,
    },
];
