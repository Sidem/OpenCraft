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

pub const SENSOR_RECIPE: Recipe = Recipe {
    output: b(SENSOR),
    group: Group::Power,
    count: 1,
    inputs: &[(CIRCUIT, 1), (IRON_PLATE, 1)],
    blurb: "Reads the storage box, silo or belt behind it and switches the machine in front of it on or off, \
            by a rule you step through with a right-click: a plain switch, or on while the box is emptier or \
            fuller than a level. Place it facing the machine (R turns it). It cuts the machine's power wire, so \
            burners and belts ignore it.",
};

pub const DRONE_PORT_RECIPE: Recipe = Recipe {
    output: b(DRONE_PORT),
    group: Group::Production,
    count: 1,
    inputs: &[(STEEL_PLATE, 24), (CIRCUIT, 12), (PROCESSOR, 2), (MOTOR, 4)],
    blurb: "A 3×3 landing pad for construction drones (R turns it before you place it). Put drones in by hand \
            or by belt; they build your ghosts and tear down what you mark in ghost mode (B), taking what they \
            need from storage boxes touching the pad and putting what they gather into them. Needs power \
            while drones are out: 40 kW.",
};
