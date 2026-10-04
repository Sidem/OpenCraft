//! Hand recipes of the railway (Milestone 9), listed in `RECIPES`. To add one: a `pub const` here, and its name in
//! `RECIPES`.

use crate::block::*;
use crate::item::*;

use super::{b, Group, Recipe};

pub const RAIL_RECIPE: Recipe = Recipe {
    output: b(RAIL),
    group: Group::Logistics,
    count: 6,
    inputs: &[(STEEL_BEAM, 1), (b(CONCRETE), 1)],
    blurb: "Track for trains, laid like power poles. Each rail is a node: click the ground to place one, click on to \
            place the next and a smooth curve joins them (Shift: full reach, 32 blocks). Click nodes to select them, \
            join them or crouch-click to cut. Nodes sit on the grid, the track between them does not.",
};
