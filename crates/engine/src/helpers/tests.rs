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

#[test]
fn helpers_round_trip_through_bytes() {
    let h = Helpers {
        thrusting: true,
        jet: 77,
        drone: drone::Drone::from_errand(IRON_PLATE, IVec3::new(-4, 70, 9)),
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
    sim.world.set_block_anywhere(pole - IVec3::new(0, 1, 0), STORAGE); // a pole needs something to stand on
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
    place_item(sim, who, ItemId::block(block), pos);
}

fn place_item(sim: &mut Sim, who: PlayerId, item: ItemId, pos: IVec3) {
    sim.apply(who, Action::Give { item, count: 1 });
    let inv = &sim.player(who).unwrap().inventory;
    let slot = inv.slots.iter().position(|s| s.item == item).unwrap();
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
    pole_game_at(ItemId::block(block), IVec3::new(0, up, 0))
}

/// The same with any pole item, put at `offset` from the feet.
fn pole_game_at(item: ItemId, offset: IVec3) -> (crate::Game, IVec3) {
    let mut g = crate::Game::new(2024, 3);
    crate::tests::run_until_ready(&mut g);
    let me = g.local.0 as usize;
    g.sim.players[me].as_mut().unwrap().inventory.worn[4] = HOVER_PACK;
    let feet = g.body().pos.floor();
    let pole = IVec3::new(feet.x, feet.y, feet.z) + offset;
    // An ordinary pole needs something solid under it (`pole_mount.rs`).
    g.sim.world.set_block_anywhere(pole - IVec3::new(0, 1, 0), STORAGE);
    place_item(&mut g.sim, g.local, item, pole);
    (g, pole)
}

fn tied(g: &crate::Game) -> Option<IVec3> {
    g.sim.player(g.local).unwrap().helpers.pole
}

/// Whether the hands tie the pack to a pole of `item` put `offset` from the feet.
fn ties(item: ItemId, offset: IVec3) -> bool {
    let (mut g, pole) = pole_game_at(item, offset);
    for _ in 0..3 {
        g.update(1.0 / 20.0);
    }
    tied(&g) == Some(pole)
}

#[test]
fn a_pole_charges_as_far_as_it_wires_to_its_own_kind() {
    use crate::factory::{tier_stats, LONG_STATS};
    use crate::item::{POLE_MK3, POLE_MK4};
    let poles = [(ItemId::block(POLE), 0), (POLE_MK3, 2), (POLE_MK4, 3)];
    for (item, tier) in poles {
        let link = tier_stats(tier).link;
        assert!(ties(item, IVec3::new(0, link - 1, 0)), "Mk{} charges from just inside {link}", tier + 1);
        assert!(!ties(item, IVec3::new(0, link + 4, 0)), "Mk{} not past {link}", tier + 1);
    }
    // A long-distance pole charges from 160 blocks: further than the world is tall, so across as well as up.
    let link = LONG_STATS.link;
    assert!(ties(ItemId::block(LONG_POLE), IVec3::new(link - 70, 60, 0)), "a long-distance pole from far away");
    assert!(!ties(ItemId::block(LONG_POLE), IVec3::new(link, 60, 0)), "but not past its own span");
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
