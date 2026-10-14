//! Milestone 12's techs (Orbit), appended after `hands.rs`'s in `techs::TECHS`, so the saved index order is: Rocketry 78,
//! Orbital Comms 79, Orbital Power 80, Orbital Science 81;
//! closed: new techs go in `later.rs`, never reorder. Needs name earlier indices: Nuclear Power 52, Electrolysis 47, Accelerators 57,
//! Data Network 58.

use crate::block::{GROUND_STATION, LAUNCH_SITE, RECTENNA};
use crate::item::{BLUE_PACK, GOLD_PACK, GREEN_PACK, RED_PACK, VIOLET_PACK};
use crate::recipes::{
    COMMS_SATELLITE_RECIPES, POWER_SATELLITE_RECIPES, ROCKET_FUEL_RECIPE, ROCKET_PART_RECIPES, ROCKET_RECIPE,
    SCIENCE_SATELLITE_RECIPES, SURVEY_SATELLITE_RECIPE,
};

use super::{r, Tech, Unlock};

pub const ORBIT: [Tech; 4] = [
    Tech {
    name: "Rocketry",
    blurb: "Light frames, rocket engines, fuel tanks and avionics come from assemblers; the chemical plant mixes \
            hydrogen and oxygen canisters into rocket fuel. The launch site (7×7×6, 2 MW) builds a rocket from 4 light \
            frames, 2 engines, 2 fuel tanks, an avionics and 8 fuel canisters, and launches it with a survey satellite \
            (an assembler builds those) into orbit.",
    needs: &[47, 57],
    packs: &[RED_PACK, GREEN_PACK, BLUE_PACK, VIOLET_PACK, GOLD_PACK],
    units: 300,
    seconds: 60.0,
    unlocks: &[
        r(LAUNCH_SITE),
        Unlock::MachineRecipe(ROCKET_PART_RECIPES[0]),
        Unlock::MachineRecipe(ROCKET_PART_RECIPES[1]),
        Unlock::MachineRecipe(ROCKET_PART_RECIPES[2]),
        Unlock::MachineRecipe(ROCKET_PART_RECIPES[3]),
        Unlock::MachineRecipe(ROCKET_FUEL_RECIPE),
        Unlock::MachineRecipe(ROCKET_RECIPE),
        Unlock::MachineRecipe(SURVEY_SATELLITE_RECIPE),
    ],
},
    Tech {
        name: "Orbital Comms",
        blurb: "Comms satellites (an assembler builds them; the launch site carries them up) cover 1024 blocks around the \
                ground below them. Under one, fibre nodes join into a single data grid however far apart they are, and \
                drone ports reach four times as far and fly cargo twice as far. It lasts only while a satellite is overhead: \
                a dozen make the cover nearly unbroken.",
        needs: &[78, 58],
        packs: &[RED_PACK, GREEN_PACK, BLUE_PACK, VIOLET_PACK, GOLD_PACK],
        units: 250,
        seconds: 60.0,
        unlocks: &[Unlock::MachineRecipe(COMMS_SATELLITE_RECIPES[0]), Unlock::MachineRecipe(COMMS_SATELLITE_RECIPES[1])],
    },
    Tech {
        name: "Orbital Power",
        blurb: "Power satellites (an assembler builds them) beam 200 kW each to the rectenna, a 5×5 field, under their \
                1024-block circle: up to 1 MW a field, day and night, with no fuel. It lasts only while a satellite is \
                overhead, so a constellation and a few accumulators keep a grid steady.",
        needs: &[78, 52],
        packs: &[RED_PACK, GREEN_PACK, BLUE_PACK, VIOLET_PACK, GOLD_PACK],
        units: 300,
        seconds: 60.0,
        unlocks: &[
            r(RECTENNA),
            Unlock::MachineRecipe(POWER_SATELLITE_RECIPES[0]),
            Unlock::MachineRecipe(POWER_SATELLITE_RECIPES[1]),
        ],
    },
    Tech {
        name: "Orbital Science",
        blurb: "Science satellites (an assembler builds them) gather data from orbit. The ground station, a 3×3 dish on \
                a power pole and a fibre node, downlinks 40 TF of compute for each one whose 1024-block circle covers \
                it, up to 100 TF, with no datacenter. It lasts only while a satellite is overhead, so build a \
                constellation.",
        needs: &[78, 58],
        packs: &[RED_PACK, GREEN_PACK, BLUE_PACK, VIOLET_PACK, GOLD_PACK],
        units: 300,
        seconds: 60.0,
        unlocks: &[
            r(GROUND_STATION),
            Unlock::MachineRecipe(SCIENCE_SATELLITE_RECIPES[0]),
            Unlock::MachineRecipe(SCIENCE_SATELLITE_RECIPES[1]),
        ],
    },
];
