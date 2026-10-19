//! Builds a chunk column's shared data (`Column`, cached in `WorldGen::columns`): heights, surface and rock, biomes,
//! water and falls, and the trees, ruins and deposits that reach into it. Branches by generator version like the rest
//! of `worldgen` (a released version's output never changes). To add per-column data: a `Column` field (`mod.rs`),
//! filled here.

use super::*;

impl WorldGen {
    pub(super) fn build_column(&self, cx: i32, cz: i32) -> Column {
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
