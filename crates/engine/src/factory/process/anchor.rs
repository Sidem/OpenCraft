//! The Elevator Anchor (M14A, tech Space Elevator) as a spec row: a 5×5×2 concrete foundation with the tether's socket.
//! Like a town depot (`Pick::Depot`) it holds only what it is told to take (`Extra::Depot`, set each tick by
//! `Sim::sync_elevator`), from belts on its back and sides, and belts never empty it; the elevator uses it up
//! (`elevator/`, through `Factory::depot_store`); the empty barrels of its rocket fuel come out of the front. It stands
//! only beside a grown town (checked in `action/multiblock.rs`). The tether above it is drawn by `elevator/view.rs`, not here.
//! The look is a placeholder from existing textures (`docs/ART_HANDOVER.md` has the request).

use crate::block::{tex, ELEVATOR_ANCHOR};

use super::super::footprint::{Footprint, Port, Role, Side, Which};
use super::model::{part, round, Look, Part};
use super::specs::{Energy, Pick, ProcessSpec, ProcessTier};

/// Stacks the anchor's store holds, and for the empty barrels it gives back at its front.
const SLOTS: usize = 32;
const BARREL_SLOTS: usize = 4;

pub const ANCHOR_SPEC: ProcessSpec = ProcessSpec {
    block: ELEVATOR_ANCHOR,
    categories: &[],
    pick: Pick::Depot,
    buffers: [0, 0, SLOTS],
    side: BARREL_SLOTS,
    tiers: &[ProcessTier { energy: Energy::Recipe, speed: 1000, fuel: 0, power: 0 }],
    footprint: Footprint {
        size: [5, 5, 2],
        ports: &[
            Port { side: Side::Front, role: Role::Side, cell: Which::All },
            Port { side: Side::Back, role: Role::In, cell: Which::All },
            Port { side: Side::Left, role: Role::In, cell: Which::All },
            Port { side: Side::Right, role: Role::In, cell: Which::All },
        ],
    },
    verb: "Receiving",
    products: "tether",
    waiting: "",
    map_colour: 0xe2e8f0,
    compute: 0,
    parts: &ANCHOR_PARTS,
};

const CONCRETE: Look = Look::Tex([tex::CONCRETE; 3]);
const STEEL: Look = Look::Tex([tex::STEEL; 3]);
const BEAM: Look = Look::Tex([tex::STEEL_BEAM; 3]);

/// A stepped concrete foundation, four steel buttresses and the round socket the tether rises from.
const ANCHOR_PARTS: [Part; 9] = [
    part([0.0, -0.8, 0.0], [4.96, 0.4, 4.96], CONCRETE),
    part([0.0, -0.45, 0.0], [4.2, 0.3, 4.2], CONCRETE),
    part([1.7, -0.05, 1.7], [0.5, 0.5, 0.5], BEAM),
    part([-1.7, -0.05, 1.7], [0.5, 0.5, 0.5], BEAM),
    part([1.7, -0.05, -1.7], [0.5, 0.5, 0.5], BEAM),
    part([-1.7, -0.05, -1.7], [0.5, 0.5, 0.5], BEAM),
    round([0.0, -0.1, 0.0], [2.6, 0.4, 2.6], STEEL),
    round([0.0, 0.35, 0.0], [1.6, 0.5, 1.6], Look::Tex([tex::STEEL, tex::FRAME, tex::STEEL])),
    round([0.0, 0.8, 0.0], [0.9, 0.4, 0.9], Look::Tex([tex::LAMP_BLUE, tex::STEEL, tex::STEEL])),
];
