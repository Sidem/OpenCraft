use super::*;
use crate::block::{LEAVES, LOG, SAPLING};
use crate::math::IVec3;
use crate::worldgen::{WorldGen, WORLD_HEIGHT_CHUNKS};
use std::collections::HashMap;

/// The blocks `species` puts down for a tree standing on the origin, trunk `trunk` blocks tall.
fn stamp(species: usize, trunk: i32) -> HashMap<(i32, i32, i32), BlockId> {
    let mut m = HashMap::new();
    tree_blocks(IVec3::new(0, 10, 0), trunk, species, 7, |p, id, _| {
        m.insert((p.x, p.y, p.z), id);
    });
    m
}

fn count(m: &HashMap<(i32, i32, i32), BlockId>, b: BlockId) -> usize {
    m.values().filter(|&&v| v == b).count()
}

fn width(m: &HashMap<(i32, i32, i32), BlockId>, b: BlockId) -> i32 {
    m.iter().filter(|(_, &v)| v == b).map(|(p, _)| p.0.abs().max(p.2.abs())).max().unwrap_or(0)
}

#[test]
fn every_species_builds_its_own_shape() {
    let oak = stamp(OAK, 5);
    assert_eq!((count(&oak, LOG), width(&oak, LEAVES)), (5, 2));
    let broad = stamp(BROAD, 3);
    assert_eq!((count(&broad, LOG), width(&broad, LEAVES)), (3, 3), "broad: low and wide");
    let pine = stamp(PINE, 7);
    assert_eq!((count(&pine, LOG), width(&pine, PINE_LEAVES)), (7, 3));
    assert_eq!(count(&pine, LEAVES), 0, "a pine has needles, not oak leaves");
    assert!(pine.contains_key(&(0, 19, 0)) && !pine.contains_key(&(0, 20, 0)), "its tip stands two above the trunk");
    let birch = stamp(BIRCH, 6);
    assert_eq!((count(&birch, BIRCH_LOG), width(&birch, BIRCH_LEAVES)), (6, 2));
    let cactus = stamp(CACTUS_PLANT, 3);
    assert_eq!((count(&cactus, CACTUS), cactus.len()), (3, 3), "a cactus is a bare column on the sand");
    let snag = stamp(SNAG, 4);
    assert_eq!(count(&snag, SNAG_LOG), 5, "a trunk and a stub of a branch");
    assert!(snag.values().all(|&b| b == SNAG_LOG), "no leaves, no dirt under it");
}

#[test]
fn only_trees_with_a_foot_replace_the_ground() {
    for (species, foot) in [(OAK, true), (BIRCH, true), (PINE, true), (CACTUS_PLANT, false), (SNAG, false)] {
        assert_eq!(stamp(species, 4).contains_key(&(0, 10, 0)), foot, "species {species}");
    }
}

#[test]
fn saplings_grow_their_own_species() {
    assert_eq!(species_of_sapling(SAPLING), OAK);
    assert_eq!(species_of_sapling(BIRCH_SAPLING), BIRCH);
    assert_eq!(species_of_sapling(PINE_SAPLING), PINE);
    for s in [OAK, BROAD, PINE, BIRCH] {
        let (min, spread) = trunk_range(s);
        assert!(min >= 3 && spread >= 2);
    }
}

/// Which of the forest's blocks a square of chunk columns around world point (x, z) holds, by block.
fn forest_around(g: &mut WorldGen, x: i32, z: i32) -> HashMap<BlockId, usize> {
    let mut seen = HashMap::new();
    for cz in (z >> 5) - 2..=(z >> 5) + 2 {
        for cx in (x >> 5) - 2..=(x >> 5) + 2 {
            for cy in 0..WORLD_HEIGHT_CHUNKS {
                let c = g.generate(IVec3::new(cx, cy, cz));
                for (lx, ly, lz) in (0..32).flat_map(|a| (0..32).flat_map(move |b| (0..32).map(move |d| (a, b, d)))) {
                    let b = c.get(lx, ly, lz);
                    if [BIRCH_LOG, PINE_LEAVES, CACTUS, SNAG_LOG].contains(&b) {
                        *seen.entry(b).or_default() += 1;
                    }
                }
            }
        }
    }
    seen
}

#[test]
fn each_biome_grows_its_own_trees_from_version_9_and_older_worlds_keep_theirs() {
    // Seed 1337: a desert, a basalt field and the highlands (found by `biomes`).
    let places = [("desert", -353, 353, CACTUS), ("basalt", 400, 0, SNAG_LOG), ("highlands", 76, -184, PINE_LEAVES)];
    let mut new = WorldGen::with_version(1337, 9);
    let mut old = WorldGen::with_version(1337, 8);
    for (name, x, z, block) in places {
        let found = forest_around(&mut new, x, z);
        assert!(found.get(&block).copied().unwrap_or(0) > 0, "{name} grows {block}: {found:?}");
        if name == "desert" {
            assert!(!found.contains_key(&PINE_LEAVES), "a desert has no pines: {found:?}");
        }
        assert!(forest_around(&mut old, x, z).is_empty(), "version 8 knows only the oak");
    }
}
