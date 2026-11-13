use crate::block::STORAGE;
use crate::helpers::drone::ANY_SLOT;
use crate::inventory::Stack;
use crate::item::{self, IRON_PLATE, PERSONAL_DRONE};
use crate::math::IVec3;
use crate::{Game, TICK_RATE};

/// A ready game whose player carries a drone, with a box of plates 10 blocks east of them; the box.
fn game() -> (Game, IVec3) {
    let mut g = Game::new(2024, 3);
    crate::tests::run_until_ready(&mut g);
    let me = g.local;
    g.sim.apply(me, crate::action::Action::Give { item: PERSONAL_DRONE, count: 1 });
    let spot = g.body().pos.floor() + IVec3::new(10, 2, 0);
    g.sim.apply(me, crate::action::Action::Give { item: STORAGE.into(), count: 1 });
    let slot = g.inventory().slots.iter().position(|s| s.item == STORAGE.into()).unwrap() as u8;
    g.sim.apply(me, crate::action::Action::PlaceBlock { pos: spot, slot, facing: 0, against: spot });
    g.sim.factory.stock(spot, IRON_PLATE, 500);
    // The drone sits in slot 0: hold the plates in another.
    g.sim.players[me.0 as usize].as_mut().unwrap().inventory.selected = 4;
    (g, spot)
}

/// Sets the hand stack to `count` plates and runs a tick, so the hands see it.
fn hold(g: &mut Game, count: u32) {
    let me = g.local.0 as usize;
    let inv = &mut g.sim.players[me].as_mut().unwrap().inventory;
    let sel = inv.selected;
    inv.slots[sel] = if count == 0 { Stack::default() } else { Stack { item: IRON_PLATE, count } };
    g.run_ticks(1);
}

fn plates(g: &Game) -> u32 {
    g.inventory().count(IRON_PLATE)
}

#[test]
fn a_hand_stack_used_down_to_a_quarter_is_topped_up_to_a_stack() {
    let (mut g, spot) = game();
    let stack = item::stack_size(IRON_PLATE);
    hold(&mut g, stack / 4 + 3);
    hold(&mut g, stack / 4 + 1);
    assert!(g.sim.player(g.local).unwrap().helpers.drone.idle(), "not low yet");
    hold(&mut g, stack / 4 - 1);
    g.run_ticks(15 * TICK_RATE);
    assert_eq!(plates(&g), stack);
    assert_eq!(g.inventory().slots[4], Stack { item: IRON_PLATE, count: stack }, "in the hand's own slot");
    assert_eq!(g.sim.factory.box_count(spot, IRON_PLATE), 500 - stack + stack / 4 - 1);
}

#[test]
fn a_slot_shows_its_fill_on_the_way_within_a_tick_then_holds_it() {
    let (mut g, _) = game();
    g.drone_fill_slot(20, IRON_PLATE.0);
    g.run_ticks(1);
    let rows = g.drone_slots();
    assert_eq!(rows[..4], [20, 0, u32::from(IRON_PLATE.0), 1], "slot 20, filling, plates, on its way");
    let (progress, secs) = (rows[4], rows[5]);
    assert!(progress < 100 && secs >= 1, "just left: {progress}‰, {secs} s");
    g.run_ticks(15 * TICK_RATE);
    assert!(g.drone_slots().is_empty());
    assert_eq!(g.inventory().slots[20], Stack { item: IRON_PLATE, count: item::stack_size(IRON_PLATE) });
}

#[test]
fn a_whole_stack_moved_away_or_restock_off_brings_nothing() {
    let (mut g, _) = game();
    let stack = item::stack_size(IRON_PLATE);
    hold(&mut g, stack);
    hold(&mut g, 0);
    g.drone_hands.restock = false;
    hold(&mut g, 10);
    hold(&mut g, 2);
    g.run_ticks(2);
    assert!(g.sim.player(g.local).unwrap().helpers.drone.idle());
}

#[test]
fn the_drones_home_follows_the_player_while_it_has_work() {
    let (mut g, _) = game();
    g.order(crate::helpers::drone::Job::Fetch { item: IRON_PLATE, count: 10, slot: ANY_SLOT });
    g.run_ticks(1);
    let start = g.sim.player(g.local).unwrap().helpers.drone.home;
    g.teleport(start.x as f64 + 20.5, start.y as f64 + 1.0, start.z as f64 + 0.5);
    g.run_ticks(2);
    let home = g.sim.player(g.local).unwrap().helpers.drone.home;
    assert_eq!((home.x, home.z), (start.x + 20, start.z), "moved along");
}
