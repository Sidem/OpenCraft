//! The AI lab (Milestone 11, the AI Labs tech): a research center on the data grid. It is the center's code
//! (`center.rs`: `Pick::Research`, eight pack kinds, belts on every side) with a different spec row:
//!
//! - one tier at twice a lab's speed and 100 kW, and every 2nd unit takes no packs (`free_every`);
//! - a data consumer (`compute` -`AI_LAB_TF`): it wants those TF while it researches, and works at its power share
//!   times its grid's satisfaction (`step_centers` reads `Data`), so with no node or no datacenter in the grid it
//!   does not work at all;
//! - the only lab that can research a tech that costs compute (`research::needs_ai_lab`): labs and centers report
//!   `LabStatus::NeedsAi` for those.
//!
//! To make a tech cost compute: add its index to `AI_ONLY` in `research/compute.rs`.

use crate::block::{tex, AI_LAB};

use super::super::footprint::{Footprint, Port, Role, Side, Which};
use super::center::CENTER_SLOTS;
use super::model::{part, Look, Part};
use super::{Energy, Pick, ProcessSpec, ProcessTier, Processor};

/// TF an AI lab uses while it researches.
pub const AI_LAB_TF: i32 = 20;
/// What an AI lab draws on its power grid, in kW.
pub const AI_LAB_KW: u32 = 100;
/// Every this many units the last one takes no packs.
const FREE_EVERY: u8 = 2;

const fn inlet(side: Side) -> Port {
    Port { side, role: Role::In, cell: Which::All }
}

pub const AI_LAB_SPEC: ProcessSpec = ProcessSpec {
    block: AI_LAB,
    categories: &[],
    pick: Pick::Research,
    buffers: [CENTER_SLOTS, 0, 0],
    side: 0,
    tiers: &[ProcessTier { energy: Energy::Electric, speed: 2000, fuel: 0, power: AI_LAB_KW }],
    footprint: Footprint {
        size: [2, 2, 2],
        ports: &[inlet(Side::Back), inlet(Side::Front), inlet(Side::Left), inlet(Side::Right)],
    },
    verb: "Researching",
    products: "research",
    waiting: "",
    map_colour: 0x6ae8ff,
    compute: -AI_LAB_TF,
    parts: &PARTS,
};

impl Processor {
    /// Whether it is an AI lab.
    pub fn is_ai_lab(&self) -> bool {
        self.spec.block == AI_LAB
    }

    /// Every this many units a center or AI lab makes one without packs (0: never).
    pub(super) fn free_every(&self, tiers: &[u8]) -> u8 {
        if self.is_ai_lab() {
            FREE_EVERY
        } else {
            tiers[self.tier as usize]
        }
    }
}

/// Across and deep 2, 2 high (±1): a banded plinth, a dark cabinet with a lit neural net, a stepped pyramid of two
/// tiers on it holding a big core that glows in the status colour (the AI lab's mark; the optimizer has a mast, the
/// swarm hub a landing pad), four copper heat pipes at the corners and a status lamp on the front.
const PARTS: [Part; 10] = [
    part([0.0, -0.9, 0.0], [1.96, 0.2, 1.96], Look::Band(tex::FRAME)),
    part([0.0, -0.25, 0.0], [1.9, 1.1, 1.9], Look::Tex([tex::AI_TOP, tex::AI_SIDE, tex::FRAME])),
    part([0.0, 0.45, 0.0], [1.4, 0.3, 1.4], Look::Tex([tex::AI_TOP, tex::AI_SIDE, tex::FRAME])),
    part([0.0, 0.72, 0.0], [0.9, 0.24, 0.9], Look::Tex([tex::AI_TOP, tex::AI_SIDE, tex::FRAME])),
    part([0.0, 1.05, 0.0], [0.5, 0.42, 0.5], Look::Lamp),
    part([-0.82, 0.5, -0.82], [0.1, 0.4, 0.1], Look::Tex([tex::COPPER_INGOT; 3])),
    part([0.82, 0.5, -0.82], [0.1, 0.4, 0.1], Look::Tex([tex::COPPER_INGOT; 3])),
    part([-0.82, 0.5, 0.82], [0.1, 0.4, 0.1], Look::Tex([tex::COPPER_INGOT; 3])),
    part([0.82, 0.5, 0.82], [0.1, 0.4, 0.1], Look::Tex([tex::COPPER_INGOT; 3])),
    part([0.6, -0.3, 0.97], [0.14, 0.1, 0.04], Look::Lamp),
];

#[cfg(test)]
mod tests;
