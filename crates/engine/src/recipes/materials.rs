//! The parts a player makes by hand from ingots (Materials): plates, rods, screws and wire. A constructor makes
//! them faster, but the first machines are built from these (`crafting.rs`). Listed in `RECIPES`.

use crate::item::*;

use super::{Group, Recipe};

pub const IRON_PLATE_RECIPE: Recipe = Recipe {
    output: IRON_PLATE,
    group: Group::Materials,
    count: 1,
    inputs: &[(IRON_INGOT, 2)],
    blurb: "Hammered flat by hand, slowly: a constructor does it for you. Machines, belts and packs use plates.",
};

pub const IRON_ROD_RECIPE: Recipe = Recipe {
    output: IRON_ROD,
    group: Group::Materials,
    count: 1,
    inputs: &[(IRON_INGOT, 1)],
    blurb: "Drawn out by hand, slowly: a constructor does it for you. Miners, lifts and screws use rods.",
};

pub const SCREW_RECIPE: Recipe = Recipe {
    output: SCREW,
    group: Group::Materials,
    count: 4,
    inputs: &[(IRON_ROD, 1)],
    blurb: "Cut from a rod by hand, slowly: a constructor does it for you.",
};

pub const COPPER_WIRE_RECIPE: Recipe = Recipe {
    output: COPPER_WIRE,
    group: Group::Materials,
    count: 2,
    inputs: &[(COPPER_INGOT, 1)],
    blurb: "Drawn from a copper ingot by hand, slowly: a constructor does it for you. Coils, poles and packs use \
            wire.",
};
