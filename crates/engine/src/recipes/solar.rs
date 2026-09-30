//! Hand recipes of solar power (`factory/process/solar.rs`), listed in `RECIPES`. To add one: a `pub const` here,
//! and its name at the end of `RECIPES`.

use crate::block::*;
use crate::item::*;

use super::{b, Group, Recipe};

pub const SOLAR_PANEL_RECIPE: Recipe = Recipe {
    output: b(SOLAR_PANEL),
    group: Group::Power,
    count: 1,
    inputs: &[(SILICON, 6), (b(GLASS), 2), (COPPER_WIRE, 4), (IRON_PLATE, 2)],
    blurb: "A flat 2×2 panel: gives up to 10 kW in full sun, less towards dawn and dusk and nothing at night. \
            Hang it on a power pole; a panel that has nothing to feed charges accumulators.",
};

pub const ACCUMULATOR_RECIPE: Recipe = Recipe {
    output: b(ACCUMULATOR),
    group: Group::Power,
    count: 1,
    inputs: &[(STEEL_PLATE, 6), (CIRCUIT, 2), (COPPER_WIRE, 12)],
    blurb: "A 2×2×2 battery: stores 10 MJ of spare solar power and gives it back, up to 60 kW, when the grid \
            wants more than the sun gives. Hang it on a power pole.",
};
