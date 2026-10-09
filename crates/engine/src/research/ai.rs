//! The techs after the bonus techs (appended after `bonus.rs`'s in `techs::TECHS`, so the saved index order is:
//! Optimizer 66, Photonics 67, Hauler Mk3 68, Hauler Mk4 69, Hauler Mk5 70; add rows at the bottom, never reorder).

use crate::block::{LASER_EMITTER, LASER_RECEIVER, OPTIMIZER};
use crate::item::{
    BLUE_PACK, GOLD_PACK, GREEN_PACK, HAULER_PACK_MK3, HAULER_PACK_MK4, HAULER_PACK_MK5, RED_PACK, VIOLET_PACK,
};

use super::{r, Tech, Unlock};

pub const AI: [Tech; 5] = [
    Tech {
        name: "Optimizer",
        blurb: "The optimizer node (2×2×2, 150 kW) uses 20 TF of the data grid to make every machine within 16 \
                blocks work 25% faster. Machines take the best optimizer in range; they do not stack.",
        needs: &[62],
        packs: &[RED_PACK, GREEN_PACK, BLUE_PACK, VIOLET_PACK, GOLD_PACK],
        units: 250,
        seconds: 60.0,
        unlocks: &[r(OPTIMIZER)],
    },
    Tech {
        name: "Photonics",
        blurb: "Laser links: an emitter shoots a beam along its front, and a receiver with a clear line to it joins \
                the power grids of the two ends into one, across up to 128 blocks and at 90% efficiency. Glass lets \
                the beam through; any other block stops it.",
        needs: &[62],
        packs: &[RED_PACK, GREEN_PACK, BLUE_PACK, VIOLET_PACK, GOLD_PACK],
        units: 150,
        seconds: 60.0,
        unlocks: &[r(LASER_EMITTER), r(LASER_RECEIVER)],
    },
    Tech {
        name: "Hauler Mk3",
        blurb: "Four blue kits raise your Mk2 hauler pack (right in the inventory screen) to Mk3: 18 more \
                backpack slots, 63 in all.",
        needs: &[35],
        packs: &[RED_PACK, GREEN_PACK, BLUE_PACK, VIOLET_PACK],
        units: 120,
        seconds: 30.0,
        unlocks: &[Unlock::Recipe(HAULER_PACK_MK3)],
    },
    Tech {
        name: "Hauler Mk4",
        blurb: "Four violet kits raise a Mk3 pack to Mk4: 18 more backpack slots, 81 in all.",
        needs: &[68],
        packs: &[RED_PACK, GREEN_PACK, BLUE_PACK, VIOLET_PACK],
        units: 200,
        seconds: 40.0,
        unlocks: &[Unlock::Recipe(HAULER_PACK_MK4)],
    },
    Tech {
        name: "Hauler Mk5",
        blurb: "Four gold kits raise a Mk4 pack to Mk5: 18 more backpack slots, 99 in all.",
        needs: &[69, 53],
        packs: &[RED_PACK, GREEN_PACK, BLUE_PACK, VIOLET_PACK],
        units: 300,
        seconds: 50.0,
        unlocks: &[Unlock::Recipe(HAULER_PACK_MK5)],
    },
];
