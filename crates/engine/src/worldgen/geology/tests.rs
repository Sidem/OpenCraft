//! Geology: ores follow the biome weights, totals match version 1, and the new ores mine.

use super::*;
use crate::block::{def, STORAGE};
use crate::tests::{build_mine, run_until_ready};
use crate::Game;

/// Every deposit seeded in the 64 × 64 columns around spawn.
fn deposits(g: &WorldGen) -> Vec<Deposit> {
    let mut out = Vec::new();
    for cz in -32..32 {
        for cx in -32..32 {
            g.seed_deposits(cx, cz, &mut out);
        }
    }
    out
}

#[test]
fn ores_follow_the_biome_weights() {
    let g = WorldGen::new(1337);
    let all = deposits(&g);
    for &(biome, weights) in &ORES_BY_BIOME {
        let here: Vec<&Deposit> = all
            .iter()
            .filter(|d| g.biome_at(d.center.x, d.center.z, g.height_at(d.center.x, d.center.z)) == biome)
            .collect();
        let total: u32 = weights.iter().map(|w| w.1).sum();
        let n = here.len() as f64;
        println!("{biome:?}: {} deposits", here.len());
        if here.len() < 400 {
            continue; // too few for shares to settle
        }
        for &(ore, weight) in weights {
            let share = here.iter().filter(|d| d.ore() == ore).count() as f64 / n;
            let want = weight as f64 / total as f64;
            assert!((share - want).abs() < 0.05, "{biome:?} {}: {share:.3} vs {want:.3}", def(ore).name);
        }
        assert!(here.iter().all(|d| weights.iter().any(|w| w.0 == d.ore())), "{biome:?} has a stray ore");
    }
}

#[test]
fn deposit_counts_match_version_1() {
    let (v1, v2) = (deposits(&WorldGen::with_version(1337, 1)), deposits(&WorldGen::new(1337)));
    for tier in [Tier::Outcrop, Tier::Vein, Tier::Lode] {
        let (a, b) = (v1.iter().filter(|d| d.tier() == tier).count(), v2.iter().filter(|d| d.tier() == tier).count());
        let off = (a as f64 - b as f64).abs() / a as f64;
        assert!(off < 0.05 || (tier == Tier::Lode && a.abs_diff(b) < 20), "{tier:?}: version 1 {a}, version 2 {b}");
    }
}

/// Stained tops within `r` blocks of (x, z).
fn hints_near(g: &mut WorldGen, x: i32, z: i32, r: i32) -> usize {
    let mut n = 0;
    for dz in -r..=r {
        for dx in -r..=r {
            let (px, pz) = (x + dx, z + dz);
            let col = g.column(px >> 5, pz >> 5);
            let top = col.surface[((pz & 31) * 32 + (px & 31)) as usize].0;
            if matches!(top, RUSTY_SOIL | DARK_SOIL | GREEN_SOIL | PALE_SOIL) {
                n += 1;
            }
        }
    }
    n
}

#[test]
fn veins_and_lodes_stain_the_soil_above_them() {
    let mut g = WorldGen::new(1337);
    let (mut deep, mut hinted) = (0, 0);
    for cz in -6..6 {
        for cx in -6..6 {
            let mut seeded = Vec::new();
            g.seed_deposits(cx, cz, &mut seeded);
            for d in seeded.iter().filter(|d| d.tier() != Tier::Outcrop) {
                let n = hints_near(&mut g, d.center.x, d.center.z, 4);
                deep += 1;
                hinted += (n > 0) as u32;
            }
        }
    }
    println!("{hinted} of {deep} veins and lodes show hints");
    assert!(hinted * 10 >= deep * 9, "{hinted} of {deep}");

    // Version 1 has none.
    let mut old = WorldGen::with_version(1337, 1);
    for (x, z) in [(0, 0), (100, -60), (-200, 150)] {
        assert_eq!(hints_near(&mut old, x, z, 40), 0);
    }
}

#[test]
fn keys_are_unique_and_found_again() {
    let g = WorldGen::new(2024);
    let mut seeded = Vec::new();
    g.seed_deposits(3, -2, &mut seeded);
    for (i, d) in seeded.iter().enumerate() {
        assert!(seeded[..i].iter().all(|o| o.key != d.key), "duplicate key {:?}", d.key);
        assert_eq!(g.deposit_by_key(d.key).map(|f| f.center), Some(d.center));
    }
}

#[test]
fn a_miner_on_limestone_or_quartz_fills_a_box() {
    for ore in [LIMESTONE, QUARTZ_ORE] {
        let mut g = Game::new(2024, 2);
        // The nearest surface outcrop of this ore.
        let gen = WorldGen::new(2024);
        let mut found = None;
        'search: for r in 1..40 {
            for cz in -r..=r {
                for cx in -r..=r {
                    let mut seeded = Vec::new();
                    gen.seed_deposits(cx, cz, &mut seeded);
                    let surface = |d: &Deposit| d.center.y >= gen.height_at(d.center.x, d.center.z) - 3;
                    if let Some(d) =
                        seeded.into_iter().find(|d| d.ore() == ore && d.tier() == Tier::Outcrop && surface(d))
                    {
                        found = Some(d);
                        break 'search;
                    }
                }
            }
        }
        let d = found.expect("an outcrop");
        g.teleport(d.center.x as f64, d.center.y as f64 + 4.0, d.center.z as f64);
        run_until_ready(&mut g);
        let (lo, hi) = d.bounds();
        let block = (lo.y..=hi.y)
            .rev()
            .flat_map(|y| (lo.z..=hi.z).flat_map(move |z| (lo.x..=hi.x).map(move |x| IVec3::new(x, y, z))))
            .find(|&p| {
                g.sim.world.get_block(p) == Some(ore) && g.sim.world.get_block(p + IVec3::new(0, 1, 0)) == Some(0)
            })
            .expect("an exposed ore block");
        let (_, chest) = build_mine(&mut g, block);
        assert_eq!(g.sim.world.get_block(chest), Some(STORAGE));
        g.skip_time(300.0);
        assert!(g.sim.factory.storage_count_at(chest, ore.into()) > 0, "{} reached the box", def(ore).name);
    }
}
