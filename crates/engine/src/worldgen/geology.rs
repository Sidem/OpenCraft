//! Geology-driven ores (generator version 2 only): the same numbers of outcrops, veins and lodes as
//! version 1 (the user kept today's rates), but each deposit's ore is drawn from the weights of the
//! biome it lies in (`biome.rs`), so copper gathers in the highlands, coal and limestone in the
//! lowlands, quartz in the deserts and iron in the basalt. Veins and lodes stain the grass and sand
//! above them (`stain_surface`: rusty over iron, dark over coal, green over copper, pale over limestone
//! and quartz); outcrops show themselves.
//!
//! Invariants: a pure function of the seed and the column, like version 1's seeding, and keys stay
//! unique per (column, tier, slot), so `deposit_by_key` finds a saved deposit again. Shapes come from
//! the same helpers as version 1 (`ore.rs`). To tune: `ORES_BY_BIOME` (weights, not rates).

use crate::block::{
    BlockId, BAUXITE_ORE, COAL_ORE, COPPER_ORE, DARK_SAND, DARK_SOIL, GRASS, GREEN_SAND, GREEN_SOIL, IRON_ORE,
    LIMESTONE, PALE_SAND, PALE_SOIL, QUARTZ_ORE, RUSTY_SAND, RUSTY_SOIL, SAND,
};
use crate::chunk::CHUNK_SIZE;
use crate::deposits::{Deposit, DepositKey, Tier};
use crate::math::{hash2, IVec3, Rng};

use super::ore::{lode_shape, LODE_CHANCE, ORE_SPAWN_CLEARING};
use super::{Biome, WorldGen};

/// Outcrops per chunk column (version 1: 4 coal + 4 iron + 3 copper).
const OUTCROPS: u32 = 11;
/// Chance of each vein slot per column (version 1's three vein chances add up to 1.3).
const VEIN_CHANCES: [f64; 2] = [1.0, 0.3];
/// Surface hints reach this many blocks past a vein's or lode's footprint.
const HINT_MARGIN: f32 = 3.0;
/// About one surface block in this many over a vein or lode is stained.
const HINT_ONE_IN: u32 = 4;
/// Which ores each biome holds, as weights.
const ORES_BY_BIOME: [(Biome, &[(BlockId, u32)]); 5] = [
    (Biome::Plains, &[(COAL_ORE, 4), (IRON_ORE, 4), (COPPER_ORE, 2), (LIMESTONE, 1)]),
    (Biome::Lowlands, &[(COAL_ORE, 6), (IRON_ORE, 2), (LIMESTONE, 3)]),
    (Biome::Highlands, &[(COPPER_ORE, 5), (IRON_ORE, 3), (QUARTZ_ORE, 3), (COAL_ORE, 1)]),
    (Biome::Desert, &[(QUARTZ_ORE, 4), (LIMESTONE, 3), (IRON_ORE, 3), (COPPER_ORE, 1)]),
    (Biome::BasaltFields, &[(IRON_ORE, 6), (COPPER_ORE, 3), (QUARTZ_ORE, 1)]),
];

/// Version 5: far from spawn, bauxite joins these biomes' ore weights (`WorldGen::bauxite_weight`).
pub const BAUXITE_FROM: i32 = 600;
const BAUXITE_WEIGHTS: [(Biome, u32); 2] = [(Biome::Desert, 4), (Biome::BasaltFields, 3)];

impl WorldGen {
    /// Version 2's deposits seeded in one chunk column.
    pub(super) fn seed_deposits_v2(&self, cx: i32, cz: i32, out: &mut Vec<Deposit>) {
        let (x0, z0) = (cx * CHUNK_SIZE, cz * CHUNK_SIZE);
        let key = |tier, ore, index: u32| DepositKey { tier, cx, cz, ore, index: index as u16 };
        let mut rng = Rng::new(hash2(self.seed ^ 0x6E01, cx, cz) as u64);
        for i in 0..OUTCROPS {
            let (center, radii, seed) = self.outcrop_shape(&mut rng, x0, z0);
            let ore = self.ore_at(&mut rng, center);
            if center.x.abs() < ORE_SPAWN_CLEARING && center.z.abs() < ORE_SPAWN_CLEARING {
                continue;
            }
            out.push(Deposit { key: key(Tier::Outcrop, ore, i), center, radii, seed });
        }
        for (i, &chance) in VEIN_CHANCES.iter().enumerate() {
            if rng.next_f64() < chance {
                let (center, radii, seed) = self.vein_shape(&mut rng, x0, z0);
                let ore = self.ore_at(&mut rng, center);
                out.push(Deposit { key: key(Tier::Vein, ore, i as u32), center, radii, seed });
            }
        }
        let mut rng = Rng::new(hash2(self.seed ^ 0x10DF, cx, cz) as u64);
        if rng.next_f64() < LODE_CHANCE {
            let (center, radii, seed) = lode_shape(&mut rng, x0, z0);
            let ore = self.ore_at(&mut rng, center);
            out.push(Deposit { key: key(Tier::Lode, ore, 0), center, radii, seed });
        }
    }

    /// Surface hints: stains about one in `HINT_ONE_IN` grass or sand tops over each vein and
    /// lode of `near` (and `HINT_MARGIN` blocks around it) in the column starting at (x0, z0).
    pub(super) fn stain_surface(&self, x0: i32, z0: i32, near: &[Deposit], surface: &mut [(BlockId, BlockId)]) {
        for d in near.iter().filter(|d| d.tier() != Tier::Outcrop) {
            let r = d.radii[0].max(d.radii[2]) + HINT_MARGIN;
            let reach = r.ceil() as i32;
            for z in (d.center.z - reach).max(z0)..=(d.center.z + reach).min(z0 + CHUNK_SIZE - 1) {
                for x in (d.center.x - reach).max(x0)..=(d.center.x + reach).min(x0 + CHUNK_SIZE - 1) {
                    let (dx, dz) = ((x - d.center.x) as f32, (z - d.center.z) as f32);
                    if dx * dx + dz * dz > r * r || !hash2(self.seed ^ 0x41E7, x, z).is_multiple_of(HINT_ONE_IN) {
                        continue;
                    }
                    let top = &mut surface[((z - z0) * CHUNK_SIZE + x - x0) as usize].0;
                    if let Some(hint) = hint_for(d.ore(), *top) {
                        *top = hint;
                    }
                }
            }
        }
    }

    /// The vein or lode whose surface hint may have left `stain` (a stained soil or sand) on column
    /// (x, z): of the ore that stain marks, the nearest whose hint area covers the column. A query.
    pub fn hint_source(&self, x: i32, z: i32, stain: BlockId) -> Option<Deposit> {
        let marks = |d: &Deposit| [GRASS, SAND].iter().any(|&top| hint_for(d.ore(), top) == Some(stain));
        let mut best: Option<(f32, Deposit)> = None;
        for d in self.deposits_near(x >> 5, z >> 5).into_iter().filter(|d| d.tier() != Tier::Outcrop && marks(d)) {
            let (dx, dz) = ((x - d.center.x) as f32, (z - d.center.z) as f32);
            let (dist, r) = (dx * dx + dz * dz, d.radii[0].max(d.radii[2]) + HINT_MARGIN);
            if dist <= r * r && best.as_ref().is_none_or(|b| dist < b.0) {
                best = Some((dist, d));
            }
        }
        best.map(|b| b.1)
    }

    /// An ore drawn from the weights of the biome above `at`.
    pub(super) fn ore_at(&self, rng: &mut Rng, at: IVec3) -> BlockId {
        let biome = self.biome_at(at.x, at.z, self.height_at(at.x, at.z));
        let ores = ores_in(biome);
        let bauxite = self.bauxite_weight(biome, at);
        let mut pick = rng.below(ores.iter().map(|o| o.1).sum::<u32>() + bauxite);
        for &(ore, weight) in ores {
            if pick < weight {
                return ore;
            }
            pick -= weight;
        }
        if pick < bauxite {
            return BAUXITE_ORE;
        }
        ores[0].0
    }

    /// The weight of bauxite among the ores of `biome` at `at`: none before version 5 or within `BAUXITE_FROM`
    /// blocks of spawn, else `BAUXITE_WEIGHTS`.
    fn bauxite_weight(&self, biome: Biome, at: IVec3) -> u32 {
        let far = (at.x as i64).pow(2) + (at.z as i64).pow(2) >= (BAUXITE_FROM as i64).pow(2);
        if self.version < 5 || !far {
            return 0;
        }
        BAUXITE_WEIGHTS.iter().find(|w| w.0 == biome).map_or(0, |w| w.1)
    }
}

/// The stained surface block that replaces `top` over `ore` underground, if `top` (grass or sand)
/// takes stains.
pub fn hint_for(ore: BlockId, top: BlockId) -> Option<BlockId> {
    let (grass, sand) = match ore {
        // Bauxite shows itself (exposed on bare rock) and leaves no stain.
        BAUXITE_ORE => return None,
        IRON_ORE => (RUSTY_SOIL, RUSTY_SAND),
        COAL_ORE => (DARK_SOIL, DARK_SAND),
        COPPER_ORE => (GREEN_SOIL, GREEN_SAND),
        _ => (PALE_SOIL, PALE_SAND),
    };
    match top {
        GRASS => Some(grass),
        SAND => Some(sand),
        _ => None,
    }
}

/// Per biome where `ore` occurs, the percentage of that biome's deposits that are `ore`, highest first
/// (version 2 on; the ore guide shows it).
pub fn ore_shares(ore: BlockId) -> Vec<(Biome, u32)> {
    let mut out: Vec<(Biome, u32)> = Vec::new();
    for (biome, weights) in ORES_BY_BIOME {
        let mut total: u32 = weights.iter().map(|w| w.1).sum();
        let mut mine = weights.iter().find(|w| w.0 == ore).map(|w| w.1);
        if ore == BAUXITE_ORE {
            mine = BAUXITE_WEIGHTS.iter().find(|w| w.0 == biome).map(|w| w.1);
            total += mine.unwrap_or(0);
        }
        if let Some(w) = mine {
            out.push((biome, (w * 100 + total / 2) / total));
        }
    }
    crate::math::sort_small_by_key(&mut out, |s| u32::MAX - s.1);
    out
}

/// The ore weights of `biome`.
pub(super) fn ores_in(biome: Biome) -> &'static [(BlockId, u32)] {
    ORES_BY_BIOME.iter().find(|b| b.0 == biome).map_or(ORES_BY_BIOME[0].1, |b| b.1)
}

#[cfg(test)]
mod tests;
