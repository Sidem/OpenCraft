//! The town depot (step 13.1, tech Settlements) as a spec row: a 3×3×2 warehouse that belts on any side fill, holding
//! what it is given like a silo (`Pick::Store`); nothing takes it out. Placed at a fitting site it founds a town
//! (`settlements/`, checked in `action/multiblock.rs`). Its orders (13.2) will decide what it takes and use it up.
//! The look is a placeholder from existing textures (`docs/ART_HANDOVER.md` has the request).

use crate::block::{tex, TOWN_DEPOT};

use super::super::footprint::{Footprint, Port, Role, Side, Which};
use super::model::{part, Look, Part};
use super::specs::{Energy, Pick, ProcessSpec, ProcessTier};

/// Stacks a depot holds.
const SLOTS: usize = 24;

const IN: Role = Role::In;

pub const DEPOT_SPEC: ProcessSpec = ProcessSpec {
    block: TOWN_DEPOT,
    categories: &[],
    pick: Pick::Store,
    buffers: [0, 0, SLOTS],
    side: 0,
    tiers: &[ProcessTier { energy: Energy::Recipe, speed: 1000, fuel: 0, power: 0 }],
    footprint: Footprint {
        size: [3, 3, 2],
        ports: &[
            Port { side: Side::Front, role: IN, cell: Which::All },
            Port { side: Side::Back, role: IN, cell: Which::All },
            Port { side: Side::Left, role: IN, cell: Which::All },
            Port { side: Side::Right, role: IN, cell: Which::All },
        ],
    },
    verb: "Receiving",
    products: "goods",
    waiting: "",
    map_colour: 0xd08a4a,
    compute: 0,
    parts: &DEPOT_PARTS,
};

const BRICK: Look = Look::Tex([tex::STONE_BRICKS; 3]);
const WOOD: Look = Look::Tex([tex::PLANKS; 3]);
const ROOF: Look = Look::Tex([tex::PLANKS, tex::FRAME, tex::FRAME]);
const PANE: Look = Look::Tex([tex::GLASS; 3]);

/// A stone-brick plinth, plank walls under a stepped roof, a door and two windows on the front.
const DEPOT_PARTS: [Part; 8] = [
    part([0.0, -0.85, 0.0], [2.96, 0.3, 2.96], BRICK),
    part([0.0, -0.2, 0.0], [2.6, 1.0, 2.6], WOOD),
    part([0.0, 0.4, 0.0], [2.9, 0.2, 2.9], ROOF),
    part([0.0, 0.6, 0.0], [2.2, 0.2, 2.2], ROOF),
    part([0.0, 0.8, 0.0], [1.4, 0.2, 1.4], ROOF),
    part([0.0, -0.35, 1.32], [0.6, 0.7, 0.04], Look::Tex([tex::FRAME; 3])),
    part([-0.85, -0.1, 1.32], [0.45, 0.35, 0.04], PANE),
    part([0.85, -0.1, 1.32], [0.45, 0.35, 0.04], PANE),
];
