//! Tests of the light hands give: which items shine, which hand, and what the frame reports.

use super::*;
use crate::block::{LAMP, STONE, TORCH};
use crate::inventory::Stack;
use crate::item::STONE_PICKAXE;
use crate::tests::run_until_ready;
use crate::Game;

#[test]
fn only_blocks_that_shine_light_a_hand() {
    assert_eq!((reach(TORCH.into()), reach(LAMP.into())), (REACH, REACH));
    assert_eq!((reach(STONE.into()), reach(STONE_PICKAXE), reach(ItemId::NONE)), (0.0, 0.0, 0.0));
}

#[test]
fn either_hand_lights_and_the_brighter_wins() {
    let mut inv = Inventory::default();
    assert_eq!(reach_of(&inv), 0.0);
    inv.offhand = Stack { item: TORCH.into(), count: 3 };
    assert_eq!(reach_of(&inv), REACH, "a torch in the left hand");
    inv.offhand = Stack::default();
    inv.slots[0] = Stack { item: TORCH.into(), count: 3 };
    assert_eq!(reach_of(&inv), REACH, "and in the right");
    inv.select(1);
    assert_eq!(reach_of(&inv), 0.0, "a torch in the backpack or on another slot lights nothing");
}

#[test]
fn the_frame_reports_one_light_for_the_player_at_the_eye_and_none_without_a_torch() {
    let mut g = Game::new(2024, 3);
    run_until_ready(&mut g);
    g.update(1.0 / 60.0);
    assert_eq!(g.held_light_count(), 0);
    g.give(u16::from(TORCH), 10);
    g.update(1.0 / 60.0);
    let slot = (0..9).find(|&i| g.slot_item(i) == u16::from(TORCH)).expect("in the hotbar");
    g.select_slot(slot);
    g.update(1.0 / 60.0);
    g.update(1.0 / 60.0);
    assert_eq!(g.held_light_count(), 1);
    let lights = &g.avatars.lights;
    assert!(lights[1] < 0.0 && lights[1] > -1.0, "just below the camera: {lights:?}");
    assert_eq!(lights[3], REACH);

    g.swap_offhand();
    g.select_slot((slot + 1) % 9);
    g.update(1.0 / 60.0);
    g.update(1.0 / 60.0);
    assert_eq!(g.held_light_count(), 1, "the torch moved to the left hand and still shines");
    g.click_offhand();
    g.update(1.0 / 60.0);
}
