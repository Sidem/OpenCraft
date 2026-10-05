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
    blurb: "Crushes 2 iron, copper or bauxite ore into 3 crushed ore, which smelt one for one, and grinds slag and \
            tailings to sand. The first step of the ore chain: the washer takes only what it crushes. Needs 30 kW.",
};

pub const WASHER_RECIPE: Recipe = Recipe {
    output: b(WASHER),
    group: Group::Production,
    count: 1,
    inputs: &[(STEEL_PLATE, 8), (STEEL_BEAM, 4), (MOTOR, 2), (b(GLASS), 4)],
    blurb: "Washes crushed ore, 2×2×2 (R turns it before you place it): 3 crushed iron, copper or bauxite and a unit \
            of water (a pipe from a pump to the blue inlet on its left) make 4 washed ore out of the front in 3 \
            seconds, which smelt one for one: 2 ingots from every ore instead of 1.5. Tailings leave by the violet \
            hatch on the right and must be taken away; the crusher grinds them to sand. Raw ore does not fit. Needs 60 kW.",
};

pub const SILO_RECIPE: Recipe = Recipe {
    output: b(SILO),
    group: Group::Logistics,
    count: 1,
    inputs: &[(STEEL_PLATE, 24), (b(CONCRETE), 8), (STEEL_BEAM, 4)],
    blurb: "A 2×2×3 store of 144 stacks: belts feed it from every side and belts leading away take items out.",
};
