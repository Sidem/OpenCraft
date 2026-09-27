//! Versions 3 and 4's deposits: rare surface ore on bare rock only, depth bands, version 2's totals, the
//! starter set near spawn.

use super::*;
use crate::chunk::CHUNK_SIZE;

const SEEDS: [u32; 3] = [2024, 1337, 7];

/// Every deposit seeded in the square of chunk columns `-r..r`.
fn seeded(g: &WorldGen, r: i32) -> Vec<Deposit> {
    let mut out = Vec::new();
    for cz in -r..r {
        for cx in -r..r {
            g.seed_deposits(cx, cz, &mut out);
        }
    }
    out
}

/// Whether the deposit's shape reaches a column's top block.
fn shows(g: &WorldGen, d: &Deposit) -> bool {
    let (lo, hi) = d.bounds();
    (lo.z..=hi.z).any(|z| (lo.x..=hi.x).any(|x| d.contains(IVec3::new(x, g.height_at(x, z), z))))
}

#[test]
fn surface_ore_is_rare_and_only_on_bare_rock() {
    for (version, range) in [(3, 0.2..=0.5), (4, 0.25..=0.65)] {
        for seed in SEEDS {
            let g = WorldGen::with_version(seed, version);
            let mut exposed = 0;
            for d in seeded(&g, 32).iter().filter(|d| d.tier() == Tier::Outcrop && d.key.index < STARTER_INDEX) {
                // A pocket this deep could only surface on a cliff face, which is bare rock anyway.
                if d.center.y < g.height_at(d.center.x, d.center.z) - 12 || !shows(&g, d) {
                    continue;
                }
                exposed += 1;
                assert!(g.bare_rock(d.center.x, d.center.z), "v{version} seed {seed}: {d:?} shows off bare rock");
            }
            let per_column = exposed as f64 / (64.0 * 64.0);
            println!("version {version}, seed {seed}: {per_column} exposed outcrops per column");
            assert!(range.contains(&per_column), "v{version} seed {seed}: {per_column} exposed per column");
        }
    }
}

#[test]
fn every_tier_keeps_version_2s_total() {
    for seed in SEEDS {
        let count = |version: u32, tier: Tier| {
            let g = WorldGen::with_version(seed, version);
            seeded(&g, 32).iter().filter(|d| d.tier() == tier && d.key.index < STARTER_INDEX).count() as f64
        };
        for tier in [Tier::Outcrop, Tier::Vein, Tier::Lode] {
            let v2 = count(2, tier);
            for version in [3, 4] {
                let n = count(version, tier);
                assert!((n / v2 - 1.0).abs() < 0.05, "seed {seed}, {tier:?}: {v2} in version 2, {n} in {version}");
            }
        }
    }
}

#[test]
fn buried_ore_keeps_to_its_band() {
    for version in [3, 4] {
        let g = WorldGen::with_version(2024, version);
        for d in seeded(&g, 16).iter().filter(|d| d.tier() != Tier::Lode && d.key.index < STARTER_INDEX) {
            let h = g.height_at(d.center.x, d.center.z);
            if d.tier() == Tier::Outcrop && d.center.y >= h - 1 {
                continue; // exposed
            }
            let (lo, hi) = g.ore_band(d.ore());
            let floor = if d.tier() == Tier::Vein { 12 } else { DEEPEST };
            let depth = h - d.center.y;
            assert!((lo..=hi).contains(&depth) || d.center.y == floor, "v{version}: {d:?} is {depth} deep, {lo}..{hi}");
        }
    }
}

#[test]
fn version_4_keeps_the_starter_metals_shallower() {
    let (v3, v4) = (WorldGen::with_version(1, 3), WorldGen::with_version(1, 4));
    for ore in [COAL_ORE, IRON_ORE, COPPER_ORE] {
        assert!(v4.ore_band(ore).1 < v3.ore_band(ore).1, "{ore}");
    }
    assert_eq!(v4.ore_band(IRON_ORE), (8, 28));
}

#[test]
fn a_starter_set_shows_near_spawn() {
    for (version, starters) in [(3, &STARTERS[..]), (4, &STARTERS_V4[..])] {
        for seed in SEEDS {
            let mut g = WorldGen::with_version(seed, version);
            let found = g.starter_outcrops();
            let ores: Vec<BlockId> = found.iter().map(|d| d.ore()).collect();
            let wanted: Vec<BlockId> = starters.iter().map(|s| s.0).collect();
            assert_eq!(ores, wanted, "v{version} seed {seed}: every starter found");
            for (d, &(_, ring)) in found.into_iter().zip(starters) {
                let (x, z) = (d.center.x, d.center.z);
                let dist = ((x * x + z * z) as f64).sqrt();
                let (near, far) = (ring.0 as f64 - 2.0, ring.1 as f64 + 1.0);
                assert!((near..=far).contains(&dist), "v{version} seed {seed}: {d:?} is {dist} from spawn");
                assert_eq!(g.deposit_by_key(d.key).map(|f| f.center), Some(d.center), "found again by key");
                let h = g.height_at(x, z);
                assert!(h > SEA_LEVEL + 1, "on dry land");
                let chunk = g.generate(IVec3::new(x >> 5, h >> 5, z >> 5));
                let top = chunk.get((x & 31) as usize, (h & 31) as usize, (z & 31) as usize);
                assert_eq!(top, d.ore(), "v{version} seed {seed}: the starter {} shows at {x}, {h}, {z}", d.ore());
                assert!(x >> 5 >= -4 && x >> 5 <= 4 && z >> 5 >= -4 && z >> 5 <= 4 && CHUNK_SIZE == 32);
            }
        }
    }
}
