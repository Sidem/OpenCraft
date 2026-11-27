use super::{FLIGHT_SECONDS, REACH};
use crate::action::Action;
use crate::block::{AIR, GRASS, STONE};
use crate::bytes::ByteWriter;
use crate::item::{BLADERANG, IRON_PLATE};
use crate::math::{IVec3, Vec3};
use crate::Game;

fn v(x: i32, y: i32, z: i32) -> IVec3 {
    IVec3::new(x, y, z)
}

/// A flat turf field (its ground level returned) with the bladerang selected, the player facing -z.
fn on_turf() -> (Game, i32) {
    let mut g = Game::new(2024, 6);
    crate::tests::run_until_ready(&mut g);
    let y = g.sim.world.generator().height_at(0, 0);
    for x in -10..=10 {
        for z in -20..=6 {
            for h in y - 2..=y + 10 {
                let id = if h < y {
                    STONE
                } else if h == y {
                    GRASS
                } else {
                    AIR
                };
                g.sim.world.set_block_anywhere_later(v(x, h, z), id);
            }
        }
    }
    g.give(BLADERANG.0, 1);
    g.run_ticks(3);
    let slot = (0..9).find(|&i| g.slot_item(i) == BLADERANG.0).expect("in the hotbar");
    g.select_slot(slot);
    g.run_ticks(2);
    let body = g.body_mut();
    body.pos = Vec3::new(0.5, (y + 1) as f64, 0.5);
    (body.yaw, body.pitch) = (0.0, 0.0);
    (g, y)
}

fn throw(g: &mut Game) {
    g.set_using(true);
    g.run_ticks(1);
    g.set_using(false);
}

fn flight_ticks() -> u32 {
    (FLIGHT_SECONDS * f64::from(crate::TICK_RATE)) as u32 + 2
}

#[test]
fn a_throw_loops_out_and_back_cutting_the_plants_under_it_and_breaking_nothing() {
    let (mut g, y) = on_turf();
    assert!(g.blade_label().unwrap().contains("RMB=Throw"));
    throw(&mut g);
    assert!(g.blade.flight.is_some(), "it flies");
    let mut furthest: f64 = 0.0;
    for _ in 0..flight_ticks() {
        g.run_ticks(1);
        if let Some(p) = g.blade_pos() {
            furthest = furthest.max(0.5 - p.z);
        }
    }
    assert!(g.blade.flight.is_none() && furthest > REACH * 0.8, "out {furthest} and back");
    let cleared = (-20..=0).filter(|&z| (-3..=6).any(|x| g.sim.world.is_cover_cleared(x, z))).count();
    assert!(cleared >= 8, "a lane of cut plants ahead: {cleared} rows");
    assert_eq!(g.sim.world.block_anywhere(v(0, y, -6)), Some(GRASS), "no block mined");
    assert_eq!(g.inventory().count(BLADERANG), 1, "it never left the slot");
}

#[test]
fn it_brings_back_the_loose_items_it_passes() {
    let (mut g, y) = on_turf();
    let ahead = Vec3::new(1.5, f64::from(y) + 1.2, -6.0);
    g.items.spawn(ahead, Vec3::ZERO, IRON_PLATE, 3, 0.0);
    g.run_ticks(1);
    throw(&mut g);
    g.run_ticks(flight_ticks() + 30);
    assert_eq!(g.inventory().count(IRON_PLATE), 3, "carried home and picked up");
}

#[test]
fn a_second_press_while_it_flies_throws_nothing_and_other_hands_ignore_it() {
    let (mut g, _) = on_turf();
    throw(&mut g);
    g.run_ticks(5);
    let t = g.blade_pos();
    g.set_using(true);
    assert!(g.update_blade(), "the press is the blade's");
    assert!(g.blade_pos() == t, "still the same throw");
    g.set_using(false);
    g.select_slot(8);
    g.run_ticks(1);
    assert!(g.blade_label().is_none() && !g.update_blade(), "something else in hand");
}

#[test]
fn only_a_player_with_a_bladerang_clears_in_the_core() {
    let mut g = Game::new(2024, 3);
    let pos = IVec3::new(10, 60, 10);
    g.act(Action::ClearCover { pos });
    g.run_ticks(2);
    assert!(!g.sim.world.is_cover_cleared(10, 10), "empty hands cut nothing");
    g.give(BLADERANG.0, 1);
    g.run_ticks(1);
    let world_bytes = |g: &Game| {
        let mut w = ByteWriter::default();
        g.sim.world.write_state(&mut w);
        w.bytes
    };
    let before = world_bytes(&g);
    g.act(Action::ClearCover { pos });
    g.run_ticks(2);
    assert!(g.sim.world.patch_cleared(pos), "a bladerang anywhere in the pack cuts (it may be in the air)");
    let after = world_bytes(&g);
    assert_ne!(after, before, "the cleared columns are part of the state");
    g.act(Action::ClearCover { pos });
    g.run_ticks(2);
    assert_eq!(world_bytes(&g), after, "cutting a cleared patch changes nothing");
}

#[test]
fn a_save_keeps_the_cleared_patches_and_differs_from_one_without() {
    let mut g = Game::new(2024, 3);
    g.give(BLADERANG.0, 1);
    g.run_ticks(2);
    let plain = g.save_bytes();
    for pos in [IVec3::new(10, 60, 10), IVec3::new(-40, 60, 31), IVec3::new(32, 60, 32)] {
        g.act(Action::ClearCover { pos });
    }
    g.run_ticks(3);
    let cut = g.save_bytes();
    assert_ne!(plain, cut, "a world with a clear saves differently");
    let loaded = Game::from_save(&cut, 3).expect("loads");
    assert_eq!(loaded.sim.state_hash(), g.sim.state_hash());
    for pos in [IVec3::new(10, 60, 10), IVec3::new(-40, 60, 31), IVec3::new(32, 60, 32)] {
        assert!(loaded.sim.world.patch_cleared(pos), "{pos:?}");
    }
    assert!(!loaded.sim.world.is_cover_cleared(13, 10));
    assert!(!Game::from_save(&plain, 3).expect("loads").sim.world.patch_cleared(IVec3::new(10, 60, 10)));
}
