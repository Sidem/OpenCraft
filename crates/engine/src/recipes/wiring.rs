//! Hand recipes of the power grid's wiring (`factory/pole.rs`), listed in `RECIPES`. To add one: a `pub const`
//! here, and its name in `RECIPES`.

use crate::block::*;
use crate::item::*;

use super::{b, Group, Recipe};

pub const CABLE_RECIPE: Recipe = Recipe {
    output: b(CABLE),
    group: Group::Power,
    count: 4,
    inputs: &[(IRON_PLATE, 1), (COPPER_WIRE, 2)],
    blurb: "A cable to hang down a shaft or along a tunnel where poles don't belong. Click near a pole and the \
            cable drops to the ground (crouch places just one); a machine within 2 blocks of a cable is powered.",
};
