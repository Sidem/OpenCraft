use super::*;
use crate::block::{COAL_ORE, IRON_ORE};

const SEED: u32 = 1337;

#[test]
fn bauxite_ground_lies_far_out_in_a_biome_that_holds_it() {
    let g = WorldGen::new(SEED);
    let b = g.bearing_to(BAUXITE_ORE, 0, 0).expect("some desert or basalt field within 3,000 blocks");
    let biome = g.biome_at(b.x, b.z, g.height_at(b.x, b.z));
    assert!(g.holds_ore(BAUXITE_ORE, biome, IVec3::new(b.x, 0, b.z)), "{biome:?}");
    assert!(b.distance >= 600 - STEP && b.distance <= MAX_RING * STEP, "{b:?}");
    assert_eq!(b.distance, ((b.x as f64).hypot(b.z as f64)).round() as i32);
}

#[test]
fn the_bearing_is_the_nearest_sampled_ground() {
    let g = WorldGen::new(SEED);
    let (x, z) = (-300, 200);
    let b = g.bearing_to(BAUXITE_ORE, x, z).unwrap();
    let holds = |px: i32, pz: i32| {
        let biome = g.biome_at(px, pz, g.height_at(px, pz));
        g.holds_ore(BAUXITE_ORE, biome, IVec3::new(px, 0, pz))
    };
    let d2 = |px: i32, pz: i32| (px as i64 - x as i64).pow(2) + (pz as i64 - z as i64).pow(2);
    let nearest = (-20..=20)
        .flat_map(|j| (-20..=20).map(move |i| (x + i * STEP, z + j * STEP)))
        .filter(|&(px, pz)| holds(px, pz))
        .map(|(px, pz)| d2(px, pz))
        .min()
        .unwrap();
    assert_eq!(d2(b.x, b.z), nearest);
}

#[test]
fn ground_that_already_holds_the_ore_is_here() {
    let g = WorldGen::new(SEED);
    // Spawn is plains: coal and iron lie there, bauxite does not.
    for ore in [COAL_ORE, IRON_ORE] {
        assert_eq!(g.bearing_to(ore, 0, 0), Some(Bearing { x: 0, z: 0, distance: 0 }));
    }
    let b = g.bearing_to(BAUXITE_ORE, 0, 0).unwrap();
    assert_eq!(g.bearing_to(BAUXITE_ORE, b.x, b.z).map(|n| n.distance), Some(0));
}

#[test]
fn old_worlds_have_no_bearings() {
    assert_eq!(WorldGen::with_version(SEED, 4).bearing_to(BAUXITE_ORE, 0, 0), None, "no bauxite before version 5");
    assert_eq!(WorldGen::with_version(SEED, 1).bearing_to(IRON_ORE, 0, 0), None, "no biome ores before version 2");
    assert!(WorldGen::with_version(SEED, 4).bearing_to(IRON_ORE, 0, 0).is_some());
}

#[test]
fn a_bearing_is_the_same_every_time_and_in_any_order() {
    let (a, b) = (WorldGen::new(SEED), WorldGen::new(SEED));
    let first = a.bearing_to(BAUXITE_ORE, 100, -50);
    b.bearing_to(BAUXITE_ORE, -900, 700); // warms b's pond cache differently
    assert_eq!(first, b.bearing_to(BAUXITE_ORE, 100, -50));
    assert_eq!(first, a.bearing_to(BAUXITE_ORE, 100, -50));
}

#[test]
fn ring_cells_walk_each_ring_once() {
    assert_eq!(ring_cells(0).collect::<Vec<_>>(), vec![(0, 0)]);
    for r in 1..6 {
        let cells: Vec<_> = ring_cells(r).collect();
        assert_eq!(cells.len() as i32, 8 * r);
    }
}
