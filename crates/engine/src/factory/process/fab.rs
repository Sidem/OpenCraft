//! The chip fab (Milestone 11, the Wafers and Accelerators techs): a 4×4×3 clean room, an ordinary recipe machine
//! (`Pick::ByInput`, `Category::Fabrication`, 1 MW). It takes what either recipe uses, so one fab does both jobs:
//! - **Wafer** (8 s): 2 silicon, an acid canister and a pure water canister; the two shells come back through the
//!   side hatches (the pure water is filled in a chemical plant: `recipes/machine.rs`).
//! - **AI accelerator** (20 s): 2 wafers, 2 processors and a plastic, so the player belts wafers back in.
//!
//! Six kinds of input share its six input slots (a stack each); `fab_text` names what a half-fed batch still lacks.
//! Numbers: docs/TECH_ERAS.md section 7.
//!
//! To make another fab product: a `Fabrication` recipe row.

use crate::block::{tex, CHIP_FAB};
use crate::item;
use crate::recipes::{Category, MachineRecipe, MACHINE_RECIPES};

use super::super::footprint::{Footprint, Port, Role, Side, Which};
use super::model::{part, Look, Part};
use super::{Energy, Pick, ProcessSpec, ProcessTier, Processor, Status};

#[cfg(test)]
mod tests;

pub const FAB_SPEC: ProcessSpec = ProcessSpec {
    block: CHIP_FAB,
    categories: &[Category::Fabrication],
    pick: Pick::ByInput,
    // Silicon, acid, pure water, wafers, processors and plastic: one slot for each.
    buffers: [6, 0, 1],
    side: 1,
    tiers: &[ProcessTier { energy: Energy::Electric, speed: 1000, fuel: 0, power: 1000 }],
    footprint: Footprint {
        size: [4, 4, 3],
        ports: &[
            Port { side: Side::Back, role: Role::In, cell: Which::All },
            Port { side: Side::Front, role: Role::Out, cell: Which::All },
            Port { side: Side::Right, role: Role::Side, cell: Which::All },
        ],
    },
    verb: "Etching",
    products: "wafers or accelerators",
    waiting: "Waiting for silicon, acid and pure water (wafers), or wafers, processors and plastic (accelerators)",
    map_colour: 0x7a6ad0,
    parts: &FAB_PARTS,
};

impl Processor {
    /// What a chip fab that holds part of a batch still lacks ("Waiting for 1 Acid Canister"); `None` for another
    /// machine, one that is working, or an empty fab (which says what it takes).
    pub(super) fn fab_text(&self) -> Option<String> {
        if self.spec.block != CHIP_FAB || self.status != Status::NoInput {
            return None;
        }
        // The recipe it is furthest into: the most inputs fully present, among those it holds anything of.
        let done = |r: &MachineRecipe| r.inputs.iter().filter(|&&(i, n)| self.input.count(i) >= n).count();
        let touched = |r: &&MachineRecipe| r.inputs.iter().any(|&(i, _)| self.input.count(i) > 0);
        let fab = MACHINE_RECIPES.iter().filter(|r| r.category == Category::Fabrication).filter(touched);
        let best = fab
            .fold(None, |b: Option<&MachineRecipe>, r| if b.is_none_or(|b| done(r) > done(b)) { Some(r) } else { b });
        let lacking: Vec<String> = best?
            .inputs
            .iter()
            .filter(|&&(i, n)| self.input.count(i) < n)
            .map(|&(i, n)| format!("{} {}", n - self.input.count(i), item::name(i)))
            .collect();
        (!lacking.is_empty()).then(|| format!("Waiting for {}", lacking.join(", ")))
    }
}

const BODY: [u16; 3] = [tex::FAB_TOP, tex::FAB_SIDE, tex::FRAME];
const FRAME: Look = Look::Tex([tex::FRAME; 3]);
const STEEL: Look = Look::Tex([tex::STEEL; 3]);

/// Across and deep 4, 3 high (±2, ±2, ±1.5): a banded plinth, the clean room (a low windowless hall), a roof deck with
/// three fan-filter units and an air duct behind them, a dark airlock door and a status lamp.
const FAB_PARTS: [Part; 10] = [
    part([0.0, -1.4, 0.0], [3.96, 0.2, 3.96], Look::Band(tex::FRAME)),
    part([0.0, -0.35, -0.1], [3.8, 1.9, 3.6], Look::Tex(BODY)),
    part([0.0, 0.65, -0.1], [3.9, 0.1, 3.7], FRAME),
    part([-1.2, 1.0, -0.4], [1.0, 0.6, 1.0], Look::Tex(BODY)),
    part([0.0, 1.0, -0.4], [1.0, 0.6, 1.0], Look::Tex(BODY)),
    part([1.2, 1.0, -0.4], [1.0, 0.6, 1.0], Look::Tex(BODY)),
    part([0.0, 1.3, -1.5], [3.4, 0.3, 0.4], STEEL),
    part([0.0, -0.75, 1.74], [1.3, 1.3, 0.12], Look::Tex([tex::GENERATOR_SIDE; 3])),
    part([0.0, -0.05, 1.74], [1.5, 0.1, 0.14], FRAME),
    part([1.6, 0.3, 1.74], [0.16, 0.12, 0.14], Look::Lamp),
];
