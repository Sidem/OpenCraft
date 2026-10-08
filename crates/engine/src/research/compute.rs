//! The techs of Milestone 11 (compute and photonics), as data (appended after `recycling.rs`'s in `techs::TECHS`, so the
//! order here is the saved index order: add rows at the bottom, never reorder). Needs name earlier indices: Acids and
//! Lubricants 45, Gold Science 53; this table's own start at 56 (Wafers 56, Accelerators 57, Data Network 58, Datacenters 59).

use crate::block::{CHIP_FAB, DATACENTER, FIBRE_NODE};
use crate::item::{BLUE_PACK, GOLD_PACK, GREEN_PACK, RED_PACK, VIOLET_PACK};
use crate::recipes::{ACCELERATOR_RECIPE, PURE_WATER_RECIPE, WAFER_RECIPE};

use super::{r, Tech, Unlock};

pub const COMPUTE: [Tech; 4] = [
    Tech {
        name: "Wafers",
        blurb:
            "The chip fab (1 MW, 4×4×3) etches wafers from 2 silicon, an acid canister and a pure water canister in \
                8 seconds, giving both shells back. The chemical plant fills pure water from an empty canister, a sand \
                and 2 units of piped water. Gold packs: research it in a research center.",
        needs: &[45, 53],
        packs: &[RED_PACK, GREEN_PACK, BLUE_PACK, VIOLET_PACK, GOLD_PACK],
        units: 220,
        seconds: 45.0,
        unlocks: &[r(CHIP_FAB), Unlock::MachineRecipe(PURE_WATER_RECIPE), Unlock::MachineRecipe(WAFER_RECIPE)],
    },
    Tech {
        name: "Accelerators",
        blurb:
            "The chip fab builds an AI accelerator from 2 wafers, 2 processors and a plastic in 20 seconds: the rack \
                part every datacenter, satellite and laser link is made of.",
        needs: &[56],
        packs: &[RED_PACK, GREEN_PACK, BLUE_PACK, VIOLET_PACK, GOLD_PACK],
        units: 260,
        seconds: 50.0,
        unlocks: &[Unlock::MachineRecipe(ACCELERATOR_RECIPE)],
    },
    Tech {
        name: "Data Network",
        blurb: "Fibre nodes (2 from 2 processors, 4 glass, 2 plastic and 6 copper wire) join into data grids by \
                themselves within 12 blocks; machines that make or use compute hang on the nearest node within 5.",
        needs: &[56],
        packs: &[RED_PACK, GREEN_PACK, BLUE_PACK, VIOLET_PACK, GOLD_PACK],
        units: 150,
        seconds: 40.0,
        unlocks: &[r(FIBRE_NODE)],
    },
    Tech {
        name: "Datacenters",
        blurb:
            "The AI datacenter (4×4×3) turns 3 MW and cooling water into 100 TF for the data grid. It needs a fibre \
                node within 5 blocks and a pump pipe to its blue inlet, or it overheats and shuts down.",
        needs: &[57, 58, 46],
        packs: &[RED_PACK, GREEN_PACK, BLUE_PACK, VIOLET_PACK, GOLD_PACK],
        units: 300,
        seconds: 60.0,
        unlocks: &[r(DATACENTER)],
    },
];
