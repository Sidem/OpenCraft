//! Biomes and rock provinces (generator version 2 only): each column gets a biome from two slow
//! climate fields (temperature, moisture), its height and a rare basalt field. The biome decides the
//! surface blocks, the rock below the soil (which also decides its ores, `geology.rs`) and how dense
//! the trees grow. Heights, caves and everything else stay as in version 1.
//!
//! Invariants: a pure function of the seed and the column, so generation stays order-independent;
//! spawn is always plains (the climate fields fade to neutral near it); borders are dithered per
//! column by a hash, a few blocks wide. Version 1 never calls into this file. To add a biome: a
//! `Biome` variant, its rule in `biome_at` and its rows in `surface`, `rock` and `tree_factor`.

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
}

impl Biome {
    #[cfg(test)]
    pub const ALL: [Biome; 5] = [Biome::Plains, Biome::Desert, Biome::Highlands, Biome::Lowlands, Biome::BasaltFields];

    /// The rock that fills the column below its soil.
    pub fn rock(self) -> BlockId {
        match self {
            Biome::Plains | Biome::Lowlands => STONE,
            Biome::Desert => SANDSTONE,
            Biome::Highlands => GRANITE,
            Biome::BasaltFields => BASALT,
        }
    }

    /// Tree density as a multiple of plains'.
    pub fn tree_factor(self) -> f64 {
        match self {
            Biome::Plains => 1.0,
            Biome::Desert | Biome::BasaltFields => 0.0,
            Biome::Highlands => 0.6,
            Biome::Lowlands => 1.7,
        }
    }
}

impl WorldGen {
    /// The biome of the column at (x, z), whose surface is at height `h`.
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
