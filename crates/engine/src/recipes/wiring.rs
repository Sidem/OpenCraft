//! Hand recipes of the power grid's wiring (`factory/pole.rs`), listed in `RECIPES`. To add one: a `pub const`
//! here, and its name in `RECIPES`.

use crate::block::*;
use crate::item::*;

use super::{b, Group, Recipe};

pub const LONG_POLE_RECIPE: Recipe = Recipe {
    output: b(LONG_POLE),
    group: Group::Power,
    count: 1,
    inputs: &[(STEEL_BEAM, 12), (ALUMINIUM_PLATE, 12), (COPPER_WIRE, 80), (b(CONCRETE), 30), (b(GLASS), 8)],
    blurb: "A lattice tower, 2×2 and 3 high, for power lines that cross the wilderness to far mines and oil wells. \
            It links to other long-distance poles up to 160 blocks away, but to an ordinary pole only within 16. \
            Wire it by hand like a pole (right-click to select it, crouch-click to link); it powers machines within 10 \
            blocks and takes 6 connections. A new one wires itself to the nearest powered pole it can reach.",
};

pub const CABLE_RECIPE: Recipe = Recipe {
    output: b(CABLE),
    group: Group::Power,
    count: 4,
    inputs: &[(IRON_PLATE, 1), (COPPER_WIRE, 2)],
    blurb: "A cable to hang down a shaft or along a tunnel where poles don't belong. Click near a pole and the \
            cable drops to the ground (crouch places just one); a machine within 2 blocks of a cable is powered.",
};
