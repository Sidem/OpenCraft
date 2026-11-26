//! Techs added after the frontier map (76 on), at the open end of [`super::TECHS`]: saves store progress by index, so
//! a new tech goes here whatever its ring, and its `ring` and `wedge` place it on the research screen.
//! To add one: a row here, its index in `ids.rs`, and `TECHS` grows by itself.

use crate::item::{BLUE_PACK, ELEVATOR_ANCHOR_ITEM, GOLD_PACK, GREEN_PACK, RED_PACK, VIOLET_PACK};
use crate::recipes::{AMBER_ROW, CLAY_BRICK_ROW, PORCELAIN_ROW, TERRACOTTA_ROW, TURQUOISE_ROW};

use super::ids::*;
use super::rings::{Ring, Wedge};
use super::{Tech, Unlock};

pub(super) const LATER: [Tech; 3] = [
    Tech {
        name: "Clay & Kilns",
        blurb: "Clay from river banks and lake beds: any furnace fires it into clay bricks, and the Glass Kiln into \
                terracotta roof tiles.",
        ring: Ring::Green,
        wedge: Wedge::Building,
        deed: None,
        needs: &[GLASS],
        packs: &[RED_PACK, GREEN_PACK],
        units: 30,
        seconds: 10.0,
        unlocks: &[Unlock::MachineRecipe(CLAY_BRICK_ROW), Unlock::MachineRecipe(TERRACOTTA_ROW)],
    },
    Tech {
        name: "Porcelain & Glazes",
        blurb: "White kaolin from weathered highland granite, fired in the Glass Kiln with granite into porcelain \
                tiles; porcelain with copper green or iron red pigment fires into turquoise or amber glazed tiles.",
        ring: Ring::Blue,
        wedge: Wedge::Building,
        deed: None,
        needs: &[GLASS],
        packs: &[RED_PACK, GREEN_PACK, BLUE_PACK],
        units: 40,
        seconds: 15.0,
        unlocks: &[
            Unlock::MachineRecipe(PORCELAIN_ROW),
            Unlock::MachineRecipe(TURQUOISE_ROW),
            Unlock::MachineRecipe(AMBER_ROW),
        ],
    },
    Tech {
        name: "Space Elevator",
        blurb: "The elevator anchor (5×5) stands by a town or a city and raises a tether into the sky, 64 blocks a \
                section for eight sections, each a large order of steel beams, aluminium, plastic, chips and rocket fuel.",
        ring: Ring::Sky,
        wedge: Wedge::TownsLife,
        deed: None,
        needs: &[ROCKETRY],
        packs: &[RED_PACK, GREEN_PACK, BLUE_PACK, VIOLET_PACK, GOLD_PACK],
        units: 400,
        seconds: 60.0,
        unlocks: &[Unlock::Recipe(ELEVATOR_ANCHOR_ITEM)],
    },
];
