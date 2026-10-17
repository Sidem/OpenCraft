//! Deterministic terrain generation: heightmap terrain, cliffs, spaghetti caves and trees. Ore
//! deposits are placed and stamped by `ore.rs`.
//!
//! Every feature that can cross a chunk border (trees, ore deposits) is derived from hashes of the
//! *source* cell, and each chunk stamps whatever part of its neighbours' features overlaps it. That
//! keeps generation order-independent, which is what later lets it move to worker threads.
//! Per-column data (heights, surface, trees, deposits) is cached in `columns` and shared by the
//! column's 8 vertical chunks.
//!
//! Versions: saves store only edited chunks and regenerate the rest, so a world keeps the generator
//! version it was made with (`WorldGen::version`, written in the save header). **What a released
//! version generates for a seed never changes** (`worldgen/tests.rs` pins each one); new rules go in a
//! new version, branched from the same code at a few named points (`self.version >= n`). 1 = the first
//! terrain (Milestones 1 to 3); 2 = Milestone 4's biomes, rock provinces and geology-driven ores
//! (`biome.rs`, `geology.rs`); 3 = Milestone 5's water and world shape (`strata.rs`: rare surface ore, depth bands, a starter set;
//! `water.rs`: sea and ponds, branching in `height_at`, `build_column`, `trees_near` and `generate`);
//! 4 = easier starter ore (`strata.rs`: shallower bands, more exposed metal, two starter patches each);
//! 5 = Milestone 9's bauxite (`geology.rs`: in far deserts and basalt fields only);
//! 6 = Milestone 10's new ground: bauxite from 300 blocks out instead of 600, oil sand and uranium (`geology.rs`
//! `EXTRAS`, deep bands in `strata.rs`, found by scanning), and far fewer caves (`caves.rs`: cave zones);
//! 7 = Milestone 10's terrain overhaul: bigger continents, taller ranges, mesas (`landform.rs`), rivers and lakes
//! (`rivers.rs`, branching in `height_at` and `water_top`);
//! 8 = R1c's natural water: winding rivers whose pools spill down as rapids and waterfalls (`falls` here, `rivers/`),
//! lakes and ponds with irregular shores (`shore.rs`);
//! 9 = R2b's forests: tree species per biome (`trees.rs`: oak and birch on plains, broad trees in lowlands, pines on
//! highlands, cacti in deserts, dead snags on basalt fields) and R5's ruins of the last factory (`ruins.rs`);
//! 10 = R5c's vaults: in about one far ruin in five a sealed room off the shaft's chamber (`ruins.rs`).

mod bearing;
mod biome;
mod caves;
mod geology;
mod landform;
mod logbook;
mod ore;
mod rivers;
mod ruins;
mod shore;
mod strata;
mod trees;
mod water;

pub use bearing::Bearing;
pub use biome::Biome;
use caves::CaveField;
pub use geology::ore_shares;
pub use ore::LODE_HEIGHTS;
pub use trees::tree_blocks;
pub(crate) use trees::{species_of_sapling, trunk_range};
use trees::{stamp_tree, Tree};
pub use water::Water;
use water::WaterGuard;

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use rustc_hash::FxHashMap;

use crate::block::*;
use crate::chunk::{index, Chunk, CHUNK_SIZE, CHUNK_VOLUME};
use crate::deposits::Deposit;
use crate::math::{hash2, hash3, smoothstep, unit, IVec3};
use crate::noise::Perlin;

/// The newest generator version, which new worlds get. A save records its world's own version.
pub const WORLDGEN_VERSION: u32 = 10;
pub const WORLD_HEIGHT_CHUNKS: i32 = 8;
pub const WORLD_HEIGHT: i32 = WORLD_HEIGHT_CHUNKS * CHUNK_SIZE;
const SAND_LEVEL: i32 = 60;
/// Version 3: the sea's surface (the top water block); land at or below it lies under water.
pub const SEA_LEVEL: i32 = 62;
const ROCK_LEVEL: i32 = 170;
const CLIFF_SLOPE: i32 = 5;
const TREE_CELL: i32 = 7;
const TREE_REACH: i32 = 2;
/// Version 9's broad and conifer canopies reach a block further.
const TREE_REACH_V9: i32 = 3;
const SPAWN_CLEARING: i32 = 6;

/// A pond cell and its pond, if it has one.
type PondCell = ((i32, i32), Option<water::Pond>);

pub struct WorldGen {
    seed: u32,
    version: u32,
    continent: Perlin,
    hills: Perlin,
    ridges: Perlin,
    forest: Perlin,
    cave_a: Perlin,
    cave_b: Perlin,
    /// Version 6: where caves may exist at all (`caves.rs`).
    cave_zone: Perlin,
    /// Version 2's climate and basalt fields (`biome.rs`).
    temperature: Perlin,
    moisture: Perlin,
    basalt: Perlin,
    /// Version 8: river bends and the wobble of widths and shores (`rivers/`).
    meander: Perlin,
    columns: FxHashMap<(i32, i32), Rc<Column>>,
    /// Version 7's river nodes and reaches (`rivers.rs`), caches.
    rivers: rivers::RiverCache,
    /// Version 3's ponds per cell (`water.rs`), a cache.
    ponds: RefCell<FxHashMap<(i32, i32), Option<water::Pond>>>,
    /// The last cell asked about, which neighbouring columns nearly always share.
    last_pond: Cell<Option<PondCell>>,
    /// Version 9's ruin per region (`ruins.rs`), a cache.
    ruins: ruins::Cache,
}

impl WorldGen {
    /// The newest generator.
    pub fn new(seed: u32) -> Self {
        Self::with_version(seed, WORLDGEN_VERSION)
    }

    /// The generator of `version` (1..=[`WORLDGEN_VERSION`]).
    pub fn with_version(seed: u32, version: u32) -> Self {
        debug_assert!((1..=WORLDGEN_VERSION).contains(&version));
        let s = seed as u64;
        Self {
            seed,
            version,
            continent: Perlin::new(s ^ 0x01),
            hills: Perlin::new(s ^ 0x02),
            ridges: Perlin::new(s ^ 0x03),
            forest: Perlin::new(s ^ 0x04),
            cave_a: Perlin::new(s ^ 0x05),
            cave_b: Perlin::new(s ^ 0x06),
            cave_zone: Perlin::new(s ^ 0x0A),
            temperature: Perlin::new(s ^ 0x07),
            moisture: Perlin::new(s ^ 0x08),
            basalt: Perlin::new(s ^ 0x09),
            meander: Perlin::new(s ^ 0x0B),
            columns: FxHashMap::default(),
            rivers: Default::default(),
            ponds: RefCell::default(),
            last_pond: Cell::new(None),
            ruins: RefCell::default(),
        }
    }

    pub fn seed(&self) -> u32 {
        self.seed
    }

    pub fn version(&self) -> u32 {
        self.version
    }

    /// Terrain surface height (y of the top solid block) at a world column.
    pub fn height_at(&self, x: i32, z: i32) -> i32 {
        let mut h = self.base_height(x, z);
        if self.version >= 7 {
            h = self.shape_rivers(x, z, h);
        }
        if self.version >= 3 {
            self.shape_ponds(x, z, h)
        } else {
            h
        }
    }

    /// The height from the terrain noise alone, before rivers (7) and ponds (3).
    fn base_height(&self, x: i32, z: i32) -> i32 {
        if self.version >= 7 {
            return self.land_height(x, z);
        }
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
        const P: usize = 36;
        let (x0, z0) = (cx * CHUNK_SIZE, cz * CHUNK_SIZE);
        let mut hm = vec![0i32; P * P];
        for z in 0..P {
            for x in 0..P {
                hm[z * P + x] = self.height_at(x0 + x as i32 - 2, z0 + z as i32 - 2);
            }
        }
        let mut heights = vec![0i32; 1024];
        let mut surface = vec![(AIR, AIR); 1024];
        let mut rock = vec![STONE; 1024];
        let mut water = vec![i32::MIN; 1024];
        // Version 3: every padded column's top water block (`i32::MIN` when dry).
        let wet: Vec<i32> = if self.version >= 3 {
            hm.iter()
                .enumerate()
                .map(|(i, &h)| self.water_top(x0 + (i % P) as i32 - 2, z0 + (i / P) as i32 - 2, h))
                .collect()
        } else {
            Vec::new()
        };
        let mut max_ground = 0;
        let mut biomes = Vec::new();
        for z in 0..32 {
            for x in 0..32 {
                let h = hm[(z + 2) * P + x + 2];
                let slope = (hm[(z + 2) * P + x + 3] - hm[(z + 2) * P + x + 1])
                    .abs()
                    .max((hm[(z + 3) * P + x + 2] - hm[(z + 1) * P + x + 2]).abs());
                let i = z * 32 + x;
                heights[i] = h;
                if self.version >= 2 {
                    let biome = self.biome_at(x0 + x as i32, z0 + z as i32, h);
                    biomes.push(biome);
                    (surface[i], rock[i]) = Self::surface_v2(biome, h, slope);
                    if self.version >= 3 {
                        water[i] = wet[(z + 2) * P + x + 2];
                        surface[i] = Self::surface_v3(surface[i], h, water[i] > h);
                    }
                } else {
                    surface[i] = Self::surface_for(h, slope);
                }
                max_ground = max_ground.max(h);
            }
        }
        let trees = self.trees_near(x0, z0);
        let ruins = self.ruins_near(x0, z0);
        let max_y = trees.iter().map(|t| t.ground + t.trunk + 2).fold(max_ground, i32::max);
        let max_y = ruins.iter().map(|r| r.top()).fold(max_y, i32::max);
        let falls = if self.version >= 8 { falls(&wet, P) } else { Vec::new() };
        let max_y = water.iter().chain(&falls).copied().fold(max_y, i32::max);
        let guard = (self.version >= 3).then(|| WaterGuard::new(hm.iter().copied().zip(wet).collect())).flatten();
        let near = self.deposits_near(cx, cz);
        if self.version >= 2 {
            self.stain_surface(x0, z0, &near, &mut surface);
        }
        let deposits = ore::in_column(near, cx, cz);
        Column {
            heights,
            surface,
            rock,
            biomes: biomes.into(),
            water,
            falls,
            guard,
            trees,
            ruins,
            deposits,
            max_y,
            max_ground,
        }
    }

    /// Trees whose trunk or canopy can overlap the column starting at (x0, z0).
    /// One candidate per TREE_CELLÂ² cell keeps trunks apart without any neighbour search.
    fn trees_near(&self, x0: i32, z0: i32) -> Vec<Tree> {
        let reach = if self.version >= 9 { TREE_REACH_V9 } else { TREE_REACH };
        let (min_x, max_x) = (x0 - reach, x0 + CHUNK_SIZE - 1 + reach);
        let (min_z, max_z) = (z0 - reach, z0 + CHUNK_SIZE - 1 + reach);
        let mut out = Vec::new();
        for gz in min_z.div_euclid(TREE_CELL)..=max_z.div_euclid(TREE_CELL) {
            for gx in min_x.div_euclid(TREE_CELL)..=max_x.div_euclid(TREE_CELL) {
                let h = hash2(self.seed ^ 0x7EE5, gx, gz);
                let x = gx * TREE_CELL + 1 + (h % 5) as i32;
                let z = gz * TREE_CELL + 1 + ((h >> 8) % 5) as i32;
                if x < min_x || x > max_x || z < min_z || z > max_z {
                    continue;
                }
                if x.abs() < SPAWN_CLEARING && z.abs() < SPAWN_CLEARING || self.ruin_covers(x, z) {
                    continue;
                }
                let f = self.forest.fbm2(x as f64 / 220.0, z as f64 / 220.0, 3);
                let ground = self.height_at(x, z);
                let biome = (self.version >= 2).then(|| self.biome_at(x, z, ground));
                let species =
                    if self.version >= 9 { biome.map_or(trees::OAK, |b| trees::pick(b, h)) } else { trees::OAK };
                let density = trees::bare_density(species).map_or_else(
                    || (smoothstep(-0.2, 0.3, f) * 0.85 + 0.03) * biome.map_or(1.0, Biome::tree_factor),
                    |per_mille| f64::from(per_mille) / 1000.0,
                );
                if unit(hash2(self.seed ^ 0x5EED, gx, gz)) >= density {
                    continue;
                }
                let slope = self.slope_at(x, z);
                let top = match biome {
                    Some(b) => Self::surface_v2(b, ground, slope).0 .0,
                    None => Self::surface_for(ground, slope).0,
                };
                if top != trees::SPECIES[species].ground || (self.version >= 3 && self.water_top(x, z, ground) > ground)
                {
                    continue;
                }
                let trunk = trees::SPECIES[species].trunk_min + ((h >> 16) % trees::SPECIES[species].spread) as i32;
                out.push(Tree { x, z, ground, trunk, species });
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
        let caves = (base.y < col.max_ground).then(|| CaveField::new(self, base)).flatten();

        for z in 0..32usize {
            for x in 0..32usize {
                let h = col.heights[z * 32 + x];
                let (top, filler) = col.surface[z * 32 + x];
                let (wx, wz) = (base.x + x as i32, base.z + z as i32);
                let y_end = h.min(base.y + CHUNK_SIZE - 1);
                for wy in base.y..=y_end {
                    let depth = h - wy;
                    // Solid bedrock at y = 0, thinning out over the next three layers.
                    let id = if wy == 0 || (wy <= 3 && hash3(self.seed, wx, wy, wz).is_multiple_of(wy as u32 + 1)) {
                        BEDROCK
                    } else if depth > 5
                        && wy > 4
                        && caves.as_ref().is_some_and(|c| c.is_cave(x, (wy - base.y) as usize, z))
                        && !col.guard.as_ref().is_some_and(|g| g.near(x, z, wy))
                    {
                        AIR
                    } else if depth == 0 {
                        top
                    } else if depth <= 3 {
                        filler
                    } else {
                        col.rock[z * 32 + x]
                    };
                    b[index(x, (wy - base.y) as usize, z)] = id;
                }
                let top = col.water[z * 32 + x].min(base.y + CHUNK_SIZE - 1);
                for wy in (h + 1).max(base.y)..=top {
                    b[index(x, (wy - base.y) as usize, z)] = WATER;
                }
                if let Some(&fall) = col.falls.get(z * 32 + x) {
                    for wy in (col.water[z * 32 + x] + 1).max(base.y)..=fall.min(base.y + CHUNK_SIZE - 1) {
                        b[index(x, (wy - base.y) as usize, z)] = FLOW_7;
                    }
                }
            }
        }

        if base.y < col.max_ground {
            let top = base + IVec3::new(CHUNK_SIZE - 1, CHUNK_SIZE - 1, CHUNK_SIZE - 1);
            for d in col.deposits.iter().filter(|d| d.intersects(base, top)) {
                ore::stamp_deposit(d, base, &mut b, self.version);
            }
        }
        for t in &col.trees {
            stamp_tree(t, base, self.seed, &mut b);
        }
        for r in &col.ruins {
            r.stamp(base, &mut b);
        }
        Chunk::from_blocks(b)
    }
}

/// Per chunk-column data shared by all 8 vertical chunks of that column.
struct Column {
    heights: Vec<i32>,
    surface: Vec<(BlockId, BlockId)>,
    /// The rock below the soil (stone everywhere in version 1).
    rock: Vec<BlockId>,
    /// Version 2: each column's biome (empty before), for the ground cover the mesher draws (`mesher/cover.rs`).
    biomes: Rc<[Biome]>,
    /// Version 3: each column's top water block, `i32::MIN` when dry (`water.rs`).
    water: Vec<i32>,
    /// Version 8: the top of the falling water above each column's water (`i32::MIN` for none; empty before).
    falls: Vec<i32>,
    guard: Option<WaterGuard>,
    trees: Vec<Tree>,
    /// Version 9: the ruins that reach into this column (`ruins.rs`).
    ruins: Vec<Rc<ruins::Ruin>>,
    /// Deposits (seeded here or in a neighbouring column) whose shape reaches into this column,
    /// sorted by key: stamping and ownership lookups both walk them in this order.
    deposits: Vec<Deposit>,
    /// Highest non-air voxel in the column, including tree canopies.
    max_y: i32,
    max_ground: i32,
}

/// Version 8: where water stands next to higher water (a pool below a drop), the cells between them hold falling water
/// up to the higher top (`FLOW_7`), exactly what `sim/water.rs` settles on there, so nothing moves until an edit.
/// `wet` holds the top water blocks of a `p`-wide grid padded by 2; the result covers the 32Â² inner columns.
fn falls(wet: &[i32], p: usize) -> Vec<i32> {
    let mut out = vec![i32::MIN; 1024];
    for z in 0..32 {
        for x in 0..32 {
            let c = (z + 2) * p + x + 2;
            let top = [c - 1, c + 1, c - p, c + p].iter().map(|&n| wet[n]).max().unwrap_or(i32::MIN);
            if wet[c] != i32::MIN && top > wet[c] {
                out[z * 32 + x] = top;
            }
        }
    }
    out
}

#[cfg(test)]
mod tests;
