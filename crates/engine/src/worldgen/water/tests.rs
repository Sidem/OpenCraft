//! Version 3's water: watertight seas and ponds, the share of the land under water, ponds near spawn,
//! dry spawn, and no water in older versions.

use rustc_hash::FxHashMap;

use super::*;
use crate::block::{AIR, WATER};
use crate::chunk::Chunk;
use crate::math::IVec3;

const SEEDS: [u32; 3] = [2024, 1337, 7];

/// Reads blocks from generated chunks, generating each once.
struct Blocks {
    g: WorldGen,
    chunks: FxHashMap<IVec3, Chunk>,
}

impl Blocks {
    fn get(&mut self, p: IVec3) -> BlockId {
        let c = IVec3::new(p.x >> 5, p.y >> 5, p.z >> 5);
        let g = &mut self.g;
        let chunk = self.chunks.entry(c).or_insert_with(|| g.generate(c));
        chunk.get((p.x & 31) as usize, (p.y & 31) as usize, (p.z & 31) as usize)
    }
}

/// Every water block in the 16 × 16 columns starting at (x0, z0) has no air beside or below it.
fn assert_watertight(seed: u32, x0: i32, z0: i32) -> usize {
    let mut w = Blocks { g: WorldGen::new(seed), chunks: FxHashMap::default() };
    let mut water = 0;
    for z in z0..z0 + 16 {
        for x in x0..x0 + 16 {
            let h = w.g.height_at(x, z);
            for y in (h + 1).max(1)..=w.g.water_top(x, z, h) {
                let p = IVec3::new(x, y, z);
                assert_eq!(w.get(p), WATER, "seed {seed}: water expected at {p:?}");
                water += 1;
                for d in [
                    IVec3::new(1, 0, 0),
                    IVec3::new(-1, 0, 0),
                    IVec3::new(0, 0, 1),
                    IVec3::new(0, 0, -1),
                    IVec3::new(0, -1, 0),
                ] {
                    assert_ne!(w.get(p + d), AIR, "seed {seed}: water at {p:?} leaks towards {:?}", p + d);
                }
            }
        }
    }
    water
}

/// The first column found below sea level, walking out from spawn.
fn a_coast(g: &WorldGen) -> (i32, i32) {
    (0..4000)
        .step_by(8)
        .flat_map(|r| [(r, 0), (-r, 0), (0, r), (0, -r)])
        .find(|&(x, z)| g.height_at(x, z) < SEA_LEVEL - 2)
        .expect("a sea")
}

/// The first pond found walking out from spawn cell by cell (within about 1500 blocks).
fn a_pond(g: &WorldGen) -> Option<Pond> {
    let n = 1500 / POND_CELL;
    (-n..=n).flat_map(|gz| (-n..=n).map(move |gx| (gx, gz))).find_map(|(gx, gz)| g.pond_in(gx, gz))
}

#[test]
fn seas_are_watertight() {
    for seed in SEEDS {
        let (x, z) = a_coast(&WorldGen::new(seed));
        assert!(assert_watertight(seed, x - 8, z - 8) > 0, "seed {seed}: some water at the coast");
    }
}

#[test]
fn ponds_lie_near_spawn_and_are_watertight() {
    for seed in SEEDS {
        let p = a_pond(&WorldGen::new(seed)).unwrap_or_else(|| panic!("seed {seed}: a pond within 1500 blocks"));
        assert!(p.level > SEA_LEVEL + 1, "above the sea");
        assert!(assert_watertight(seed, p.x - 8, p.z - 8) > 0, "seed {seed}: the pond at {p:?} holds water");
    }
}

#[test]
fn about_a_fifth_of_the_land_is_under_water_and_spawn_is_dry() {
    for seed in SEEDS {
        let g = WorldGen::new(seed);
        let (mut wet, mut all) = (0, 0);
        for z in (-2000..2000).step_by(16) {
            for x in (-2000..2000).step_by(16) {
                let h = g.height_at(x, z);
                wet += usize::from(g.water_top(x, z, h) > h);
                all += 1;
            }
        }
        let share = wet as f64 / all as f64;
        assert!((0.15..=0.35).contains(&share), "seed {seed}: {share} of the land under water");
        let h = g.height_at(0, 0);
        assert!(g.water_top(0, 0, h) < h && h > SEA_LEVEL, "seed {seed}: spawn stands on dry land ({h})");
    }
}

#[test]
fn older_versions_have_no_water() {
    for version in [1, 2] {
        let mut g = WorldGen::with_version(1337, version);
        let (x, z) = a_coast(&g);
        let h = g.height_at(x, z);
        let chunk = g.generate(IVec3::new(x >> 5, SEA_LEVEL >> 5, z >> 5));
        assert_eq!(chunk.get((x & 31) as usize, (SEA_LEVEL & 31) as usize, (z & 31) as usize), AIR, "{h}");
    }
}
