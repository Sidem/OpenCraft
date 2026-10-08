//! The cooling tower (Milestone 11, the Cooling tech): a 3×3×5 fan tower that closes a datacenter's water loop.
//!
//! - It draws `TOWER_KW` whenever it hangs on a pole, and has a water inlet (`Role::Water`) like a datacenter: a pipe
//!   from a pump keeps its small tank (`steam.water`, `draw_water`) at a few units, the tower's makeup water.
//! - A datacenter on one of its water networks (`seat`, derived at every relink: at most `TOWER_SERVES` to a tower,
//!   lower processor indices first) takes its coolant from the tower instead of the pump while the tower is powered
//!   and has water: the hot water goes to the tower and comes back cool (`circulate`), and the tower loses only one
//!   unit in `LOOP_LOSS` that way, so the pump's pool is drained at a twentieth of the open-loop rate.
//! - Without power or water the datacenter simply falls back to the pump, as before.
//! - Saved like any water-using processor: the tank is `steam.water`, the circulation count is `progress`. The seats
//!   (`steam.tower`, and the tower's `steam.boilers`: its datacenters) and `steam.looped` are derived, never saved.
//!
//! To add a second kind of tower: a spec row with `Energy::Cooling`; the pairing takes any.

use crate::block::{tex, COOLING_TOWER};

use super::super::footprint::{Footprint, Port, Role, Side, Which};
use super::super::power::NOT_WIRED;
use super::model::{part, Look, Part};
use super::{Energy, Pick, ProcessSpec, ProcessTier, Processor, Status};

/// What a tower's fans draw on its grid, in kW.
pub const TOWER_KW: u32 = 300;
/// Datacenters one tower cools.
pub const TOWER_SERVES: usize = 2;
/// Of this many units a datacenter takes through the loop, one is lost (to evaporation) from the tower's tank.
pub const LOOP_LOSS: u32 = 20;

pub const TOWER_SPEC: ProcessSpec = ProcessSpec {
    block: COOLING_TOWER,
    categories: &[],
    pick: Pick::ByInput,
    buffers: [0, 0, 0],
    side: 0,
    tiers: &[ProcessTier { energy: Energy::Cooling, speed: 1000, fuel: 0, power: TOWER_KW }],
    footprint: Footprint {
        size: [3, 3, 5],
        ports: &[Port { side: Side::Left, role: Role::Water, cell: Which::Nth(1) }],
    },
    verb: "Cooling",
    products: "water",
    waiting: "Idle",
    map_colour: 0x8aa0b4,
    compute: 0,
    parts: &PARTS,
};

impl Processor {
    /// The status line of a cooling tower (`None`: another processor).
    pub(super) fn tower_text(&self) -> Option<String> {
        if self.energy() != Energy::Cooling {
            return None;
        }
        let n = self.steam.boilers.len();
        Some(match self.status {
            Status::Working if self.steam.water == 0 => {
                "Out of makeup water: pipe its blue inlet to a pump with water in reach".to_string()
            }
            Status::Working if n == 0 => {
                "No datacenter: pipe its blue inlet to the same pipes as a datacenter's inlet".to_string()
            }
            Status::Working => {
                format!("Cooling {n} of {TOWER_SERVES} datacenters · {} units of makeup water", self.steam.water)
            }
            _ => NOT_WIRED.to_string(),
        })
    }

    /// Whether a datacenter can draw its coolant through this tower now: powered and holding makeup water.
    pub(super) fn tower_ready(&self) -> bool {
        self.energy() == Energy::Cooling && self.status == Status::Working && self.steam.water > 0
    }
}

/// Gives datacenter `d` a unit of coolant (`unit` of its tank's measure) through tower `t`, which loses one unit of
/// its own water in every `LOOP_LOSS` circulated.
pub(super) fn circulate(processors: &mut [Processor], d: usize, t: usize, unit: u32) {
    processors[d].steam.water += unit;
    let tower = &mut processors[t];
    tower.progress += 1;
    if tower.progress >= LOOP_LOSS {
        tower.progress = 0;
        tower.steam.water -= 1;
    }
}

/// Pairs every datacenter with a tower on one of its water networks (`link`, after the networks are known).
pub(super) fn seat(processors: &mut [Processor]) {
    let towers: Vec<usize> = (0..processors.len()).filter(|&i| processors[i].energy() == Energy::Cooling).collect();
    for d in 0..processors.len() {
        if processors[d].energy() != Energy::Compute {
            continue;
        }
        let shares = |t: usize| processors[t].steam.nets.iter().any(|n| processors[d].steam.nets.contains(n));
        let found = towers.iter().copied().find(|&t| processors[t].steam.boilers.len() < TOWER_SERVES && shares(t));
        processors[d].steam.tower = found.map(|t| t as u32);
        if let Some(t) = found {
            processors[t].steam.boilers.push(d as u32);
        }
    }
}

/// Across and deep 3, 5 high (±1.5, ±1.5, ±2.5): a banded plinth, a louvred body, a narrower upper stage, a basin
/// ring, a fan deck, a blue water stub on the left and a status lamp.
const PARTS: [Part; 7] = [
    part([0.0, -2.4, 0.0], [2.96, 0.2, 2.96], Look::Band(tex::FRAME)),
    part([0.0, -1.2, 0.0], [2.8, 2.2, 2.8], Look::Tex([tex::TOWER_TOP, tex::TOWER_SIDE, tex::FRAME])),
    part([0.0, 0.85, 0.0], [2.5, 1.9, 2.5], Look::Tex([tex::TOWER_TOP, tex::TOWER_SIDE, tex::FRAME])),
    part([0.0, 2.05, 0.0], [2.9, 0.6, 2.9], Look::Tex([tex::FRAME; 3])),
    part([0.0, 2.4, 0.0], [2.0, 0.2, 2.0], Look::Tex([tex::TOWER_TOP; 3])),
    part([-1.46, -2.0, 0.0], [0.2, 0.5, 0.5], Look::Tex([tex::PIPE_WATER; 3])),
    part([1.2, -1.8, 1.44], [0.16, 0.12, 0.14], Look::Lamp),
];

#[cfg(test)]
mod tests;
