use super::*;
use crate::action::Action;
use crate::block::{AIR, STORAGE};
use crate::item::IRON_PLATE;
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
    assert!(!helpers(&sim).thrusting, "no jetpack in the pack");
    sim.apply(P, Action::Give { item: JETPACK, count: 1 });
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
    sim.apply(P, Action::Give { item: coal(), count: 1 });
    sim.apply(P, Action::Jetpack { on: true });
    run(&mut sim, 120);
    sim.apply(P, Action::Jetpack { on: false });
    assert!(!helpers(&sim).thrusting);
    assert_eq!(helpers(&sim).jet, JET_TICKS_PER_COAL - 120);
    sim.apply(P, Action::Jetpack { on: true });
    assert_eq!(pack(&sim, coal()), 0, "the tank is used before another coal");
    let inv = &mut sim.players[0].as_mut().unwrap().inventory;
    inv.remove(JETPACK, 1);
    run(&mut sim, 1);
    assert!(!helpers(&sim).thrusting, "no jetpack, no thrust");
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
    };
    let mut w = ByteWriter::default();
    h.write_state(&mut w);
    assert_eq!(Helpers::read_state(&mut ByteReader::new(&w.bytes)), Some(h));
}
