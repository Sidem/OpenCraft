//! The optimizer node (Milestone 11, the Optimizer tech): a 2×2×2 data consumer that makes the machines around it
//! faster, as long as it gets its compute.
//!
//! - It always wants `OPTIMIZER_TF` of its data grid and `OPTIMIZER_KW` of power (`Energy::Optimizer`, a power sink like
//!   the winch and the tower) and runs at its power share times its grid's satisfaction, so a datacenter shortage or a
//!   brownout weakens it smoothly instead of switching it off.
//! - Its effect is `BONUS_PERMILLE` (+25%) at full speed, for every processor and miner whose anchor is within
//!   `OPTIMIZER_RANGE` blocks of it, whatever grid they are on. Several optimizers do not stack: a machine takes the
//!   best one in range (`bonus_at`). The effect multiplies into the machine's `boost` in `Factory::update`, next to the
//!   research bonuses (`research/bonus.rs`).
//! - `bonuses` is computed from this tick's power and data balance, never from last tick's derived `speed`, so a loaded
//!   or resynced core behaves exactly like one that kept running. Nothing is saved.
//!
//! To make another booster: a spec row with `Energy::Optimizer` and its own numbers here.

use crate::block::{tex, OPTIMIZER};
use crate::math::IVec3;

use super::super::fibre::Data;
use super::super::footprint::Footprint;
use super::super::pole::dist2;
use super::super::power::{Power, FULL_SPEED};
use super::model::{part, Look, Part};
use super::{Energy, Pick, ProcessSpec, ProcessTier, Processor, Status};

/// TF it uses.
pub const OPTIMIZER_TF: i32 = 20;
/// What it draws on its power grid, in kW.
pub const OPTIMIZER_KW: u32 = 150;
/// Machines this close (blocks between anchors) are sped up.
pub const OPTIMIZER_RANGE: i32 = 16;
/// The speed it adds at full power and compute, in thousandths.
pub const BONUS_PERMILLE: u32 = 250;

pub const OPTIMIZER_SPEC: ProcessSpec = ProcessSpec {
    block: OPTIMIZER,
    categories: &[],
    pick: Pick::ByInput,
    buffers: [0, 0, 0],
    side: 0,
    tiers: &[ProcessTier { energy: Energy::Optimizer, speed: 1000, fuel: 0, power: OPTIMIZER_KW }],
    footprint: Footprint { size: [2, 2, 2], ports: &[] },
    verb: "Optimizing",
    products: "speed",
    waiting: "Idle",
    map_colour: 0xffb347,
    compute: -OPTIMIZER_TF,
    parts: &PARTS,
};

impl Processor {
    /// The status line of an optimizer (`None`: another processor).
    pub(super) fn optimizer_text(&self) -> Option<String> {
        if self.energy() != Energy::Optimizer {
            return None;
        }
        Some(match self.status {
            Status::Working => format!(
                "Machines within {OPTIMIZER_RANGE} blocks run {}% faster{}",
                BONUS_PERMILLE * self.speed / FULL_SPEED / 10,
                if self.speed < FULL_SPEED { " (low power or compute)" } else { "" }
            ),
            _ => "No power or no compute: hang it on a pole and a fibre node whose grid has a datacenter".to_string(),
        })
    }
}

/// Every optimizer's anchor and the speed it adds right now, in thousandths (from this tick's balance).
pub(in crate::factory) fn bonuses(processors: &[Processor], power: &Power, data: &Data) -> Vec<(IVec3, u32)> {
    let mut out = Vec::new();
    for (i, p) in processors.iter().enumerate().filter(|(_, p)| p.energy() == Energy::Optimizer) {
        let share = power.speed(power.process_pole[i]) as u64 * data.satisfaction(data.process_node[i]) as u64;
        out.push((p.pos, (BONUS_PERMILLE as u64 * share / (FULL_SPEED as u64 * FULL_SPEED as u64)) as u32));
    }
    out
}

/// The multiplier in thousandths (1000: none) the best optimizer of `bonuses` in range gives a machine at `pos`.
pub(in crate::factory) fn bonus_at(bonuses: &[(IVec3, u32)], pos: IVec3) -> u32 {
    let best = bonuses.iter().filter(|(at, _)| dist2(*at, pos) <= OPTIMIZER_RANGE * OPTIMIZER_RANGE);
    FULL_SPEED + best.map(|&(_, extra)| extra).max().unwrap_or(0)
}

/// Across and deep 2, 2 high (±1): a banded plinth, a dark cabinet, a crown ring, a tall mast with a lit tip and a status
/// lamp.
const PARTS: [Part; 6] = [
    part([0.0, -0.9, 0.0], [1.96, 0.2, 1.96], Look::Band(tex::FRAME)),
    part([0.0, -0.25, 0.0], [1.9, 1.1, 1.9], Look::Tex([tex::OPT_TOP, tex::OPT_SIDE, tex::FRAME])),
    part([0.0, 0.5, 0.0], [1.5, 0.4, 1.5], Look::Tex([tex::OPT_TOP, tex::OPT_SIDE, tex::FRAME])),
    part([0.0, 1.1, 0.0], [0.12, 0.8, 0.12], Look::Tex([tex::FRAME; 3])),
    part([0.0, 1.55, 0.0], [0.24, 0.24, 0.24], Look::Lamp),
    part([0.78, -0.3, 0.92], [0.14, 0.1, 0.14], Look::Lamp),
];

#[cfg(test)]
mod tests;
