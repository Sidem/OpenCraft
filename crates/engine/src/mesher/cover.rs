//! Ground cover (R2a): grass tufts, flowers, ferns, reeds, dry brush and lichen drawn as plants on top of
//! natural ground, chosen per biome by a hash of the world column. Presentation only: never blocks, never
//! saved, not in the state hash; a remesh redraws it, so it follows the ground. Only open sky gets any
//! (sky light at least `SKY_MIN`), so floors under roofs and in caves stay bare.
//!
//! Invariants: a pure function of the chunk's blocks, light, biomes, world position and `Cover::cleared`; generator
//! version 1 has no biomes and so no cover. To add a plant: a texture (`block::tex`,
//! `textures/hires.rs`) and a rule in [`pick`]. Over stained ground (`block::RUSTY_SOIL..`) grow the indicator
//! plants (R2c): each stain has its own, so a patch of one tells the ore below before anyone points at it.
//! A column the player cleared with the Bladerang (U3, `world/cover_clear.rs`, which is core state) grows nothing: one
//! bit test per candidate column, and an empty `cleared` (a chunk column never touched) costs no more than that.

use super::foliage::{sheds_litter, species_of};
use super::shaped::{pack, Shaped, Vert, KIND_CARD};
use super::{neighbor_index, pidx, Mesher, N, P, P2};
use crate::block::{
    tex, BlockId, AIR, BASALT, DIRT, FACE_TEX, GRANITE, GRASS, LIQUID, PALE_SAND, PALE_SOIL, RUSTY_SAND, RUSTY_SOIL,
    SAND,
};
use crate::chunk::{Chunk, HOLDS_LEAVES};
use crate::math::hash2;
use crate::worldgen::Biome;

mod shape;

/// What a chunk needs to grow cover: its biomes and its lowest world corner.
pub struct Cover<'a> {
    pub x0: i32,
    pub z0: i32,
    /// 32 × 32, z rows, x fastest (`WorldGen::biomes`).
    pub biomes: &'a [Biome],
    /// The columns the player cleared: 32 rows by z, bit x of a row set for a cleared column; empty for none.
    pub cleared: &'a [u32],
}

/// Least sky light (of 15) a cell needs to grow ground cover, and under a shaded canopy for fallen leaf litter.
const SKY_MIN: u8 = 12;
const LITTER_SKY_MIN: u8 = 6;
const SEED: u32 = 0xC0FE_7A1F;
const LITTER_SEED: u32 = 0xFA11_1EAF;
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

/// Whether plants can grow over block `b`: what the Bladerang cuts under its loop (`bladerang.rs`) and what a clear looks for.
pub fn bears_cover(b: BlockId) -> bool {
    ground_of(b).is_some()
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
    pub(super) fn mesh_cover(&mut self, n: &[&Chunk; 27], light: &[u8], cover: &Cover) {
        if cover.biomes.len() != N * N {
            return;
        }
        let has_litter = has_shedding_leaves(&self.pad, n);
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
                    let sky = light[p] & 15;
                    if sky < LITTER_SKY_MIN {
                        continue;
                    }
                    if cover.cleared.get(z).is_some_and(|row| row >> x & 1 != 0) {
                        break;
                    }
                    if has_litter && matches!(ground, Ground::Turf | Ground::Dirt) {
                        if let Some((leaf_b, direct)) = self.canopy_above(n, x as i32, y as i32, z as i32) {
                            push_litter(
                                &mut self.shaped,
                                [x as i32, y as i32, z as i32],
                                [wx, wz],
                                leaf_b,
                                direct,
                                light[p],
                            );
                        }
                    }
                    if sky < SKY_MIN {
                        continue;
                    }
                    let (h, meadow) = *rolls.get_or_insert_with(|| {
                        (hash2(SEED, wx, wz), hash2(SEED ^ 1, wx >> 2, wz >> 2) % 1000 < MEADOW_PERMILLE)
                    });
                    let wet = !matches!(ground, Ground::Rock(_)) && self.water_beside(p, y);
                    let stain = indicator_of(below).filter(|_| h % 1000 < INDICATOR_PERMILLE);
                    if let Some(layer) = stain.or_else(|| pick(cover.biomes[z * N + x], ground, h, meadow, wet)) {
                        let seed = h ^ (y as u32).wrapping_mul(0x9E37_79B9);
                        shape::push(&mut self.shaped, [x as i32, y as i32, z as i32], layer, light[p], seed);
                    }
                }
            }
        }
    }

    /// Block at chunk-local `(x, y, z)` where `x, z` in `-1..=32` and `y` in `0..64` (reads `pad` then upper chunks).
    fn block_at(&self, n: &[&Chunk; 27], x: i32, y: i32, z: i32) -> BlockId {
        if y < N as i32 {
            self.pad[pidx((x + 1) as usize, (y + 1) as usize, (z + 1) as usize)]
        } else {
            let map = |v: i32| {
                if v < 0 {
                    (-1, N - 1)
                } else if v >= N as i32 {
                    (1, 0)
                } else {
                    (0, v as usize)
                }
            };
            let (cx, lx) = map(x);
            let (cz, lz) = map(z);
            n[neighbor_index(cx, 1, cz)].get(lx, (y - N as i32) as usize, lz)
        }
    }

    /// The shedding leaf block overhead of `(x, y, z)` (`direct = true` straight above, `false` on the 1-block drip edge).
    fn canopy_above(&self, n: &[&Chunk; 27], x: i32, y: i32, z: i32) -> Option<(BlockId, bool)> {
        let top = (y + 16).min(2 * N as i32 - 1);
        for yy in (y + 2)..=top {
            let b = self.block_at(n, x, yy, z);
            if sheds_litter(b) {
                return Some((b, true));
            }
        }
        let edge_top = (y + 12).min(2 * N as i32 - 1);
        for (dx, dz) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
            for yy in (y + 2)..=edge_top {
                let b = self.block_at(n, x + dx, yy, z + dz);
                if sheds_litter(b) {
                    return Some((b, false));
                }
            }
        }
        None
    }

    /// Water at the ground level of a side neighbour of air cell `p` (chunk row `y`), or one lower: a bank.
    fn water_beside(&self, p: usize, y: usize) -> bool {
        let lowest = if y >= 1 { 2 } else { 1 };
        [1, -1, P as isize, -(P as isize)].iter().any(|&side| {
            (1..=lowest).any(|down| LIQUID[self.pad[(p as isize + side - down * P2 as isize) as usize] as usize])
        })
    }
}

/// Fast pre-check: whether the chunk or its upper neighbours hold any deciduous/shedding leaves at all. Shedding leaves
/// are leaves, so a neighbourhood without any (`Chunk::holds`, cached) needs no scan.
fn has_shedding_leaves(pad: &[BlockId], n: &[&Chunk; 27]) -> bool {
    if !n.iter().any(|c| c.holds(HOLDS_LEAVES)) {
        return false;
    }
    if pad.iter().any(|&b| sheds_litter(b)) {
        return true;
    }
    for dz in -1..=1 {
        for dx in -1..=1 {
            let c = n[neighbor_index(dx, 1, dz)];
            match c.dense() {
                Some(d) if d.iter().any(|&b| sheds_litter(b)) => return true,
                None if sheds_litter(c.get(0, 0, 0)) => return true,
                _ => {}
            }
        }
    }
    false
}

/// Emits one flat ground-hugging fallen-leaf scatter card (`KIND_CARD`, `is_litter = 1`), revealed in the shader when
/// its overhead species sheds (autumn leaves or spring cherry petals).
fn push_litter(out: &mut Shaped, cell: [i32; 3], world_xz: [i32; 2], leaf: BlockId, direct: bool, light: u8) {
    let h = hash2(LITTER_SEED, world_xz[0], world_xz[1]);
    let chance = if direct { 72 } else { 32 };
    if h % 100 >= chance {
        return;
    }
    let f = |shift: u32| ((h >> shift) & 31) as f32 / 31.0;
    let cx = 0.25 + f(5) * 0.50;
    let cz = 0.25 + f(10) * 0.50;
    let cy = 0.016 + ((h >> 15) & 7) as f32 * 0.0012;
    let half = 0.44 + f(18) * 0.22;
    let yaw = f(23) * std::f32::consts::TAU;
    let (s, c) = yaw.sin_cos();
    let (rx, rz) = (c * half, -s * half);
    let (fx, fz) = (s * half, c * half);
    let at = |su: f32, sv: f32| [cx + rx * su + fx * sv, cy, cz + rz * su + fz * sv];
    let part = (h >> 12) & 15;
    let drop_hi = (h >> 28) & 3;
    let span_bits = 9u32; // ~1.125 blocks of leaf texture across the scatter patch
    let corners = [(0, 0, at(-1.0, -1.0)), (1, 0, at(1.0, -1.0)), (1, 2, at(1.0, 1.0)), (0, 2, at(-1.0, 1.0))];
    let q = corners.map(|(u, v, off)| {
        let bits = u | v << 1 | part << 3 | span_bits << 7;
        let mut w = pack(&Vert {
            cell,
            off,
            normal: [0.0, 1.0, 0.0],
            u: bits as f32 / 256.0,
            kind: KIND_CARD,
            occlusion: 6,
            light,
            layer: 0,
        });
        w[0] |= drop_hi << 30;
        w
    });
    let packed_layer = (FACE_TEX[leaf as usize][0] & 0x7FF) | (u16::from(species_of(leaf)) << 11) | (1u16 << 15);
    out.packed(true, q, packed_layer);
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

/// Cover on grass, by biome: (tufts, ferns, dry brush, flowers) in thousandths of the columns, and the flowers it
/// takes.
fn turf(biome: Biome, roll: u32, kind: u32, meadow: bool) -> Option<u16> {
    const PLAINS: [u16; 3] = [tex::DAISY, tex::POPPY, tex::BLUEBELL];
    const WET: [u16; 2] = [tex::BLUEBELL, tex::DAISY];
    const HILLS: [u16; 2] = [tex::BLUEBELL, tex::BLUEBELL];
    const DRY: [u16; 1] = [tex::POPPY];
    // Grassland is lush (docs/ART_STYLE.md, "The key-art look"): about a quarter of plains columns carry a tuft.
    let (tufts, ferns, brush, flowers, kinds): (u32, u32, u32, u32, &[u16]) = match biome {
        Biome::Plains => (250, 0, 0, 26, &PLAINS),
        Biome::Lowlands => (150, 80, 0, 16, &WET),
        Biome::Highlands => (100, 40, 0, 16, &HILLS),
        Biome::Desert => (20, 0, 0, 0, &[]),
        Biome::BasaltFields => (40, 0, 0, 0, &[]),
        Biome::Savanna => (300, 0, 90, 6, &DRY),
        Biome::Boreal => (110, 110, 0, 10, &HILLS),
        Biome::Woodland => (220, 20, 30, 14, &PLAINS),
        Biome::BambooForest => (120, 150, 0, 10, &WET),
        Biome::Redwoods => (60, 200, 0, 8, &HILLS),
    };
    let flowers = flowers * if meadow { MEADOW_BOOST } else { 1 };
    let ferns = tufts + ferns;
    let brush = ferns + brush;
    if roll < tufts {
        Some(tex::GRASS_TUFT)
    } else if roll < ferns {
        Some(tex::FERN)
    } else if roll < brush {
        Some(tex::DRY_BRUSH)
    } else if roll < brush + flowers {
        Some(kinds[kind as usize % kinds.len()])
    } else {
        None
    }
}

#[cfg(test)]
mod tests;
