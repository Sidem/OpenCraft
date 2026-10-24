use super::*;
use crate::block::IRON_ORE;
use crate::math::IVec3;
use crate::tests::run_until_ready;

fn v(x: i32, y: i32, z: i32) -> IVec3 {
    IVec3::new(x, y, z)
}

/// A game with one belt carrying one ore, the crosshair on it.
fn belt_with_ore() -> (Game, IVec3) {
    let mut g = Game::new(2024, 3);
    run_until_ready(&mut g);
    let cell = v(0, 200, 0);
    g.sim.factory.add_belt(cell, 1);
    g.sim.factory.put_on_belt(cell, IRON_ORE.into(), v(-4, 200, 0), 0.5);
    g.render_eye = cell.as_vec3() + Vec3::new(0.5, 2.0, 0.5);
    let down = Vec3::new(0.0, -1.0, 0.0);
    g.target_item = g.sim.factory.belt_item_under(cell, g.render_eye, down);
    assert!(g.target_item.is_some());
    (g, cell)
}

fn ore_held(g: &Game) -> u32 {
    g.sim.players[0].as_ref().unwrap().inventory.count(IRON_ORE.into())
}

#[test]
fn a_click_selects_the_item_and_its_readout_stays_when_the_crosshair_leaves() {
    let (mut g, _) = belt_with_ore();
    let hash = g.sim.state_hash();
    assert_eq!(g.item_state(), ItemState::Pointed, "pointed at, not selected yet");
    g.click_item();
    assert_eq!(g.item_state(), ItemState::Selected);

    g.target_item = None;
    g.track_selected_item();
    assert_eq!(g.item_state(), ItemState::Selected, "still up with nothing pointed at");
    assert!(g.target_item_name().contains("Iron Ore"), "{}", g.target_item_name());
    assert!(g.target_detail().contains("4 west"), "{}", g.target_detail());
    assert_eq!(g.selected_box().len(), 6);
    assert_eq!(g.sim.state_hash(), hash, "selecting changes nothing in the core");

    // A click on anything else lets go.
    g.click_item();
    assert_eq!(g.item_state(), ItemState::None);
    assert!(g.selected_box().is_empty());
}

#[test]
fn a_selected_item_far_away_is_shown_but_not_taken_until_in_reach() {
    let (mut g, cell) = belt_with_ore();
    g.click_item();
    g.target_item = None;
    g.render_eye = cell.as_vec3() + Vec3::new(0.5, 40.0, 0.5);
    assert_eq!(g.item_state(), ItemState::OutOfReach);
    g.take_target_item();
    g.run_ticks(2);
    assert_eq!(ore_held(&g), 0, "too far to pick up");

    g.render_eye = cell.as_vec3() + Vec3::new(0.5, 2.5, 0.5);
    assert_eq!(g.item_state(), ItemState::Selected);
    g.take_target_item();
    g.run_ticks(2);
    assert_eq!(ore_held(&g), 1, "in reach again: picked up");
    g.track_selected_item();
    assert_eq!(g.item_state(), ItemState::None, "a taken item ends the selection");
}

#[test]
fn pointing_at_another_item_shows_that_one_and_clicking_it_moves_the_selection() {
    let (mut g, cell) = belt_with_ore();
    g.sim.factory.put_on_belt(cell, IRON_ORE.into(), v(-9, 200, 0), 0.0);
    let down = Vec3::new(0.0, -1.0, 0.0);
    let first = g.target_item.unwrap();
    g.click_item();
    let back = g.sim.factory.belt_item_under(cell, cell.as_vec3() + Vec3::new(0.15, 2.0, 0.5), down).unwrap();
    assert_ne!(back.tag, first.tag);
    g.target_item = Some(back);
    assert_eq!(g.item_state(), ItemState::Pointed);
    assert!(g.target_detail().contains("9 west"), "the pointed item's origin, not the selected one's");
    g.click_item();
    g.target_item = None;
    assert_eq!(g.shown_item().map(|a| a.tag), Some(back.tag), "the click moved the selection");
}
