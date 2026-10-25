//! Tests of the doors: the pairing, swinging a stack, the drops, and the view refusing to shut one on the body.

use crate::action::Action;
use crate::block::{self, AIR, IRON_DOOR, IRON_DOOR_OPEN, STONE, TORCH, WOOD_DOOR, WOOD_DOOR_OPEN, WOOD_FLOOR};
use crate::item::ItemId;
use crate::math::IVec3;
use crate::sim::{PlayerId, Sim, SimEvent};
use crate::tests::run_until_ready;
use crate::Game;

const P: PlayerId = PlayerId(0);
const AT: IVec3 = IVec3 { x: 3, y: 200, z: 3 };

fn up(n: i32) -> IVec3 {
    AT + IVec3::new(0, n, 0)
}

fn block_at(sim: &mut Sim, p: IVec3) -> block::BlockId {
    sim.world.block_anywhere_or_generate(p)
}

#[test]
fn the_pairs_are_one_solid_item_and_one_see_through_twin_that_breaks_into_it() {
    for (shut, open) in [(WOOD_DOOR, WOOD_DOOR_OPEN), (IRON_DOOR, IRON_DOOR_OPEN)] {
        assert_eq!((block::toggled(shut), block::toggled(open)), (Some(open), Some(shut)));
        assert!(block::def(shut).solid && block::def(shut).placeable && block::def(shut).drop == shut);
        assert!(!block::is_open_door(shut) && block::is_open_door(open));
        let twin = block::def(open);
        assert!(!twin.solid && !twin.placeable && twin.drop == shut, "{} breaks into the closed door", twin.name);
        assert_eq!(twin.break_time, block::def(shut).break_time);
    }
    assert_eq!(block::toggled(STONE), None);
}

#[test]
fn right_clicking_swings_the_whole_stack_and_back() {
    let mut sim = Sim::new(7, 2);
    for n in 0..2 {
        sim.world.set_block_anywhere(up(n), WOOD_DOOR);
    }
    sim.world.set_block_anywhere(up(2), WOOD_FLOOR);
    sim.events.clear();
    sim.apply(P, Action::ToggleDoor { pos: AT });
    assert_eq!((block_at(&mut sim, up(0)), block_at(&mut sim, up(1))), (WOOD_DOOR_OPEN, WOOD_DOOR_OPEN));
    assert_eq!(block_at(&mut sim, up(2)), WOOD_FLOOR, "only doors swing");
    let moved =
        sim.events.iter().filter(|e| matches!(e, SimEvent::DoorMoved { block, .. } if *block == WOOD_DOOR_OPEN));
    assert_eq!(moved.count(), 2);
    sim.apply(P, Action::ToggleDoor { pos: up(1) });
    assert_eq!((block_at(&mut sim, up(0)), block_at(&mut sim, up(1))), (WOOD_DOOR, WOOD_DOOR));
}

#[test]
fn a_door_of_the_other_state_or_kind_is_left_alone() {
    let mut sim = Sim::new(7, 2);
    sim.world.set_block_anywhere(up(0), IRON_DOOR);
    sim.world.set_block_anywhere(up(1), WOOD_DOOR_OPEN);
    sim.apply(P, Action::ToggleDoor { pos: up(0) });
    assert_eq!((block_at(&mut sim, up(0)), block_at(&mut sim, up(1))), (IRON_DOOR_OPEN, WOOD_DOOR_OPEN));
    sim.world.set_block_anywhere(up(5), STONE);
    sim.events.clear();
    sim.apply(P, Action::ToggleDoor { pos: up(5) });
    assert!(sim.events.is_empty() && block_at(&mut sim, up(5)) == STONE, "not a door: nothing happens");
}

#[test]
fn an_open_door_breaks_into_a_closed_one() {
    let mut sim = Sim::new(7, 2);
    sim.world.set_block_anywhere(AT, WOOD_DOOR_OPEN);
    sim.events.clear();
    sim.apply(P, Action::BreakBlock { pos: AT });
    assert_eq!(block_at(&mut sim, AT), AIR);
    let dropped = sim.events.iter().find_map(|e| match e {
        SimEvent::Dropped { item, count, .. } => Some((*item, *count)),
        _ => None,
    });
    assert_eq!(dropped, Some((ItemId::block(WOOD_DOOR), 1)));
}

#[test]
fn a_torch_on_a_closed_door_falls_when_it_opens() {
    let mut sim = Sim::new(7, 2);
    sim.world.set_block_anywhere(AT, WOOD_DOOR);
    sim.world.set_block_anywhere(up(1), TORCH);
    sim.apply(P, Action::ToggleDoor { pos: AT });
    assert_eq!(block_at(&mut sim, up(1)), AIR, "nothing to stand on any more");
}

#[test]
fn the_view_will_not_shut_a_door_on_the_player_but_swings_it_otherwise() {
    let mut g = Game::new(2024, 3);
    run_until_ready(&mut g);
    g.toggle_fly();
    g.teleport(3.5, 200.0, 3.5);
    g.sim.world.set_block_anywhere(AT, WOOD_DOOR_OPEN);
    g.sim.world.set_block_anywhere(up(1), WOOD_DOOR_OPEN);
    g.update(1.0 / 60.0);
    g.set_using(true);
    g.update(1.0 / 60.0);
    g.run_ticks(2);
    assert_eq!(g.sim.world.block_anywhere_or_generate(AT), WOOD_DOOR_OPEN, "the player stands in it");

    g.teleport(8.5, 200.0, 3.5);
    g.set_look(-std::f64::consts::FRAC_PI_2, 0.0); // towards -X, at the door
    g.update(1.0 / 60.0);
    g.set_using(false);
    g.update(1.0 / 60.0);
    g.set_using(true);
    g.update(1.0 / 60.0);
    g.run_ticks(2);
    let now = (g.sim.world.block_anywhere_or_generate(AT), g.sim.world.block_anywhere_or_generate(up(1)));
    assert_eq!(now, (WOOD_DOOR, WOOD_DOOR), "from outside it closes, both halves");
}
