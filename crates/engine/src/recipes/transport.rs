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
    blurb: "Track for trains. Hold rails and drag along the ground like belts (up to 64 a drag): a rail joins the \
            rails beside it, level or a block up or down, so a track climbs one block a cell. Level the ground with \
            the planner first for a long run.",
};
