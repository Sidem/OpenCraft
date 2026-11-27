//! Read-only presentation survey of nearby rhythmic machinery for the host's factory instrument
//! (`web/src/audio/factory-beat.ts`, F-BEAT). Maps working machines into [`FAMILY_COUNT`] musical
//! families within [`BEAT_RADIUS`] blocks of the camera and packs per-family proximity, nearest
//! relative position, health (working vs stalled), power pace, cycle phase and tier into a flat
//! float array. Presentation only: never mutates or feeds back into core state (`DEV_PLAN` §3.4).
//!
//! To assign a new machine block to a voice: add its `BlockId` to [`family_of`].

use crate::block::{
    BlockId, AI_LAB, ARC_FURNACE, ASSEMBLER, BLAST_FURNACE, BOILER, CENTRIFUGE, CHEMICAL_PLANT, CHIP_FAB, COAL_MINER,
    CONSTRUCTOR, CRACKER, CRUSHER, DATACENTER, DIESEL_GENERATOR, ELECTROLYSER, ELECTROLYTIC_CELL, FORESTER, GAS_PLANT,
    GAS_TURBINE, GAS_WELL, GENERATOR, GLASS_KILN, LAB, LAUNCH_SITE, MINER, MINER_MK2, OPTIMIZER, PUMPJACK, QUARRY,
    REACTOR, RECYCLER, REFINERY, RESEARCH_CENTER, SMELTER, STEAM_REFORMER, SWARM_HUB, TURBINE, UPGRADER, WASHER,
    WATER_WHEEL, WELLHEAD_SEPARATOR,
};
use crate::math::{IVec3, Vec3};
use crate::recipes::MACHINE_RECIPES;

use super::lab::LabStatus;
use super::miner::MinerStatus;
use super::power::FULL_SPEED;
use super::process::{Energy, Processor, Status};
use super::quarry::QuarryStatus;
use super::{ticks, Factory};

/// Hard hearing distance cap for the factory instrument (blocks). Beyond this, silence.
pub const BEAT_RADIUS: f64 = 26.0;
/// Musical machine families (Power, Mining, Smelting, Refining, Shaping, Assembly, Chemistry, Research).
pub const FAMILY_COUNT: usize = 8;
/// Floats emitted per family: strength, dist, rx, rz, health, pace, phase, tier.
pub const FAMILY_STRIDE: usize = 8;
pub const BEAT_FLOATS: usize = FAMILY_COUNT * FAMILY_STRIDE;

pub const FAMILY_POWER: usize = 0;
pub const FAMILY_MINING: usize = 1;
pub const FAMILY_SMELTING: usize = 2;
pub const FAMILY_REFINING: usize = 3;
pub const FAMILY_SHAPING: usize = 4;
pub const FAMILY_ASSEMBLY: usize = 5;
pub const FAMILY_CHEMISTRY: usize = 6;
pub const FAMILY_RESEARCH: usize = 7;

/// Maps an active machine block to its musical family, or `None` for passive/silent blocks.
pub fn family_of(block: BlockId) -> Option<usize> {
    match block {
        GENERATOR | BOILER | TURBINE | DIESEL_GENERATOR | GAS_TURBINE | WATER_WHEEL | REACTOR => Some(FAMILY_POWER),
        MINER | COAL_MINER | MINER_MK2 | QUARRY | PUMPJACK | GAS_WELL | FORESTER => Some(FAMILY_MINING),
        SMELTER | BLAST_FURNACE | ARC_FURNACE | GLASS_KILN => Some(FAMILY_SMELTING),
        CRUSHER | WASHER | ELECTROLYTIC_CELL | RECYCLER | WELLHEAD_SEPARATOR => Some(FAMILY_REFINING),
        CONSTRUCTOR | UPGRADER => Some(FAMILY_SHAPING),
        ASSEMBLER | CHIP_FAB | LAUNCH_SITE => Some(FAMILY_ASSEMBLY),
        REFINERY | CRACKER | CHEMICAL_PLANT | ELECTROLYSER | GAS_PLANT | STEAM_REFORMER | CENTRIFUGE => {
            Some(FAMILY_CHEMISTRY)
        }
        LAB | RESEARCH_CENTER | AI_LAB | DATACENTER | OPTIMIZER | SWARM_HUB => Some(FAMILY_RESEARCH),
        _ => None,
    }
}

#[derive(Default)]
struct Acc {
    working: u32,
    stalled: u32,
    prox_sum: f64,
    max_prox: f64,
    pace_sum: f64,
    nearest_d: f64,
    rx: f64,
    rz: f64,
    phase: f64,
    max_tier: u8,
}

impl Acc {
    fn new() -> Self {
        Self { nearest_d: BEAT_RADIUS + 1.0, ..Self::default() }
    }

    fn note_working(&mut self, pos: IVec3, at: Vec3, pace: f64, phase: f64, tier: u8) {
        let rel = Vec3::new(f64::from(pos.x) + 0.5, f64::from(pos.y) + 0.5, f64::from(pos.z) + 0.5) - at;
        let d = rel.length();
        if d >= BEAT_RADIUS {
            return;
        }
        let prox = 1.0 - d / BEAT_RADIUS;
        self.working += 1;
        self.prox_sum += prox;
        if prox > self.max_prox {
            self.max_prox = prox;
        }
        self.pace_sum += pace.clamp(0.0, 1.0) * prox;
        if tier > self.max_tier {
            self.max_tier = tier;
        }
        if d < self.nearest_d {
            self.nearest_d = d;
            self.rx = rel.x;
            self.rz = rel.z;
            self.phase = phase.clamp(0.0, 1.0);
        }
    }

    fn note_stalled(&mut self, pos: IVec3, at: Vec3) {
        let rel = Vec3::new(f64::from(pos.x) + 0.5, f64::from(pos.y) + 0.5, f64::from(pos.z) + 0.5) - at;
        if rel.length() < BEAT_RADIUS {
            self.stalled += 1;
        }
    }

    fn write(&self, out: &mut [f32]) {
        if self.working == 0 {
            out[0] = 0.0;
            out[1] = (BEAT_RADIUS + 1.0) as f32;
            out[2] = 0.0;
            out[3] = 0.0;
            out[4] = 0.0;
            out[5] = 0.0;
            out[6] = 0.0;
            out[7] = 0.0;
            return;
        }
        // One nearby machine already carries the voice cleanly; extra machines add gentle warmth, never noise.
        let fullness = 0.72 + 0.28 * (self.prox_sum / (1.0 + self.prox_sum));
        let strength = (self.max_prox * fullness).clamp(0.0, 1.0);
        let total = self.working + self.stalled;
        let health = f64::from(self.working) / f64::from(total);
        let pace = if self.prox_sum > 0.0 { self.pace_sum / self.prox_sum } else { 1.0 };
        out[0] = strength as f32;
        out[1] = self.nearest_d as f32;
        out[2] = self.rx as f32;
        out[3] = self.rz as f32;
        out[4] = health as f32;
        out[5] = pace.clamp(0.0, 1.0) as f32;
        out[6] = self.phase as f32;
        out[7] = f32::from(self.max_tier);
    }
}

fn processor_stalled(p: &Processor) -> bool {
    let source = p.energy().is_source() || p.energy() == Energy::Boiler;
    match p.status {
        Status::OutputFull
        | Status::NoPower
        | Status::NoFuel
        | Status::NoWater
        | Status::Overheated
        | Status::Exhausted => true,
        // Power sources use NoInput to mean "grid demand is already satisfied", not starved.
        Status::NoInput => !source,
        Status::Working | Status::NoRecipe | Status::NoDeposit | Status::Off => false,
    }
}

fn processor_phase(p: &Processor) -> f64 {
    let Some(i) = p.batch else { return 0.0 };
    let Some(r) = MACHINE_RECIPES.get(i as usize) else { return 0.0 };
    let total = f64::from(ticks(r.seconds) * FULL_SPEED);
    if total > 0.0 {
        f64::from(p.progress) / total
    } else {
        0.0
    }
}

fn processor_pace(p: &Processor) -> f64 {
    if p.draws_power() || p.spec.compute < 0 {
        f64::from(p.speed) / f64::from(FULL_SPEED)
    } else {
        1.0
    }
}

impl Factory {
    /// Summarises nearby rhythmic machinery by musical family within [`BEAT_RADIUS`] of `at`.
    pub fn beat_near(&self, at: Vec3) -> [f32; BEAT_FLOATS] {
        let mut acc: [Acc; FAMILY_COUNT] = std::array::from_fn(|_| Acc::new());

        for p in &self.processors {
            let Some(fam) = family_of(p.spec.block) else { continue };
            if p.status == Status::Working {
                acc[fam].note_working(p.pos, at, processor_pace(p), processor_phase(p), p.tier);
            } else if processor_stalled(p) {
                acc[fam].note_stalled(p.pos, at);
            }
        }

        for m in &self.miners {
            let fam = FAMILY_MINING;
            if m.status == MinerStatus::Running && m.draw_rate > 0.01 {
                let pace = f64::from(m.speed) / f64::from(FULL_SPEED);
                let phase = 1.0 - (f64::from(m.pulse) / 54.0).clamp(0.0, 1.0);
                let tier = if m.is_coal() { 0 } else { m.tier };
                acc[fam].note_working(m.pos, at, pace, phase, tier);
            } else if matches!(
                m.status,
                MinerStatus::OutputFull | MinerStatus::NoPower | MinerStatus::NoFuel | MinerStatus::Exhausted
            ) {
                acc[fam].note_stalled(m.pos, at);
            }
        }

        for q in &self.quarries {
            if q.status == QuarryStatus::Digging {
                let pace = f64::from(q.speed) / f64::from(FULL_SPEED);
                acc[FAMILY_MINING].note_working(q.pos, at, pace, 0.0, q.tier);
            } else if matches!(q.status, QuarryStatus::OutputFull | QuarryStatus::NoPower | QuarryStatus::Flooded) {
                acc[FAMILY_MINING].note_stalled(q.pos, at);
            }
        }

        for g in &self.generators {
            if g.output > 0 {
                acc[FAMILY_POWER].note_working(g.pos, at, 1.0, 0.0, g.tier);
            } else if g.fuel.total() == 0 && g.energy == 0 {
                acc[FAMILY_POWER].note_stalled(g.pos, at);
            }
        }

        for l in &self.labs {
            if l.status == LabStatus::Working {
                let pace = f64::from(l.speed) / f64::from(FULL_SPEED);
                acc[FAMILY_RESEARCH].note_working(l.pos, at, pace, 0.0, l.tier);
            } else if matches!(l.status, LabStatus::NoPacks | LabStatus::NoPower) {
                acc[FAMILY_RESEARCH].note_stalled(l.pos, at);
            }
        }

        let mut out = [0.0f32; BEAT_FLOATS];
        for (i, a) in acc.iter().enumerate() {
            a.write(&mut out[i * FAMILY_STRIDE..(i + 1) * FAMILY_STRIDE]);
        }
        out
    }
}
