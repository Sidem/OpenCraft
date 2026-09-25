//! Deterministic terrain generation: heightmap terrain, cliffs, spaghetti caves and trees. Ore
//! deposits are placed and stamped by `ore.rs`.
//!
//! Every feature that can cross a chunk border (trees, ore deposits) is derived from hashes of the
//! *source* cell, and each chunk stamps whatever part of its neighbours' features overlaps it. That
//! keeps generation order-independent, which is what later lets it move to worker threads.
//! Per-column data (heights, surface, trees, deposits) is cached in `columns` and shared by the
//! column's 8 vertical chunks. **Any change to generated output for a given seed must bump
//! [`WORLDGEN_VERSION`]:** saves store only edited chunks and regenerate the rest.

mod ore;

use std::rc::Rc;

use rustc_hash::FxHashMap;

use crate::block::*;
use crate::chunk::{index, Chunk, CHUNK_SIZE, CHUNK_VOLUME};
use crate::deposits::Deposit;
use crate::math::{hash2, hash3, smoothstep, unit, IVec3};
use crate::noise::Perlin;

/// Saves record this and refuse to load under a different one, since their untouched terrain and
/// deposits would come back different under the player's edits.
pub const WORLDGEN_VERSION: u32 = 1;
pub const WORLD_HEIGHT_CHUNKS: i32 = 8;
pub const WORLD_HEIGHT: i32 = WORLD_HEIGHT_CHUNKS * CHUNK_SIZE;
const SAND_LEVEL: i32 = 60;
const ROCK_LEVEL: i32 = 170;
const CLIFF_SLOPE: i32 = 5;
const TREE_CELL: i32 = 7;
const TREE_REACH: i32 = 2;
const SPAWN_CLEARING: i32 = 6;

pub struct WorldGen {
    seed: u32,
    continent: Perlin,
    hills: Perlin,
    ridges: Perlin,
    forest: Perlin,
    cave_a: Perlin,
    cave_b: Perlin,
    columns: FxHashMap<(i32, i32), Rc<Column>>,
}

impl WorldGen {
    pub fn new(seed: u32) -> Self {
        let s = seed as u64;
        Self {
            seed,
            continent: Perlin::new(s ^ 0x01),
            hills: Perlin::new(s ^ 0x02),
            ridges: Perlin::new(s ^ 0x03),
            forest: Perlin::new(s ^ 0x04),
            cave_a: Perlin::new(s ^ 0x05),
            cave_b: Perlin::new(s ^ 0x06),
            columns: FxHashMap::default(),
        }
    }

    pub fn seed(&self) -> u32 {
        self.seed
    }

    /// Terrain surface height (y of the top solid block) at a world column.
    pub fn height_at(&self, x: i32, z: i32) -> i32 {
        let (fx, fz) = (x as f64, z as f64);
        let c = self.continent.fbm2(fx / 900.0, fz / 900.0, 4);
        let hills = self.hills.fbm2(fx / 170.0, fz / 170.0, 5);
        let r = self.ridges.fbm2(fx / 360.0, fz / 360.0, 4);
        let ridge = (1.0 - r.abs() * 2.4).max(0.0);
        let mountains = ridge * ridge * smoothstep(-0.05, 0.3, c) * 100.0;
        let h = 68.0 + c * 50.0 + hills * 22.0 + mountains;
        (h.round() as i32).clamp(4, WORLD_HEIGHT - 16)
    }

    fn surface_for(h: i32, slope: i32) -> (BlockId, BlockId) {
        if slope >= CLIFF_SLOPE || h > ROCK_LEVEL {
            (STONE, STONE)
        } else if h <= SAND_LEVEL {
            (SAND, SAND)
        } else {
            (GRASS, DIRT)
        }
    }

    fn slope_at(&self, x: i32, z: i32) -> i32 {
        let dx = (self.height_at(x + 1, z) - self.height_at(x - 1, z)).abs();
        let dz = (self.height_at(x, z + 1) - self.height_at(x, z - 1)).abs();
        dx.max(dz)
    }

    /// Drops cached column data the streamer no longer needs.
    pub fn retain_columns(&mut self, mut keep: impl FnMut(i32, i32) -> bool) {
        self.columns.retain(|&(x, z), _| keep(x, z));
    }

    fn column(&mut self, cx: i32, cz: i32) -> Rc<Column> {
        if let Some(c) = self.columns.get(&(cx, cz)) {
            return c.clone();
        }
        let col = Rc::new(self.build_column(cx, cz));
        self.columns.insert((cx, cz), col.clone());
        col
    }

    fn build_column(&self, cx: i32, cz: i32) -> Column {
        const P: usize = 34;
        let (x0, z0) = (cx * CHUNK_SIZE, cz * CHUNK_SIZE);
        let mut hm = vec![0i32; P * P];
        for z in 0..P {
            for x in 0..P {
                hm[z * P + x] = self.height_at(x0 + x as i32 - 1, z0 + z as i32 - 1);
            }
        }
        let mut heights = vec![0i32; 1024];
        let mut surface = vec![(AIR, AIR); 1024];
        let mut max_ground = 0;
        for z in 0..32 {
            for x in 0..32 {
                let h = hm[(z + 1) * P + x + 1];
                let slope = (hm[(z + 1) * P + x + 2] - hm[(z + 1) * P + x])
                    .abs()
                    .max((hm[(z + 2) * P + x + 1] - hm[z * P + x + 1]).abs());
                heights[z * 32 + x] = h;
                surface[z * 32 + x] = Self::surface_for(h, slope);
                max_ground = max_ground.max(h);
            }
        }
        let trees = self.trees_near(x0, z0);
        let max_y = trees.iter().map(|t| t.ground + t.trunk + 2).fold(max_ground, i32::max);
        let deposits = self.deposits_for_column(cx, cz);
        Column { heights, surface, trees, deposits, max_y, max_ground }
    }

    /// Trees whose trunk or canopy can overlap the column starting at (x0, z0).
    /// One candidate per TREE_CELL² cell keeps trunks apart without any neighbour search.
    fn trees_near(&self, x0: i32, z0: i32) -> Vec<Tree> {
        let (min_x, max_x) = (x0 - TREE_REACH, x0 + CHUNK_SIZE - 1 + TREE_REACH);
        let (min_z, max_z) = (z0 - TREE_REACH, z0 + CHUNK_SIZE - 1 + TREE_REACH);
        let mut out = Vec::new();
        for gz in min_z.div_euclid(TREE_CELL)..=max_z.div_euclid(TREE_CELL) {
            for gx in min_x.div_euclid(TREE_CELL)..=max_x.div_euclid(TREE_CELL) {
                let h = hash2(self.seed ^ 0x7EE5, gx, gz);
                let x = gx * TREE_CELL + 1 + (h % 5) as i32;
                let z = gz * TREE_CELL + 1 + ((h >> 8) % 5) as i32;
                if x < min_x || x > max_x || z < min_z || z > max_z {
                    continue;
                }
                if x.abs() < SPAWN_CLEARING && z.abs() < SPAWN_CLEARING {
                    continue;
                }
                let f = self.forest.fbm2(x as f64 / 220.0, z as f64 / 220.0, 3);
                let density = smoothstep(-0.2, 0.3, f) * 0.85 + 0.03;
                if unit(hash2(self.seed ^ 0x5EED, gx, gz)) >= density {
                    continue;
                }
                let ground = self.height_at(x, z);
                if Self::surface_for(ground, self.slope_at(x, z)).0 != GRASS {
                    continue;
                }
                out.push(Tree { x, z, ground, trunk: 4 + ((h >> 16) % 3) as i32 });
            }
        }
        out
    }

    pub fn generate(&mut self, pos: IVec3) -> Chunk {
        if pos.y < 0 || pos.y >= WORLD_HEIGHT_CHUNKS {
            return Chunk::uniform(AIR);
        }
        let col = self.column(pos.x, pos.z);
        let base = IVec3::new(pos.x * CHUNK_SIZE, pos.y * CHUNK_SIZE, pos.z * CHUNK_SIZE);
        if base.y > col.max_y {
            return Chunk::uniform(AIR);
        }

        let mut b = vec![AIR; CHUNK_VOLUME];
        let caves = (base.y < col.max_ground).then(|| CaveField::new(self, base));

        for z in 0..32usize {
            for x in 0..32usize {
                let h = col.heights[z * 32 + x];
                let (top, filler) = col.surface[z * 32 + x];
                let (wx, wz) = (base.x + x as i32, base.z + z as i32);
                let y_end = h.min(base.y + CHUNK_SIZE - 1);
                for wy in base.y..=y_end {
                    let depth = h - wy;
                    // Solid bedrock at y = 0, thinning out over the next three layers.
                    let id = if wy == 0 || (wy <= 3 && hash3(self.seed, wx, wy, wz) % (wy as u32 + 1) == 0) {
                        BEDROCK
                    } else if depth > 5
                        && wy > 4
                        && caves.as_ref().is_some_and(|c| c.is_cave(x, (wy - base.y) as usize, z))
                    {
                        AIR
                    } else if depth == 0 {
                        top
                    } else if depth <= 3 {
                        filler
                    } else {
                        STONE
                    };
                    b[index(x, (wy - base.y) as usize, z)] = id;
                }
            }
        }

        if base.y < col.max_ground {
            let top = base + IVec3::new(CHUNK_SIZE - 1, CHUNK_SIZE - 1, CHUNK_SIZE - 1);
            for d in col.deposits.iter().filter(|d| d.intersects(base, top)) {
                ore::stamp_deposit(d, base, &mut b);
            }
        }
        for t in &col.trees {
            stamp_tree(t, base, self.seed, &mut b);
        }
        Chunk::from_blocks(b)
    }
}

struct Tree {
    x: i32,
    z: i32,
    ground: i32,
    trunk: i32,
}

/// Per chunk-column data shared by all 8 vertical chunks of that column.
struct Column {
    heights: Vec<i32>,
    surface: Vec<(BlockId, BlockId)>,
    trees: Vec<Tree>,
    /// Deposits (seeded here or in a neighbouring column) whose shape reaches into this column,
    /// sorted by key: stamping and ownership lookups both walk them in this order.
    deposits: Vec<Deposit>,
    /// Highest non-air voxel in the column, including tree canopies.
    max_y: i32,
    max_ground: i32,
}

fn stamp_tree(t: &Tree, base: IVec3, seed: u32, b: &mut [BlockId]) {
    let mut put = |x: i32, y: i32, z: i32, id: BlockId, replace_solid: bool| {
        let (lx, ly, lz) = (x - base.x, y - base.y, z - base.z);
        if !(0..CHUNK_SIZE).contains(&lx) || !(0..CHUNK_SIZE).contains(&ly) || !(0..CHUNK_SIZE).contains(&lz) {
            return;
        }
        let i = index(lx as usize, ly as usize, lz as usize);
        if replace_solid || b[i] == AIR {
            b[i] = id;
        }
    };

    let top = t.ground + t.trunk;
    // Canopy: two wide layers, then two narrow ones; corners are randomly trimmed.
    for y in (top - 2)..=(top + 1) {
        let r: i32 = if y < top { 2 } else { 1 };
        for dz in -r..=r {
            for dx in -r..=r {
                let corner = dx.abs() == r && dz.abs() == r;
                if corner && (y == top + 1 || hash3(seed ^ 0x1EAF, t.x + dx, y, t.z + dz) & 1 == 0) {
                    continue;
                }
                put(t.x + dx, y, t.z + dz, LEAVES, false);
            }
        }
    }
    for y in (t.ground + 1)..=top {
        put(t.x, y, t.z, LOG, true);
    }
    put(t.x, t.ground, t.z, DIRT, true);
}

/// Spaghetti caves: tunnels where two independent noise fields are both near zero.
/// Noise is sampled on a 4-block lattice and trilinearly interpolated (~45x fewer noise calls).
struct CaveField {
    a: [f32; 729],
    b: [f32; 729],
}

impl CaveField {
    fn new(g: &WorldGen, base: IVec3) -> Self {
        let mut f = CaveField { a: [0.0; 729], b: [0.0; 729] };
        for iy in 0..9 {
            for iz in 0..9 {
                for ix in 0..9 {
                    let x = (base.x + ix * 4) as f64;
                    let y = (base.y + iy * 4) as f64;
                    let z = (base.z + iz * 4) as f64;
                    let i = ((iy * 9 + iz) * 9 + ix) as usize;
                    f.a[i] = g.cave_a.noise3(x / 52.0, y / 30.0, z / 52.0) as f32;
                    f.b[i] = g.cave_b.noise3(x / 52.0, y / 30.0, z / 52.0) as f32;
                }
            }
        }
        f
    }

    #[inline]
    fn is_cave(&self, x: usize, y: usize, z: usize) -> bool {
        let (ix, iy, iz) = (x >> 2, y >> 2, z >> 2);
        let (tx, ty, tz) = ((x & 3) as f32 * 0.25, (y & 3) as f32 * 0.25, (z & 3) as f32 * 0.25);
        let s = |f: &[f32; 729]| {
            let at = |dx: usize, dy: usize, dz: usize| f[((iy + dy) * 9 + iz + dz) * 9 + ix + dx];
            let l = |a: f32, b: f32, t: f32| a + (b - a) * t;
            let c00 = l(at(0, 0, 0), at(1, 0, 0), tx);
            let c10 = l(at(0, 1, 0), at(1, 1, 0), tx);
            let c01 = l(at(0, 0, 1), at(1, 0, 1), tx);
            let c11 = l(at(0, 1, 1), at(1, 1, 1), tx);
            l(l(c00, c10, ty), l(c01, c11, ty), tz)
        };
        let (a, b) = (s(&self.a), s(&self.b));
        a * a + b * b < 0.0045
    }
}

#[cfg(test)]
mod tests;
