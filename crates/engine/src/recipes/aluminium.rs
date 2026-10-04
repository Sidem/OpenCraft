//! Hand recipes of Aluminium (Milestone 9), listed in `RECIPES`. Crushed bauxite, the ingot, plate and battery are
//! machine recipes (`machine.rs`). To add one: a `pub const` here, and its name in `RECIPES`.

use crate::block::*;
use crate::item::*;

use super::{b, Group, Recipe};

pub const ELECTROLYTIC_CELL_RECIPE: Recipe = Recipe {
    output: b(ELECTROLYTIC_CELL),
    group: Group::Production,
    count: 1,
    inputs: &[(STEEL_PLATE, 12), (b(STONE_BRICKS), 16), (COPPER_WIRE, 24), (CIRCUIT, 4)],
    blurb: "Makes aluminium from 2 crushed bauxite and a quicklime, 6 seconds an ingot, and leaves a slag. It is \
            3×2×2 (R turns it before you place it): belts bring both in at the hatches on its back and left, the \
            ingot leaves by the front and the slag by the violet hatch on the right. Needs 300 kW.",
};
