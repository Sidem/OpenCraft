//! The town depot (step 13.1, tech Settlements) as a spec row: a 3×3×2 warehouse that belts on any side fill. It holds
//! what it is given like a silo (`Pick::Depot`), but only what its town wants (`Processor::wants`, set each tick by
//! `Sim::sync_depots`); belts never empty it, its town uses it up (`settlements/run.rs`, through `depot_store`). Placed
//! at a fitting site it founds a town (`settlements/`, checked in `action/multiblock.rs`).
//! The look is a placeholder from existing textures (`docs/ART_HANDOVER.md` has the request).

use crate::block::{tex, TOWN_DEPOT};
use crate::item::ItemId;
use crate::math::IVec3;

use super::super::buffer::Buffer;
use super::super::footprint::{Footprint, Port, Role, Side, Which};
use super::super::Factory;
use super::model::{part, Look, Part};
use super::specs::{Energy, Pick, ProcessSpec, ProcessTier};

/// Stacks a depot holds.
const SLOTS: usize = 24;

const IN: Role = Role::In;

pub const DEPOT_SPEC: ProcessSpec = ProcessSpec {
    block: TOWN_DEPOT,
    categories: &[],
    pick: Pick::Depot,
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

impl Factory {
    /// The store of the depot anchored at `pos`, for its town to use up; `None` when no depot stands there.
    pub fn depot_store(&mut self, pos: IVec3) -> Option<&mut Buffer> {
        let p = self.processors.iter_mut().find(|p| p.pos == pos && p.spec.block == TOWN_DEPOT)?;
        Some(&mut p.out)
    }

    /// What the depot anchored at `pos` holds, for the readout.
    pub fn depot_held(&self, pos: IVec3) -> Option<&Buffer> {
        let p = self.processors.iter().find(|p| p.pos == pos && p.spec.block == TOWN_DEPOT)?;
        Some(&p.out)
    }

    /// Sets what the depot anchored at `pos` takes in (its town's goods) and up to how many it holds of each.
    pub fn set_depot_wants(&mut self, pos: IVec3, wants: Vec<(ItemId, u32)>) {
        if let Some(p) = self.processors.iter_mut().find(|p| p.pos == pos && p.spec.block == TOWN_DEPOT) {
            p.wants = wants;
        }
    }
}

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
