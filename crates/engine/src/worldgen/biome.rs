//! Biomes and rock provinces (generator version 2 only): each column gets a biome from two slow
//! climate fields (temperature, moisture), its height and a rare basalt field. The biome decides the
//! surface blocks, the rock below the soil (which also decides its ores, `geology.rs`) and how dense
//! the trees grow. Heights, caves and everything else stay as in version 1.
//!
//! Invariants: a pure function of the seed and the column, so generation stays order-independent;
//! spawn is always plains (the climate fields fade to neutral near it); borders are dithered per
//! column by a hash, a few blocks wide. Version 1 never calls into this file. To add a biome: a
//! `Biome` variant, its rule in `biome_at` and its rows in `surface`, `rock` and `tree_factor`.
//!
//! Generator 12 splits the broad biomes into forest biomes by the same climate (`local_biome_at`): savanna,
//! boreal forest, eucalyptus woodland (plains), bamboo forest (lowlands), redwood forest (highlands). Only trees
//! and ground cover read them; rock, ores, towns, rivers and sound keep the broad one (`base`, `biome_at`).

use std::rc::Rc;

use crate::block::*;
use crate::math::{hash2, smoothstep};

use super::{WorldGen, ROCK_LEVEL, SAND_LEVEL};

/// Columns higher than this are highlands.
const HIGHLAND_LEVEL: i32 = 112;
/// Lowlands lie at or below this height.
const LOWLAND_LEVEL: i32 = 74;
/// Climate fields fade to neutral within this many blocks of spawn (fully neutral inside the first).
const SPAWN_CALM: (f64, f64) = (64.0, 320.0);
/// How many blocks the border dither moves where a column reads its climate (and its highland line).
const DITHER: u32 = 2;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Biome {
    Plains,
    Desert,
    Highlands,
    Lowlands,
    BasaltFields,
    /// Warm, dry plains: acacia and baobab over dry grass.
    Savanna,
    /// Cold plains: pine and birch.
    Boreal,
    /// Warm, wet plains: eucalyptus.
    Woodland,
    /// Warm lowlands: bamboo stands and eucalyptus.
    BambooForest,
    /// Wet highlands: redwoods.
    Redwoods,
}

impl Biome {
    #[cfg(test)]
    pub const ALL: [Biome; 5] = [Biome::Plains, Biome::Desert, Biome::Highlands, Biome::Lowlands, Biome::BasaltFields];

    /// The broad biome a forest biome lies in (itself for a broad one).
    pub fn base(self) -> Biome {
        match self {
            Biome::Savanna | Biome::Boreal | Biome::Woodland => Biome::Plains,
            Biome::BambooForest => Biome::Lowlands,
            Biome::Redwoods => Biome::Highlands,
            b => b,
        }
    }

    /// The rock that fills the column below its soil.
    pub fn rock(self) -> BlockId {
        match self.base() {
            Biome::Plains | Biome::Lowlands => STONE,
            Biome::Desert => SANDSTONE,
            Biome::Highlands => GRANITE,
            _ => BASALT,
        }
    }

    /// Its name for players, with how to tell it (the ore guide).
    pub fn name(self) -> &'static str {
        match self {
            Biome::Plains => "plains (grassland)",
            Biome::Desert => "deserts (sand over sandstone)",
            Biome::Highlands => "highlands (granite hills and mountains)",
            Biome::Lowlands => "lowlands (low, wet forest)",
            Biome::BasaltFields => "basalt fields (black rock)",
            Biome::Savanna => "savanna (acacias and baobabs)",
            Biome::Boreal => "boreal forest (pines and birches)",
            Biome::Woodland => "eucalyptus woodland",
            Biome::BambooForest => "bamboo forest",
            Biome::Redwoods => "redwood forest",
        }
    }

    /// Tree density as a multiple of plains'.
    pub fn tree_factor(self) -> f64 {
        match self {
            Biome::Plains => 1.0,
            Biome::Desert | Biome::BasaltFields => 0.0,
            Biome::Highlands => 0.6,
            Biome::Lowlands | Biome::BambooForest => 1.7,
            Biome::Savanna | Biome::Woodland => 1.0,
            Biome::Boreal => 1.1,
            Biome::Redwoods => 0.9,
        }
    }
}

impl WorldGen {
    /// The biomes of chunk column (cx, cz), 32 × 32 with z rows (x fastest); empty before version 2.
    pub fn biomes(&mut self, cx: i32, cz: i32) -> Rc<[Biome]> {
        self.column(cx, cz).biomes.clone()
    }

    /// The biome of the column at (x, z) as trees and ground cover see it: from generator 12 the forest biome
    /// inside the broad one (by the same frayed climate), before it the broad one.
    pub fn local_biome_at(&self, x: i32, z: i32, h: i32) -> Biome {
        let base = self.biome_at(x, z, h);
        if self.version < 12 {
            return base;
        }
        let (t, m) = self.frayed_climate(x, z);
        match base {
            Biome::Plains if t > 0.06 && m < 0.04 => Biome::Savanna,
            Biome::Plains if t > 0.05 && m > 0.06 => Biome::Woodland,
            Biome::Plains if t < -0.1 => Biome::Boreal,
            Biome::Lowlands if t > 0.02 => Biome::BambooForest,
            Biome::Highlands if m > 0.02 => Biome::Redwoods,
            b => b,
        }
    }

    /// The climate `biome_at` reads at (x, z): a little off the column, calm near spawn.
    fn frayed_climate(&self, x: i32, z: i32) -> (f64, f64) {
        let j = hash2(self.seed ^ 0xB10E, x, z);
        let jitter = |bits: u32| ((j >> bits) % (2 * DITHER + 1)) as i32 - DITHER as i32;
        let (fx, fz) = ((x + jitter(0)) as f64, (z + jitter(8)) as f64);
        let calm = smoothstep(SPAWN_CALM.0, SPAWN_CALM.1, (fx * fx + fz * fz).sqrt());
        (
            self.temperature.fbm2(fx / 1100.0, fz / 1100.0, 3) * calm,
            self.moisture.fbm2(fx / 900.0, fz / 900.0, 3) * calm,
        )
    }

    /// The broad biome of the column at (x, z), whose surface is at height `h`.
    pub fn biome_at(&self, x: i32, z: i32, h: i32) -> Biome {
        // The climate is read a little off the column (by a hash), which frays the borders.
        let j = hash2(self.seed ^ 0xB10E, x, z);
        let jitter = |bits: u32| ((j >> bits) % (2 * DITHER + 1)) as i32 - DITHER as i32;
        let (fx, fz) = ((x + jitter(0)) as f64, (z + jitter(8)) as f64);
        let calm = smoothstep(SPAWN_CALM.0, SPAWN_CALM.1, (fx * fx + fz * fz).sqrt());
        if self.basalt.fbm2(fx / 260.0, fz / 260.0, 2) * calm > 0.34 {
            return Biome::BasaltFields;
        }
        if h > HIGHLAND_LEVEL + jitter(16) {
            return Biome::Highlands;
        }
        let t = self.temperature.fbm2(fx / 1100.0, fz / 1100.0, 3) * calm;
        let m = self.moisture.fbm2(fx / 900.0, fz / 900.0, 3) * calm;
        if t > 0.12 && m < 0.0 {
            Biome::Desert
        } else if h <= LOWLAND_LEVEL && m > 0.08 {
            Biome::Lowlands
        } else {
            Biome::Plains
        }
    }

    /// The climate at (x, z), unfrayed: warmth and wetness (about -0.6..0.6, calm near spawn). Version 12's trees.
    pub(super) fn climate_at(&self, x: i32, z: i32) -> (f64, f64) {
        let (fx, fz) = (x as f64, z as f64);
        let calm = smoothstep(SPAWN_CALM.0, SPAWN_CALM.1, (fx * fx + fz * fz).sqrt());
        (
            self.temperature.fbm2(fx / 1100.0, fz / 1100.0, 3) * calm,
            self.moisture.fbm2(fx / 900.0, fz / 900.0, 3) * calm,
        )
    }

    /// Version 2's (top, filler) blocks and rock for a column: version 1's rules on the biome's rock,
    /// with sand over sandstone in deserts and bare rock on basalt fields.
    pub(super) fn surface_v2(biome: Biome, h: i32, slope: i32) -> ((BlockId, BlockId), BlockId) {
        let rock = biome.rock();
        let pair = if slope >= super::CLIFF_SLOPE || h > ROCK_LEVEL || biome == Biome::BasaltFields {
            (rock, rock)
        } else if h <= SAND_LEVEL || biome == Biome::Desert {
            (SAND, SAND)
        } else {
            (GRASS, DIRT)
        };
        (pair, rock)
    }
}

#[cfg(test)]
mod tests;
