//! The hoist (Milestone 10): a stack of hoist shaft blocks (`block::HOIST`, climbable like a ladder, no machine) and a
//! winch (a 1×1×1 processor, a power sink) beside the shaft's top cell. A body in a shaft whose winch is powered is
//! carried at `HOIST_SPEED` times the winch's power share (never slower than a ladder); with the winch dark or short
//! of power the shaft is just a ladder. The winch draws `WINCH_KW` whenever it hangs on a pole, used or not.
//!
//! Invariants: the core only holds the winch's grid share (`Processor::speed`, derived); `Factory::hoist_rate` is a
//! query over the world's blocks and that share, called by `authority.rs` each tick to set `Player::hoist` on bodies
//! (bodies are not core). Nothing here is saved.
//!
//! To add a second kind of winch: a spec row like this and its block; the lookup takes any `Energy::Hoist` processor.

use crate::block::{tex, BlockId, HOIST, WINCH};
use crate::math::{IVec3, Vec3};

use super::super::footprint::SINGLE;
use super::super::links::Slot;
use super::super::Factory;
use super::model::{drum_x, drum_y, part, Look, Part};
use super::{Energy, Pick, ProcessSpec, ProcessTier, Processor, Status};

/// Blocks a second at full power (a ladder is 3).
pub const HOIST_SPEED: f64 = 9.0;
/// What a winch draws on its grid, in kW.
pub const WINCH_KW: u32 = 20;
/// Shaft cells searched above a body for the top.
const MAX_SHAFT: i32 = 256;

pub const WINCH_SPEC: ProcessSpec = ProcessSpec {
    block: WINCH,
    categories: &[],
    pick: Pick::ByInput,
    buffers: [0, 0, 0],
    side: 0,
    tiers: &[ProcessTier { energy: Energy::Hoist, speed: 1000, fuel: 0, power: WINCH_KW }],
    footprint: SINGLE,
    verb: "Hoisting",
    products: "riders",
    waiting: "Idle",
    map_colour: 0xe2b428,
    compute: 0,
    parts: &WINCH_PARTS,
};

/// A banded plinth, two side cheeks carrying a cable drum lying across (two crossed parts), the cable running up
/// off it, a copper motor behind and a status lamp.
const WINCH_PARTS: [Part; 8] = [
    part([0.0, -0.4, 0.0], [0.96, 0.2, 0.96], Look::Band(tex::FRAME)),
    part([-0.38, 0.0, 0.0], [0.08, 0.6, 0.7], Look::Tex([tex::FRAME, tex::WINCH_SIDE, tex::FRAME])),
    part([0.38, 0.0, 0.0], [0.08, 0.6, 0.7], Look::Tex([tex::FRAME, tex::WINCH_SIDE, tex::FRAME])),
    drum_x([0.0, 0.05, 0.0], [0.64, 0.5, 0.5], Look::Tex([tex::FRAME, tex::GEAR, tex::FRAME])),
    drum_y([0.0, 0.05, 0.0], [0.64, 0.5, 0.5], Look::Tex([tex::FRAME, tex::GEAR, tex::FRAME])),
    part([0.1, 0.4, 0.24], [0.05, 0.6, 0.05], Look::Tex([tex::FRAME; 3])),
    part([0.0, -0.15, -0.36], [0.4, 0.3, 0.2], Look::Tex([tex::COPPER_INGOT; 3])),
    part([0.38, 0.34, 0.3], [0.1, 0.08, 0.1], Look::Lamp),
];

impl Processor {
    /// The status line of a winch (`None`: another processor).
    pub(super) fn hoist_text(&self) -> Option<String> {
        if self.energy() != Energy::Hoist {
            return None;
        }
        Some(match self.status {
            Status::Working => format!("Shaft running at {} blocks a second", self.speed * HOIST_SPEED as u32 / 1000),
            _ => "No power: the shaft is only a ladder (hang the winch on a pole)".to_string(),
        })
    }
}

impl Factory {
    /// Blocks a second a body with its feet at `feet` is carried by a powered hoist (0: not in a shaft, or the shaft
    /// has no winch beside its top cell, or the winch has no power). `block` reads the world's blocks.
    pub fn hoist_rate(&self, feet: Vec3, block: &mut impl FnMut(i32, i32, i32) -> BlockId) -> f64 {
        let base = feet.floor();
        let mut cell = [base, base + IVec3::new(0, 1, 0)]
            .into_iter()
            .find(|c| block(c.x, c.y, c.z) == HOIST)
            .unwrap_or(IVec3::new(0, i32::MIN, 0));
        if cell.y == i32::MIN {
            return 0.0;
        }
        for _ in 0..MAX_SHAFT {
            if block(cell.x, cell.y + 1, cell.z) != HOIST {
                break;
            }
            cell.y += 1;
        }
        let around = [(0, 1, 0), (1, 0, 0), (-1, 0, 0), (0, 0, 1), (0, 0, -1)];
        around
            .into_iter()
            .filter_map(|(dx, dy, dz)| match self.at.get(&(cell + IVec3::new(dx, dy, dz))) {
                Some(&Slot::Process(i)) => Some(&self.processors[i as usize]),
                _ => None,
            })
            .filter(|p| p.energy() == Energy::Hoist)
            .map(|p| HOIST_SPEED * p.speed as f64 / 1000.0)
            .fold(0.0, f64::max)
    }
}

#[cfg(test)]
mod tests;
