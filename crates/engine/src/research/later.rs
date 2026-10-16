//! The open end of `techs::TECHS`: techs added after Milestone 12's first steps, so that `orbit.rs` stays closed and no saved
//! index moves. Saved index order: Long-Distance Power 82, Drop Pods 83, Settlements 84. Add rows at the bottom, never reorder (new
//! Orbit techs go here too). Needs name earlier indices: Blue Science 10, Mk4 Logistics 19, Bauxite Processing 36,
//! Rocketry 78, Orbital Comms 79.

use crate::block::{LANDING_PAD, LONG_POLE, POD_LAUNCHER, TOWN_DEPOT};
use crate::item::{BLUE_PACK, GOLD_PACK, GREEN_PACK, RED_PACK, VIOLET_PACK};
use crate::recipes::{DROP_POD_RECIPE, LONG_POLE_ROW};

use super::{r, Tech, Unlock};

pub const LATER: [Tech; 3] = [
    Tech {
        name: "Long-Distance Power",
        blurb: "Lattice towers, 2×2 and 3 high, that link to each other 160 blocks apart (an ordinary pole only 16), \
                to carry power across the wilderness to far mines and oil wells.",
        needs: &[19, 36],
        packs: &[RED_PACK, GREEN_PACK, BLUE_PACK, VIOLET_PACK],
        units: 150,
        seconds: 30.0,
        unlocks: &[r(LONG_POLE), Unlock::MachineRecipe(LONG_POLE_ROW)],
    },
    Tech {
        name: "Drop Pods",
        blurb: "Ship cargo through the sky. An assembler builds drop pods; a pod launcher (3×3×3, 500 kW) loads up to 6 \
                stacks from its belts into one and sends it up, and 20 seconds later it lands on a landing pad (3×3) \
                that belts empty. Both need a comms satellite overhead, and a launcher and a pad pair up when they face \
                the same way (four channels): a pod goes to the nearest pad on its channel.",
        needs: &[78, 79],
        packs: &[RED_PACK, GREEN_PACK, BLUE_PACK, VIOLET_PACK, GOLD_PACK],
        units: 250,
        seconds: 60.0,
        unlocks: &[r(POD_LAUNCHER), r(LANDING_PAD), Unlock::MachineRecipe(DROP_POD_RECIPE)],
    },
    Tech {
        name: "Settlements",
        blurb: "Found towns. A town depot (3×3, 2 high) placed at a fitting site founds one, of a kind that site decides: \
                a harbour on a sea shore, a river town on a river or lake bank, a highland workshop in the granite \
                highlands, a basalt forge in the basalt fields. Towns stand at least 300 blocks apart and away from \
                the spawn plains; while you hold a depot it says whether the spot fits. Belts fill it from any side.",
        needs: &[10],
        packs: &[RED_PACK, GREEN_PACK, BLUE_PACK],
        units: 100,
        seconds: 20.0,
        unlocks: &[r(TOWN_DEPOT)],
    },
];
