//! Where generator 12's trees stand: the same candidate per `TREE_CELL` square as version 9, now reaching `REACH`
//! across, with the species from `kinds::pick` (forest biome, surface, warmth, beach, grove) and each tree grown whole
//! (`grow`) so every chunk it touches stamps the same blocks. Deserts and basalt fields keep version 9's cactus and
//! snag. To change where a species grows: `kinds::pick`.

use super::kinds::{pick, KINDS, PALM};
use super::{grow, height_of, REACH};
use crate::block::{is_log, BlockId, GRASS, SAND};
use crate::chunk::CHUNK_SIZE;
use crate::math::{hash2, smoothstep, unit, IVec3};
use crate::worldgen::trees::{self, Tree};
use crate::worldgen::{Biome, WorldGen, SEA_LEVEL, SPAWN_CLEARING, TREE_CELL};

/// Ground at most this far over the sea counts as a shore, where palms grow (when water is near).
const SHORE: i32 = 3;
/// How far from a palm the water it looks for may be.
const SHORE_REACH: i32 = 6;
/// The size of the stands cherries and bamboo grow in (blocks across a noise period).
const GROVE_SCALE: f64 = 70.0;

impl WorldGen {
    /// Version 12's trees whose blocks can reach the column starting at (x0, z0).
    pub(in crate::worldgen) fn trees_v12(&self, x0: i32, z0: i32) -> Vec<Tree> {
        let (min_x, max_x) = (x0 - REACH, x0 + CHUNK_SIZE - 1 + REACH);
        let (min_z, max_z) = (z0 - REACH, z0 + CHUNK_SIZE - 1 + REACH);
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
                let ground = self.height_at(x, z);
                if self.water_top(x, z, ground) > ground {
                    continue;
                }
                let biome = self.local_biome_at(x, z, ground);
                let top = Self::surface_v3(Self::surface_v2(biome, ground, self.slope_at(x, z)).0, ground, false).0;
                let warmth = self.climate_at(x, z).0;
                let shore = ground <= SEA_LEVEL + SHORE && (top == SAND || top == GRASS) && self.water_near(x, z);
                let grove = self.forest.fbm2(x as f64 / GROVE_SCALE + 91.0, z as f64 / GROVE_SCALE - 37.0, 2) > 0.12;
                let roll = unit(hash2(self.seed ^ 0x5EED, gx, gz));
                let f = self.forest.fbm2(x as f64 / 220.0, z as f64 / 220.0, 3);
                let forest = smoothstep(-0.2, 0.3, f) * 0.85 + 0.03;
                match pick(biome, top, warmth, shore, grove, h) {
                    Some(kind) => {
                        let factor = if kind == PALM { 1.0 } else { biome.tree_factor() };
                        if roll >= forest * factor * KINDS[kind].density {
                            continue;
                        }
                        let foot = IVec3::new(x, ground, z);
                        let height = height_of(kind, h >> 16);
                        let mut grown = Vec::new();
                        grow(kind, foot, height, self.seed, |p, id, replace| grown.push((p, id, replace)));
                        if !self.fits(ground, &grown) {
                            continue;
                        }
                        out.push(Tree { x, z, ground, trunk: height, species: kind, grown });
                    }
                    None if matches!(biome, Biome::Desert | Biome::BasaltFields) => {
                        let species = trees::pick(biome, h);
                        let density = trees::bare_density(species).map_or(0.0, |d| f64::from(d) / 1000.0);
                        if roll < density && top == trees::SPECIES[species].ground {
                            let trunk =
                                trees::SPECIES[species].trunk_min + ((h >> 16) % trees::SPECIES[species].spread) as i32;
                            out.push(Tree { x, z, ground, trunk, species, grown: Vec::new() });
                        }
                    }
                    None => {}
                }
            }
        }
        out
    }

    /// Whether a grown tree stands clear: no log over a ruin, and every log that reaches the ground (a wide trunk's
    /// roots, bamboo's culms) on dry ground no lower than it roots.
    fn fits(&self, ground: i32, grown: &[(IVec3, BlockId, bool)]) -> bool {
        let mut columns: Vec<(i32, i32, i32)> = Vec::new();
        for &(p, _, _) in grown.iter().filter(|g| is_log(g.1)) {
            match columns.iter_mut().find(|c| c.0 == p.x && c.1 == p.z) {
                Some(c) => c.2 = c.2.min(p.y),
                None => columns.push((p.x, p.z, p.y)),
            }
        }
        columns.iter().all(|&(x, z, low)| {
            if self.ruin_covers(x, z) {
                return false;
            }
            if low > ground + 1 {
                return true;
            }
            let g = self.height_at(x, z);
            g >= low - 1 && self.water_top(x, z, g) <= g
        })
    }

    /// Whether the sea, a river or a lake lies within `SHORE_REACH` of (x, z).
    fn water_near(&self, x: i32, z: i32) -> bool {
        let r = SHORE_REACH;
        [(r, 0), (-r, 0), (0, r), (0, -r), (r, r), (-r, -r), (r, -r), (-r, r)].iter().any(|&(dx, dz)| {
            let (wx, wz) = (x + dx, z + dz);
            let g = self.height_at(wx, wz);
            self.water_top(wx, wz, g) > g
        })
    }
}
