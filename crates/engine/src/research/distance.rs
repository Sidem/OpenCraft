//! The techs of Milestone 9 (distance), as data (appended after `personal.rs`'s in `techs::TECHS`, so the order here
//! is the saved index order: add rows at the bottom, never reorder). Needs name main-tree indices (Ore Crushing 15,
//! Violet Science 18) and personal ones from 30 (Earthworks 32); this table's own start at 36.

use crate::block::{ELECTROLYTIC_CELL, RAIL};
use crate::item::{BLUE_PACK, GREEN_PACK, RED_PACK, VIOLET_PACK};
use crate::recipes::BAUXITE_RECIPES;

use super::{r, Tech, Unlock};

pub const DISTANCE: [Tech; 2] = [
    Tech {
        name: "Bauxite Processing",
        blurb: "Crushers grind bauxite ore, and electrolytic cells (300 kW) turn 2 crushed bauxite and a quicklime \
                into an aluminium ingot and a slag. Constructors press plates; assemblers build batteries from a \
                plate, a circuit and copper wire.",
        needs: &[15, 18],
        packs: &[RED_PACK, GREEN_PACK, BLUE_PACK, VIOLET_PACK],
        units: 120,
        seconds: 30.0,
        unlocks: &[
            r(ELECTROLYTIC_CELL),
            Unlock::MachineRecipe(BAUXITE_RECIPES[0]),
            Unlock::MachineRecipe(BAUXITE_RECIPES[1]),
            Unlock::MachineRecipe(BAUXITE_RECIPES[2]),
            Unlock::MachineRecipe(BAUXITE_RECIPES[3]),
        ],
    },
    Tech {
        name: "Rails",
        blurb: "Rails from a steel beam and a concrete, six at a time: place rail nodes like power poles and a smooth \
                curve of track joins each pair, up to 32 blocks apart, climbing one block in three at most. Level and \
                cut the ground with the planner first.",
        needs: &[18, 32],
        packs: &[RED_PACK, GREEN_PACK, BLUE_PACK, VIOLET_PACK],
        units: 100,
        seconds: 30.0,
        unlocks: &[r(RAIL)],
    },
];
