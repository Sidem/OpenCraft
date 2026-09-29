//! Hand recipes of Electronics (Milestone 7: `factory/process/`), listed in `RECIPES`. Silicon and circuits are
//! machine recipes (`machine.rs`). To add one: a `pub const` here, and its name in `RECIPES`.

use crate::block::*;
use crate::item::*;

use super::{b, Group, Recipe};

pub const ARC_FURNACE_RECIPE: Recipe = Recipe {
    output: b(ARC_FURNACE),
    group: Group::Production,
    count: 1,
    inputs: &[(STEEL_PLATE, 10), (b(STONE_BRICKS), 16), (COPPER_WIRE, 24)],
    blurb: "Makes silicon from a quartz ore and a coal, 4 seconds a piece. It is 2×2×2 (R turns it before you \
            place it): belts bring both in at the hatches on its back and sides, silicon leaves by the front. \
            Choose its recipe in its panel. Needs 120 kW.",
};
