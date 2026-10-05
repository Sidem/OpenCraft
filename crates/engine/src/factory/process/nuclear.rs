//! Nuclear power (Milestone 10): the centrifuge and the reactor.
//!
//! - The **centrifuge** is an ordinary recipe machine (`Pick::ByInput`, `Category::Enrichment`, 200 kW): 4 uranium ore
//!   and a steel plate make a fuel cell.
//! - The **reactor** is a power source like the diesel generator (`diesel.rs`) but water-cooled. Belts bring fuel cells
//!   in at the back; when its grid lacks power and what it holds can't cover this tick it lights one (`CELL_KJ`). It
//!   gives only what the grid lacks, up to `REACTOR_KW`. Whatever it gives heats it; a pipe from a pump to its blue
//!   inlet (`Role::Water`, `steam.rs` links it and `draw_water` fills its tank) cools it. Water is spent as the reactor
//!   gives (`COOLANT_UNIT` of output a unit); with water it sheds heat as fast as it makes it, without it only
//!   `PASSIVE_KW`. At `HEAT_LIMIT` it shuts down (`Status::Overheated`) and stays off until the heat has halved.
//! - Saved: the lit cell's energy (`Store::charge`), the coolant tank (`steam.water`, in kW·ticks of output) and the
//!   heat, kept in `progress` (a reactor has no batches). Power order (`Power::balance`): sun, hydro, reactor, coal
//!   generators, steam turbines, diesel.
//!
//! To add another water-cooled source: an `Energy` variant like this one's and a spec row.

use crate::block::{tex, CENTRIFUGE, REACTOR};
use crate::item::{ItemId, DIESEL_CANISTER, FUEL_CELL};
use crate::recipes::Category;
use crate::TICK_RATE;

use super::super::footprint::{Footprint, Port, Role, Side, Which};
use super::super::power::{Power, NOT_WIRED};
use super::model::{part, Look, Part};
use super::{Energy, Pick, ProcessSpec, ProcessTier, Processor, Status};

/// The most a reactor gives, in kW.
pub const REACTOR_KW: u32 = 2_000;
/// The energy of a fuel cell in kJ: 2 MW for 150 s.
pub const CELL_KJ: u32 = 300_000;
pub(super) const CELL_CHARGE: u32 = CELL_KJ * TICK_RATE;
/// One pumped unit of water cools 10 s of full output; the tank asks for another below two units.
pub(super) const COOLANT_UNIT: u32 = REACTOR_KW * TICK_RATE * 10;
pub(super) const COOLANT_LOW: u32 = COOLANT_UNIT * 2;
/// Heat (kW·ticks) at which the reactor shuts down: 20 s of full output without coolant.
const HEAT_LIMIT: u32 = REACTOR_KW * TICK_RATE * 20;
/// Heat shed a tick with coolant, and without (kW).
const COOLED_KW: u32 = REACTOR_KW;
const PASSIVE_KW: u32 = REACTOR_KW / 20;

const IN_BACK: Port = Port { side: Side::Back, role: Role::In, cell: Which::All };
const OUT_FRONT: Port = Port { side: Side::Front, role: Role::Out, cell: Which::All };

pub const CENTRIFUGE_SPEC: ProcessSpec = ProcessSpec {
    block: CENTRIFUGE,
    categories: &[Category::Enrichment],
    pick: Pick::ByInput,
    buffers: [2, 0, 1],
    side: 0,
    tiers: &[ProcessTier { energy: Energy::Electric, speed: 1000, fuel: 0, power: 200 }],
    footprint: Footprint { size: [2, 2, 3], ports: &[IN_BACK, OUT_FRONT] },
    verb: "Enriching",
    products: "fuel cells",
    waiting: "Waiting for uranium ore and steel plates",
    map_colour: 0x9ad84a,
    parts: &CENTRIFUGE_PARTS,
};

pub const REACTOR_SPEC: ProcessSpec = ProcessSpec {
    block: REACTOR,
    categories: &[],
    pick: Pick::ByInput,
    buffers: [1, 0, 0],
    side: 0,
    tiers: &[ProcessTier { energy: Energy::Reactor, speed: 1000, fuel: 0, power: 0 }],
    footprint: Footprint {
        size: [3, 3, 3],
        ports: &[IN_BACK, Port { side: Side::Left, role: Role::Water, cell: Which::Nth(1) }],
    },
    verb: "Generating",
    products: "power",
    waiting: "Idle",
    map_colour: 0x7ad84a,
    parts: &REACTOR_PARTS,
};

const STEEL: Look = Look::Tex([tex::STEEL; 3]);
const FRAME: Look = Look::Tex([tex::FRAME; 3]);
const CENTRIFUGE_BODY: [u16; 3] = [tex::CENTRIFUGE_TOP, tex::CENTRIFUGE_SIDE, tex::FRAME];
const REACTOR_BODY: [u16; 3] = [tex::REACTOR_TOP, tex::REACTOR_SIDE, tex::FRAME];

/// Across and deep 2, 3 high (±1.5): a banded plinth, a tall drum, its lid, a motor housing on top and a status lamp.
const CENTRIFUGE_PARTS: [Part; 5] = [
    part([0.0, -1.4, 0.0], [1.96, 0.2, 1.96], Look::Band(tex::FRAME)),
    part([0.0, -0.1, 0.0], [1.5, 2.4, 1.5], Look::Tex(CENTRIFUGE_BODY)),
    part([0.0, 1.2, 0.0], [1.7, 0.15, 1.7], FRAME),
    part([0.0, 1.38, 0.0], [0.6, 0.2, 0.6], STEEL),
    part([0.78, -1.2, 0.92], [0.14, 0.1, 0.14], Look::Lamp),
];

/// Across, deep and high 3 (±1.5): a banded plinth, the concrete containment with a stepped lid, a glowing window on
/// the front, a steel stack and a status lamp.
const REACTOR_PARTS: [Part; 7] = [
    part([0.0, -1.4, 0.0], [2.96, 0.2, 2.96], Look::Band(tex::FRAME)),
    part([0.0, -0.2, 0.0], [2.4, 2.4, 2.4], Look::Tex(REACTOR_BODY)),
    part([0.0, 1.15, 0.0], [1.6, 0.3, 1.6], Look::Tex([tex::REACTOR_TOP; 3])),
    part([0.0, 0.0, 1.22], [1.0, 0.5, 0.06], Look::Tex([tex::FUEL_CELL; 3])),
    part([0.7, 1.05, -0.7], [0.4, 0.8, 0.4], STEEL),
    part([-0.7, 1.05, -0.7], [0.4, 0.8, 0.4], FRAME),
    part([1.2, -1.2, 1.35], [0.14, 0.1, 0.14], Look::Lamp),
];

impl Processor {
    /// The one item a generator that burns items takes: diesel canisters, or fuel cells.
    pub(super) fn generator_fuel(&self) -> ItemId {
        if self.energy() == Energy::Reactor {
            FUEL_CELL
        } else {
            DIESEL_CANISTER
        }
    }

    /// The status line of a reactor (`None`: another processor).
    pub(super) fn reactor_text(&self) -> Option<String> {
        if self.energy() != Energy::Reactor {
            return None;
        }
        let heat = self.progress as u64 * 100 / HEAT_LIMIT as u64;
        Some(match self.status {
            Status::Working if self.steam.water == 0 => {
                format!(
                    "Giving {} of {REACTOR_KW} kW · heat {heat}% and rising: pipe water to its blue inlet",
                    self.store.given
                )
            }
            Status::Working => format!("Giving {} of {REACTOR_KW} kW · heat {heat}%", self.store.given),
            Status::Overheated => {
                format!("Overheated and shut down, cooling ({heat}% heat): pipe water to its blue inlet")
            }
            Status::NoFuel => "Out of fuel: bring fuel cells".to_string(),
            Status::NoPower => NOT_WIRED.to_string(),
            _ => format!("Standing by: the grid wants no more power · heat {heat}%"),
        })
    }
}

/// Whether a reactor with `heat` is shut down: it trips at the limit and restarts once the heat has halved.
fn scrammed(was: bool, heat: u32) -> bool {
    if was {
        heat > HEAT_LIMIT / 2
    } else {
        heat >= HEAT_LIMIT
    }
}

/// One tick of every reactor on a grid: each gives what its grid still lacks, lighting a fuel cell when it must, heats by
/// what it gives and sheds heat to its coolant. Called by `Power::balance` after the sun and the water wheels.
pub(in crate::factory) fn run(power: &mut Power, processors: &mut [Processor]) {
    for (i, p) in processors.iter_mut().enumerate().filter(|(_, p)| p.energy() == Energy::Reactor) {
        p.store.given = 0;
        let Some(pole) = power.process_pole[i] else {
            p.status = Status::NoPower;
            continue;
        };
        let grid = power.pole_grid[pole as usize] as usize;
        let tripped = scrammed(p.status == Status::Overheated, p.progress);
        let unmet = power.demand[grid] - power.supply[grid].min(power.demand[grid]);
        let want = if tripped { 0 } else { unmet.min(REACTOR_KW) };
        if want > p.store.charge && p.input.count(FUEL_CELL) > 0 {
            p.input.remove(FUEL_CELL, 1);
            p.store.charge += CELL_CHARGE;
        }
        let given = want.min(p.store.charge);
        p.store.charge -= given;
        p.store.given = given;
        power.supply[grid] += given;
        let cooled = p.steam.water >= given.max(1);
        if cooled {
            p.steam.water -= given;
        }
        p.progress = (p.progress + given).saturating_sub(if cooled { COOLED_KW } else { PASSIVE_KW });
        let hot = scrammed(tripped, p.progress);
        let fuelled = p.store.charge > 0 || p.input.count(FUEL_CELL) > 0;
        if fuelled && !hot {
            power.capacity[grid] += REACTOR_KW;
        }
        p.status = match () {
            _ if hot => Status::Overheated,
            _ if given > 0 => Status::Working,
            _ if fuelled => Status::NoInput,
            _ => Status::NoFuel,
        };
    }
}

#[cfg(test)]
mod tests;
