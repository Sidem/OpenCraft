//! The swarm hub (Milestone 11, the Drone Swarms tech): a 2×2×2 data consumer that lets the drone ports around it keep
//! more drones, as long as it gets its compute.
//!
//! - It always wants `HUB_TF` of its data grid and `HUB_KW` of power (`Energy::Optimizer`, the booster drive it shares
//!   with the optimizer: a power sink that runs at its power share times its grid's satisfaction).
//! - Its effect is `BONUS_PERMILLE` (+50%) of a port's fleet at full power and compute, for every drone port whose anchor
//!   is within `HUB_RANGE` blocks of it. Hubs do not stack: a port takes the best one in range (`bonus_at`). The extra is
//!   `Hangar::bonus`, derived each tick in `Factory::update` and never saved: `Processor::fleet` adds it to the tier's
//!   fleet. Without compute the bonus is 0 and the fleet drops back; drones already home or out stay, and the port
//!   simply takes no more until it is under its fleet again.
//! - Like the optimizer, `bonuses` reads this tick's balance, never a derived `speed`.
//!
//! To make another hub: a spec row with `Energy::Optimizer` and its own numbers here.

use crate::block::{tex, SWARM_HUB};
use crate::math::IVec3;

use super::super::fibre::Data;
use super::super::footprint::Footprint;
use super::super::pole::dist2;
use super::super::power::{Power, FULL_SPEED};
use super::model::{part, Look, Part};
use super::{Energy, Pick, ProcessSpec, ProcessTier, Processor, Status};

/// TF it uses.
pub const HUB_TF: i32 = 30;
/// What it draws on its power grid, in kW.
pub const HUB_KW: u32 = 100;
/// Ports this close (blocks between anchors) keep more drones.
pub const HUB_RANGE: i32 = 12;
/// The share of a port's fleet it adds at full power and compute, in thousandths.
pub const BONUS_PERMILLE: u32 = 500;

pub const HUB_SPEC: ProcessSpec = ProcessSpec {
    block: SWARM_HUB,
    categories: &[],
    pick: Pick::ByInput,
    buffers: [0, 0, 0],
    side: 0,
    tiers: &[ProcessTier { energy: Energy::Optimizer, speed: 1000, fuel: 0, power: HUB_KW }],
    footprint: Footprint { size: [2, 2, 2], ports: &[] },
    verb: "Coordinating",
    products: "drones",
    waiting: "Idle",
    map_colour: 0x5fd8ff,
    compute: -HUB_TF,
    parts: &PARTS,
};

impl Processor {
    /// The status line of a swarm hub (`None`: another processor).
    pub(super) fn hub_text(&self) -> Option<String> {
        if self.spec.block != SWARM_HUB {
            return None;
        }
        Some(match self.status {
            Status::Working => format!(
                "Drone ports within {HUB_RANGE} blocks keep {}% more drones{}",
                BONUS_PERMILLE * self.speed / FULL_SPEED / 10,
                if self.speed < FULL_SPEED { " (low power or compute)" } else { "" }
            ),
            _ => "No power or no compute: hang it on a pole and a fibre node whose grid has a datacenter".to_string(),
        })
    }
}

/// Every hub's anchor and the share of a fleet it adds right now, in thousandths (from this tick's balance).
pub(in crate::factory) fn bonuses(processors: &[Processor], power: &Power, data: &Data) -> Vec<(IVec3, u32)> {
    let mut out = Vec::new();
    for (i, p) in processors.iter().enumerate().filter(|(_, p)| p.spec.block == SWARM_HUB) {
        let share = power.speed(power.process_pole[i]) as u64 * data.satisfaction(data.process_node[i]) as u64;
        out.push((p.pos, (BONUS_PERMILLE as u64 * share / (FULL_SPEED as u64 * FULL_SPEED as u64)) as u32));
    }
    out
}

/// The share of its fleet, in thousandths (0: none), the best hub of `bonuses` in range adds to a port at `pos`.
pub(in crate::factory) fn bonus_at(bonuses: &[(IVec3, u32)], pos: IVec3) -> u32 {
    let near = bonuses.iter().filter(|(at, _)| dist2(*at, pos) <= HUB_RANGE * HUB_RANGE);
    near.map(|&(_, extra)| extra).max().unwrap_or(0)
}

/// A banded plinth, a dark body, a wide landing pad overhanging it on a short neck with two drones parked on it, a
/// corner mast with a small dish, and a status lamp.
const PARTS: [Part; 9] = [
    part([0.0, -0.9, 0.0], [1.96, 0.2, 1.96], Look::Band(tex::FRAME)),
    part([0.0, -0.3, 0.0], [1.9, 1.0, 1.9], Look::Tex([tex::HUB_TOP, tex::HUB_SIDE, tex::FRAME])),
    part([0.0, 0.3, 0.0], [1.2, 0.2, 1.2], Look::Tex([tex::FRAME; 3])),
    part([0.0, 0.45, 0.0], [2.2, 0.1, 2.2], Look::Tex([tex::HUB_TOP, tex::FRAME, tex::FRAME])),
    part([-0.45, 0.56, 0.35], [0.36, 0.12, 0.36], Look::Tex([tex::HUB_SIDE; 3])),
    part([0.45, 0.56, -0.3], [0.36, 0.12, 0.36], Look::Tex([tex::HUB_SIDE; 3])),
    part([-0.95, 0.85, -0.95], [0.08, 0.8, 0.08], Look::Tex([tex::FRAME; 3])),
    part([-0.95, 1.2, -0.75], [0.36, 0.36, 0.05], Look::Tex([tex::HUB_TOP; 3])),
    part([0.6, -0.3, 0.97], [0.14, 0.1, 0.04], Look::Lamp),
];

#[cfg(test)]
mod tests;
