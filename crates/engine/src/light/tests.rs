use super::*;
use crate::block::{BlockId, AIR, LAMP, LEAVES, STONE, TORCH, WATER};
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
fn a_grown_trees_wood_lets_light_round_it_but_a_built_log_wall_does_not() {
    use crate::block::{BAMBOO, BAOBAB_LOG, LOG};
    // A bamboo stalk and a 3Ã—3 baobab bole on open ground, and a placed log box round (24, 17, 24).
    let world = |x: i32, y: i32, z: i32| match (x, y, z) {
        (8, 16..=30, 8) | (3, 15..=30, 3) => BAMBOO,
        (14..=16, 16..=30, 14..=16) => BAOBAB_LOG,
        (23..=25, 16..=18, 23..=25) if (x, y, z) != (24, 17, 24) => LOG,
        _ => ground(y, 16),
    };
    let pad = light_chunk(&world, 0, 0, 0);
    // The ground under a round trunk shows round it: it must not be black.
    assert!(at(&pad, 8, 16, 8).0 >= 13, "under the bamboo: {:?}", at(&pad, 8, 16, 8));
    assert!(at(&pad, 14, 16, 14).0 >= 13, "under the bole's edge: {:?}", at(&pad, 14, 16, 14));
    assert!(at(&pad, 3, 15, 3).0 >= 12, "a culm sunk into the ground: {:?}", at(&pad, 3, 15, 3));
    assert_eq!(at(&pad, 24, 17, 24).0, 0, "inside a log box");
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
    assert_eq!(east[idx(9, 21, 17)] >> 4, 15, "the lamp at x = 30 lights x = 40, ten blocks away, fully");
    for a in 0..PAD {
        for b in 0..PAD {
            assert_eq!(here[idx(33, a, b)], east[idx(1, a, b)]);
            assert_eq!(here[idx(32, a, b)], east[idx(0, a, b)]);
            assert_eq!(here[idx(a, 33, b)], up[idx(a, 1, b)]);
        }
    }
}

/// A sealed cave room (x and z 8..17, y 5..9) with `light` at (12, 5, 12) if given.
fn cave_room(light: Option<BlockId>) -> impl Fn(i32, i32, i32) -> BlockId {
    let room = |x: i32, y: i32, z: i32| (8..17).contains(&x) && (5..9).contains(&y) && (8..17).contains(&z);
    move |x, y, z| match light {
        Some(b) if (x, y, z) == (12, 5, 12) => b,
        _ if room(x, y, z) => AIR,
        _ => STONE,
    }
}

#[test]
fn a_lamp_lights_a_cave_room_and_removing_it_darkens_it() {
    let lit = light_chunk(&cave_room(Some(LAMP)), 0, 0, 0);
    assert_eq!(at(&lit, 12, 6, 12), (0, 15), "right above the lamp");
    assert_eq!(at(&lit, 14, 7, 13), (0, 15), "five steps away, around a corner: still full");
    assert_eq!(at(&lit, 16, 8, 16), (0, 15), "eleven steps away: 32 âˆ’ 11 is still full");
    assert_eq!(at(&lit, 12, 12, 12), (0, 0), "no light inside the rock");
    let dark = light_chunk(&cave_room(None), 0, 0, 0);
    assert_eq!(at(&dark, 15, 5, 12), (0, 0), "dark again without it");
}

#[test]
fn a_torch_is_full_up_close_and_fades_out_by_eleven_blocks() {
    let lit = light_chunk(&cave_room(Some(TORCH)), 0, 0, 0);
    let row: Vec<u8> = (12..17).map(|x| at(&lit, x, 5, 12).1).collect();
    assert_eq!(row, [15; 5], "full within four blocks");
    assert_eq!(at(&lit, 16, 5, 13).1, 14, "five steps: 24 âˆ’ 10");
    assert_eq!(at(&lit, 16, 7, 16).1, 4, "ten steps");
    assert_eq!(at(&lit, 16, 8, 16).1, 2, "eleven steps: the last lit");
    // Beside a lamp, a cell shows the brighter of the two.
    let both = |x, y, z| if (x, y, z) == (16, 5, 16) { LAMP } else { cave_room(Some(TORCH))(x, y, z) };
    let pad = light_chunk(&both, 0, 0, 0);
    assert_eq!(at(&pad, 8, 8, 8).1, 13, "the lamp 19 steps away (13) outshines the torch 11 steps away (2)");
    assert_eq!(at(&pad, 12, 6, 12).1, 15, "full beside the torch");
}

/// A sealed one-cell tunnel along x at (y 10, z 16) through solid rock, with `lamp` standing at x = 33.
fn tunnel_with(lamp: BlockId) -> impl Fn(i32, i32, i32) -> BlockId {
    move |x, y, z| match (y, z) {
        (10, 16) if x == 33 => lamp,
        (10, 16) if (-100..=140).contains(&x) => AIR,
        _ => STONE,
    }
}

#[test]
fn each_led_step_is_full_then_fades_and_is_dark_by_the_field_margin() {
    use crate::block::{LED, LED_STEPS};
    for (step, lamp) in LED_STEPS.into_iter().enumerate() {
        let (strength, loss) = SOURCES[crate::block::def(lamp).light as usize];
        assert_eq!((loss, lamp >= LED), (2, true));
        let pad = light_chunk(&tunnel_with(lamp), 1, 0, 0);
        for local in -1..=32i32 {
            let d = (local + 32 - 33).abs();
            let expect = (i32::from(strength) - 2 * d).clamp(0, 15) as u8;
            assert_eq!(at(&pad, local, 10, 16).1, expect, "step {step}, {d} blocks from the lamp");
        }
        // The farthest light a step gives lies inside the margin: strength / 2 blocks at most.
        assert!(usize::from(strength) / 2 <= MARGIN, "step {step} reaches {} blocks", strength / 2);
    }
    // Steps are strictly brighter, farther out.
    let at_distance = |lamp: BlockId, d: i32| {
        let pad = light_chunk(&tunnel_with(lamp), 1, 0, 0);
        at(&pad, 33 - 32 + d, 10, 16).1
    };
    assert_eq!(LED_STEPS.map(|l| at_distance(l, 20)), [0, 12, 15], "20 blocks out: dark, dim, still full");
    assert_eq!(LED_STEPS.map(|l| at_distance(l, 13)), [14, 15, 15], "13 blocks out");
}

#[test]
fn the_led_you_place_is_never_weaker_than_a_lamp_and_its_dimmer_steps_are_dimmer() {
    use crate::block::{LED, LED_STEPS};
    assert_eq!(LED_STEPS[2], LED, "the item is the brightest step");
    let row = |lamp: BlockId| -> Vec<u8> {
        let pad = light_chunk(&tunnel_with(lamp), 1, 0, 0);
        (-1..=32).map(|x| at(&pad, x, 10, 16).1).collect()
    };
    let (lamp, led) = (row(LAMP), row(LED));
    assert!(led.iter().zip(&lamp).all(|(l, p)| l >= p), "LED {led:?} against lamp {lamp:?}");
    assert!(led.iter().zip(&lamp).any(|(l, p)| l > p), "and better somewhere");
    for dim in [LED_STEPS[0], LED_STEPS[1]] {
        let dimmer = row(dim);
        assert!(dimmer.iter().zip(&led).all(|(d, l)| d <= l), "a dimmer step never beats the item");
        assert!(dimmer.iter().zip(&lamp).any(|(d, p)| d < p), "a dim step is the one that falls short of a lamp");
    }
}

/// Stone below y 10 and a street lamp (three posts, head on top) based at (0, 10, 16).
fn street_lamp_on_ground(x: i32, y: i32, z: i32) -> BlockId {
    use crate::block::{street_post, STREET};
    match (x, z, y) {
        (0, 16, 10..=12) => street_post(0),
        (0, 16, 13) => STREET,
        _ => ground(y, 10),
    }
}

#[test]
fn a_street_lamp_head_lights_the_ground_four_blocks_below_it_as_an_led_in_the_air_would() {
    use crate::block::{LAMP, LED, STREET};
    let pad = light_chunk(&street_lamp_on_ground, 0, 0, 0);
    assert_eq!(at(&pad, 0, 10, 16).1, 15, "the base cell, four blocks below the head");
    assert_eq!(at(&pad, 1, 10, 16).1, 15, "the ground beside the post");
    let (strength, loss) = SOURCES[crate::block::def(STREET).light as usize];
    assert_eq!(crate::block::def(STREET).light, crate::block::def(LED).light, "the head is the brightest LED row");
    // The field steps by faces, so the ground k blocks out is k + 3 blocks from the head.
    let street = |k: i32| at(&pad, k, 10, 16).1;
    for k in 0..=29 {
        let expect = (i32::from(strength) - i32::from(loss) * (k + 3)).clamp(0, 15) as u8;
        assert_eq!(street(k), expect, "ground {k} blocks from the post");
    }
    // Against the same lamps standing on that ground: an LED reaches 3 blocks (6 levels) farther along it, and a plain lamp
    // lights less up to the last few blocks of its reach, where it is under a third as bright.
    let on_ground =
        |lamp: BlockId| light_chunk(&|x, y, z| if (x, y, z) == (0, 10, 16) { lamp } else { ground(y, 10) }, 0, 0, 0);
    let (led, lamp) = (on_ground(LED), on_ground(LAMP));
    for k in 0..=29 {
        let (l, p) = (at(&led, k, 10, 16).1, at(&lamp, k, 10, 16).1);
        assert!(street(k) <= l && l - street(k) <= 6, "ground {k}: street {} led {l}", street(k));
        assert!(p <= street(k) || p <= 5, "ground {k}: a lamp's {p} against the street lamp's {}", street(k));
    }
    // What its height buys: a wall across the way, two high, shades a lamp on the ground but not one four blocks up.
    let walled = |x: i32, y: i32| x == 2 && (10..12).contains(&y);
    let street_wall =
        light_chunk(&|x, y, z| if walled(x, y) { STONE } else { street_lamp_on_ground(x, y, z) }, 0, 0, 0);
    let led_wall = light_chunk(
        &|x, y, z| match (x, y, z) {
            _ if walled(x, y) => STONE,
            (0, 10, 16) => LED,
            _ => ground(y, 10),
        },
        0,
        0,
        0,
    );
    assert!(at(&street_wall, 24, 10, 16).1 > at(&led_wall, 24, 10, 16).1, "behind the wall, 24 blocks out");
}
#[test]
fn led_light_matches_across_chunk_borders_even_at_the_far_end() {
    use crate::block::LED_STEPS;
    const LED_BRIGHTEST: BlockId = LED_STEPS[2];
    // The lamp in chunk 1 (x 32..63) lights the border with chunk 2 (x 64) from 31 blocks away, where it is nearly out.
    let world = tunnel_with(LED_BRIGHTEST);
    let (one, two) = (light_chunk(&world, 1, 0, 0), light_chunk(&world, 2, 0, 0));
    let idx = |x: usize, y: usize, z: usize| (y * PAD + z) * PAD + x;
    assert_eq!(one[idx(33, 11, 17)] >> 4, 2, "x = 64 is 31 blocks away: 64 âˆ’ 62");
    for a in 0..PAD {
        for b in 0..PAD {
            assert_eq!(one[idx(33, a, b)], two[idx(1, a, b)]);
            assert_eq!(one[idx(32, a, b)], two[idx(0, a, b)]);
        }
    }
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

/// `cargo test --release -q bench_light -- --ignored --nocapture`: lighting one chunk beside no lamp, a lamp and an LED.
#[test]
#[ignore]
fn bench_light_one_chunk_by_source() {
    use crate::block::LED_STEPS;
    use std::time::Instant;
    let mut l = Lighting::default();
    for (name, lamp) in [
        ("none", AIR),
        ("lamp (loss 1)", LAMP),
        ("torch (loss 2, 24)", TORCH),
        ("LED brightest (loss 2, 64)", LED_STEPS[2]),
    ] {
        let world = move |x: i32, y: i32, z: i32| {
            if (x, y, z) == (16, 20, 16) {
                lamp
            } else {
                ground(y, 16)
            }
        };
        let chunk = |cx: i32, cy: i32, cz: i32| {
            let mut blocks = vec![AIR; 32 * 32 * 32];
            for ly in 0..32usize {
                for lz in 0..32usize {
                    for lx in 0..32usize {
                        blocks[index(lx, ly, lz)] =
                            world(cx * 32 + lx as i32, cy * 32 + ly as i32, cz * 32 + lz as i32);
                    }
                }
            }
            Chunk::from_blocks(blocks)
        };
        let n: Vec<Chunk> = (0..27).map(|i| chunk(i % 3 - 1, i / 3 % 3 - 1, i / 9 - 1)).collect();
        let refs: [&Chunk; 27] = std::array::from_fn(|i| &n[i]);
        let above: [&[&Chunk]; 9] = [&[]; 9];
        let t = Instant::now();
        for _ in 0..40 {
            l.light(&refs, &above);
        }
        println!("{name}: {:.2} ms a chunk", t.elapsed().as_secs_f64() * 1e3 / 40.0);
    }
}
