//! Hand recipes for towns and what grows from them, listed in `RECIPES`: the town depot (Milestone 13, the Settlements
//! tech) and the space elevator's anchor (M14A, the Space Elevator tech).

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

pub const ELEVATOR_ANCHOR_RECIPE: Recipe = Recipe {
    output: ELEVATOR_ANCHOR_ITEM,
    group: Group::Space,
    count: 1,
    inputs: &[(b(CONCRETE), 200), (STEEL_BEAM, 80), (MOTOR, 20), (AVIONICS, 4)],
    blurb: "The foundation of the space elevator, 5×5 and 2 high: it stands near the depot of a town or a city, one \
            per world. Belts bring each section's order into its back and sides, and the tether climbs 64 blocks a \
            section, eight in all; from the second section on, its climbers use up goods every minute. While you hold \
            it, it says whether the spot fits.",
};
