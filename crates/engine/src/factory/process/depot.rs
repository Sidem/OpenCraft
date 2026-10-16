//! The town depot (step 13.1, tech Settlements) as a spec row: a 3×3×2 warehouse that belts on its back and sides fill.
//! It holds what it is given like a silo (`Pick::Depot`), but only what its town wants (`Processor::wants`, set each
//! tick by `Sim::sync_depots`); belts never empty its store (`out`), its town uses it up (`settlements/run.rs`, through
//! `depot_store`). Placed at a fitting site it founds a town (`settlements/`, checked in `action/multiblock.rs`).
//!
//! The town's perk (step 13.4, `settlements/perks/`) has buffers of its own, so it never crowds the store: what the
//! perk takes in (a `Want` with `perk` set) waits in `input`, and what it gives waits in `side`, which belts take from
//! at the front (the byproduct port) and the player by hand. Saves before version 52 had neither.
//! The look is a placeholder from existing textures (`docs/ART_HANDOVER.md` has the request).

use crate::block::{tex, TOWN_DEPOT};
use crate::item::ItemId;
use crate::math::IVec3;

use super::super::buffer::Buffer;
use super::super::footprint::{Footprint, Port, Role, Side, Which};
use super::super::Factory;
use super::model::{part, Look, Part};
use super::specs::{Energy, Pick, ProcessSpec, ProcessTier};
use super::Processor;

/// Stacks a depot's store holds.
const SLOTS: usize = 24;
/// Stacks for what its perk takes in, and for what it gives.
const PERK_SLOTS: usize = 8;

const IN: Role = Role::In;

/// One good a depot takes in, up to `cap` held: for its town's store, or (`perk`) for its perk.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Want {
    pub item: ItemId,
    pub cap: u32,
    pub perk: bool,
}

pub const DEPOT_SPEC: ProcessSpec = ProcessSpec {
    block: TOWN_DEPOT,
    categories: &[],
    pick: Pick::Depot,
    buffers: [PERK_SLOTS, 0, SLOTS],
    side: PERK_SLOTS,
    tiers: &[ProcessTier { energy: Energy::Recipe, speed: 1000, fuel: 0, power: 0 }],
    footprint: Footprint {
        size: [3, 3, 2],
        ports: &[
            Port { side: Side::Front, role: Role::Side, cell: Which::All },
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

    /// What the perk of the depot anchored at `pos` has taken in, and where it puts what it gives.
    pub fn depot_perk(&mut self, pos: IVec3) -> Option<(&mut Buffer, &mut Buffer)> {
        let p = self.processors.iter_mut().find(|p| p.pos == pos && p.spec.block == TOWN_DEPOT)?;
        Some((&mut p.input, &mut p.side))
    }

    /// Sets what the depot anchored at `pos` takes in (its town's goods) and up to how many it holds of each.
    pub fn set_depot_wants(&mut self, pos: IVec3, wants: Vec<Want>) {
        if let Some(p) = self.processors.iter_mut().find(|p| p.pos == pos && p.spec.block == TOWN_DEPOT) {
            p.wants = wants;
        }
    }
}

impl Processor {
    /// How many of `item` a depot takes now: up to its want's cap, in the buffer the want names.
    pub(super) fn depot_room(&self, item: ItemId) -> u32 {
        let Some(w) = self.wants.iter().find(|w| w.item == item) else { return 0 };
        let buf = if w.perk { &self.input } else { &self.out };
        w.cap.saturating_sub(buf.count(item)).min(buf.space_for(item))
    }

    /// Whether `item` goes to the depot's perk (`input`) rather than its store.
    pub(super) fn for_perk(&self, item: ItemId) -> bool {
        self.wants.iter().any(|w| w.item == item && w.perk)
    }
}

/// Whether a saved processor of `spec` is a depot from before the perks (save 52), saved without their buffers.
pub(super) fn before_perks(spec: &ProcessSpec, version: u32) -> bool {
    spec.block == TOWN_DEPOT && version < 52
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
