//! Solar power (Milestone 7): the solar panel (2×2×1) and the accumulator (2×2×2), processors that are power
//! sources and sinks on the grid of the pole they hang on, like a steam turbine (`steam.rs`).
//!
//! - A panel gives `SOLAR_KW` in full sun, scaled by `daytime::sunlight` of the core tick (nothing at night),
//!   and only what its grid lacks. Sun a grid doesn't use is spare and charges its accumulators.
//! - An accumulator holds `CHARGE_CAP` and gives up to `ACCUMULATOR_KW` when the sun falls short, before
//!   any generator or turbine burns fuel; it charges from spare sun only, at the same rate. Charge is in
//!   kW·ticks (1 kJ = `TICK_RATE`), saved for accumulators only. Nothing else here is saved.
//! - Power order each tick (`run`, called by `Power::balance` after the demands are added): panels, then
//!   accumulators discharge, then (in `power.rs`) generators and turbines, then spare sun charges.
//!
//! To add another source or store: an `Energy` variant, a spec here and its arm in `run`.

use crate::block::tex;
use crate::bytes::{ByteReader, ByteWriter};
use crate::daytime::sunlight;
use crate::TICK_RATE;

use super::super::footprint::{Footprint, Port};
use super::super::power::Power;
use super::model::{part, Look, Part};
use super::{Energy, Pick, ProcessSpec, ProcessTier, Processor, Status};

/// What a panel gives in full sun, in kW.
pub const SOLAR_KW: u32 = 10;
/// What an accumulator holds: 10 MJ, in kW·ticks.
pub const CHARGE_CAP: u32 = 10_000 * TICK_RATE;
/// The most an accumulator gives or takes, in kW.
pub const ACCUMULATOR_KW: u32 = 60;

/// A panel's or accumulator's state; other processors keep the default.
#[derive(Default)]
pub struct Store {
    /// An accumulator's charge, in kW·ticks.
    pub charge: u32,
    /// kW of sun a panel had last tick, and kW it or an accumulator gave and took (derived).
    pub sun: u32,
    pub given: u32,
    pub taken: u32,
    /// kW the water around a water wheel gives (derived; `hydro.rs`).
    pub flow: u32,
}

const TIER: [ProcessTier; 1] = [ProcessTier { energy: Energy::Solar, speed: 1000, fuel: 0, power: 0 }];
const NO_PORTS: &[Port] = &[];

pub const SOLAR_SPEC: ProcessSpec = ProcessSpec {
    block: crate::block::SOLAR_PANEL,
    categories: &[],
    pick: Pick::ByInput,
    buffers: [0, 0, 0],
    side: 0,
    tiers: &TIER,
    footprint: Footprint { size: [2, 2, 1], ports: NO_PORTS },
    verb: "Generating",
    products: "power",
    waiting: "Idle",
    map_colour: 0x2f4fa8,
    compute: 0,
    parts: &SOLAR_PARTS,
};

pub const ACCUMULATOR_SPEC: ProcessSpec = ProcessSpec {
    block: crate::block::ACCUMULATOR,
    categories: &[],
    pick: Pick::ByInput,
    buffers: [0, 0, 0],
    side: 0,
    tiers: &[ProcessTier { energy: Energy::Accumulator, speed: 1000, fuel: 0, power: 0 }],
    footprint: Footprint { size: [2, 2, 2], ports: NO_PORTS },
    verb: "Storing",
    products: "power",
    waiting: "Idle",
    map_colour: 0x3fae6a,
    compute: 0,
    parts: &ACCUMULATOR_PARTS,
};

const CELLS: [u16; 3] = [tex::SOLAR_TOP, tex::SOLAR_SIDE, tex::FRAME];
const CASING: [u16; 3] = [tex::ACCUMULATOR_TOP, tex::ACCUMULATOR_SIDE, tex::FRAME];
const FRAME: Look = Look::Tex([tex::FRAME; 3]);

/// A flat panel of cells on a banded plinth, a status lamp on a corner.
const SOLAR_PARTS: [Part; 3] = [
    part([0.0, -0.42, 0.0], [1.96, 0.16, 1.96], Look::Band(tex::FRAME)),
    part([0.0, -0.2, 0.0], [1.84, 0.28, 1.84], Look::Tex(CELLS)),
    part([0.85, -0.03, 0.85], [0.12, 0.08, 0.12], Look::Lamp),
];

/// A casing with lit charge cells on the front (they glow while it charges or gives), a roof plate, two
/// copper terminals and a status lamp.
const ACCUMULATOR_PARTS: [Part; 6] = [
    part([0.0, -0.9, 0.0], [1.96, 0.2, 1.96], Look::Band(tex::FRAME)),
    part([0.0, -0.05, 0.0], [1.8, 1.5, 1.8], Look::Tex(CASING)),
    part([0.0, 0.74, 0.0], [1.9, 0.1, 1.9], FRAME),
    part([-0.45, 0.95, 0.0], [0.24, 0.3, 0.24], Look::Tex([tex::COPPER_INGOT; 3])),
    part([0.45, 0.95, 0.0], [0.24, 0.3, 0.24], Look::Tex([tex::COPPER_INGOT; 3])),
    part([0.78, 0.86, 0.78], [0.14, 0.12, 0.14], Look::Lamp),
];

impl Processor {
    /// Writes the energy an accumulator, a diesel generator or a reactor holds (nothing for other processors).
    pub(super) fn write_store(&self, w: &mut ByteWriter) {
        if matches!(self.energy(), Energy::Accumulator | Energy::Diesel | Energy::Reactor) {
            w.u32(self.store.charge);
        }
    }

    pub(super) fn read_store(&mut self, r: &mut ByteReader) -> Option<()> {
        if matches!(self.energy(), Energy::Accumulator | Energy::Diesel | Energy::Reactor) {
            let cap = CHARGE_CAP.max(super::diesel::CANISTER_KJ * TICK_RATE).max(super::nuclear::CELL_CHARGE);
            self.store.charge = r.u32()?.min(cap);
        }
        Some(())
    }

    /// The status line of a panel or an accumulator (`None`: another processor).
    pub(super) fn solar_text(&self) -> Option<String> {
        let (s, kj) = (&self.store, |v: u32| v / TICK_RATE);
        match self.energy() {
            Energy::Solar if s.sun == 0 => Some("Night: no sunlight".to_string()),
            Energy::Solar => Some(format!("Sun {} of {SOLAR_KW} kW · giving {} kW", s.sun, s.given)),
            Energy::Accumulator => {
                let flow = match (s.given, s.taken) {
                    (0, 0) => "idle".to_string(),
                    (g, 0) => format!("giving {g} kW"),
                    (_, t) => format!("charging {t} kW"),
                };
                Some(format!("Charge {} of {} kJ · {flow}", kj(s.charge), kj(CHARGE_CAP)))
            }
            _ => None,
        }
    }
}

/// One tick of every panel and accumulator on a grid: panels give what their grid lacks, accumulators fill
/// what is still lacking, and the spare sun charges accumulators. `Power::balance` adds the demands first.
pub(in crate::factory) fn run(power: &mut Power, processors: &mut [Processor], tick: u64) {
    let sun = sunlight(tick) * SOLAR_KW / 1000;
    let grid_of = |power: &Power, i: usize| power.process_pole[i].map(|p| power.pole_grid[p as usize] as usize);
    let mut spare = vec![0u32; power.supply.len()];
    for (i, p) in processors.iter_mut().enumerate() {
        if p.energy() != Energy::Solar {
            continue;
        }
        p.store.sun = sun;
        p.store.given = 0;
        if let Some(grid) = grid_of(power, i) {
            let given = sun.min(power.demand[grid] - power.supply[grid].min(power.demand[grid]));
            power.supply[grid] += given;
            power.capacity[grid] += sun;
            spare[grid] += sun - given;
            p.store.given = given;
        }
        p.status = if p.store.given > 0 { Status::Working } else { Status::NoInput };
    }
    for (i, p) in processors.iter_mut().enumerate() {
        if p.energy() != Energy::Accumulator {
            continue;
        }
        p.store.given = 0;
        p.store.taken = 0;
        if let Some(grid) = grid_of(power, i) {
            let unmet = power.demand[grid] - power.supply[grid].min(power.demand[grid]);
            let given = unmet.min(ACCUMULATOR_KW).min(p.store.charge);
            p.store.charge -= given;
            p.store.given = given;
            power.supply[grid] += given;
            power.capacity[grid] += ACCUMULATOR_KW.min(p.store.charge);
            let taken = spare[grid].min(ACCUMULATOR_KW).min(CHARGE_CAP - p.store.charge);
            p.store.charge += taken;
            p.store.taken = taken;
            spare[grid] -= taken;
        }
        p.status = if p.store.given + p.store.taken > 0 { Status::Working } else { Status::NoInput };
    }
}

#[cfg(test)]
mod tests;
