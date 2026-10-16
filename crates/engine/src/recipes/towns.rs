//! Milestone 13's hand recipes (the Settlements tech): the town depot, listed in `RECIPES`.

use crate::block::*;
use crate::item::*;

use super::{b, Group, Recipe};

pub const TOWN_DEPOT_RECIPE: Recipe = Recipe {
    output: b(TOWN_DEPOT),
    group: Group::Building,
    count: 1,
    inputs: &[(b(STONE_BRICKS), 40), (b(PLANKS), 40), (b(GLASS), 8), (STEEL_PLATE, 10)],
    blurb:
        "A warehouse, 3×3 and 2 high, that founds a town where it stands: a harbour on a sea shore, a river town on \
            a river or lake bank, a highland workshop in the granite highlands, a basalt forge in the basalt fields. \
            Towns stand at least 300 blocks apart and away from the spawn plains; while you hold it, it says whether \
            the spot fits. Belts fill it from any side.",
};
