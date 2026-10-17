use super::*;
use crate::block::{LEAVES, STONE, WATER};
use crate::tests::{build_mine, find_outcrop_block, run_until_ready};
use crate::Game;

fn world() -> Game {
    let mut g = Game::new(2024, 3);
    run_until_ready(&mut g);
    g
}

fn read(g: &Game, eye: Vec3) -> [f32; FLOATS] {
    survey(&g.sim.world, &g.sim.factory, eye)
}

/// An eye a few blocks above the ground at (x, z).
fn above_ground(g: &Game, x: i32, z: i32) -> Vec3 {
    Vec3::new(f64::from(x) + 0.5, f64::from(g.sim.world.generator().height_at(x, z)) + 4.5, f64::from(z) + 0.5)
}

/// A 41 × 41 slab of water at height `y` around `cell`.
fn fill(g: &mut Game, cell: IVec3, y: i32) {
    for dx in -20..=20 {
        for dz in -20..=20 {
            g.sim.world.set_block_anywhere_later(IVec3::new(cell.x + dx, y, cell.z + dz), WATER);
        }
    }
}

#[test]
fn open_ground_reads_as_sky_and_the_height_it_is_at() {
    let g = world();
    let eye = above_ground(&g, 0, 0);
    let r = read(&g, eye);
    assert_eq!((r[BIOME], r[SKY], r[DEPTH]), (0.0, 1.0, 0.0), "spawn is plains under open sky");
    assert!((r[HEIGHT] - (eye.y as f32 - SEA_LEVEL as f32)).abs() < 1e-3);
    assert!(r[OPEN] > 0.8, "the spawn is mostly open: {}", r[OPEN]);
    assert_eq!((r[SEA], r[FRESH], r[DIN]), (0.0, 0.0, 0.0));
}

#[test]
fn a_roof_closes_the_sky_and_leaves_only_dim_it() {
    let mut g = world();
    let eye = above_ground(&g, 0, 0);
    let cell = eye.floor();
    g.sim.world.set_block_anywhere(IVec3::new(cell.x, cell.y + 5, cell.z), LEAVES);
    let leafy = read(&g, eye)[SKY];
    assert!(leafy > 0.0 && leafy < 1.0, "one leaf block dims the sky to {leafy}");
    g.sim.world.set_block_anywhere(IVec3::new(cell.x, cell.y + 6, cell.z), STONE);
    assert_eq!(read(&g, eye)[SKY], 0.0, "stone closes it");
}

#[test]
fn depth_counts_blocks_below_the_surface() {
    let g = world();
    let top = above_ground(&g, 0, 0);
    let deep = Vec3::new(top.x, top.y - 24.0, top.z);
    let r = read(&g, deep);
    assert!((r[DEPTH] - 19.5).abs() < 1e-3, "{}", r[DEPTH]);
}

#[test]
fn water_at_sea_level_is_sea_and_other_water_is_fresh() {
    let mut g = world();
    let cell = above_ground(&g, 0, 0).floor();
    fill(&mut g, cell, SEA_LEVEL);
    let eye = Vec3::new(f64::from(cell.x) + 0.5, f64::from(SEA_LEVEL) + 3.5, f64::from(cell.z) + 0.5);
    let r = read(&g, eye);
    assert!(r[SEA] > 0.9, "standing over a sea: {}", r[SEA]);
    assert_eq!(r[FRESH], 0.0, "sea water is not fresh water");
    fill(&mut g, cell, SEA_LEVEL + 12);
    let high = read(&g, Vec3::new(eye.x, f64::from(SEA_LEVEL) + 14.5, eye.z));
    assert!(high[FRESH] > 0.5, "a pool just below: {}", high[FRESH]);
    assert!(high[SEA] < r[SEA], "the sea is quieter from 12 blocks up: {} against {}", high[SEA], r[SEA]);
}

/// What a stream (a strip of still water 3 blocks wide and 1 deep, 20 blocks each way) reads as `d` blocks to the side.
fn stream_at(g: &mut Game, d: i32) -> f32 {
    let cell = above_ground(g, 0, 0).floor();
    let y = SEA_LEVEL + 30;
    for dx in -20..=20 {
        for dz in 0..3 {
            g.sim.world.set_block_anywhere_later(IVec3::new(cell.x + dx, y, cell.z + dz), WATER);
        }
    }
    read(g, Vec3::new(f64::from(cell.x) + 0.5, f64::from(y) + 2.5, f64::from(cell.z) + 1.5 + f64::from(d)))[FRESH]
}

#[test]
fn a_stream_fades_quickly_with_distance() {
    let mut g = world();
    let levels = [0, 3, 6, 9, 12].map(|d| stream_at(&mut g, d));
    println!("stream by distance {levels:?}");
    assert!(levels[0] > 0.6, "beside the stream: {levels:?}");
    assert!(levels.windows(2).all(|w| w[1] < w[0]), "fades steadily: {levels:?}");
    assert!(levels[2] < 0.5 * levels[0], "half the din by 6 blocks: {levels:?}");
    assert!(levels[3] < 0.15, "a whisper at 9 blocks: {levels:?}");
    assert_eq!(levels[4], 0.0, "out of earshot at 12: {levels:?}");
}

#[test]
fn leaves_around_read_as_trees() {
    let mut g = world();
    let eye = above_ground(&g, 0, 0);
    let cell = eye.floor();
    assert!(read(&g, eye)[TREES] < 0.5, "a few natural trees near spawn at most");
    for dx in -9..=9 {
        for dz in -9..=9 {
            for dy in [2, 4, 6] {
                g.sim.world.set_block_anywhere_later(IVec3::new(cell.x + dx, cell.y + dy, cell.z + dz), LEAVES);
            }
        }
    }
    assert!(read(&g, eye)[TREES] > 0.9);
}

#[test]
fn only_working_machines_make_din_and_it_fades_with_distance() {
    let mut g = world();
    let (p, _) = find_outcrop_block(&mut g, 6);
    let (m, _) = build_mine(&mut g, p);
    let at = Vec3::new(f64::from(m.x) + 0.5, f64::from(m.y) + 1.5, f64::from(m.z) + 0.5);
    g.skip_time(5.0);
    let near = read(&g, at)[DIN];
    let farther = read(&g, Vec3::new(at.x + 14.0, at.y, at.z))[DIN];
    assert!(near > 0.2 && farther > 0.0 && farther < near, "near {near}, farther {farther}");
    assert_eq!(read(&g, Vec3::new(at.x + 200.0, at.y, at.z))[DIN], 0.0);
}
