//! Hand recipes of the heavy machines (Steam Power, Ore Crushing, Bulk Storage: `factory/process/`), listed
//! in `RECIPES`. To add one: a `pub const` here, and its name in `RECIPES`.

use crate::block::*;
use crate::item::*;

use super::{b, Group, Recipe};

pub const BOILER_RECIPE: Recipe = Recipe {
    output: b(BOILER),
    group: Group::Power,
    count: 1,
    inputs: &[(b(STONE_BRICKS), 24), (STEEL_PLATE, 8), (b(PIPE), 6)],
    blurb: "A 2×2×2 boiler: each of its back and side faces has a coal inlet (dark chute) for a belt and a water inlet \
            (blue pipe) for a pump's pipe. It makes steam with twice the energy of a generator's fire and sends it out of \
            its two front outlets (red banded pipes) to the turbines.",
};

pub const TURBINE_RECIPE: Recipe = Recipe {
    output: b(TURBINE),
    group: Group::Power,
    count: 1,
    inputs: &[(STEEL_PLATE, 12), (MOTOR, 4), (COPPER_WIRE, 16)],
    blurb: "A 3×2×2 steam turbine: pipe its steam inlet (the end away from the generator) to a boiler's steam outlet \
            (a boiler drives two) and hang it on a power pole; it gives up to 240 kW.",
};

pub const CRUSHER_RECIPE: Recipe = Recipe {
    output: b(CRUSHER),
    group: Group::Production,
    count: 1,
    inputs: &[(STEEL_PLATE, 6), (MOTOR, 2), (GEAR, 4)],
    blurb: "Crushes 2 iron or copper ore into 3 crushed ore, which smelt one for one, and grinds slag to sand. \
            Needs 30 kW.",
};

pub const SILO_RECIPE: Recipe = Recipe {
    output: b(SILO),
    group: Group::Logistics,
    count: 1,
    inputs: &[(STEEL_PLATE, 24), (b(CONCRETE), 8), (STEEL_BEAM, 4)],
    blurb: "A 2×2×3 store of 144 stacks: belts feed it from every side and belts leading away take items out.",
};
