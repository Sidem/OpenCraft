use crate::action::Action;
use crate::block::{GRASS, STONE};
use crate::bytes::ByteWriter;
use crate::item::SICKLE;
use crate::math::IVec3;
use crate::world::PATCH_RADIUS;
use crate::Game;

const DT: f64 = 1.0 / 60.0;

fn hold_sickle() -> Game {
    let mut g = Game::new(2024, 6);
    crate::tests::run_until_ready(&mut g);
    g.give(SICKLE.0, 1);
    g.update(DT);
    let slot = (0..9).find(|&i| g.slot_item(i) == SICKLE.0).expect("in the hotbar");
    g.select_slot(slot);
    g.update(DT);
    assert!(g.sickle_held());
    g
}

/// Looks at the ground just ahead of the feet and makes the block there turf, so no biome can get in the way.
fn aim_at_turf(g: &mut Game) -> IVec3 {
    g.set_look(0.0, -1.2);
    g.update(DT);
    let hit = g.target.expect("ground in the crosshair");
    assert!(g.sim.world.set_block_anywhere(hit.block, GRASS) || g.sim.world.block_anywhere(hit.block) == Some(GRASS));
    g.update(DT);
    g.target.expect("still aimed").block
}

fn click_use(g: &mut Game) {
    g.set_using(true);
    g.run_ticks(2);
    g.set_using(false);
    g.run_ticks(2);
}

fn cleared_count(g: &Game, centre: IVec3, reach: i32) -> usize {
    let r = reach;
    (centre.z - r..=centre.z + r)
        .flat_map(|z| (centre.x - r..=centre.x + r).map(move |x| (x, z)))
        .filter(|&(x, z)| g.sim.world.is_cover_cleared(x, z))
        .count()
}

#[test]
fn right_click_cuts_the_patch_and_breaks_and_places_nothing() {
    let mut g = hold_sickle();
    let ground = aim_at_turf(&mut g);
    assert!(g.sickle_label().unwrap().contains("Right-click cuts"));
    assert_eq!(g.sickle_box().len(), 7, "the patch is outlined before the cut");
    let blocks = (g.sim.world.block_anywhere(ground), g.sim.world.block_anywhere(ground + IVec3::new(0, 1, 0)));
    let hand = g.inventory().count(SICKLE);

    click_use(&mut g);
    assert_eq!(cleared_count(&g, ground, PATCH_RADIUS), 9, "all nine columns are cleared");
    assert_eq!(cleared_count(&g, ground, PATCH_RADIUS + 3), 9, "and no other");
    assert_eq!(
        (g.sim.world.block_anywhere(ground), g.sim.world.block_anywhere(ground + IVec3::new(0, 1, 0))),
        blocks,
        "no block mined or placed"
    );
    assert_eq!(g.inventory().count(SICKLE), hand, "the sickle never wears");
    g.update(DT);
    assert!(g.sickle_label().unwrap().contains("Nothing left"));
    assert!(g.sickle_box().is_empty(), "a cleared patch has no outline");
}

#[test]
fn holding_the_button_sweeps_on_and_a_cleared_patch_sends_nothing() {
    let mut g = hold_sickle();
    let ground = aim_at_turf(&mut g);
    g.set_using(true);
    g.run_ticks(2);
    assert!(g.update_sickle(0.0), "a swing is the click");
    let cooldown = g.use_cooldown;
    g.update(DT);
    assert!(g.update_sickle(0.0) && g.use_cooldown == cooldown, "an already cleared patch swings no more");
    g.set_using(false);
    // Turn to fresh ground: the held button swings again, after the repeat time.
    g.set_look(1.2, -1.2);
    g.update(DT);
    let fresh = g.target.expect("ground").block;
    g.set_using(true);
    g.run_ticks(40);
    assert!(g.sim.world.is_cover_cleared(fresh.x, fresh.z) || g.target.is_none());
    assert!(g.sim.world.is_cover_cleared(ground.x, ground.z));
}

#[test]
fn the_click_falls_through_when_the_aim_bears_no_cover() {
    let mut g = hold_sickle();
    let ground = aim_at_turf(&mut g);
    g.sim.world.set_block_anywhere(ground, STONE);
    g.update(DT);
    g.set_using(true);
    assert!(!g.update_sickle(0.0), "stone grows nothing: the click is left to the other hands");
    g.run_ticks(3);
    assert_eq!(cleared_count(&g, ground, 2), 0);
    assert!(g.sickle_box().is_empty() && g.sickle_label().unwrap().contains("Aim at the ground"));
    g.set_using(false);
    g.select_slot(8);
    g.update(DT);
    assert!(g.sickle_label().is_none() && !g.update_sickle(0.0), "something else in hand");
}

#[test]
fn only_a_held_sickle_clears_in_the_core() {
    let mut g = Game::new(2024, 3);
    let pos = IVec3::new(10, 60, 10);
    g.act(Action::ClearCover { pos });
    g.run_ticks(2);
    assert!(!g.sim.world.is_cover_cleared(10, 10), "empty hands cut nothing");
    g.give(SICKLE.0, 1);
    g.run_ticks(1);
    let slot = (0..9).find(|&i| g.slot_item(i) == SICKLE.0).expect("in the hotbar");
    g.select_slot(slot);
    g.run_ticks(1);
    let world_bytes = |g: &Game| {
        let mut w = ByteWriter::default();
        g.sim.world.write_state(&mut w);
        w.bytes
    };
    let before = world_bytes(&g);
    g.act(Action::ClearCover { pos });
    g.run_ticks(2);
    assert!(g.sim.world.patch_cleared(pos));
    let after = world_bytes(&g);
    assert_ne!(after, before, "the cleared columns are part of the state");
    g.act(Action::ClearCover { pos });
    g.run_ticks(2);
    assert_eq!(world_bytes(&g), after, "cutting a cleared patch changes nothing");
}

#[test]
fn a_save_keeps_the_cleared_patches_and_differs_from_one_without() {
    let mut g = Game::new(2024, 3);
    g.give(SICKLE.0, 1);
    g.run_ticks(1);
    let slot = (0..9).find(|&i| g.slot_item(i) == SICKLE.0).expect("in the hotbar");
    g.select_slot(slot);
    g.run_ticks(1);
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
