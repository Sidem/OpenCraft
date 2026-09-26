//! Biomes: all of them near spawn, spawn itself plains, the right rock underneath, and the cost.

use super::*;
use crate::chunk::CHUNK_SIZE;
use crate::math::IVec3;

/// Samples every 16 blocks within `radius` of spawn: the share of each biome and one column of each.
fn survey(g: &WorldGen, radius: i32) -> Vec<(Biome, u32, (i32, i32))> {
    let mut found: Vec<(Biome, u32, (i32, i32))> = Biome::ALL.iter().map(|&b| (b, 0, (0, 0))).collect();
    for z in (-radius..=radius).step_by(16) {
        for x in (-radius..=radius).step_by(16) {
            if x * x + z * z > radius * radius {
                continue;
            }
            let b = g.biome_at(x, z, g.height_at(x, z));
            let entry = found.iter_mut().find(|e| e.0 == b).unwrap();
            entry.1 += 1;
            entry.2 = (x, z);
        }
    }
    found
}

#[test]
fn every_biome_is_near_spawn() {
    for seed in [2024, 1337, 7] {
        let g = WorldGen::new(seed);
        let found = survey(&g, 1500);
        let total: u32 = found.iter().map(|f| f.1).sum();
        let shares: Vec<_> = found.iter().map(|f| (f.0, f.1 * 100 / total, f.2)).collect();
        println!("seed {seed}: {shares:?}");
        assert!(found.iter().all(|f| f.1 > 0), "seed {seed}: {shares:?}");
    }
}

#[test]
fn spawn_is_plains() {
    for seed in [2024, 1337, 7, 1, 99, 4242] {
        let g = WorldGen::new(seed);
        for z in (-48..=48).step_by(4) {
            for x in (-48..=48).step_by(4) {
                let h = g.height_at(x, z);
                if h <= HIGHLAND_LEVEL {
                    assert_eq!(g.biome_at(x, z, h), Biome::Plains, "seed {seed} at {x}, {z}");
                }
            }
        }
    }
}

#[test]
fn each_biome_has_its_rock_underground() {
    let mut g = WorldGen::new(2024);
    for (biome, _, (x, z)) in survey(&g, 1500) {
        let h = g.height_at(x, z);
        // Several blocks below the soil, where caves and ore permit.
        let mut rocks = 0;
        for y in (h - 14)..(h - 6) {
            let c = g.generate(IVec3::new(x >> 5, y >> 5, z >> 5));
            let b = c.get(
                x.rem_euclid(CHUNK_SIZE) as usize,
                y.rem_euclid(CHUNK_SIZE) as usize,
                z.rem_euclid(CHUNK_SIZE) as usize,
            );
            rocks += (b == biome.rock()) as u32;
        }
        assert!(rocks >= 4, "{biome:?} at {x}, {z}: {rocks} of 8 blocks are {}", def(biome.rock()).name);
    }
}

#[test]
fn version_1_has_no_biome_rocks() {
    let mut g = WorldGen::with_version(2024, 1);
    for (x, z) in [(0, 0), (40, -80), (-300, 200)] {
        let h = g.height_at(x, z);
        for cy in 0..=(h >> 5) {
            let c = g.generate(IVec3::new(x >> 5, cy, z >> 5));
            for i in 0..crate::chunk::CHUNK_VOLUME {
                let b = c.get(i & 31, i >> 10, (i >> 5) & 31);
                assert!(!matches!(b, GRANITE | SANDSTONE | BASALT));
            }
        }
    }
}

/// Generation cost of version 2 against version 1 (run with `--ignored --nocapture`).
#[test]
#[ignore]
fn bench_generation() {
    for version in [1, 2] {
        let mut g = WorldGen::with_version(1337, version);
        let start = std::time::Instant::now();
        for cz in -6..6 {
            for cx in -6..6 {
                for cy in 0..4 {
                    g.generate(IVec3::new(cx, cy, cz));
                }
            }
        }
        println!("version {version}: {:?} for 576 chunks", start.elapsed());
    }
}
