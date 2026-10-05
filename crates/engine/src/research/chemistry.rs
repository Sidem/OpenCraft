//! The techs of Milestone 10 (fluids and chemistry), as data (appended after `distance.rs`'s in `techs::TECHS`, so the
//! order here is the saved index order: add rows at the bottom, never reorder). Needs name earlier indices (Fluid
//! Handling 6, Electronics 17); this table's own start at 42 (Oil Processing 42, Refining 43).

use crate::block::{CRACKER, PUMPJACK, REFINERY};
use crate::item::{BLUE_PACK, EMPTY_CANISTER, GREEN_PACK, RED_PACK, VIOLET_PACK};
use crate::recipes::{CANISTER_MACHINE_RECIPE, CRACK_RECIPE, DISTIL_RECIPE};

use super::{r, Tech, Unlock};

pub const CHEMISTRY: [Tech; 2] = [
    Tech {
        name: "Oil Processing",
        blurb: "Pumpjacks (90 kW) drill down to an oil reservoir and fill canisters with crude oil, a steel drum each \
                that belts carry like any item. Press empty canisters from steel plates, by hand or in a constructor. \
                Oil sand burns as a weak fuel; refining comes next.",
        needs: &[6, 17],
        packs: &[RED_PACK, GREEN_PACK, BLUE_PACK, VIOLET_PACK],
        units: 140,
        seconds: 40.0,
        unlocks: &[r(PUMPJACK), Unlock::Recipe(EMPTY_CANISTER), Unlock::MachineRecipe(CANISTER_MACHINE_RECIPE)],
    },
    Tech {
        name: "Refining",
        blurb:
            "The refinery (150 kW, a water pipe) distils 3 crude oil canisters into a naphtha, a diesel and a heavy \
                oil canister and a sulfur; the cracker (90 kW, a water pipe) splits 2 heavy oil canisters into a \
                naphtha and a diesel. Every stream must be used or stored, or the plant stops.",
        needs: &[42],
        packs: &[RED_PACK, GREEN_PACK, BLUE_PACK, VIOLET_PACK],
        units: 160,
        seconds: 40.0,
        unlocks: &[r(REFINERY), r(CRACKER), Unlock::MachineRecipe(DISTIL_RECIPE), Unlock::MachineRecipe(CRACK_RECIPE)],
    },
];
