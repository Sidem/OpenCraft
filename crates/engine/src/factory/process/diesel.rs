//! The diesel generator (Milestone 10): a processor that is a power source, like a steam turbine or a solar panel
//! (`steam.rs`, `solar.rs`) but fuelled by canisters. Belts bring diesel canisters in at the back; when its grid
//! lacks power and what it holds can't cover this tick it lights one, which gives `CANISTER_KJ` and hands the empty
//! canister to the output at once (a full output stops it lighting). It gives only what the grid lacks, up to
//! `DIESEL_KW`, so a canister lasts exactly as long as the load allows.
//!
//! - The energy it holds is `Store::charge` in kW·ticks (1 kJ = `TICK_RATE`), saved for it like an accumulator's.
//! - Power order (`Power::balance`): sun and accumulators, coal generators, steam turbines, then diesel.
//!
//! To add another canister-fuelled source: an `Energy` variant, a spec like this and its arm in `run`.

use crate::block::{tex, DIESEL_GENERATOR};
use crate::item::{DIESEL_CANISTER, EMPTY_CANISTER};
use crate::TICK_RATE;

use super::super::footprint::{Footprint, Port, Role, Side, Which};
use super::super::power::{Power, NOT_WIRED};
use super::model::{part, Look, Part};
use super::{Energy, Pick, ProcessSpec, ProcessTier, Processor, Status};

/// The most a diesel generator gives, in kW.
pub const DIESEL_KW: u32 = 400;
/// The energy of a diesel canister in kJ: 400 kW for 100 s (coal is 270 kJ an item).
pub const CANISTER_KJ: u32 = 40_000;
const CANISTER_CHARGE: u32 = CANISTER_KJ * TICK_RATE;

pub const DIESEL_SPEC: ProcessSpec = ProcessSpec {
    block: DIESEL_GENERATOR,
    categories: &[],
    pick: Pick::ByInput,
    buffers: [1, 0, 1],
    side: 0,
    tiers: &[ProcessTier { energy: Energy::Diesel, speed: 1000, fuel: 0, power: 0 }],
    footprint: Footprint {
        size: [2, 2, 2],
        ports: &[
            Port { side: Side::Back, role: Role::In, cell: Which::All },
            Port { side: Side::Front, role: Role::Out, cell: Which::All },
        ],
    },
    verb: "Generating",
    products: "empty canisters",
    waiting: "Waiting for diesel canisters",
    map_colour: 0xc8742a,
    parts: &DIESEL_PARTS,
};

const BODY: [u16; 3] = [tex::DIESEL_TOP, tex::DIESEL_SIDE, tex::FRAME];
const STEEL: Look = Look::Tex([tex::STEEL; 3]);
const FRAME: Look = Look::Tex([tex::FRAME; 3]);

/// Across and deep 2, 2 high (±1): a banded plinth, an engine block with a roof plate, a tall exhaust stack at the
/// back with a flame that burns while it gives power, a copper terminal and a status lamp.
const DIESEL_PARTS: [Part; 7] = [
    part([0.0, -0.9, 0.0], [1.96, 0.2, 1.96], Look::Band(tex::FRAME)),
    part([0.0, -0.2, 0.15], [1.8, 1.2, 1.5], Look::Tex(BODY)),
    part([0.0, 0.46, 0.15], [1.9, 0.1, 1.6], FRAME),
    part([-0.55, 0.6, -0.65], [0.3, 0.9, 0.3], STEEL),
    part([-0.55, 1.1, -0.65], [0.2, 0.3, 0.2], Look::Fire(tex::GENERATOR_SIDE)),
    part([0.5, 0.62, 0.15], [0.3, 0.2, 0.3], Look::Tex([tex::COPPER_INGOT; 3])),
    part([0.78, 0.0, 0.92], [0.14, 0.1, 0.14], Look::Lamp),
];

impl Processor {
    /// The status line of a diesel generator (`None`: another processor).
    pub(super) fn diesel_text(&self) -> Option<String> {
        if self.energy() != Energy::Diesel {
            return None;
        }
        Some(match self.status {
            Status::Working => {
                format!("Giving {} of {DIESEL_KW} kW · {} kJ lit", self.store.given, self.store.charge / TICK_RATE)
            }
            Status::NoFuel => "Out of fuel: bring diesel canisters".to_string(),
            Status::OutputFull => {
                "Empty canisters have nowhere to go: put a belt leading away, or take them".to_string()
            }
            Status::NoPower => NOT_WIRED.to_string(),
            _ => "Standing by: the grid wants no more power".to_string(),
        })
    }
}

/// One tick of every diesel generator on a grid: each gives what its grid still lacks, lighting a canister when it
/// must. Called by `Power::balance` after the demands, the sun, the generators and the turbines.
pub(in crate::factory) fn run(power: &mut Power, processors: &mut [Processor]) {
    for (i, p) in processors.iter_mut().enumerate().filter(|(_, p)| p.energy() == Energy::Diesel) {
        p.store.given = 0;
        let Some(pole) = power.process_pole[i] else {
            p.status = Status::NoPower;
            continue;
        };
        let grid = power.pole_grid[pole as usize] as usize;
        let want = (power.demand[grid] - power.supply[grid].min(power.demand[grid])).min(DIESEL_KW);
        let mut stuck = false;
        if want > p.store.charge && p.input.count(DIESEL_CANISTER) > 0 {
            if p.out.space_for(EMPTY_CANISTER) > 0 {
                p.input.remove(DIESEL_CANISTER, 1);
                p.out.add(EMPTY_CANISTER, 1);
                p.store.charge += CANISTER_CHARGE;
            } else {
                stuck = true;
            }
        }
        let given = want.min(p.store.charge);
        p.store.charge -= given;
        p.store.given = given;
        power.supply[grid] += given;
        let fuelled = p.store.charge > 0 || p.input.count(DIESEL_CANISTER) > 0;
        if fuelled {
            power.capacity[grid] += DIESEL_KW;
        }
        p.status = match () {
            _ if given > 0 => Status::Working,
            _ if stuck => Status::OutputFull,
            _ if fuelled => Status::NoInput,
            _ => Status::NoFuel,
        };
    }
}

#[cfg(test)]
mod tests;
