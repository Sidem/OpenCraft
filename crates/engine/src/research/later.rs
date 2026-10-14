//! The open end of `techs::TECHS`: techs added after Milestone 12's first steps, so that `orbit.rs` stays closed and no saved
//! index moves. Saved index order: Long-Distance Power 82. Add rows at the bottom, never reorder (new Orbit techs go here
//! too). Needs name earlier indices: Mk4 Logistics 19, Bauxite Processing 36.

use crate::block::LONG_POLE;
use crate::item::{BLUE_PACK, GREEN_PACK, RED_PACK, VIOLET_PACK};

use super::{r, Tech};

pub const LATER: [Tech; 1] = [Tech {
    name: "Long-Distance Power",
    blurb:
        "Lattice towers, 2×2 and 3 high, that link to each other 160 blocks apart (an ordinary pole only 16), to carry \
            power across the wilderness to far mines and oil wells.",
    needs: &[19, 36],
    packs: &[RED_PACK, GREEN_PACK, BLUE_PACK, VIOLET_PACK],
    units: 150,
    seconds: 30.0,
    unlocks: &[r(LONG_POLE)],
}];
