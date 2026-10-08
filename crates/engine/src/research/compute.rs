//! The techs of Milestone 11 (compute and photonics), as data (appended after `recycling.rs`'s in `techs::TECHS`, so the
//! order here is the saved index order: add rows at the bottom, never reorder). Needs name earlier indices: Acids and
//! Lubricants 45, Gold Science 53; this table's own start at 56 (Wafers 56, Accelerators 57, Data Network 58, Datacenters 59, Cooling 60, AI Labs 61, AI Research 62).

use crate::block::{AI_LAB, CHIP_FAB, COOLING_TOWER, DATACENTER, FIBRE_NODE};
use crate::item::{BLUE_PACK, GOLD_PACK, GREEN_PACK, RED_PACK, VIOLET_PACK};
use crate::recipes::{ACCELERATOR_RECIPE, PURE_WATER_RECIPE, WAFER_RECIPE};

use super::{is_bonus, r, Tech, Unlock};

/// Techs only an AI lab can research (they cost compute: `factory/process/ailab.rs`), by index; the bonus techs
/// (`bonus.rs`) are too.
const AI_ONLY: [u8; 1] = [62];

/// Whether `tech` can be researched only in an AI lab.
pub fn needs_ai_lab(tech: u8) -> bool {
    AI_ONLY.contains(&tech) || is_bonus(tech)
}

pub const COMPUTE: [Tech; 7] = [
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
    Tech {
        name: "Cooling",
        blurb: "The cooling tower (300 kW, 3×3×5) closes a datacenter's water loop: piped to the same network, it takes the \
                coolant through itself and loses only one unit in twenty. One tower cools two datacenters.",
        needs: &[59],
        packs: &[RED_PACK, GREEN_PACK, BLUE_PACK, VIOLET_PACK, GOLD_PACK],
        units: 200,
        seconds: 50.0,
        unlocks: &[r(COOLING_TOWER)],
    },
    Tech {
        name: "AI Labs",
        blurb: "The AI lab (2×2×2, 100 kW) researches at twice a lab's speed, takes every pack, makes every 2nd unit free of \
                packs, and uses 20 TF of the data grid it hangs on while it works. It is the only lab that can research \
                techs that cost compute.",
        needs: &[59],
        packs: &[RED_PACK, GREEN_PACK, BLUE_PACK, VIOLET_PACK, GOLD_PACK],
        units: 200,
        seconds: 60.0,
        unlocks: &[r(AI_LAB)],
    },
    Tech {
        name: "AI Research",
        blurb: "Trains the first research models. Costs compute: only an AI lab on a data grid with a datacenter can \
                research it. The endless bonus techs build on it.",
        needs: &[61],
        packs: &[RED_PACK, GREEN_PACK, BLUE_PACK, VIOLET_PACK, GOLD_PACK],
        units: 100,
        seconds: 60.0,
        unlocks: &[],
    },
];
