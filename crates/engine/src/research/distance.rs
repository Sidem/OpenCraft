//! The techs of Milestone 9 (distance), as data (appended after `personal.rs`'s in `techs::TECHS`, so the order here
//! is the saved index order: add rows at the bottom, never reorder). Needs name main-tree indices (Ore Crushing 15,
//! Violet Science 18) and personal ones from 30 (Earthworks 32); this table's own start at 36.

use crate::block::{ELECTROLYTIC_CELL, LOADING_DOCK, RAIL, UNLOADING_DOCK};
use crate::item::{BLUE_PACK, GREEN_PACK, LOCOMOTIVE, RED_PACK, VIOLET_PACK, WAGON};
use crate::recipes::BAUXITE_RECIPES;

use super::{r, Tech, Unlock};

pub const DISTANCE: [Tech; 4] = [
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
    Tech {
        name: "Trains",
        blurb: "Locomotives from motors, circuits and steel: put one on a rail node and it runs the track at 9 blocks \
                a second, turning round at the ends. Freight adds wagons and docks.",
        needs: &[37],
        packs: &[RED_PACK, GREEN_PACK, BLUE_PACK, VIOLET_PACK],
        units: 150,
        seconds: 40.0,
        unlocks: &[Unlock::Recipe(LOCOMOTIVE)],
    },
    Tech {
        name: "Freight",
        blurb: "Wagons with a box's worth of slots coupled behind a locomotive, and loading and unloading docks: \
                belts fill a loading dock, a train stopped beside it takes the items, and an unloading dock empties \
                the cargo onto belts at the other end.",
        needs: &[38],
        packs: &[RED_PACK, GREEN_PACK, BLUE_PACK, VIOLET_PACK],
        units: 120,
        seconds: 40.0,
        unlocks: &[Unlock::Recipe(WAGON), r(LOADING_DOCK), r(UNLOADING_DOCK)],
    },
];
