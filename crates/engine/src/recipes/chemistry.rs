//! Hand recipes of Chemistry (Milestone 10), listed in `RECIPES`. Canisters are also pressed by constructors
//! (`machine.rs`). To add one: a `pub const` here, and its name in `RECIPES`.

use crate::block::*;
use crate::item::*;

use super::{b, Group, Recipe};

pub const PUMPJACK_RECIPE: Recipe = Recipe {
    output: b(PUMPJACK),
    group: Group::Production,
    count: 1,
    inputs: &[(STEEL_PLATE, 12), (STEEL_BEAM, 4), (MOTOR, 2), (CIRCUIT, 2)],
    blurb: "Stand it on the ground right above an oil reservoir (the scanner's oil filter and the map show where): it \
            drills straight down to the oil sand. It fills an empty canister with crude oil every few seconds, \
            from the reservoir's own pace (a vein gives about 20 canisters a minute), until the field runs dry. \
            Belts bring empty canisters and take the full ones. Needs 90 kW.",
};

pub const WATER_WHEEL_RECIPE: Recipe = Recipe {
    output: b(WATER_WHEEL),
    group: Group::Power,
    count: 1,
    inputs: &[(b(PLANKS), 12), (IRON_ROD, 8), (GEAR, 4), (COPPER_WIRE, 6)],
    blurb: "A 3×3×1 wheel that stands in a river or beside a lake: every water block touching it gives 2 kW (flowing \
            water 4), up to 48 kW, day and night with no fuel. A river's weir or a pool you pipe in works as a dam. \
            Hang it on a power pole.",
};

pub const HOIST_RECIPE: Recipe = Recipe {
    output: b(HOIST),
    group: Group::Building,
    count: 2,
    inputs: &[(STEEL_BEAM, 2), (IRON_ROD, 4)],
    blurb:
        "A steel climbing frame, stacked like ladders down a shaft: climb it with jump and crouch (3 blocks a second). \
            With a hoist winch beside the shaft's top block and power on the wire it carries you at up to 9.",
};

pub const WINCH_RECIPE: Recipe = Recipe {
    output: b(WINCH),
    group: Group::Power,
    count: 1,
    inputs: &[(STEEL_PLATE, 10), (STEEL_BEAM, 4), (GEAR, 4), (MOTOR, 2), (CIRCUIT, 2)],
    blurb: "Place it against the top block of a hoist shaft (above or beside it) and hang it on a power pole: riders \
            in the shaft then climb at up to 9 blocks a second, slower when the grid is short. Draws 20 kW.",
};

pub const CENTRIFUGE_RECIPE: Recipe = Recipe {
    output: b(CENTRIFUGE),
    group: Group::Production,
    count: 1,
    inputs: &[(STEEL_PLATE, 14), (STEEL_BEAM, 6), (MOTOR, 4), (CIRCUIT, 6), (PLASTIC, 4)],
    blurb:
        "Enriches uranium, 2×2×3 (R turns it before you place it): belts bring 4 uranium ore and a steel plate in at \
            the back and take a fuel cell from the front every 10 seconds. Needs 200 kW.",
};

pub const REACTOR_RECIPE: Recipe = Recipe {
    output: b(REACTOR),
    group: Group::Power,
    count: 1,
    inputs: &[(STEEL_PLATE, 40), (STEEL_BEAM, 20), (b(CONCRETE), 32), (CIRCUIT, 12), (PLASTIC, 8)],
    blurb: "Up to 2 MW from fuel cells (150 seconds each at full load), 3×3×3 (R turns it before you place it): belts \
            bring fuel cells in at the back, a pipe from a pump brings coolant water to the blue inlet on its left. \
            It burns only what the grid asks for; without water it heats up, and past its limit it shuts down until \
            it has cooled. Wire it to a pole like any power source.",
};

pub const REFINERY_RECIPE: Recipe = Recipe {
    output: b(REFINERY),
    group: Group::Production,
    count: 1,
    inputs: &[(STEEL_PLATE, 16), (STEEL_BEAM, 8), (b(STONE_BRICKS), 12), (MOTOR, 2), (CIRCUIT, 6)],
    blurb:
        "Fractional distillation, 3×3×4 (R turns it before you place it). Belts bring crude oil canisters in at the \
            hatches on its back, a pipe from a pump brings water to the blue inlet on its left; a batch of 3 crude \
            canisters and a unit of water takes 6 seconds and gives a naphtha canister out of the front and, out of \
            the violet side hatches on the right, a diesel canister, a heavy oil canister and a sulfur. Every stream \
            must be used or stored, or it stops. Needs 150 kW.",
};

pub const CRACKER_RECIPE: Recipe = Recipe {
    output: b(CRACKER),
    group: Group::Production,
    count: 1,
    inputs: &[(STEEL_PLATE, 10), (STEEL_BEAM, 4), (MOTOR, 1), (CIRCUIT, 4)],
    blurb: "Splits heavy oil, 2×2×3: 2 heavy oil canisters and a unit of water (blue inlet on the left) make a \
            naphtha and a diesel canister in 4 seconds. Belts bring heavy oil in at the back; naphtha leaves by the \
            front and diesel by the violet hatches on the right. Needs 90 kW.",
};

pub const CHEMICAL_PLANT_RECIPE: Recipe = Recipe {
    output: b(CHEMICAL_PLANT),
    group: Group::Production,
    count: 1,
    inputs: &[(STEEL_PLATE, 14), (STEEL_BEAM, 6), (b(GLASS), 8), (MOTOR, 2), (CIRCUIT, 6)],
    blurb: "Plastic, acid and lubricant, 3×2×3 (R turns it before you place it); choose what it makes in its panel. \
            Belts bring the inputs in at the hatches on its back and take the product from the front; plastic's two \
            spare empty canisters leave by the violet hatches on the right. Acid needs a unit of water from a pipe \
            to the blue inlet on its left. Needs 120 kW.",
};

pub const DIESEL_GENERATOR_RECIPE: Recipe = Recipe {
    output: b(DIESEL_GENERATOR),
    group: Group::Production,
    count: 1,
    inputs: &[(STEEL_PLATE, 14), (STEEL_BEAM, 6), (MOTOR, 4), (CIRCUIT, 4)],
    blurb: "Burns diesel canisters for 400 kW, 2×2×2 (R turns it before you place it): one canister lasts 100 \
            seconds at full load, far more energy than the oil cost to make. Belts bring diesel canisters in at the \
            back and take the empties from the front. Wire it to a pole like any power source; it burns only what \
            the grid asks for, after solar, coal and steam.",
};

pub const ELECTROLYSER_RECIPE: Recipe = Recipe {
    output: b(ELECTROLYSER),
    group: Group::Production,
    count: 1,
    inputs: &[(STEEL_PLATE, 12), (STEEL_BEAM, 4), (b(GLASS), 6), (COPPER_WIRE, 24), (CIRCUIT, 4)],
    blurb: "Splits water, 2×2×2 (R turns it before you place it): 3 empty canisters and 2 units of water (a pipe to \
            the blue inlet on its left) make 2 hydrogen canisters out of the front and an oxygen canister out of \
            the violet hatch on the right in 6 seconds. Both gases must be used or stored. Needs 500 kW.",
};
pub const CANISTER_RECIPE: Recipe = Recipe {
    output: EMPTY_CANISTER,
    group: Group::Materials,
    count: 2,
    inputs: &[(STEEL_PLATE, 1)],
    blurb: "A steel drum that carries any fluid but water on belts. Every machine that empties one gives it back; a \
            constructor presses them faster.",
};
