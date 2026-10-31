use super::*;
use crate::block::{BELT, STONE};
use crate::factory::{belt_facing, BEND_LEFT, BEND_RIGHT, BEND_STRAIGHT};
use crate::inventory::Stack;
use crate::math::IVec3;
use crate::raycast::RayHit;
use crate::tests::{belt_test_slab, run_until_ready};

const EAST: u8 = 1;

/// A game over the test slab with ten belts in the selected slot.
fn holding_belts() -> Game {
    let mut g = Game::new(2024, 3);
    run_until_ready(&mut g);
    belt_test_slab(&mut g);
    let inv = &mut g.sim.players[0].as_mut().unwrap().inventory;
    inv.slots[0] = Stack { item: BELT.into(), count: 10 };
    inv.selected = 0;
    g
}

#[test]
fn shift_r_steps_the_bend_of_the_belt_you_look_at() {
    let mut g = holding_belts();
    let pos = IVec3::new(0, 201, 0);
    g.act(Action::PlaceBlock { pos, slot: 0, facing: EAST, against: pos - IVec3::new(0, 1, 0) });
    g.run_ticks(1);
    for want in [BEND_LEFT, BEND_RIGHT, BEND_STRAIGHT] {
        // Each tick aims anew from the view, so aim at the belt before each press.
        g.target = Some(RayHit { block: pos, normal: IVec3::ZERO, id: BELT });
        assert!(g.bend_target_belt());
        g.run_ticks(3);
        assert_eq!(g.sim.factory.bend_at(pos), Some(want));
    }
    assert_eq!(g.next_bend(), BEND_STRAIGHT, "aimed at a belt, the held bend stays");
    g.target = Some(RayHit { block: pos - IVec3::new(0, 1, 0), normal: IVec3::new(0, 1, 0), id: STONE });
    assert!(g.bend_target_belt(), "aimed at the ground with belts in hand");
    assert_eq!(g.next_bend(), BEND_LEFT);
    assert!(g.bend_label().is_some_and(|l| l.contains("from its left")), "the HUD says so");
    g.sim.players[0].as_mut().unwrap().inventory.selected = 5;
    assert!(!g.bend_target_belt(), "nothing to bend with an empty hand");
}

#[test]
fn the_held_bend_goes_on_the_first_belt_of_the_next_line() {
    let mut g = holding_belts();
    g.held_bend = BEND_RIGHT;
    let (start, end) = (IVec3::new(-2, 201, 0), IVec3::new(2, 201, 0));
    g.line.cells = crate::belt_line::plan(|p| g.sim.world.get_block(p), start, end, EAST).0;
    g.bend_line_start();
    assert_eq!(g.line.cells[0].dir, belt_facing(EAST, BEND_RIGHT));
    assert!(g.line.cells[1..].iter().all(|c| c.dir == EAST), "only the first");
    g.instances.clear();
    g.write_line_preview(crate::math::Vec3::ZERO, 0.0);
    assert!(!g.instances.is_empty(), "the preview draws the bent belt");
    let planned = g.line.cells.clone();
    g.queue_build(&planned, false);
    for _ in 0..30 {
        g.update(1.0 / 60.0);
    }
    assert_eq!(g.sim.factory.bend_at(start), Some(BEND_RIGHT));
    assert_eq!(g.sim.factory.bend_at(start + IVec3::new(1, 0, 0)), Some(BEND_STRAIGHT));
}
