//! The hand recipe of the recycler (the Recycling tech: `factory/process/recycler.rs`), listed in `RECIPES`. What it
//! pays for items is worked out from all the recipes: `recycling.rs`.

use crate::block::*;
use crate::item::*;

use super::{b, Group, Recipe};

pub const RECYCLER_RECIPE: Recipe = Recipe {
    output: b(RECYCLER),
    group: Group::Production,
    count: 1,
    inputs: &[(STEEL_PLATE, 10), (STEEL_BEAM, 4), (MOTOR, 4), (b(CONCRETE), 8)],
    blurb: "Destroys any item and pays recycling coins, 2×2×2 (R turns it before you place it). Belts bring items in \
            at the hatches on its back and sides; the coins (1024 to a stack) leave by the two front hatches. Raw \
            things pay 1 coin and every step of processing doubles it: an ingot 2, a plate 8. Slag and tailings, and \
            what is crushed from them, pay only 1. Needs 90 kW.",
};
