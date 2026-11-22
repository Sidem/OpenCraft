//! Unit, physics, meshing, and save round-trip tests for block shape modifiers (`L-SLAB`).

use super::*;
use crate::action::Action;
use crate::block;
use crate::chunk::Chunk;
use crate::math::{IVec3, Vec3};
use crate::mesher::{neighbor_index, Mesher, PAD_VOLUME};
use crate::player::{Player, PlayerInput};
use crate::sim::{PlayerId, Sim};

const P: PlayerId = PlayerId(0);

#[test]
fn roof_tiles_and_building_blocks_are_shapeable() {
    for b in [
        block::STONE,
        block::STONE_BRICKS,
        block::CONCRETE,
        block::RED_CONCRETE,
        block::BLUE_CONCRETE,
        block::PLANKS,
        block::WOOD_FLOOR,
        block::planks_of(0),
        block::floor_of(3),
        block::RED_ROOF,
        block::SLATE_ROOF,
        block::GREEN_ROOF,
        block::THATCH,
        block::PLASTER,
        block::TIMBER_FRAME,
        block::METAL_PANEL,
        block::CATWALK,
        block::GLASS,
        block::FRAMED_GLASS,
        block::WINDOW,
        block::STAINED_GLASS_FIRST,
        block::STAINED_GLASS_FIRST + 15,
    ] {
        assert!(is_shapeable(b), "expected block {b} ({}) to be shapeable", block::def(b).name);
    }
    for b in [
        block::AIR,
        block::WATER,
        block::BELT,
        block::MINER,
        block::ASSEMBLER,
        block::WOOD_DOOR,
        block::TORCH,
        block::COAL_ORE,
    ] {
        assert!(!is_shapeable(b), "expected block {b} not to be shapeable");
    }
}

#[test]
fn shape_encode_decode_and_rotation_round_trip() {
    for s in 0..=31u8 {
        if is_valid(s) {
            assert_eq!(encode(kind(s)), s, "shape {s} failed encode/decode round-trip");
            if is_rotatable(s) {
                let mut cur = s;
                for _ in 0..4 {
                    cur = rotate_cw(cur);
                }
                assert_eq!(cur, s, "4 clockwise turns must return to shape {s}");
            }
        }
    }
}

#[test]
fn slab_stacking_grows_quarters_halves_and_full() {
    assert_eq!(stack_slab(SLAB_B1, MODE_SLAB_QUARTER, IVec3::new(0, 1, 0)), Some(SLAB_B2));
    assert_eq!(stack_slab(SLAB_B2, MODE_SLAB_QUARTER, IVec3::new(0, 1, 0)), Some(SLAB_B3));
    assert_eq!(stack_slab(SLAB_B3, MODE_SLAB_QUARTER, IVec3::new(0, 1, 0)), Some(FULL));
    assert_eq!(stack_slab(SLAB_B2, MODE_SLAB_HALF, IVec3::new(0, 1, 0)), Some(FULL));
    assert_eq!(stack_slab(SLAB_T1, MODE_SLAB_HALF, IVec3::new(0, -1, 0)), Some(SLAB_T3));
    assert_eq!(stack_slab(SLAB_T2, MODE_SLAB_HALF, IVec3::new(0, -1, 0)), Some(FULL));
    // Clicking the bottom of a bottom slab does not stack it
    assert_eq!(stack_slab(SLAB_B2, MODE_SLAB_HALF, IVec3::new(0, -1, 0)), None);
}

#[test]
fn hip_roof_corners_auto_connect() {
    // 3x3 ring of slopes facing inward toward (1, 0, 1):
    // NW corner (0, 0, 0) faces South (dir=2); behind it at (0, 0, 1) is West eaves facing East (dir=1).
    let s_south = encode(Kind::Slope { dir: 2, inverted: false });
    let s_east = encode(Kind::Slope { dir: 1, inverted: false });
    let corner = corner_of(s_south, IVec3::new(0, 0, 0), |q| if q == IVec3::new(0, 0, 1) { s_east } else { FULL });
    assert_eq!(corner, Corner::OuterLeft);
}

#[test]
fn walking_on_quarter_half_and_three_quarter_slabs_sets_exact_heights() {
    // Floor at y=0, then at x=1 quarter slab (0.25), x=2 half slab (0.50), x=3 three-quarter slab (0.75), x=4 full (1.0).
    let slab_at = |x: i32, y: i32| -> u8 {
        if y == 0 {
            return FULL;
        }
        if y == 1 {
            return match x {
                1 => SLAB_B1,
                2 => SLAB_B2,
                3 => SLAB_B3,
                4 => FULL,
                _ => 0,
            };
        }
        0
    };
    let mut boxes = |x: i32, y: i32, z: i32| -> CellBoxes {
        if y == 0 {
            return CellBoxes::full(IVec3::new(x, y, z));
        }
        if y == 1 && x >= 1 && (-2..=2).contains(&z) {
            let s = if x <= 3 { slab_at(x, y) } else { FULL };
            return boxes_for(IVec3::new(x, y, z), s, Corner::Straight);
        }
        CellBoxes::EMPTY
    };
    let mut blk = |_: i32, _: i32, _: i32| block::AIR;

    // Test dropping onto each slab height directly
    for (x, expected_y) in [(1, 1.25), (2, 1.50), (3, 1.75), (4, 2.00)] {
        let mut p = Player::new(Vec3::new(x as f64 + 0.5, 2.5, 0.5));
        for _ in 0..120 {
            p.step_boxes(1.0 / 120.0, &mut boxes, &mut blk);
        }
        assert!(p.on_ground, "player should land on slab at x={x}");
        assert!((p.pos.y - expected_y).abs() < 1e-4, "slab at x={x}: expected feet at {expected_y}, got {}", p.pos.y);
    }

    // Walk continuously from x=0.5 (y=1.0) across ¼ -> ½ -> ¾ -> Full without jumping!
    let mut p = Player::new(Vec3::new(0.5, 1.0, 0.5));
    p.yaw = std::f64::consts::FRAC_PI_2; // face +X
    p.input = PlayerInput { forward: 1.0, ..Default::default() };
    for _ in 0..240 {
        p.step_boxes(1.0 / 120.0, &mut boxes, &mut blk);
    }
    assert!(p.pos.x > 4.2, "player should smoothly walk across all slab heights to full block, x={}", p.pos.x);
    assert!((p.pos.y - 2.0).abs() < 1e-4, "player should reach y=2.0, got {}", p.pos.y);
}

#[test]
fn walking_up_and_down_stairs_is_smooth_and_grounded() {
    // 5-block staircase ascending toward +X (dir=1) from (1,1,0) up to (5,5,0), with landings at x<=0 (y=0) and x>=6 (y=5).
    let stair = encode(Kind::Stair { dir: 1, inverted: false });
    let mut boxes = |x: i32, y: i32, z: i32| -> CellBoxes {
        if (-2..=2).contains(&z) {
            if y <= 0 {
                return CellBoxes::full(IVec3::new(x, y, z));
            }
            if (1..=5).contains(&x) && y == x {
                return boxes_for(IVec3::new(x, y, z), stair, Corner::Straight);
            }
            if (1..=5).contains(&x) && y < x {
                return CellBoxes::full(IVec3::new(x, y, z));
            }
            if x >= 6 && y <= 5 {
                return CellBoxes::full(IVec3::new(x, y, z));
            }
        }
        CellBoxes::EMPTY
    };
    let mut blk = |_: i32, _: i32, _: i32| block::AIR;

    // Walk UP stairs until reaching the top landing (x >= 6.5) without jumping
    let mut p = Player::new(Vec3::new(0.3, 1.0, 0.5));
    p.yaw = std::f64::consts::FRAC_PI_2; // +X
    p.input = PlayerInput { forward: 1.0, ..Default::default() };
    for _ in 0..240 {
        p.step_boxes(1.0 / 120.0, &mut boxes, &mut blk);
        if p.pos.x >= 6.5 {
            break;
        }
    }
    assert!(p.pos.x >= 6.5, "player should climb entire 5-block staircase without jumping, x={}", p.pos.x);
    assert!((p.pos.y - 6.0).abs() < 1e-4, "expected top landing y=6.0, got {}", p.pos.y);
    assert!(p.on_ground);

    // Turn around and walk DOWN stairs: must stay grounded without bouncing or triggering landing thuds
    p.yaw = -std::f64::consts::FRAC_PI_2; // -X
    p.landing_speed = 0.0;
    let mut airborne_steps = 0u32;
    for _ in 0..300 {
        p.step_boxes(1.0 / 120.0, &mut boxes, &mut blk);
        if !p.on_ground {
            airborne_steps += 1;
        }
        if p.pos.x <= 0.4 {
            break;
        }
    }
    assert!(p.pos.x < 0.5, "player should descend to bottom landing, x={}", p.pos.x);
    assert!((p.pos.y - 1.0).abs() < 1e-4, "expected bottom landing y=1.0, got {}", p.pos.y);
    assert_eq!(airborne_steps, 0, "walking down stairs should stay smoothly grounded via step-down");
    assert_eq!(p.landing_speed, 0.0, "smooth stair descent must not trigger fall landing thuds");
}

#[test]
fn walking_up_and_down_pitched_roof_slopes_is_smooth() {
    let slope = encode(Kind::Slope { dir: 1, inverted: false });
    let mut boxes = |x: i32, y: i32, z: i32| -> CellBoxes {
        if (-2..=2).contains(&z) {
            if y <= 0 {
                return CellBoxes::full(IVec3::new(x, y, z));
            }
            if (1..=4).contains(&x) && y == x {
                return boxes_for(IVec3::new(x, y, z), slope, Corner::Straight);
            }
            if (1..=4).contains(&x) && y < x {
                return CellBoxes::full(IVec3::new(x, y, z));
            }
            if x >= 5 && y <= 4 {
                return CellBoxes::full(IVec3::new(x, y, z));
            }
        }
        CellBoxes::EMPTY
    };
    let mut blk = |_: i32, _: i32, _: i32| block::AIR;

    let mut p = Player::new(Vec3::new(0.3, 1.0, 0.5));
    p.yaw = std::f64::consts::FRAC_PI_2;
    p.input = PlayerInput { forward: 1.0, ..Default::default() };
    for _ in 0..240 {
        p.step_boxes(1.0 / 120.0, &mut boxes, &mut blk);
        if p.pos.x >= 5.5 {
            break;
        }
    }
    assert!(p.pos.x >= 5.5 && (p.pos.y - 5.0).abs() < 1e-4, "climbed pitched slope to ({}, {})", p.pos.x, p.pos.y);

    p.yaw = -std::f64::consts::FRAC_PI_2;
    let mut airborne = 0u32;
    for _ in 0..240 {
        p.step_boxes(1.0 / 120.0, &mut boxes, &mut blk);
        if !p.on_ground {
            airborne += 1;
        }
        if p.pos.x <= 0.4 {
            break;
        }
    }
    assert!(p.pos.x < 0.5 && (p.pos.y - 1.0).abs() < 1e-4, "descended pitched slope to ({}, {})", p.pos.x, p.pos.y);
    assert_eq!(airborne, 0, "descending pitched roof slope should stay smoothly grounded");
}

#[test]
fn mesher_emits_opaque_cutout_and_translucent_shaped_quads() {
    let air = Chunk::uniform(block::AIR);
    let mut center = Chunk::uniform(block::AIR);
    center.set(4, 4, 4, block::RED_ROOF);
    center.set_shape(4, 4, 4, encode(Kind::Slope { dir: 1, inverted: false }));
    center.set(6, 4, 4, block::CATWALK);
    center.set_shape(6, 4, 4, encode(Kind::Stair { dir: 0, inverted: false }));
    center.set(8, 4, 4, block::STAINED_GLASS_FIRST);
    center.set_shape(8, 4, 4, SLAB_B2);

    let mut refs = [&air; 27];
    refs[neighbor_index(0, 0, 0)] = &center;
    let light = vec![15u8; PAD_VOLUME];
    let mut mesher = Mesher::new();
    let out = mesher.mesh(&refs, &light, None);
    assert_eq!(out.opaque_quads, 0, "shaped blocks must not also emit cube quads");
    assert!(out.shaped_opaque > 0, "red roof slope emits shaped opaque quads");
    assert!(out.shaped_cutout > 0, "catwalk stair emits shaped cutout quads");
    assert!(out.shaped_translucent > 0, "stained glass slab emits shaped translucent quads");
}

#[test]
fn place_stack_rotate_break_and_save_round_trip() {
    let mut sim = Sim::new(7, 2);
    let pos = IVec3::new(10, 120, 10);
    sim.apply(P, Action::Give { item: block::RED_ROOF.into(), count: 2 });
    // Place quarter slab -> consumes 1 item
    sim.apply(P, Action::PlaceShaped { pos, slot: 0, shape: SLAB_B1 });
    assert_eq!(sim.world.block_anywhere(pos), Some(block::RED_ROOF));
    assert_eq!(sim.world.shape_anywhere(pos), SLAB_B1);
    assert_eq!(sim.player(P).unwrap().inventory.count(block::RED_ROOF.into()), 1);

    // Grow slab in-place to half -> 0 extra item cost
    sim.apply(P, Action::PlaceShaped { pos, slot: 0, shape: SLAB_B2 });
    assert_eq!(sim.world.shape_anywhere(pos), SLAB_B2);
    assert_eq!(sim.player(P).unwrap().inventory.count(block::RED_ROOF.into()), 1);

    // Place a pitched roof slope next to it and rotate it clockwise
    let pos2 = IVec3::new(11, 120, 10);
    let slope0 = encode(Kind::Slope { dir: 0, inverted: false });
    sim.apply(P, Action::PlaceShaped { pos: pos2, slot: 0, shape: slope0 });
    assert_eq!(sim.world.shape_anywhere(pos2), slope0);
    assert_eq!(sim.player(P).unwrap().inventory.count(block::RED_ROOF.into()), 0);
    sim.apply(P, Action::Rotate { pos: pos2 });
    assert_eq!(sim.world.shape_anywhere(pos2), encode(Kind::Slope { dir: 1, inverted: false }));

    // Save/load state round-trip preserves exact shapes and state hash
    let mut w = crate::bytes::ByteWriter::default();
    sim.write_state(&mut w);
    let mut loaded = Sim::new(7, 2);
    let mut r = crate::bytes::ByteReader::new(&w.bytes);
    r.version = crate::save::SAVE_VERSION;
    assert!(loaded.read_state(&mut r).is_some());
    assert_eq!(loaded.world.shape_anywhere(pos), SLAB_B2);
    assert_eq!(loaded.world.shape_anywhere(pos2), encode(Kind::Slope { dir: 1, inverted: false }));
    assert_eq!(loaded.state_hash(), sim.state_hash());

    // Breaking a shaped block clears its shape cleanly
    sim.apply(P, Action::BreakBlock { pos: pos2 });
    assert_eq!(sim.world.block_anywhere(pos2), Some(block::AIR));
    assert_eq!(sim.world.shape_anywhere(pos2), FULL);
}
