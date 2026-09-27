//! Version 3's deposits: rare surface ore on bare rock only, depth bands, version 2's totals, the
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

fn band(ore: BlockId) -> (i32, i32) {
    ORE_DEPTH.iter().find(|b| b.0 == ore).map(|b| (b.1, b.2)).expect("every ore has a band")
}

#[test]
fn surface_ore_is_rare_and_only_on_bare_rock() {
    for seed in SEEDS {
        let g = WorldGen::new(seed);
        let mut exposed = 0;
        for d in seeded(&g, 32).iter().filter(|d| d.tier() == Tier::Outcrop && d.key.index < STARTER_INDEX) {
            // A pocket this deep could only surface on a cliff face, which is bare rock anyway.
            if d.center.y < g.height_at(d.center.x, d.center.z) - 12 || !shows(&g, d) {
                continue;
            }
            exposed += 1;
            assert!(g.bare_rock(d.center.x, d.center.z), "seed {seed}: {d:?} shows off bare rock");
        }
        let per_column = exposed as f64 / (64.0 * 64.0);
        assert!((0.2..=0.5).contains(&per_column), "seed {seed}: {per_column} exposed outcrops per column");
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
            let (v2, v3) = (count(2, tier), count(3, tier));
            assert!((v3 / v2 - 1.0).abs() < 0.05, "seed {seed}, {tier:?}: {v2} in version 2, {v3} in version 3");
        }
    }
}

#[test]
fn buried_ore_keeps_to_its_band() {
    let g = WorldGen::new(2024);
    for d in seeded(&g, 16).iter().filter(|d| d.tier() != Tier::Lode && d.key.index < STARTER_INDEX) {
        let h = g.height_at(d.center.x, d.center.z);
        if d.tier() == Tier::Outcrop && d.center.y >= h - 1 {
            continue; // exposed
        }
        let (lo, hi) = band(d.ore());
        let floor = if d.tier() == Tier::Vein { 12 } else { DEEPEST };
        let depth = h - d.center.y;
        assert!((lo..=hi).contains(&depth) || d.center.y == floor, "{d:?} is {depth} deep, band {lo}..{hi}");
    }
}

#[test]
fn a_starter_set_shows_near_spawn() {
    for seed in SEEDS {
        let mut g = WorldGen::new(seed);
        let starters = g.starter_outcrops();
        let ores: Vec<BlockId> = starters.iter().map(|d| d.ore()).collect();
        assert_eq!(ores, STARTER_ORES, "seed {seed}: one of each");
        for d in starters {
            let (x, z) = (d.center.x, d.center.z);
            let dist = ((x * x + z * z) as f64).sqrt();
            assert!((38.0..=81.0).contains(&dist), "seed {seed}: {d:?} is {dist} from spawn");
            assert_eq!(g.deposit_by_key(d.key).map(|f| f.center), Some(d.center), "found again by key");
            let h = g.height_at(x, z);
            assert!(h > SEA_LEVEL + 1, "on dry land");
            let chunk = g.generate(IVec3::new(x >> 5, h >> 5, z >> 5));
            let top = chunk.get((x & 31) as usize, (h & 31) as usize, (z & 31) as usize);
            assert_eq!(top, d.ore(), "seed {seed}: the starter {} shows at {x}, {h}, {z}", d.ore());
            assert!(x >> 5 >= -3 && x >> 5 <= 3 && z >> 5 >= -3 && z >> 5 <= 3 && CHUNK_SIZE == 32);
        }
    }
}
