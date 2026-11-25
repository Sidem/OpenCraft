//! Seismic block classification, tier masking and subsurface anomaly clustering (`seismic/mod.rs`).
//! Pure read-only helpers: never create deposit tracking state or mutate the core.

use rustc_hash::FxHashMap;

use crate::block::{
    BlockId, AIR, BASALT, BAUXITE_ORE, BEDROCK, CLAY, COAL_ORE, COPPER_ORE, DIRT, GAS_SHALE, GRANITE, GRASS, IRON_ORE,
    KAOLIN, LIMESTONE, LIQUID, OIL_SAND, QUARTZ_ORE, SAND, SANDSTONE, SOLID, SPENT_ROCK, STONE, URANIUM_ORE,
};
use crate::chunk::{Chunk, CHUNK_MASK};
use crate::math::{sort_small_by_key, IVec3};
use crate::prospect::FILTER_ORES;
use crate::world::chunk_of;

use super::{
    ANOMALY_FIELDS, CELL_BYTES, CLASS_BEDROCK, CLASS_CAVE, CLASS_ORE_CARBON, CLASS_ORE_METAL, CLASS_ORE_MINERAL,
    CLASS_ORE_UNKNOWN, CLASS_ROCK, CLASS_SOIL, CLASS_STRUCTURE, CLASS_TUNNEL, CLASS_WATER, SLICE_WIDTH,
};

const MAX_ANOMALIES: usize = 6;

pub(super) fn ore_detail(b: BlockId) -> u8 {
    FILTER_ORES.iter().position(|&o| o == b).map_or(0, |i| (i + 1) as u8)
}

pub(super) fn classify_block(
    b: BlockId,
    p: IVec3,
    surf: i32,
    factory: &crate::factory::Factory,
    world: &mut crate::world::World,
    orig_cache: &mut FxHashMap<IVec3, Chunk>,
) -> (f32, u8, u8) {
    if b == BEDROCK || p.y == 0 {
        return (1.0, CLASS_BEDROCK, 0);
    }
    if LIQUID[b as usize] {
        return (0.20, CLASS_WATER, 0);
    }
    if factory.anchor_of(p).is_some() {
        return (0.74, CLASS_STRUCTURE, 0);
    }
    if b == AIR {
        let is_tunnel = p.y <= surf && was_originally_solid(p, world, orig_cache);
        return if is_tunnel { (0.08, CLASS_TUNNEL, 0) } else { (0.04, CLASS_CAVE, 0) };
    }
    let det = ore_detail(b);
    if det > 0 {
        let (imp, cls) = match b {
            COAL_ORE => (0.44, CLASS_ORE_CARBON),
            LIMESTONE => (0.66, CLASS_ORE_MINERAL),
            QUARTZ_ORE => (0.78, CLASS_ORE_MINERAL),
            OIL_SAND => (0.74, CLASS_ORE_CARBON),
            BAUXITE_ORE => (0.80, CLASS_ORE_METAL),
            COPPER_ORE => (0.83, CLASS_ORE_METAL),
            IRON_ORE => (0.86, CLASS_ORE_METAL),
            GAS_SHALE => (0.90, CLASS_ORE_CARBON),
            URANIUM_ORE => (0.96, CLASS_ORE_METAL),
            CLAY => (0.36, CLASS_ORE_MINERAL),
            KAOLIN => (0.42, CLASS_ORE_MINERAL),
            _ => (0.80, CLASS_ORE_UNKNOWN),
        };
        return (imp, cls, det);
    }
    if matches!(b, STONE | GRANITE | SANDSTONE | BASALT | SPENT_ROCK) {
        let imp = match b {
            SANDSTONE => 0.46,
            SPENT_ROCK => 0.49,
            STONE => 0.54,
            GRANITE => 0.61,
            BASALT => 0.66,
            _ => 0.54,
        };
        return (imp, CLASS_ROCK, 0);
    }
    if matches!(b, DIRT | GRASS | SAND) || !SOLID[b as usize] {
        return (0.31, CLASS_SOIL, 0);
    }
    if was_modified_block(p, b, world, orig_cache) {
        return (0.72, CLASS_STRUCTURE, 0);
    }
    (0.35, CLASS_SOIL, 0)
}

fn was_originally_solid(p: IVec3, world: &mut crate::world::World, cache: &mut FxHashMap<IVec3, Chunk>) -> bool {
    let c = chunk_of(p);
    if !world.chunk_modified(c) {
        return false;
    }
    let orig = cache.entry(c).or_insert_with(|| world.original_chunk(c));
    let (lx, ly, lz) = ((p.x & CHUNK_MASK) as usize, (p.y & CHUNK_MASK) as usize, (p.z & CHUNK_MASK) as usize);
    SOLID[orig.get(lx, ly, lz) as usize]
}

fn was_modified_block(
    p: IVec3,
    now: BlockId,
    world: &mut crate::world::World,
    cache: &mut FxHashMap<IVec3, Chunk>,
) -> bool {
    let c = chunk_of(p);
    if !world.chunk_modified(c) {
        return false;
    }
    let orig = cache.entry(c).or_insert_with(|| world.original_chunk(c));
    let (lx, ly, lz) = ((p.x & CHUNK_MASK) as usize, (p.y & CHUNK_MASK) as usize, (p.z & CHUNK_MASK) as usize);
    orig.get(lx, ly, lz) != now
}

/// Applies tier-based information masking: Mk1 shows acoustic density bands and caves, Mk2 separates tunnels and
/// mineral families, Mk3 exposes confident ore identification.
pub(super) fn mask_for_tier(cls: u8, det: u8, imp: f32, tier: u8) -> (u8, u8) {
    match tier {
        0 => match cls {
            CLASS_ORE_METAL | CLASS_ORE_CARBON | CLASS_ORE_MINERAL | CLASS_ORE_UNKNOWN => (CLASS_ORE_UNKNOWN, 0),
            CLASS_TUNNEL => (CLASS_CAVE, 0),
            CLASS_STRUCTURE => (CLASS_ROCK, 0),
            CLASS_ROCK if imp > 0.80 => (CLASS_ORE_UNKNOWN, 0),
            other => (other, 0),
        },
        1 => (cls, 0),
        _ => (cls, det),
    }
}

/// Finds up to [`MAX_ANOMALIES`] strongest distinct subsurface clusters from the tier-masked cross-section.
pub(super) fn detect_anomalies(cells: &[u8], surface: &[i32; SLICE_WIDTH], h: usize) -> Vec<i32> {
    let mut visited = vec![false; SLICE_WIDTH * h];
    let mut found: Vec<(i32, [i32; ANOMALY_FIELDS])> = Vec::new();
    let mut queue = Vec::new();

    for y in 1..h {
        for (s, &surf_y) in surface.iter().enumerate() {
            let idx = y * SLICE_WIDTH + s;
            if visited[idx] {
                continue;
            }
            let cls = cells[idx * CELL_BYTES + 2];
            if !is_notable(cls, y as i32, surf_y) {
                continue;
            }
            let bucket = anomaly_bucket(cls);
            visited[idx] = true;
            queue.clear();
            queue.push((s, y));
            let (mut sum_s, mut sum_y, mut count, mut max_imp, mut best_cls, mut best_det) =
                (0i32, 0i32, 0i32, 0u8, cls, cells[idx * CELL_BYTES + 3]);

            while let Some((cs, cy)) = queue.pop() {
                let cidx = cy * SLICE_WIDTH + cs;
                let imp = cells[cidx * CELL_BYTES];
                let ccls = cells[cidx * CELL_BYTES + 2];
                let cdet = cells[cidx * CELL_BYTES + 3];
                sum_s += cs as i32;
                sum_y += cy as i32;
                count += 1;
                if imp >= max_imp {
                    max_imp = imp;
                    best_cls = ccls;
                    if cdet > 0 {
                        best_det = cdet;
                    }
                }
                for (ds, dy) in [(-1i32, 0i32), (1, 0), (0, -1), (0, 1)] {
                    let (ns, ny) = (cs as i32 + ds, cy as i32 + dy);
                    if ns < 0 || ns >= SLICE_WIDTH as i32 || ny < 1 || ny >= h as i32 {
                        continue;
                    }
                    let nidx = ny as usize * SLICE_WIDTH + ns as usize;
                    if visited[nidx] {
                        continue;
                    }
                    let ncls = cells[nidx * CELL_BYTES + 2];
                    if anomaly_bucket(ncls) == bucket && is_notable(ncls, ny, surface[ns as usize]) {
                        visited[nidx] = true;
                        queue.push((ns as usize, ny as usize));
                    }
                }
            }

            let min_cells = if bucket == 1 { 4 } else { 8 };
            if count < min_cells {
                continue;
            }
            let strength =
                if bucket == 1 { (i32::from(max_imp) * 100 / 255).clamp(20, 100) } else { (count * 3).min(100) };
            let priority = count
                * if bucket == 1 {
                    4
                } else if bucket == 3 {
                    3
                } else {
                    2
                };
            found.push((-priority, [best_cls as i32, sum_s / count, sum_y / count, strength, best_det as i32, count]));
        }
    }

    sort_small_by_key(&mut found, |f| f.0);
    found.into_iter().take(MAX_ANOMALIES).flat_map(|(_, row)| row).collect()
}

fn is_notable(cls: u8, y: i32, surf: i32) -> bool {
    match cls {
        CLASS_ORE_UNKNOWN | CLASS_ORE_METAL | CLASS_ORE_CARBON | CLASS_ORE_MINERAL => true,
        CLASS_TUNNEL | CLASS_CAVE | CLASS_WATER => y <= surf - 2,
        _ => false,
    }
}

fn anomaly_bucket(cls: u8) -> u8 {
    match cls {
        CLASS_ORE_UNKNOWN | CLASS_ORE_METAL | CLASS_ORE_CARBON | CLASS_ORE_MINERAL => 1,
        CLASS_CAVE => 2,
        CLASS_TUNNEL => 3,
        CLASS_WATER => 4,
        _ => 0,
    }
}
