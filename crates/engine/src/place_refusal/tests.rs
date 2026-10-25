//! Tests of the refusals: the body in the way, a taken footprint, one sound per click, the red box and the words.

use crate::block::{self, AIR, LONG_POLE, STONE};
use crate::math::IVec3;
use crate::sound;
use crate::tests::run_until_ready;
use crate::upgrade_aim::BLOCKED_RED;
use crate::Game;

const FRAME: f64 = 1.0 / 60.0;

/// A flat stone floor with `item` selected and the player looking down at the floor from `(x, z)` cell centre.
fn on_a_floor(item: u16, x: f64, z: f64) -> (Game, i32) {
    let mut g = Game::new(2024, 3);
    run_until_ready(&mut g);
    let y = g.sim.world.generator().height_at(0, 0);
    for x in 0..=14 {
        for z in 0..=8 {
            for h in y - 1..=y + 6 {
                g.sim.world.set_block_anywhere(IVec3::new(x, h, z), if h <= y { STONE } else { AIR });
            }
        }
    }
    g.give(item, 5);
    g.update(FRAME);
    let slot = (0..9).find(|&i| g.slot_item(i) == item).expect("in the hotbar");
    g.select_slot(slot);
    g.teleport(x, (y + 1) as f64, z);
    g.update(FRAME);
    g.update(FRAME);
    (g, y)
}

fn denials(g: &Game) -> usize {
    g.sounds.kinds().iter().filter(|&&k| k == sound::DENIED).count()
}

#[test]
fn a_block_where_the_player_stands_is_refused_with_words_a_red_box_and_one_knock_a_click() {
    let (mut g, y) = on_a_floor(u16::from(STONE), 3.5, 3.5);
    g.set_look(0.0, -1.5); // straight down: the floor under the feet, so the new block would fill the player's cell
    g.update(FRAME);
    assert!(g.line_label().contains("step aside"), "{}", g.line_label());
    let boxes = g.placement_box();
    assert_eq!(boxes.len(), 7);
    assert_eq!((boxes[1], boxes[6]), (y + 1, BLOCKED_RED));

    g.sounds.clear();
    g.set_using(true);
    for _ in 0..10 {
        g.update(FRAME);
    }
    assert_eq!(denials(&g), 1, "once, not every frame");
    for _ in 0..70 {
        g.update(FRAME);
    }
    assert!(denials(&g) >= 2, "a held button knocks again");
    assert_eq!(g.sim.world.block_anywhere_or_generate(IVec3::new(3, y + 1, 3)), AIR);
}

#[test]
fn stepping_aside_places_the_block_without_a_knock() {
    let (mut g, y) = on_a_floor(u16::from(STONE), 3.5, 3.5);
    g.set_look(0.0, -1.5);
    g.teleport(5.5, (y + 1) as f64, 3.5);
    g.set_look(std::f64::consts::PI * 1.5, -0.9); // towards -x and down: the floor a block or two ahead
    g.update(FRAME);
    assert!(!g.line_label().contains("step aside"));
    assert!(!g.placement_box().contains(&BLOCKED_RED));
    g.sounds.clear();
    g.set_using(true);
    for _ in 0..5 {
        g.update(FRAME);
    }
    g.run_ticks(3);
    assert_eq!(denials(&g), 0);
    let placed = (2..=5).filter(|&x| g.sim.world.block_anywhere_or_generate(IVec3::new(x, y + 1, 3)) == STONE).count();
    assert_eq!(placed, 1, "the block went down in front of the player");
}

#[test]
fn a_machine_that_does_not_fit_knocks_and_one_that_does_is_placed_in_silence() {
    let (mut g, y) = on_a_floor(u16::from(LONG_POLE), 3.5, 3.5);
    g.set_look(0.0, -1.5); // its footprint would take the player's own cells
    g.update(FRAME);
    g.sounds.clear();
    g.set_using(true);
    for _ in 0..5 {
        g.update(FRAME);
    }
    assert_eq!(denials(&g), 1);

    g.set_using(false);
    g.update(FRAME);
    g.teleport(12.5, (y + 1) as f64, 3.5);
    g.set_look(std::f64::consts::PI * 1.5, -0.6); // towards -x, down at the open floor
    g.update(FRAME);
    let anchor = g.footprint_ghost().expect("aimed at the floor").1;
    g.sounds.clear();
    g.set_using(true);
    for _ in 0..5 {
        g.update(FRAME);
    }
    g.run_ticks(3);
    assert_eq!(denials(&g), 0);
    assert_eq!(g.sim.world.block_anywhere_or_generate(anchor), block::LONG_POLE, "it stands where the ghost was");
}
