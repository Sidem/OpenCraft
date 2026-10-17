//! Ground cover (R2a): grass tufts, flowers, ferns, reeds, dry brush and lichen drawn as plants on top of
//! natural ground, chosen per biome by a hash of the world column. Presentation only: never blocks, never
//! saved, not in the state hash; a remesh redraws it, so it follows the ground. Only open sky gets any
//! (sky light at least `SKY_MIN`), so floors under roofs and in caves stay bare.
//!
//! Invariants: a pure function of the chunk's blocks, light, biomes and world position; generator
//! version 1 has no biomes and so no cover. To add a plant: a texture (`block::tex`,
//! `textures/hires.rs`) and a rule in [`pick`]. Over stained ground (`block::RUSTY_SOIL..`) grow the indicator
//! plants (R2c): each stain has its own, so a patch of one tells the ore below before anyone points at it.

use super::{pidx, Mesher, N, P, P2};
use crate::block::{
    tex, BlockId, AIR, BASALT, DIRT, GRANITE, GRASS, LIQUID, PALE_SAND, PALE_SOIL, RUSTY_SAND, RUSTY_SOIL, SAND,
};
use crate::math::hash2;
use crate::worldgen::Biome;

/// What a chunk needs to grow cover: its biomes and its lowest world corner.
pub struct Cover<'a> {
    pub x0: i32,
    pub z0: i32,
    /// 32 × 32, z rows, x fastest (`WorldGen::biomes`).
    pub biomes: &'a [Biome],
}

/// Least sky light (of 15) a cell needs to grow anything.
const SKY_MIN: u8 = 12;
const SEED: u32 = 0xC0FE_7A1F;
/// Flower chance is multiplied by this inside a meadow, a 4 × 4 patch of columns picked by hash.
const MEADOW_BOOST: u32 = 7;
const MEADOW_PERMILLE: u32 = 220;

#[derive(Clone, Copy, PartialEq)]
enum Ground {
    Turf,
    Dirt,
    Sand,
    Rock(BlockId),
}

/// The plant that grows over each stain (R2c), by the stain's order in `block`: rusty (iron), dark (coal), green
/// (copper), pale (limestone and quartz). Real geobotany: some plants thrive on metal-rich soil.
const INDICATORS: [u16; 4] = [tex::INDICATOR_IRON, tex::INDICATOR_COAL, tex::INDICATOR_COPPER, tex::INDICATOR_PALE];
/// Chance in thousandths that a stained column carries its indicator.
const INDICATOR_PERMILLE: u32 = 450;

/// The indicator of a stained soil or stained sand block, if it is one.
fn indicator_of(b: BlockId) -> Option<u16> {
    match b {
        RUSTY_SOIL..=PALE_SOIL => Some(INDICATORS[(b - RUSTY_SOIL) as usize]),
        RUSTY_SAND..=PALE_SAND => Some(INDICATORS[(b - RUSTY_SAND) as usize]),
        _ => None,
    }
}

fn ground_of(b: BlockId) -> Option<Ground> {
    match b {
        GRASS | RUSTY_SOIL..=PALE_SOIL => Some(Ground::Turf),
        DIRT => Some(Ground::Dirt),
        SAND | RUSTY_SAND..=PALE_SAND => Some(Ground::Sand),
        BASALT | GRANITE => Some(Ground::Rock(b)),
        _ => None,
    }
}

impl Mesher {
    /// Adds the cover of the centre chunk, after its blocks are in `pad` (`light` is the padded light).
    pub(super) fn mesh_cover(&mut self, light: &[u8], cover: &Cover) {
        if cover.biomes.len() != N * N {
            return;
        }
        for z in 0..N {
            for x in 0..N {
                let (wx, wz) = (cover.x0 + x as i32, cover.z0 + z as i32);
                // The column's hashes, rolled at its first candidate cell only: most columns have none.
                let mut rolls = None;
                for y in 0..N {
                    let p = pidx(x + 1, y + 1, z + 1);
                    if self.pad[p] != AIR {
                        continue;
                    }
                    let below = self.pad[p - P2];
                    let Some(ground) = ground_of(below) else { continue };
                    if (light[p] & 15) < SKY_MIN {
                        continue;
                    }
                    let (h, meadow) = *rolls.get_or_insert_with(|| {
                        (hash2(SEED, wx, wz), hash2(SEED ^ 1, wx >> 2, wz >> 2) % 1000 < MEADOW_PERMILLE)
                    });
                    let wet = !matches!(ground, Ground::Rock(_)) && self.water_beside(p, y);
                    let stain = indicator_of(below).filter(|_| h % 1000 < INDICATOR_PERMILLE);
                    if let Some(layer) = stain.or_else(|| pick(cover.biomes[z * N + x], ground, h, meadow, wet)) {
                        self.push_plant(x as u32, y as u32, z as u32, u32::from(layer), light[p]);
                    }
                }
            }
        }
    }

    /// Water at the ground level of a side neighbour of air cell `p` (chunk row `y`), or one lower: a bank.
    fn water_beside(&self, p: usize, y: usize) -> bool {
        let lowest = if y >= 1 { 2 } else { 1 };
        [1, -1, P as isize, -(P as isize)].iter().any(|&side| {
            (1..=lowest).any(|down| LIQUID[self.pad[(p as isize + side - down * P2 as isize) as usize] as usize])
        })
    }
}

/// The plant (a texture layer) for a column of this biome over this ground, if any. `h` is the column's
/// hash: its low bits roll the chance in thousandths, the rest pick between kinds.
fn pick(biome: Biome, ground: Ground, h: u32, meadow: bool, wet: bool) -> Option<u16> {
    let (roll, kind) = (h % 1000, h / 1000);
    if wet && roll < 380 {
        return Some(tex::REED);
    }
    match ground {
        Ground::Turf => turf(biome, roll, kind, meadow),
        Ground::Dirt => (roll < 30).then_some(tex::GRASS_TUFT),
        Ground::Sand => (roll < if biome == Biome::Desert { 45 } else { 14 }).then_some(tex::DRY_BRUSH),
        Ground::Rock(b) => {
            let lichen = if b == BASALT { 70 } else { 45 };
            if roll < lichen {
                Some(tex::LICHEN)
            } else if b == BASALT && roll < lichen + 25 {
                Some(tex::DRY_BRUSH)
            } else {
                None
            }
        }
    }
}

/// Cover on grass, by biome: (tufts, ferns, flowers) in thousandths of the columns, and the flowers it takes.
fn turf(biome: Biome, roll: u32, kind: u32, meadow: bool) -> Option<u16> {
    const PLAINS: [u16; 3] = [tex::DAISY, tex::POPPY, tex::BLUEBELL];
    const WET: [u16; 2] = [tex::BLUEBELL, tex::DAISY];
    const HILLS: [u16; 2] = [tex::BLUEBELL, tex::BLUEBELL];
    let (tufts, ferns, flowers, kinds): (u32, u32, u32, &[u16]) = match biome {
        Biome::Plains => (130, 0, 14, &PLAINS),
        Biome::Lowlands => (90, 80, 10, &WET),
        Biome::Highlands => (70, 40, 12, &HILLS),
        Biome::Desert => (20, 0, 0, &[]),
        Biome::BasaltFields => (40, 0, 0, &[]),
    };
    let flowers = flowers * if meadow { MEADOW_BOOST } else { 1 };
    if roll < tufts {
        Some(tex::GRASS_TUFT)
    } else if roll < tufts + ferns {
        Some(tex::FERN)
    } else if roll < tufts + ferns + flowers {
        Some(kinds[kind as usize % kinds.len()])
    } else {
        None
    }
}

#[cfg(test)]
mod tests;
