//! Deterministic terrain generation: heightmap terrain, cliffs, spaghetti caves, ore deposits and trees.
//!
//! Every feature that can cross a chunk border (trees, ore deposits) is derived from hashes of the
//! *source* cell, and each chunk stamps whatever part of its neighbours' features overlaps it. That
//! keeps generation order-independent, which is what later lets it move to worker threads.

use std::rc::Rc;

use rustc_hash::FxHashMap;

use crate::block::*;
use crate::chunk::{index, Chunk, CHUNK_SIZE, CHUNK_VOLUME};
use crate::deposits::{Deposit, DepositKey, Tier};
use crate::math::{hash2, hash3, smoothstep, unit, IVec3, Rng};
use crate::noise::Perlin;

pub const WORLD_HEIGHT_CHUNKS: i32 = 8;
pub const WORLD_HEIGHT: i32 = WORLD_HEIGHT_CHUNKS * CHUNK_SIZE;
const SAND_LEVEL: i32 = 60;
const ROCK_LEVEL: i32 = 170;
const CLIFF_SLOPE: i32 = 5;
const TREE_CELL: i32 = 7;
const TREE_REACH: i32 = 2;
const SPAWN_CLEARING: i32 = 6;
/// No surface outcrops this close to spawn, so the player starts on plain ground.
const ORE_SPAWN_CLEARING: i32 = 10;

/// Per ore: (block, outcrops per chunk column, chance of a vein per column, weight when picking a lode's ore).
const ORE_GEN: [(BlockId, u32, f64, u32); 3] =
    [(COAL_ORE, 4, 0.5, 1), (IRON_ORE, 4, 0.45, 2), (COPPER_ORE, 3, 0.35, 1)];
/// Chance that a chunk column seeds a lode (roughly one per 40 columns, i.e. per ~200 x 200 blocks).
const LODE_CHANCE: f64 = 1.0 / 40.0;

/// Blocks a deposit may turn into ore. Outcrops that reach the surface replace grass and dirt, which
/// is what makes them visible.
#[inline]
fn ore_replaceable(b: BlockId) -> bool {
    matches!(b, STONE | DIRT | GRASS)
}

/// Sorts deposits into ownership order. The lists are a few columns' worth at most, and an
/// insertion sort keeps another instantiation of the general-purpose sort out of the wasm.
fn sort_by_ownership(v: &mut [Deposit]) {
    for i in 1..v.len() {
        let mut j = i;
        while j > 0 && v[j - 1].key > v[j].key {
            v.swap(j - 1, j);
            j -= 1;
        }
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

    /// Deposits from this column's 3x3 neighbourhood that reach into it, in ownership order.
    fn deposits_for_column(&self, cx: i32, cz: i32) -> Vec<Deposit> {
        let mut all = Vec::new();
        for dz in -1..=1 {
            for dx in -1..=1 {
                self.seed_deposits(cx + dx, cz + dz, &mut all);
            }
        }
        let lo = IVec3::new(cx * CHUNK_SIZE, 0, cz * CHUNK_SIZE);
        let hi = IVec3::new(lo.x + CHUNK_SIZE - 1, WORLD_HEIGHT - 1, lo.z + CHUNK_SIZE - 1);
        all.retain(|d| d.intersects(lo, hi));
        sort_by_ownership(&mut all);
        all
    }

    /// Every deposit seeded in one chunk column. A deposit's shape stays within one column of where
    /// it was seeded, so a chunk only ever needs its 3x3 neighbourhood.
    fn seed_deposits(&self, cx: i32, cz: i32, out: &mut Vec<Deposit>) {
        let (x0, z0) = (cx * CHUNK_SIZE, cz * CHUNK_SIZE);
        for (oi, &(ore, outcrops, vein_chance, _)) in ORE_GEN.iter().enumerate() {
            let mut rng = Rng::new(hash3(self.seed ^ 0xDE90, cx, oi as i32, cz) as u64);
            let key = |tier, index| DepositKey { tier, cx, cz, ore, index };
            for i in 0..outcrops {
                let x = x0 + rng.below(32) as i32;
                let z = z0 + rng.below(32) as i32;
                let surface = self.height_at(x, z);
                // Half sit right at the surface (visible patches), half are pockets deeper down.
                let y = if rng.next_f64() < 0.5 {
                    surface - rng.below(4) as i32
                } else {
                    8 + rng.below((surface - 14).max(1) as u32) as i32
                };
                let r = rng.range(1.3, 2.3) as f32;
                let squash = rng.range(0.7, 1.0) as f32;
                let seed = rng.next_u32();
                if x.abs() < ORE_SPAWN_CLEARING && z.abs() < ORE_SPAWN_CLEARING {
                    continue;
                }
                out.push(Deposit {
                    key: key(Tier::Outcrop, i as u16),
                    center: IVec3::new(x, y, z),
                    radii: [r, r * squash, r],
                    seed,
                });
            }
            if rng.next_f64() < vein_chance {
                let x = x0 + rng.below(32) as i32;
                let z = z0 + rng.below(32) as i32;
                let y = (self.height_at(x, z) - rng.range(20.0, 60.0) as i32).max(12);
                let major = rng.range(4.5, 7.0) as f32;
                let minor = rng.range(2.0, 3.0) as f32;
                let tall = rng.range(1.8, 2.6) as f32;
                let radii = if rng.below(2) == 0 { [major, tall, minor] } else { [minor, tall, major] };
                out.push(Deposit { key: key(Tier::Vein, 0), center: IVec3::new(x, y, z), radii, seed: rng.next_u32() });
            }
        }

        let mut rng = Rng::new(hash2(self.seed ^ 0x10DE, cx, cz) as u64);
        if rng.next_f64() < LODE_CHANCE {
            let total: u32 = ORE_GEN.iter().map(|o| o.3).sum();
            let mut pick = rng.below(total);
            let mut ore = ORE_GEN[0].0;
            for &(o, _, _, weight) in &ORE_GEN {
                if pick < weight {
                    ore = o;
                    break;
                }
                pick -= weight;
            }
            let x = x0 + rng.below(32) as i32;
            let z = z0 + rng.below(32) as i32;
            let y = 12 + rng.below(14) as i32;
            let r = rng.range(7.0, 8.5) as f32;
            out.push(Deposit {
                key: DepositKey { tier: Tier::Lode, cx, cz, ore, index: 0 },
                center: IVec3::new(x, y, z),
                radii: [r, r * 0.75, r],
                seed: rng.next_u32(),
            });
        }
    }

    /// The deposit that owns the ore block at `p`: the first one in ownership order whose shape
    /// contains it (the same one that stamped it during generation).
    pub fn deposit_at(&mut self, p: IVec3) -> Option<Deposit> {
        let col = self.column(p.x >> 5, p.z >> 5);
        col.deposits.iter().find(|d| d.contains(p)).copied()
    }

    /// Every deposit whose bounds overlap the box `lo..=hi`, in ownership order.
    pub fn deposits_touching(&mut self, lo: IVec3, hi: IVec3) -> Vec<Deposit> {
        let mut out: Vec<Deposit> = Vec::new();
        for cz in (lo.z >> 5)..=(hi.z >> 5) {
            for cx in (lo.x >> 5)..=(hi.x >> 5) {
                let col = self.column(cx, cz);
                out.extend(col.deposits.iter().filter(|d| d.intersects(lo, hi)));
            }
        }
        sort_by_ownership(&mut out);
        out.dedup_by_key(|d| d.key);
        out
    }

    /// Nearest deposit of `tier` seeded within `radius` chunk columns of `near` (a prospecting aid).
    pub fn find_deposit(&self, near: IVec3, tier: Tier, radius: i32) -> Option<Deposit> {
        let (pcx, pcz) = (near.x >> 5, near.z >> 5);
        let mut seeded = Vec::new();
        let mut best: Option<(i64, Deposit)> = None;
        for cz in pcz - radius..=pcz + radius {
            for cx in pcx - radius..=pcx + radius {
                seeded.clear();
                self.seed_deposits(cx, cz, &mut seeded);
                for d in seeded.iter().filter(|d| d.tier() == tier) {
                    let v = d.center - near;
                    let dist = (v.x as i64).pow(2) + (v.y as i64).pow(2) + (v.z as i64).pow(2);
                    if best.as_ref().is_none_or(|(b, _)| dist < *b) {
                        best = Some((dist, *d));
                    }
                }
            }
        }
        best.map(|(_, d)| d)
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
                    let id = if wy == 0 {
                        BEDROCK
                    } else if wy <= 3 && hash3(self.seed, wx, wy, wz) % (wy as u32 + 1) == 0 {
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
                stamp_deposit(d, base, &mut b);
            }
        }
        for t in &col.trees {
            stamp_tree(t, base, self.seed, &mut b);
        }
        Chunk::from_blocks(b)
    }
}

/// Writes a deposit's ore into the chunk at `base`. Deposits are stamped in ownership order and never
/// overwrite ore, so where shapes overlap the first (smallest key) wins, matching [`WorldGen::deposit_at`].
fn stamp_deposit(d: &Deposit, base: IVec3, b: &mut [BlockId]) {
    let (lo, hi) = d.bounds();
    let from = |v: i32, o: i32| (v - o).max(0);
    let to = |v: i32, o: i32| (v - o).min(CHUNK_SIZE - 1);
    for y in from(lo.y, base.y)..=to(hi.y, base.y) {
        for z in from(lo.z, base.z)..=to(hi.z, base.z) {
            for x in from(lo.x, base.x)..=to(hi.x, base.x) {
                let i = index(x as usize, y as usize, z as usize);
                if ore_replaceable(b[i]) && d.contains(base + IVec3::new(x, y, z)) {
                    b[i] = d.ore();
                }
            }
        }
    }
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
mod tests {
    use super::*;

    #[test]
    fn generation_is_deterministic() {
        let mut a = WorldGen::new(7);
        let mut b = WorldGen::new(7);
        for p in [IVec3::new(0, 1, 0), IVec3::new(-3, 2, 5), IVec3::new(10, 0, -10)] {
            let (ca, cb) = (a.generate(p), b.generate(p));
            for i in 0..CHUNK_VOLUME {
                let (x, y, z) = (i & 31, i >> 10, (i >> 5) & 31);
                assert_eq!(ca.get(x, y, z), cb.get(x, y, z));
            }
        }
    }

    #[test]
    fn bottom_is_bedrock_and_sky_is_empty() {
        let mut g = WorldGen::new(1);
        let bottom = g.generate(IVec3::new(0, 0, 0));
        assert_eq!(bottom.get(5, 0, 5), BEDROCK);
        assert_eq!(g.generate(IVec3::new(0, WORLD_HEIGHT_CHUNKS - 1, 0)).as_uniform(), Some(AIR));
    }

    #[test]
    fn spawn_column_is_solid_below_surface() {
        let mut g = WorldGen::new(99);
        let h = g.height_at(0, 0);
        let c = g.generate(IVec3::new(0, h >> 5, 0));
        assert_ne!(c.get(0, (h & 31) as usize, 0), AIR);
        assert!(h > 4 && h < WORLD_HEIGHT - 16);
    }

    #[test]
    fn ore_is_owned_and_some_is_visible_near_spawn() {
        let mut g = WorldGen::new(1337);
        let (mut ore, mut exposed) = (0, 0);
        let mut tiers = [0u32; 3];
        for cz in -2..=2 {
            for cx in -2..=2 {
                for cy in 0..WORLD_HEIGHT_CHUNKS {
                    let c = g.generate(IVec3::new(cx, cy, cz));
                    for i in 0..CHUNK_VOLUME {
                        let (x, y, z) = (i & 31, i >> 10, (i >> 5) & 31);
                        let b = c.get(x, y, z);
                        if !is_ore(b) {
                            continue;
                        }
                        ore += 1;
                        let p = IVec3::new(cx * 32 + x as i32, cy * 32 + y as i32, cz * 32 + z as i32);
                        let d = g.deposit_at(p).expect("every ore block belongs to a deposit");
                        assert_eq!(d.ore(), b);
                        tiers[d.tier() as usize] += 1;
                        if y < 31 && c.get(x, y + 1, z) == AIR {
                            exposed += 1;
                        }
                    }
                }
            }
        }
        println!("ore blocks {ore}, exposed {exposed}, by tier (lode, vein, outcrop) {tiers:?}");
        assert!(tiers[Tier::Outcrop as usize] > 0 && tiers[Tier::Vein as usize] > 0);
        assert!(exposed > 20, "only {exposed} ore blocks see the sky or a cave");
    }

    #[test]
    fn lodes_exist_within_prospecting_range() {
        let g = WorldGen::new(1337);
        let lode = g.find_deposit(IVec3::new(0, 64, 0), Tier::Lode, 16).expect("a lode within ~500 blocks");
        assert!(lode.center.y < 32, "lodes sit near bedrock");
    }

    #[test]
    fn terrain_height_distribution_is_reasonable() {
        let g = WorldGen::new(1337);
        let (mut lo, mut hi, mut sum, mut n) = (i32::MAX, i32::MIN, 0i64, 0i64);
        for z in (-4000..4000).step_by(37) {
            for x in (-4000..4000).step_by(37) {
                let h = g.height_at(x, z);
                lo = lo.min(h);
                hi = hi.max(h);
                sum += h as i64;
                n += 1;
            }
        }
        let mean = sum / n;
        println!("height min {lo} max {hi} mean {mean}");
        assert!(lo >= 4 && hi <= WORLD_HEIGHT - 16);
        assert!(hi - lo > 60, "terrain too flat: {lo}..{hi}");
    }
}
