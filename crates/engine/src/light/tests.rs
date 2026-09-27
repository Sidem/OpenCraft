use super::*;
use crate::block::{BlockId, AIR, LAMP, LEAVES, STONE, WATER};
use crate::worldgen::WORLD_HEIGHT_CHUNKS;

/// Lights chunk (`cx`, `cy`, `cz`) of a world given by `block` at world coordinates; returns the pad.
fn light_chunk(block: &dyn Fn(i32, i32, i32) -> BlockId, cx: i32, cy: i32, cz: i32) -> Vec<u8> {
    let chunk = |cx: i32, cy: i32, cz: i32| {
        if cy < 0 {
            return Chunk::uniform(STONE);
        }
        if cy >= WORLD_HEIGHT_CHUNKS {
            return Chunk::uniform(AIR);
        }
        let mut blocks = vec![AIR; 32 * 32 * 32];
        for y in 0..32 {
            for z in 0..32 {
                for x in 0..32 {
                    blocks[index(x, y, z)] = block(cx * 32 + x as i32, cy * 32 + y as i32, cz * 32 + z as i32);
                }
            }
        }
        Chunk::from_blocks(blocks)
    };
    let n: Vec<Chunk> = (0..27).map(|i| chunk(cx + i % 3 - 1, cy + i / 3 % 3 - 1, cz + i / 9 - 1)).collect();
    let above: Vec<Vec<Chunk>> = (0..9)
        .map(|i| (cy + 2..WORLD_HEIGHT_CHUNKS).map(|y| chunk(cx + i % 3 - 1, y, cz + i / 3 - 1)).collect())
        .collect();
    let refs: [&Chunk; 27] = std::array::from_fn(|i| &n[i]);
    let above_refs: Vec<Vec<&Chunk>> = above.iter().map(|c| c.iter().collect()).collect();
    let mut l = Lighting::default();
    l.light(&refs, &std::array::from_fn(|i| above_refs[i].as_slice()));
    l.pad
}

/// Sky and block light at world (x, y, z), which must lie in the pad of chunk (0, 0, 0).
fn at(pad: &[u8], x: i32, y: i32, z: i32) -> (u8, u8) {
    let l = pad[((y + 1) as usize * PAD + (z + 1) as usize) * PAD + (x + 1) as usize];
    (l & 15, l >> 4)
}

/// Stone below `top`, air above.
fn ground(y: i32, top: i32) -> BlockId {
    if y < top {
        STONE
    } else {
        AIR
    }
}

/// A flat square at height `y` reaching past the lit field of chunk (0, 0, 0) on three sides.
fn sheet(x: i32, y: i32, z: i32, at_y: i32) -> bool {
    y == at_y && (-40..=40).contains(&x) && (-40..=40).contains(&z)
}

#[test]
fn open_ground_is_sky_lit_and_caves_are_dark() {
    let pocket = |x: i32, y: i32, z: i32| (10..13).contains(&x) && (5..8).contains(&y) && (10..13).contains(&z);
    let pad = light_chunk(&|x, y, z| if pocket(x, y, z) { AIR } else { ground(y, 16) }, 0, 0, 0);
    assert_eq!(at(&pad, 5, 16, 5), (15, 0));
    assert_eq!(at(&pad, 11, 6, 11), (0, 0), "a sealed cave is dark");
    assert_eq!(at(&pad, 5, 5, 5), (0, 0), "stone holds no light");
}

#[test]
fn a_roof_shades_the_ground_and_light_creeps_in_from_its_edge() {
    // The roof ends at x = 20: open sky from x = 21.
    let roof = |x: i32, y: i32, z: i32| sheet(x, y, z, 24) && x <= 20;
    let pad = light_chunk(&|x, y, z| if roof(x, y, z) { STONE } else { ground(y, 16) }, 0, 0, 0);
    assert_eq!(at(&pad, 0, 17, 16).0, 0, "deep under the roof");
    assert_eq!(at(&pad, 15, 17, 16).0, 9, "six blocks in from the edge");
    assert_eq!(at(&pad, 25, 17, 16).0, 15, "beside the roof");
    assert_eq!(at(&pad, 16, 30, 16).0, 15, "on the roof");
}

#[test]
fn a_shaft_lets_the_sky_down() {
    // A tunnel at y = 10 under 18 blocks of stone, with a shaft up to the surface at x = z = 16.
    let world = |open: bool| {
        move |x: i32, y: i32, z: i32| {
            let shaft = x == 16 && z == 16 && y >= 10 && (open || y < 27);
            let tunnel = y == 10 && z == 16 && (16..=22).contains(&x);
            if shaft || tunnel {
                AIR
            } else {
                ground(y, 28)
            }
        }
    };
    let capped = light_chunk(&world(false), 0, 0, 0);
    assert_eq!(at(&capped, 16, 10, 16).0, 0, "a capped shaft stays dark");
    let pad = light_chunk(&world(true), 0, 0, 0);
    assert_eq!(at(&pad, 16, 10, 16).0, 15, "straight down the shaft");
    assert_eq!(at(&pad, 19, 10, 16).0, 12, "three blocks along the side tunnel");
}

#[test]
fn leaves_shade_the_column_but_let_light_through() {
    let pad = light_chunk(&|x, y, z| if sheet(x, y, z, 20) { LEAVES } else { ground(y, 16) }, 0, 0, 0);
    assert_eq!(at(&pad, 16, 17, 16).0, 11, "15 above the leaves, then 1 less per block");
}

#[test]
fn light_matches_across_chunk_borders() {
    let world = |x: i32, y: i32, z: i32| {
        if (x, y, z) == (30, 20, 16) {
            LAMP
        } else if sheet(x, y, z, 24) && (x + z) % 7 != 0 {
            STONE
        } else {
            ground(y, 16 + (x * 3 + z * 5).rem_euclid(9) / 4)
        }
    };
    let here = light_chunk(&world, 0, 0, 0);
    let east = light_chunk(&world, 1, 0, 0);
    let up = light_chunk(&world, 0, 1, 0);
    let idx = |x: usize, y: usize, z: usize| (y * PAD + z) * PAD + x;
    assert_eq!(east[idx(3, 21, 17)] >> 4, 11, "the lamp at x = 30 lights x = 34, four blocks away");
    for a in 0..PAD {
        for b in 0..PAD {
            assert_eq!(here[idx(33, a, b)], east[idx(1, a, b)]);
            assert_eq!(here[idx(32, a, b)], east[idx(0, a, b)]);
            assert_eq!(here[idx(a, 33, b)], up[idx(a, 1, b)]);
        }
    }
}

#[test]
fn a_lamp_lights_a_cave_room_and_removing_it_darkens_it() {
    let room = |x: i32, y: i32, z: i32| (8..17).contains(&x) && (5..9).contains(&y) && (8..17).contains(&z);
    let cave = |lamp: bool| {
        move |x: i32, y: i32, z: i32| {
            if lamp && (x, y, z) == (12, 5, 12) {
                LAMP
            } else if room(x, y, z) {
                AIR
            } else {
                STONE
            }
        }
    };
    let lit = light_chunk(&cave(true), 0, 0, 0);
    assert_eq!(at(&lit, 12, 6, 12), (0, 14), "right above the lamp");
    assert_eq!(at(&lit, 15, 5, 12), (0, 12), "three blocks away");
    assert_eq!(at(&lit, 14, 7, 13), (0, 10), "five steps away, around a corner");
    assert_eq!(at(&lit, 12, 12, 12), (0, 0), "no light inside the rock");
    let dark = light_chunk(&cave(false), 0, 0, 0);
    assert_eq!(at(&dark, 15, 5, 12), (0, 0), "dark again without it");
}

#[test]
fn water_dims_light_by_two_per_block() {
    // A lake with its surface at y 20 over ground at y 8, wide enough to reach past the lit field.
    let pad = light_chunk(
        &|_, y, _| {
            if y < 8 {
                STONE
            } else if y <= 20 {
                WATER
            } else {
                AIR
            }
        },
        0,
        0,
        0,
    );
    assert_eq!(at(&pad, 5, 21, 5).0, 15, "open sky above the water");
    let sky: Vec<u8> = (15..=20).rev().map(|y| at(&pad, 5, y, 5).0).collect();
    assert_eq!(sky, [13, 11, 9, 7, 5, 3], "each block of water takes 2");
    assert_eq!(at(&pad, 5, 8, 5).0, 0, "the deep bottom is dark");
}
