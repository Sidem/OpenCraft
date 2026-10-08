//! The AI datacenter (Milestone 11, the Datacenters tech): a 4×4×3 hall that turns power and coolant into compute.
//!
//! - It draws `DATACENTER_KW` (3 MW) from its power grid at all times (a brownout slows it) and, hung on a fibre node
//!   (`fibre.rs`), gives the data grid `DATACENTER_TF` TF times the power share it gets: its spec's `compute`.
//! - **Heat** is the reactor's model (`nuclear.rs`): it heats by the kW it draws and sheds that much while it has
//!   coolant, a pipe from a pump to its blue inlet (`Role::Water`; `steam.rs` links it and `draw_water` fills its tank,
//!   a unit per 10 s of full load). Without coolant it only sheds `PASSIVE_KW`, so it trips at `HEAT_LIMIT` (20 s of
//!   full load): `Status::Overheated`, no power drawn and no compute given, until the heat has halved.
//! - Saved like any processor: the heat is `progress` (a datacenter has no batches) and the coolant tank is `steam.water`
//!   (it has a water inlet), so nothing new in the save. Hand recipe: `recipes/compute.rs`.
//!
//! A cooling tower on its water network closes the loop (`tower.rs`): the water comes back cool and only a trickle is lost.
//!
//! Not yet: Mk tiers with more racks.

use crate::block::{tex, DATACENTER};
use crate::TICK_RATE;

use super::super::footprint::{Footprint, Port, Role, Side, Which};
use super::super::power::NOT_WIRED;
use super::model::{part, Look, Part};
use super::{Energy, Pick, ProcessSpec, ProcessTier, Processor, Status};

/// What a datacenter draws at full load, in kW.
pub const DATACENTER_KW: u32 = 3_000;
/// The compute it gives at full power, in TF.
pub const DATACENTER_TF: i32 = 100;
/// One pumped unit of water cools 10 s of full load; the tank asks for another below two units.
pub(super) const COOLANT_UNIT: u32 = DATACENTER_KW * TICK_RATE * 10;
pub(super) const COOLANT_LOW: u32 = COOLANT_UNIT * 2;
/// Heat (kW·ticks) at which it shuts down: 20 s of full load without coolant.
const HEAT_LIMIT: u32 = DATACENTER_KW * TICK_RATE * 20;
/// Heat shed a tick with coolant, and without (kW).
const COOLED_KW: u32 = DATACENTER_KW;
const PASSIVE_KW: u32 = DATACENTER_KW / 20;

pub const DATACENTER_SPEC: ProcessSpec = ProcessSpec {
    block: DATACENTER,
    categories: &[],
    pick: Pick::ByInput,
    buffers: [0, 0, 0],
    side: 0,
    tiers: &[ProcessTier { energy: Energy::Compute, speed: 1000, fuel: 0, power: DATACENTER_KW }],
    footprint: Footprint {
        size: [4, 4, 3],
        ports: &[Port { side: Side::Left, role: Role::Water, cell: Which::Nth(1) }],
    },
    verb: "Computing",
    products: "compute",
    waiting: "Idle",
    map_colour: 0x4ad0e0,
    compute: DATACENTER_TF,
    parts: &PARTS,
};

impl Processor {
    /// The status line of a datacenter (`None`: another processor).
    pub(super) fn datacenter_text(&self) -> Option<String> {
        if self.energy() != Energy::Compute {
            return None;
        }
        let heat = self.progress as u64 * 100 / HEAT_LIMIT as u64;
        let tf = DATACENTER_TF as u32 * self.speed / 1000;
        Some(match self.status {
            Status::Working if self.steam.water == 0 => {
                format!("Making {tf} of {DATACENTER_TF} TF · heat {heat}% and rising: pipe water to its blue inlet")
            }
            Status::Working if self.steam.looped => {
                format!("Making {tf} of {DATACENTER_TF} TF · heat {heat}% · cooling loop closed")
            }
            Status::Working => format!("Making {tf} of {DATACENTER_TF} TF · heat {heat}%"),
            Status::Overheated => {
                format!("Overheated and shut down, cooling ({heat}% heat): pipe water to its blue inlet")
            }
            _ => NOT_WIRED.to_string(),
        })
    }

    /// Whether it draws power this tick: always, unless it has shut down.
    pub(super) fn datacenter_wants_power(&self) -> bool {
        self.energy() == Energy::Compute && self.status != Status::Overheated
    }

    /// One tick at power share `power` (thousandths): heats by the load, sheds heat to its coolant, trips at the limit.
    pub(super) fn run_datacenter(&mut self, power: u32) {
        let tripped = scrammed(self.status == Status::Overheated, self.progress);
        let load = if tripped { 0 } else { DATACENTER_KW * power / 1000 };
        let cooled = self.steam.water >= load.max(1);
        if cooled {
            self.steam.water -= load;
        }
        self.progress = (self.progress + load).saturating_sub(if cooled { COOLED_KW } else { PASSIVE_KW });
        self.status = match () {
            _ if scrammed(tripped, self.progress) => Status::Overheated,
            _ if power > 0 => Status::Working,
            _ => Status::NoPower,
        };
    }
}

/// Whether a datacenter with `heat` is shut down: it trips at the limit and restarts once the heat has halved.
fn scrammed(was: bool, heat: u32) -> bool {
    if was {
        heat > HEAT_LIMIT / 2
    } else {
        heat >= HEAT_LIMIT
    }
}

const BODY: [u16; 3] = [tex::DC_TOP, tex::DC_SIDE, tex::FRAME];
const FRAME: Look = Look::Tex([tex::FRAME; 3]);
const STEEL: Look = Look::Tex([tex::STEEL; 3]);

/// Across and deep 4, 3 high (±2, ±2, ±1.5): a banded plinth, the server hall, its roof deck with two cooling fan
/// units and a steel manifold, a blue water stub on the left, a dark door on the front and a status lamp.
const PARTS: [Part; 9] = [
    part([0.0, -1.4, 0.0], [3.96, 0.2, 3.96], Look::Band(tex::FRAME)),
    part([0.0, -0.3, 0.0], [3.8, 2.0, 3.8], Look::Tex(BODY)),
    part([0.0, 0.78, 0.0], [3.9, 0.12, 3.9], FRAME),
    part([-1.0, 1.1, -0.6], [1.3, 0.5, 1.3], Look::Tex([tex::DC_TOP; 3])),
    part([1.0, 1.1, -0.6], [1.3, 0.5, 1.3], Look::Tex([tex::DC_TOP; 3])),
    part([0.0, 1.05, 1.1], [3.2, 0.3, 0.5], STEEL),
    part([-1.96, -0.3, -0.5], [0.2, 0.5, 0.5], Look::Tex([tex::PIPE_WATER; 3])),
    part([0.0, -0.6, 1.92], [1.6, 1.4, 0.1], Look::Tex([tex::DC_SIDE; 3])),
    part([1.6, 0.3, 1.94], [0.16, 0.12, 0.14], Look::Lamp),
];

#[cfg(test)]
mod tests;
