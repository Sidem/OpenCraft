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

pub const CANISTER_RECIPE: Recipe = Recipe {
    output: EMPTY_CANISTER,
    group: Group::Materials,
    count: 2,
    inputs: &[(STEEL_PLATE, 1)],
    blurb: "A steel drum that carries any fluid but water on belts. Every machine that empties one gives it back; a \
            constructor presses them faster.",
};
