//! The hand recipes of Milestone 11's compute machines, listed in `RECIPES`. Starts with the chip fab (the Wafers tech:
//! `factory/process/fab.rs`).

use crate::block::*;
use crate::item::*;

use super::{b, Group, Recipe};

pub const CHIP_FAB_RECIPE: Recipe = Recipe {
    output: b(CHIP_FAB),
    group: Group::Production,
    count: 1,
    inputs: &[(STEEL_PLATE, 60), (STEEL_BEAM, 24), (b(GLASS), 24), (PROCESSOR, 16), (CIRCUIT, 24)],
    blurb:
        "A clean room, 4×4×3 (R turns it before you place it). Wafers: 2 silicon, an acid canister and a pure water \
            canister make a wafer in 8 seconds and give both canisters back as empties (violet hatches on the right). \
            AI accelerators: 2 wafers, 2 processors and a plastic, 20 seconds. Belts bring everything in at the back \
            hatches and take the product from the front; it takes whatever its recipes use. Needs 1 MW.",
};
