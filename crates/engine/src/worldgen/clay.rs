//! Clay beds and kaolin pockets (generator version 14, MAT-CLAY): shallow deposits on their own hash streams, apart
//! from `ore_at`, `EXTRAS` and the gas fields, so upgrading a world's `deposits` layer to 14 adds them in unedited
//! chunks without moving any existing deposit (`worldgen/layers/tests.rs`).
//!
//! Real geology: clay settles where slow water drops its finest silt, so beds lie flat in river banks, under lakes and
//! ponds and in the shallows where they meet the sea, a block or two down; kaolin is granite weathered in place, so it lies in pockets just under the
//! highlands' granite. Both are outcrops (`Tier::Outcrop`): shallow, seen where they break the surface, and they leave
//! no stain (`geology::hint_for`). To add another shallow deposit: a seeding block here with its own salt and index.

use crate::block::{CLAY, KAOLIN};
use crate::chunk::CHUNK_SIZE;
use crate::deposits::{Deposit, DepositKey, Tier};
use crate::math::{hash2, IVec3, Rng};

use super::ore::ORE_SPAWN_CLEARING;
use super::{Biome, WorldGen, SEA_LEVEL};

/// The generator `deposits` version that introduced clay beds and kaolin pockets.
pub const CLAY_SINCE: u32 = 14;
/// The depth (blocks below the ground) clay and kaolin lie in: right under it, often breaking through.
pub const CLAY_BAND: (i32, i32) = (0, 4);
/// How far from spawn (blocks) kaolin first appears; clay is found from the start.
pub const KAOLIN_FROM: i32 = 150;
/// Points tried per chunk column for a clay bed; the first beside a river, lake or pond takes it.
const CLAY_TRIES: u32 = 4;
/// How far from standing water (blocks) a clay bed may lie.
const BANK_REACH: i32 = 3;
/// Chance per chunk column of a kaolin pocket (kept only where the highlands' granite lies).
const KAOLIN_CHANCE: f64 = 0.12;
/// Distinct `DepositKey::index`es, past the ores' and the gas fields' (200).
const CLAY_INDEX: u16 = 201;
const KAOLIN_INDEX: u16 = 202;

/// Whether `ore` can lie in `biome` at `at` in this world, for the guides (a query; `None` for other ores).
pub(super) fn holds(gen: &WorldGen, ore: u16, biome: Biome, at: IVec3) -> Option<bool> {
    let new = gen.layers.deposits >= CLAY_SINCE;
    match ore {
        CLAY => Some(new && gen.by_water(at.x, at.z)),
        KAOLIN => Some(new && biome.base() == Biome::Highlands && far(at, KAOLIN_FROM)),
        _ => None,
    }
}

fn far(at: IVec3, from: i32) -> bool {
    (at.x as i64).pow(2) + (at.z as i64).pow(2) >= (from as i64).pow(2)
}

impl WorldGen {
    /// Seeds version 14's clay bed and kaolin pocket for chunk column `(cx, cz)`, if it has them.
    pub(super) fn seed_clay(&self, cx: i32, cz: i32, out: &mut Vec<Deposit>) {
        if self.layers.deposits < CLAY_SINCE {
            return;
        }
        let (x0, z0) = (cx * CHUNK_SIZE, cz * CHUNK_SIZE);
        let key = |ore, index| DepositKey { tier: Tier::Outcrop, cx, cz, ore, index };

        let mut rng = Rng::new(hash2(self.seed ^ 0xC1A7, cx, cz) as u64);
        for _ in 0..CLAY_TRIES {
            let (x, z) = (x0 + rng.below(32) as i32, z0 + rng.below(32) as i32);
            let (major, minor, tall) = (rng.range(4.5, 7.0) as f32, rng.range(3.0, 4.5) as f32, rng.range(1.4, 2.0));
            let radii = if rng.below(2) == 0 { [major, tall as f32, minor] } else { [minor, tall as f32, major] };
            let seed = rng.next_u32();
            if !self.by_water(x, z) || (x.abs() < ORE_SPAWN_CLEARING && z.abs() < ORE_SPAWN_CLEARING) {
                continue;
            }
            let h = self.height_at(x, z);
            if h < SEA_LEVEL - 2 {
                continue; // the sea floor: the beds are the rivers' and lakes'
            }
            out.push(Deposit { key: key(CLAY, CLAY_INDEX), center: IVec3::new(x, h - 1, z), radii, seed });
            break;
        }

        let mut rng = Rng::new(hash2(self.seed ^ 0xCA01, cx, cz) as u64);
        if rng.next_f64() < KAOLIN_CHANCE {
            let (x, z) = (x0 + rng.below(32) as i32, z0 + rng.below(32) as i32);
            let h = self.height_at(x, z);
            let at = IVec3::new(x, h - 2, z);
            let r = rng.range(4.0, 6.0) as f32;
            let radii = [r, rng.range(2.0, 3.0) as f32, r * rng.range(0.7, 1.0) as f32];
            if self.biome_at(x, z, h).base() == Biome::Highlands && far(at, KAOLIN_FROM) {
                out.push(Deposit { key: key(KAOLIN, KAOLIN_INDEX), center: at, radii, seed: rng.next_u32() });
            }
        }
    }

    /// Whether (x, z) lies under or within `BANK_REACH` blocks of a river, lake or pond as generated, or in the
    /// shallows at the sea's edge where they run out (a query, like `water_top`).
    pub(super) fn by_water(&self, x: i32, z: i32) -> bool {
        let near_inland = (self.layers.water >= 7 && self.river_near(x, z)) || self.pond_near(x, z).is_some();
        if !near_inland && self.height_at(x, z) > SEA_LEVEL + BANK_REACH {
            return false; // dry upland: no river, lake, pond or shore within reach (cheap, cached checks first)
        }
        let r = BANK_REACH;
        [(0, 0), (r, 0), (-r, 0), (0, r), (0, -r)].iter().any(|&(dx, dz)| {
            let (x, z) = (x + dx, z + dz);
            let h = self.height_at(x, z);
            h >= SEA_LEVEL - 1 && self.water_top(x, z, h) > h
        })
    }
}

#[cfg(test)]
mod tests;
