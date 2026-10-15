use super::*;
use crate::action::Action;
use crate::block::{AIR, LONG_POLE, POLE, STORAGE};
use crate::item::{HOVER_PACK, IRON_PLATE, JETPACK};
use crate::player::Player;

const P: PlayerId = PlayerId(0);

fn pack(sim: &Sim, item: ItemId) -> u32 {
    sim.player(P).unwrap().inventory.count(item)
}

fn helpers(sim: &Sim) -> &Helpers {
    &sim.player(P).unwrap().helpers
}

fn run(sim: &mut Sim, ticks: u32) {
    for _ in 0..ticks {
        sim.step();
    }
}

#[test]
fn the_jetpack_burns_a_coal_per_ten_seconds_and_stops_when_the_pack_is_dry() {
    let mut sim = Sim::new(7, 2);
    sim.apply(P, Action::Jetpack { on: true });
    assert!(!helpers(&sim).thrusting, "no jetpack");
    sim.apply(P, Action::Give { item: JETPACK, count: 1 });
    sim.apply(P, Action::Give { item: coal(), count: 2 });
    sim.apply(P, Action::Jetpack { on: true });
    assert!(!helpers(&sim).thrusting, "a jetpack in the pack does nothing");
    sim.apply(P, Action::ClickSlot { slot: 0, shift: true });
    sim.players[0].as_mut().unwrap().inventory.remove(coal(), 2);
    sim.apply(P, Action::Jetpack { on: true });
    assert!(!helpers(&sim).thrusting, "no coal");
    sim.apply(P, Action::Give { item: coal(), count: 2 });
    sim.apply(P, Action::Jetpack { on: true });
    assert!(helpers(&sim).thrusting);
    assert_eq!((pack(&sim, coal()), helpers(&sim).jet), (1, JET_TICKS_PER_COAL));
    run(&mut sim, JET_TICKS_PER_COAL - 1);
    assert!(helpers(&sim).thrusting && pack(&sim, coal()) == 1, "the first coal has not run out");
    run(&mut sim, 1);
    assert!(helpers(&sim).thrusting && pack(&sim, coal()) == 0, "the second was taken without a gap");
    run(&mut sim, JET_TICKS_PER_COAL);
    assert!(!helpers(&sim).thrusting, "dry");
    assert_eq!(helpers(&sim).jet, 0);
}

#[test]
fn leftover_fuel_waits_in_the_tank_and_losing_the_jetpack_stops_the_thrust() {
    let mut sim = Sim::new(7, 2);
    sim.apply(P, Action::Give { item: JETPACK, count: 1 });
    sim.apply(P, Action::ClickSlot { slot: 0, shift: true });
    sim.apply(P, Action::Give { item: coal(), count: 1 });
    sim.apply(P, Action::Jetpack { on: true });
    run(&mut sim, 120);
    sim.apply(P, Action::Jetpack { on: false });
    assert!(!helpers(&sim).thrusting);
    assert_eq!(helpers(&sim).jet, JET_TICKS_PER_COAL - 120);
    sim.apply(P, Action::Jetpack { on: true });
    assert_eq!(pack(&sim, coal()), 0, "the tank is used before another coal");
    let inv = &mut sim.players[0].as_mut().unwrap().inventory;
    inv.worn[4] = ItemId::NONE;
    run(&mut sim, 1);
    assert!(!helpers(&sim).thrusting, "taken off, no thrust");
}

#[test]
fn a_world_loaded_mid_thrust_is_switched_off_by_the_hands() {
    let mut g = crate::Game::new(2024, 3);
    crate::tests::run_until_ready(&mut g);
    let me = g.local.0 as usize;
    let core = g.sim.players[me].as_mut().unwrap();
    core.inventory.worn[4] = JETPACK;
    core.helpers.thrusting = true;
    core.helpers.jet = 10_000;
    g.hands_synced = false; // as in a game just loaded
    for _ in 0..30 {
        g.update(1.0 / 60.0);
    }
    assert!(!g.sim.player(g.local).unwrap().helpers.thrusting, "nobody holds jump, so the thrust stops");
}

#[test]
fn a_thrusting_body_climbs_to_a_steady_rate_and_falls_without() {
    let step = |thrust: bool| {
        let mut p = Player::new(crate::math::Vec3::new(0.5, 100.0, 0.5));
        p.thrust = thrust;
        let (mut solid, mut block) = (|_, _, _| false, |_, _, _| AIR);
        for _ in 0..120 {
            p.step(1.0 / 120.0, &mut solid, &mut block);
        }
        p.vel.y
    };
    let up = step(true);
    assert!(up > 4.0 && up <= 6.0 + 1e-9, "settles at the climb rate: {up}");
    assert!(step(false) < -10.0, "falls");
}

fn errand_sim(box_has: u32) -> (Sim, IVec3) {
    let mut sim = Sim::new(7, 2);
    let at = IVec3::new(900, 200, -900);
    let spot = at + IVec3::new(6, 0, 0);
    sim.apply(P, Action::Give { item: STORAGE.into(), count: 1 });
    sim.apply(P, Action::PlaceBlock { pos: spot, slot: 0, facing: 0, against: spot });
    sim.factory.stock(spot, IRON_PLATE, box_has);
    (sim, at)
}

#[test]
fn the_personal_drone_brings_a_stack_from_the_nearest_box() {
    let (mut sim, at) = errand_sim(100);
    sim.apply(P, Action::Fetch { item: IRON_PLATE, at });
    assert!(helpers(&sim).fetch.is_none(), "no drone in the pack");
    sim.apply(P, Action::Give { item: PERSONAL_DRONE, count: 1 });
    sim.apply(P, Action::Fetch { item: IRON_PLATE, at });
    let errand = helpers(&sim).fetch.expect("on its way");
    assert_eq!(errand.total, TICK_RATE, "a short hop takes the minimum");
    sim.apply(P, Action::Fetch { item: IRON_PLATE, at });
    assert_eq!(helpers(&sim).fetch, Some(errand), "one errand at a time");
    run(&mut sim, TICK_RATE);
    assert!(helpers(&sim).fetch.is_none());
    assert_eq!(pack(&sim, IRON_PLATE), 64, "one stack");
    assert_eq!(sim.factory.box_count(errand.from, IRON_PLATE), 36);
}

#[test]
fn it_fetches_nothing_from_out_of_reach_or_empty_boxes_and_returns_what_does_not_fit() {
    let (mut sim, at) = errand_sim(5);
    sim.apply(P, Action::Give { item: PERSONAL_DRONE, count: 1 });
    sim.apply(P, Action::Fetch { item: IRON_PLATE, at: at + IVec3::new(-100, 0, 0) });
    assert!(helpers(&sim).fetch.is_none(), "too far");
    sim.apply(P, Action::Fetch { item: item::STEEL_PLATE, at });
    assert!(helpers(&sim).fetch.is_none(), "the box holds none");
    // A full pack: the plates stay in the box.
    let inv = &mut sim.players[0].as_mut().unwrap().inventory;
    for s in inv.slots.iter_mut().filter(|s| s.is_empty()) {
        *s = Stack { item: item::STEEL_PLATE, count: 64 };
    }
    sim.apply(P, Action::Fetch { item: IRON_PLATE, at });
    run(&mut sim, 2 * TICK_RATE);
    assert_eq!(pack(&sim, IRON_PLATE), 0);
    let from = at + IVec3::new(6, 0, 0);
    assert_eq!(sim.factory.box_count(from, IRON_PLATE), 5, "all went back");
}

#[test]
fn helpers_round_trip_through_bytes() {
    let h = Helpers {
        thrusting: true,
        jet: 77,
        fetch: Some(Fetch { item: IRON_PLATE, from: IVec3::new(-4, 70, 9), left: 12, total: 90 }),
        hover: true,
        charge: 1234,
        pole: Some(IVec3::new(5, 64, -3)),
    };
    let mut w = ByteWriter::default();
    h.write_state(&mut w);
    assert_eq!(Helpers::read_state(&mut ByteReader::new(&w.bytes)), Some(h));
}

#[test]
fn helpers_saved_before_the_hover_pack_read_with_no_charge() {
    let old = Helpers { thrusting: true, jet: 5, ..Helpers::default() };
    let mut w = ByteWriter::default();
    w.bool(true);
    w.u32(5);
    w.bool(false);
    let mut r = ByteReader::new(&w.bytes);
    r.version = 35;
    assert_eq!(Helpers::read_state(&mut r), Some(old));
}

/// Gives the player a hover pack and puts it on (it does nothing in the backpack).
fn wear_hover_pack(sim: &mut Sim) {
    sim.apply(P, Action::Give { item: HOVER_PACK, count: 1 });
    let inv = &mut sim.players[0].as_mut().unwrap().inventory;
    let slot = inv.slots.iter().position(|s| s.item == HOVER_PACK).unwrap();
    assert!(inv.wear_from(slot));
}

fn hover_sim() -> Sim {
    let mut sim = Sim::new(7, 2);
    wear_hover_pack(&mut sim);
    sim
}

fn charged(sim: &mut Sim, ticks: u32) {
    sim.players[0].as_mut().unwrap().helpers.charge = ticks;
}

#[test]
fn the_hover_pack_hovers_on_charge_and_stops_when_it_runs_out() {
    let mut sim = Sim::new(7, 2);
    sim.apply(P, Action::Hover { on: true });
    assert!(!helpers(&sim).hover, "no pack");
    sim.apply(P, Action::Give { item: HOVER_PACK, count: 1 });
    sim.apply(P, Action::Hover { on: true });
    assert!(!helpers(&sim).hover, "in the backpack it does nothing");
    sim.players[0].as_mut().unwrap().inventory.remove(HOVER_PACK, 1);
    wear_hover_pack(&mut sim);
    sim.apply(P, Action::Hover { on: true });
    assert!(!helpers(&sim).hover, "no charge");
    charged(&mut sim, 100);
    sim.apply(P, Action::Hover { on: true });
    assert!(helpers(&sim).hover);
    run(&mut sim, 99);
    assert!(helpers(&sim).hover && helpers(&sim).charge == 1);
    run(&mut sim, 1);
    assert!(!helpers(&sim).hover && helpers(&sim).charge == 0, "empty");
}

#[test]
fn losing_the_hover_pack_stops_the_hover() {
    let mut sim = hover_sim();
    charged(&mut sim, 500);
    sim.apply(P, Action::Hover { on: true });
    sim.players[0].as_mut().unwrap().inventory.click_gear(4, true);
    run(&mut sim, 1);
    assert!(!helpers(&sim).hover);
}

#[test]
fn a_pole_charges_the_pack_to_its_cap_and_losing_the_pole_stops_it() {
    let mut sim = hover_sim();
    let pole = IVec3::new(900, 200, -900);
    sim.apply(P, Action::Charge { pole, on: true });
    assert_eq!(helpers(&sim).pole, None, "no pole stands there");
    sim.apply(P, Action::Give { item: ItemId::block(POLE), count: 1 });
    let slot = sim.player(P).unwrap().inventory.slots.iter().position(|s| s.item == ItemId::block(POLE)).unwrap();
    sim.apply(P, Action::PlaceBlock { pos: pole, slot: slot as u8, facing: 0, against: pole });
    sim.apply(P, Action::Charge { pole, on: true });
    assert_eq!(helpers(&sim).pole, Some(pole));
    run(&mut sim, 10);
    assert_eq!(helpers(&sim).charge, 10 * CHARGE_RATE);
    run(&mut sim, HOVER_CAP);
    assert_eq!(helpers(&sim).charge, HOVER_CAP, "full");
    sim.apply(P, Action::BreakBlock { pos: pole });
    run(&mut sim, 1);
    assert_eq!(helpers(&sim).pole, None, "the pole is gone");
    sim.apply(P, Action::Charge { pole, on: false });
    assert_eq!(helpers(&sim).pole, None);
}

/// Puts `block` down at `pos` for player `who` (a bare placement: it gives the block first).
fn place(sim: &mut Sim, who: PlayerId, block: u8, pos: IVec3) {
    sim.apply(who, Action::Give { item: ItemId::block(block), count: 1 });
    let inv = &sim.player(who).unwrap().inventory;
    let slot = inv.slots.iter().position(|s| s.item == ItemId::block(block)).unwrap();
    sim.apply(who, Action::PlaceBlock { pos, slot: slot as u8, facing: 0, against: pos });
}

#[test]
fn a_long_distance_pole_can_charge_the_pack_too() {
    let mut sim = hover_sim();
    let pole = IVec3::new(900, 200, -900);
    place(&mut sim, P, LONG_POLE, pole);
    sim.apply(P, Action::Charge { pole, on: true });
    assert_eq!(helpers(&sim).pole, Some(pole));
    run(&mut sim, 10);
    assert_eq!(helpers(&sim).charge, 10 * CHARGE_RATE);
}

/// A game with the local player wearing a hover pack, and a pole of `block` put `up` blocks above them.
fn pole_game(block: u8, up: i32) -> (crate::Game, IVec3) {
    let mut g = crate::Game::new(2024, 3);
    crate::tests::run_until_ready(&mut g);
    let me = g.local.0 as usize;
    g.sim.players[me].as_mut().unwrap().inventory.worn[4] = HOVER_PACK;
    let feet = g.body().pos.floor();
    let pole = IVec3::new(feet.x, feet.y + up, feet.z);
    place(&mut g.sim, g.local, block, pole);
    (g, pole)
}

fn tied(g: &crate::Game) -> Option<IVec3> {
    g.sim.player(g.local).unwrap().helpers.pole
}

#[test]
fn the_hands_tie_the_pack_to_a_long_pole_from_far_away_but_not_to_an_ordinary_one() {
    let (mut g, _) = pole_game(POLE, 25);
    g.update(1.0 / 20.0);
    g.update(1.0 / 20.0);
    assert_eq!(tied(&g), None, "an ordinary pole 25 blocks up is out of reach");
    let (mut g, pole) = pole_game(LONG_POLE, 25);
    for _ in 0..3 {
        g.update(1.0 / 20.0);
    }
    assert_eq!(tied(&g), Some(pole), "a long-distance pole reaches that far");
    let (mut g, _) = pole_game(LONG_POLE, CHARGE_RANGE_LONG + 5);
    g.update(1.0 / 20.0);
    g.update(1.0 / 20.0);
    assert_eq!(tied(&g), None, "but not past its own range");
}

#[test]
fn the_pack_powers_up_when_it_starts_charging_and_down_when_it_loses_the_pole() {
    use crate::sound::{POWER_DOWN, POWER_UP};
    let (mut g, pole) = pole_game(LONG_POLE, 10);
    let mut heard = Vec::new();
    let frames = |g: &mut crate::Game, n: u32, heard: &mut Vec<u8>| {
        for _ in 0..n {
            g.update(1.0 / 20.0);
            heard.extend(g.sounds.kinds());
            g.sounds.clear();
        }
    };
    frames(&mut g, 4, &mut heard);
    assert_eq!(tied(&g), Some(pole));
    assert_eq!(heard.iter().filter(|&&k| k == POWER_UP).count(), 1, "one power-up: {heard:?}");
    assert!(!heard.contains(&POWER_DOWN));
    g.sim.apply(g.local, Action::BreakBlock { pos: pole });
    frames(&mut g, 4, &mut heard);
    assert_eq!(tied(&g), None);
    assert_eq!(heard.iter().filter(|&&k| k == POWER_DOWN).count(), 1, "one power-down: {heard:?}");
}

#[test]
fn a_hovering_body_holds_its_height_and_moves_faster() {
    let step = |jump: bool, crouch: bool, hover: bool| {
        let mut p = Player::new(crate::math::Vec3::new(0.5, 100.0, 0.5));
        p.hover = hover;
        p.input.forward = 1.0;
        p.input.jump = jump;
        p.input.crouch = crouch;
        let (mut solid, mut block) = (|_, _, _| false, |_, _, _| AIR);
        for _ in 0..240 {
            p.step(1.0 / 120.0, &mut solid, &mut block);
        }
        (p.pos.y, p.vel.y, p.vel.length())
    };
    let (y, vy, speed) = step(false, false, true);
    assert!((y - 100.0).abs() < 0.01 && vy.abs() < 0.01, "holds its height: {y}");
    assert!(speed > 7.0, "faster than walking: {speed}");
    assert!(step(true, false, true).0 > 105.0, "jump rises");
    assert!(step(false, true, true).0 < 95.0, "crouch sinks");
    assert!(step(false, false, false).0 < 90.0, "without it the body falls");
}
