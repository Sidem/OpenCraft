//! Milestone 12's techs (Orbit), appended after `hands.rs`'s in `techs::TECHS`, so the saved index order is: Rocketry 78;
//! add rows at the bottom, never reorder. Needs name earlier indices: Electrolysis 47, Accelerators 57.

use crate::block::LAUNCH_SITE;
use crate::item::{BLUE_PACK, GOLD_PACK, GREEN_PACK, RED_PACK, VIOLET_PACK};
use crate::recipes::{ROCKET_FUEL_RECIPE, ROCKET_PART_RECIPES, ROCKET_RECIPE, SURVEY_SATELLITE_RECIPE};

use super::{r, Tech, Unlock};

pub const ORBIT: [Tech; 1] = [Tech {
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
}];
