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

pub const LOCOMOTIVE_RECIPE: Recipe = Recipe {
    output: LOCOMOTIVE,
    group: Group::Logistics,
    count: 1,
    inputs: &[(MOTOR, 4), (CIRCUIT, 4), (STEEL_PLATE, 10), (STEEL_BEAM, 4)],
    blurb: "A locomotive. Hold it and click a rail node that has track: it sets off along the line at 9 blocks a \
            second, turns round at the end of the track, and takes the straightest track at a junction. \
            Crouch-click near it to pick it up. Wagons and docks come with Freight.",
};

pub const WAGON_RECIPE: Recipe = Recipe {
    output: WAGON,
    group: Group::Logistics,
    count: 1,
    inputs: &[(STEEL_PLATE, 8), (STEEL_BEAM, 2), (b(STORAGE), 1)],
    blurb: "A wagon with a box's worth of slots (24). Hold it and click a rail node beside a train to couple it on \
            behind (up to 6). Trains stop at docks and trade their cargo there. Crouch-click picks the whole train up.",
};

pub const RAIL_SIGNAL_RECIPE: Recipe = Recipe {
    output: RAIL_SIGNAL,
    group: Group::Logistics,
    count: 2,
    inputs: &[(STEEL_PLATE, 1), (CIRCUIT, 1)],
    blurb: "A rail signal. Hold it and click a rail node to put it there (click again to take it back). The track \
            between signals holds one train at a time: a train that would enter a stretch another train is on \
            waits at the signal, or takes a free branch at a junction. A passing loop lets trains meet and pass.",
};

pub const LOADING_DOCK_RECIPE: Recipe = Recipe {
    output: b(LOADING_DOCK),
    group: Group::Logistics,
    count: 1,
    inputs: &[(STEEL_PLATE, 10), (MOTOR, 2), (CIRCUIT, 2), (b(STORAGE), 1)],
    blurb: "A dock beside the track (within 2 blocks of a rail node). Belts feed it on any side; a train with wagons \
            that stops there takes the items on board, and leaves when full or after 5 idle seconds.",
};

pub const UNLOADING_DOCK_RECIPE: Recipe = Recipe {
    output: b(UNLOADING_DOCK),
    group: Group::Logistics,
    count: 1,
    inputs: &[(STEEL_PLATE, 10), (MOTOR, 2), (CIRCUIT, 2), (b(STORAGE), 1)],
    blurb: "A dock beside the track (within 2 blocks of a rail node). A train with wagons that stops there empties \
            its cargo into it; belts leading away from any side carry the items off.",
};
