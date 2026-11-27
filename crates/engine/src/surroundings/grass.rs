//! Nearby lit turf cells around the camera for GPU-instanced 3D grass blades (`web/src/render/grass.ts`).
//! Presentation only: reads loaded chunks, cached light, bare sod and cleared cover; never generates chunks or touches
//! core state. Polled by the host when the camera moves or on a quiet interval (`Game::grass_cells`).
//!
//! Output layout (`Vec<u32>`):
//! - words `0..3`: origin `(ox, oy, oz)` as `i32` bit patterns (`eye.floor() - 32`).
//! - words `3..`: up to [`MAX_CELLS`] packed turf cells, one `u32` each:
//!   - bits `0..7`: `x - ox` (`0..=64`)
//!   - bits `7..14`: `y - oy` (`0..=64`)
//!   - bits `14..21`: `z - oz` (`0..=64`)
//!   - bits `21..29`: light byte at `(x, y + 1, z)` (`sky | block << 4`)
//!   - bits `29..32`: tone class (`0` plains, `1` lush forest/lowland, `2` dry savanna, `3..=6` ore stains, `7` alpine)

use crate::block::{is_turf, BlockId, AIR, DARK_SOIL, GREEN_SOIL, PALE_SOIL, RUSTY_SOIL};
use crate::chunk::{index, CHUNK_SHIFT, CHUNK_SIZE};
use crate::math::{sort_small_by_key, IVec3, Vec3};
use crate::world::World;
use crate::worldgen::{Biome, WORLD_HEIGHT};

/// Horizontal and vertical reach around the camera, in blocks (fits within `0..=64` around `eye - 32`).
pub const RADIUS: i32 = 28;
pub const VERT: i32 = 24;
/// Maximum grass blocks returned (fits in a 64 × 64 `R32UI` texture on the GPU).
pub const MAX_CELLS: usize = 4096;
/// Same sky-light threshold `mesher/face_layer.rs` uses to draw turf green.
const GREEN_SKY: u8 = 8;
/// Top face (`+Y`) bit in `Chunk::bares` (`mesher::FACES` index 2).
const TOP_FACE_BIT: u8 = 1 << 2;

/// Collects nearby lit, open, non-cleared, grown turf blocks around `eye` for the instanced grass pass.
pub fn grass_cells(world: &mut World, eye: Vec3) -> Vec<u32> {
    let center = eye.floor();
    let (ox, oy, oz) = (center.x - 32, center.y - 32, center.z - 32);
    let min_x = center.x - RADIUS;
    let max_x = center.x + RADIUS;
    let min_y = (center.y - VERT).max(0);
    let max_y = (center.y + VERT).min(WORLD_HEIGHT - 2);
    let min_z = center.z - RADIUS;
    let max_z = center.z + RADIUS;

    // Visit overlapping loaded chunks nearest-first so dense multi-level terrain always keeps the closest turf.
    let (cx0, cx1) = (min_x >> CHUNK_SHIFT, max_x >> CHUNK_SHIFT);
    let (cy0, cy1) = (min_y >> CHUNK_SHIFT, max_y >> CHUNK_SHIFT);
    let (cz0, cz1) = (min_z >> CHUNK_SHIFT, max_z >> CHUNK_SHIFT);
    let (ecx, ecy, ecz) = (center.x >> CHUNK_SHIFT, center.y >> CHUNK_SHIFT, center.z >> CHUNK_SHIFT);
    let mut chunks: Vec<IVec3> = Vec::with_capacity(((cx1 - cx0 + 1) * (cy1 - cy0 + 1) * (cz1 - cz0 + 1)) as usize);
    for cy in cy0..=cy1 {
        for cz in cz0..=cz1 {
            for cx in cx0..=cx1 {
                chunks.push(IVec3::new(cx, cy, cz));
            }
        }
    }
    sort_small_by_key(&mut chunks, |c| {
        let (dx, dy, dz) = (c.x - ecx, c.y - ecy, c.z - ecz);
        dx * dx + dy * dy + dz * dz
    });

    let r2 = RADIUS * RADIUS;
    let mut raw: Vec<(i32, i32, i32, u8)> = Vec::with_capacity(2048);
    for c in chunks {
        if raw.len() >= MAX_CELLS {
            break;
        }
        let Some(chunk) = world.loaded_chunk(c) else { continue };
        if let Some(u) = chunk.as_uniform() {
            if !is_turf(u) {
                continue;
            }
        }
        let (bx, by, bz) = (c.x << CHUNK_SHIFT, c.y << CHUNK_SHIFT, c.z << CHUNK_SHIFT);
        let lx0 = (min_x - bx).max(0) as usize;
        let lx1 = (max_x - bx).min(CHUNK_SIZE - 1) as usize;
        let ly0 = (min_y - by).max(0) as usize;
        let ly1 = (max_y - by).min(CHUNK_SIZE - 1) as usize;
        let lz0 = (min_z - bz).max(0) as usize;
        let lz1 = (max_z - bz).min(CHUNK_SIZE - 1) as usize;
        let dense = chunk.dense();
        let uniform = chunk.get(0, 0, 0);
        let has_bares = !chunk.bares().is_empty();
        for lz in lz0..=lz1 {
            let wz = bz + lz as i32;
            let dz = wz - center.z;
            for lx in lx0..=lx1 {
                let wx = bx + lx as i32;
                let dx = wx - center.x;
                if dx * dx + dz * dz > r2 || world.is_cover_cleared(wx, wz) {
                    continue;
                }
                for ly in ly0..=ly1 {
                    let b = match dense {
                        Some(d) => d[index(lx, ly, lz)],
                        None => uniform,
                    };
                    if !is_turf(b) {
                        continue;
                    }
                    let wy = by + ly as i32;
                    if world.get_block(IVec3::new(wx, wy + 1, wz)) != Some(AIR) {
                        continue;
                    }
                    if has_bares && chunk.get_bare(lx, ly, lz) & TOP_FACE_BIT != 0 {
                        continue;
                    }
                    let tone = tone_of(b, world.generator().biome_at(wx, wz, wy));
                    raw.push((wx, wy, wz, tone));
                    if raw.len() >= MAX_CELLS {
                        break;
                    }
                }
                if raw.len() >= MAX_CELLS {
                    break;
                }
            }
            if raw.len() >= MAX_CELLS {
                break;
            }
        }
    }

    let mut out = Vec::with_capacity(3 + raw.len());
    out.push(ox as u32);
    out.push(oy as u32);
    out.push(oz as u32);
    for (wx, wy, wz, tone) in raw {
        let light = world.light_at(IVec3::new(wx, wy + 1, wz));
        if (light & 15) < GREEN_SKY {
            continue;
        }
        let packed = ((wx - ox) as u32 & 127)
            | (((wy - oy) as u32 & 127) << 7)
            | (((wz - oz) as u32 & 127) << 14)
            | (u32::from(light) << 21)
            | (u32::from(tone & 7) << 29);
        out.push(packed);
    }
    out
}

fn tone_of(b: BlockId, biome: Biome) -> u8 {
    match b {
        RUSTY_SOIL => 3,
        DARK_SOIL => 4,
        GREEN_SOIL => 5,
        PALE_SOIL => 6,
        _ => match biome {
            Biome::Lowlands | Biome::Woodland | Biome::BambooForest | Biome::Redwoods => 1,
            Biome::Savanna | Biome::Desert | Biome::BasaltFields => 2,
            Biome::Boreal | Biome::Highlands => 7,
            Biome::Plains => 0,
        },
    }
}
