//! The water wheel (Milestone 10's hydro): a 3×3×1 processor that is a power source like a solar panel (`solar.rs`),
//! but driven by the water that touches it. Every tick (`sense`) it counts the blocks around its footprint: still
//! water is worth `POINT_KW`, flowing water twice that, up to `MAX_KW`. Placed in a river, beside a lake or against a
//! weir it gives power day and night with no fuel; a pool the player pipes (pump and outlet) drives it too. Like
//! a panel it gives only what its grid lacks.
//!
//! Invariants: it reads the world with `block_anywhere_or_generate` every tick and keeps nothing (`Store::flow` is
//! derived), so a loaded game and a running one agree. Nothing here is saved.
//!
//! To add another water-driven source: an `Energy` variant like this one's and a spec row.

use crate::block::{flow_level, tex, BlockId, WATER, WATER_WHEEL};
use crate::math::IVec3;
use crate::world::World;

use super::super::footprint::{Footprint, Port};
use super::super::power::Power;
use super::model::{part, Look, Part};
use super::{Energy, Pick, ProcessSpec, ProcessTier, Processor, Status};

/// kW for each still water block touching the wheel (flowing water counts double).
const POINT_KW: u32 = 2;
/// The most a wheel gives, in kW.
pub const MAX_KW: u32 = 48;
const FACES: [IVec3; 6] = [
    IVec3::new(1, 0, 0),
    IVec3::new(-1, 0, 0),
    IVec3::new(0, 1, 0),
    IVec3::new(0, -1, 0),
    IVec3::new(0, 0, 1),
    IVec3::new(0, 0, -1),
];

const NO_PORTS: &[Port] = &[];

pub const WHEEL_SPEC: ProcessSpec = ProcessSpec {
    block: WATER_WHEEL,
    categories: &[],
    pick: Pick::ByInput,
    buffers: [0, 0, 0],
    side: 0,
    tiers: &[ProcessTier { energy: Energy::Hydro, speed: 1000, fuel: 0, power: 0 }],
    footprint: Footprint { size: [3, 3, 1], ports: NO_PORTS },
    verb: "Generating",
    products: "power",
    waiting: "Idle",
    map_colour: 0x3d78b8,
    compute: 0,
    parts: &WHEEL_PARTS,
};

const WOOD: Look = Look::Tex([tex::PLANKS; 3]);
const HUB: Look = Look::Tex([tex::LOG_TOP, tex::LOG_SIDE, tex::LOG_TOP]);

/// Across and high 3 (±1.5), deep 1: a log hub, two spokes, a square rim and four paddles at the corners, a status
/// lamp.
const WHEEL_PARTS: [Part; 12] = [
    part([0.0, 0.0, 0.0], [0.7, 0.7, 0.9], HUB),
    part([0.0, 0.0, 0.0], [2.7, 0.2, 0.4], WOOD),
    part([0.0, 0.0, 0.0], [0.2, 2.7, 0.36], WOOD),
    part([0.0, 1.3, 0.0], [2.7, 0.2, 0.5], WOOD),
    part([0.0, -1.3, 0.0], [2.7, 0.2, 0.5], WOOD),
    part([-1.3, 0.0, 0.0], [0.2, 2.7, 0.46], WOOD),
    part([1.3, 0.0, 0.0], [0.2, 2.7, 0.46], WOOD),
    part([-1.3, 1.3, 0.0], [0.5, 0.5, 0.9], WOOD),
    part([1.3, 1.3, 0.0], [0.5, 0.5, 0.9], WOOD),
    part([-1.3, -1.3, 0.0], [0.5, 0.5, 0.9], WOOD),
    part([1.3, -1.3, 0.0], [0.5, 0.5, 0.9], WOOD),
    part([0.0, 0.0, 0.46], [0.14, 0.14, 0.1], Look::Lamp),
];

/// What a block around a wheel is worth: 0 for anything but water, 1 for still water, 2 for flowing.
fn points(id: BlockId) -> u32 {
    match () {
        _ if id == WATER => 1,
        _ if flow_level(id).is_some() => 2,
        _ => 0,
    }
}

impl Processor {
    /// The status line of a water wheel (`None`: another processor).
    pub(super) fn hydro_text(&self) -> Option<String> {
        if self.energy() != Energy::Hydro {
            return None;
        }
        Some(match self.store.flow {
            0 => "Dry: stand it in or beside water".to_string(),
            flow => format!("Water gives {flow} of {MAX_KW} kW · giving {} kW", self.store.given),
        })
    }
}

/// Counts the water around every wheel (see the header); called by `Factory::update` before the power balance.
pub(in crate::factory) fn sense(processors: &mut [Processor], world: &mut World) {
    for p in processors.iter_mut().filter(|p| p.energy() == Energy::Hydro) {
        let cells = p.cells();
        let mut wet = 0;
        for &c in &cells {
            for f in FACES {
                if !cells.contains(&(c + f)) {
                    wet += points(world.block_anywhere_or_generate(c + f));
                }
            }
        }
        p.store.flow = (wet * POINT_KW).min(MAX_KW);
    }
}

/// One tick of every water wheel on a grid: each gives what its grid still lacks, up to what its water gives.
/// Called by `Power::balance` with the sun, before any fuel burns.
pub(in crate::factory) fn run(power: &mut Power, processors: &mut [Processor]) {
    for (i, p) in processors.iter_mut().enumerate().filter(|(_, p)| p.energy() == Energy::Hydro) {
        p.store.given = 0;
        let Some(pole) = power.process_pole[i] else {
            p.status = Status::NoPower;
            continue;
        };
        let grid = power.pole_grid[pole as usize] as usize;
        let given = p.store.flow.min(power.demand[grid] - power.supply[grid].min(power.demand[grid]));
        power.supply[grid] += given;
        power.capacity[grid] += p.store.flow;
        p.store.given = given;
        p.status = if given > 0 { Status::Working } else { Status::NoInput };
    }
}

#[cfg(test)]
mod tests;
