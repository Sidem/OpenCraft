//! The rectenna (Milestone 12, step 12.3): a 5×5×1 field of receiving antennas, a power source that lives off the sky.
//! Every power satellite (`satellites.rs`, kind `POWER`) whose footprint covers the field beams it [`SATELLITE_KW`]; the
//! sum, up to [`MAX_KW`], is what it gives, day and night, and only for as long as a satellite is overhead (they fly on
//! at 6 blocks a second, so a field wants a constellation). Like a panel it gives only what its grid lacks.
//!
//! Invariants: the power on offer (`Store::flow`) is derived every tick (`sense_sources`, before the power balance)
//! from the satellites and the tick alone, so a loaded game and a running one agree. Nothing here is saved.
//!
//! To add another sky-driven source: an `Energy` variant like this one's and a spec row.

use crate::block::{tex, RECTENNA};
use crate::satellites::{Satellites, POWER};
use crate::world::World;

use super::super::footprint::{Footprint, Port};
use super::super::power::Power;
use super::super::power_mix::Source;
use super::model::{part, Look, Part};
use super::{Energy, Pick, ProcessSpec, ProcessTier, Processor, Status};

/// kW one power satellite beams to a field it covers.
pub const SATELLITE_KW: u32 = 200;
/// The most a field gives, in kW (five satellites' worth).
pub const MAX_KW: u32 = 1000;

const NO_PORTS: &[Port] = &[];

pub const RECTENNA_SPEC: ProcessSpec = ProcessSpec {
    block: RECTENNA,
    categories: &[],
    pick: Pick::ByInput,
    buffers: [0, 0, 0],
    side: 0,
    tiers: &[ProcessTier { energy: Energy::Rectenna, speed: 1000, fuel: 0, power: 0 }],
    footprint: Footprint { size: [5, 5, 1], ports: NO_PORTS },
    verb: "Receiving",
    products: "power",
    waiting: "Idle",
    map_colour: 0xf472b6,
    compute: 0,
    parts: &RECTENNA_PARTS,
};

const FIELD: Look = Look::Tex([tex::RECTENNA_TOP, tex::RECTENNA_SIDE, tex::FRAME]);
const MAST: Look = Look::Tex([tex::RECTENNA_SIDE; 3]);

/// A banded plinth, the field of diodes, a mast with a status lamp on top, and a post at two corners.
const RECTENNA_PARTS: [Part; 5] = [
    part([0.0, -0.42, 0.0], [4.96, 0.16, 4.96], Look::Band(tex::FRAME)),
    part([0.0, -0.28, 0.0], [4.8, 0.12, 4.8], FIELD),
    part([0.0, 0.1, 0.0], [0.22, 0.8, 0.22], MAST),
    part([0.0, 0.55, 0.0], [0.3, 0.12, 0.3], Look::Lamp),
    part([-2.2, -0.1, -2.2], [0.2, 0.4, 0.2], MAST),
];

impl Processor {
    /// The status line of a rectenna (`None`: another processor).
    pub(super) fn rectenna_text(&self) -> Option<String> {
        if self.energy() != Energy::Rectenna {
            return None;
        }
        Some(match self.store.flow {
            0 => "No power satellite overhead".to_string(),
            flow => format!("Satellites beam {flow} of {MAX_KW} kW · giving {} kW", self.store.given),
        })
    }
}

/// Measures what every renewable source is given this tick: the water around each wheel (`hydro.rs`) and the power
/// satellites over each rectenna (and, for the data grid, over each ground station). Called by `Factory::update` before
/// the power balance.
pub(in crate::factory) fn sense_sources(processors: &mut [Processor], world: &mut World, sats: &Satellites, tick: u64) {
    super::hydro::sense(processors, world);
    super::ground::sense(processors, sats, tick);
    for p in processors.iter_mut().filter(|p| p.energy() == Energy::Rectenna) {
        let overhead = sats.covering(POWER, p.pos.x, p.pos.z, tick) as u32;
        p.store.flow = (overhead * SATELLITE_KW).min(MAX_KW);
    }
}

/// One tick of every rectenna on a grid: each gives what its grid still lacks, up to what the sky gives. Called by
/// `Power::balance` after the water wheels, before any fuel burns.
pub(in crate::factory) fn run(power: &mut Power, processors: &mut [Processor]) {
    for (i, p) in processors.iter_mut().enumerate().filter(|(_, p)| p.energy() == Energy::Rectenna) {
        p.store.given = 0;
        let Some(pole) = power.process_pole[i] else {
            p.status = Status::NoPower;
            continue;
        };
        let grid = power.pole_grid[pole as usize] as usize;
        let given = p.store.flow.min(power.demand[grid] - power.supply[grid].min(power.demand[grid]));
        power.give(Source::Rectenna, grid, given, p.store.flow);
        p.store.given = given;
        p.status = if given > 0 { Status::Working } else { Status::NoInput };
    }
}

#[cfg(test)]
mod tests;
