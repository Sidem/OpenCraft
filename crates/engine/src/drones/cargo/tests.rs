use super::*;
use crate::action::Action;
use crate::block::{COAL_ORE, DRONE_PORT, GENERATOR, POLE, STORAGE};
use crate::item::{ItemId, IRON_PLATE};
use crate::sim::PlayerId;

const P: PlayerId = PlayerId(0);

fn put(sim: &mut Sim, item: ItemId, pos: IVec3) {
    sim.apply(P, Action::Give { item, count: 1 });
    sim.apply(P, Action::PlaceBlock { pos, slot: 0, facing: 0, against: pos });
}

fn run(sim: &mut Sim, seconds: u32) {
    for _ in 0..seconds * TICK_RATE {
        sim.step();
    }
}

/// A powered port `dx` blocks east of (900, 200, -900) with `drones` cargo drones at home and a box beside the pad;
/// returns (port anchor, box cell).
fn site(sim: &mut Sim, dx: i32, drones: u32) -> (IVec3, IVec3) {
    let at = |x: i32| IVec3::new(900 + dx + x, 200, -900);
    let (gen, pole, port) = (at(0), at(4), at(9));
    put(sim, GENERATOR.into(), gen);
    put(sim, POLE.into(), pole);
    put(sim, DRONE_PORT.into(), port);
    sim.apply(P, Action::Give { item: COAL_ORE.into(), count: 20 });
    sim.apply(P, Action::Insert { pos: gen, item: COAL_ORE.into() });
    sim.apply(P, Action::Give { item: CARGO_DRONE, count: drones });
    for _ in 0..drones {
        sim.apply(P, Action::Insert { pos: port, item: CARGO_DRONE });
    }
    let centre = sim.factory.port_centre(port).unwrap();
    let supply = IVec3::new(centre.x.floor() as i32 + 2, port.y, centre.z.floor() as i32);
    put(sim, STORAGE.into(), supply);
    run(sim, 1);
    (port, supply)
}

/// Two sites `gap` blocks apart, the first with `drones` cargo drones; (from port, from box, to port, to box).
fn pair(sim: &mut Sim, gap: i32, drones: u32) -> (IVec3, IVec3, IVec3, IVec3) {
    let (a, a_box) = site(sim, 0, drones);
    let (b, b_box) = site(sim, gap, 0);
    (a, a_box, b, b_box)
}

fn route(sim: &mut Sim, from: IVec3, to: IVec3) {
    sim.apply(P, Action::SetRoute { from, to, clear: false });
}

fn fleet(sim: &Sim) -> u32 {
    sim.factory.ports().iter().map(|p| p.couriers).sum::<u32>() + sim.cargo.list.len() as u32
}

#[test]
fn a_cargo_drone_hauls_a_stack_to_the_other_port_and_burns_batteries() {
    let mut sim = Sim::new(7, 2);
    let (a, a_box, b, b_box) = pair(&mut sim, 80, 1);
    sim.factory.stock(a_box, IRON_PLATE, 200);
    sim.factory.stock(a_box, BATTERY, 10);
    route(&mut sim, a, b);
    assert_eq!(sim.cargo.route_from(a), Some(b));
    let mut flew = false;
    for _ in 0..40 * TICK_RATE {
        sim.step();
        flew |= !sim.cargo.list.is_empty();
        assert_eq!(fleet(&sim), 1, "the drone is on the pad or in the air, never both or neither");
    }
    assert!(flew);
    let dist = (sim.factory.port_centre(b).unwrap() - sim.factory.port_centre(a).unwrap()).length();
    let per_trip = trip_batteries(dist);
    assert_eq!(per_trip, 2, "80 blocks there and back is 160, over one battery's 150");
    let delivered = sim.factory.box_count(b_box, IRON_PLATE);
    assert!(delivered >= LOAD && delivered.is_multiple_of(LOAD), "whole loads arrive: {delivered}");
    let trips = delivered / LOAD;
    assert_eq!(sim.factory.box_count(a_box, BATTERY), 10 - trips * per_trip);
    assert_eq!(sim.factory.box_count(a_box, IRON_PLATE), 200 - delivered, "nothing is made or lost");
}

#[test]
fn it_waits_for_batteries_and_room_and_a_battery_is_never_cargo() {
    let mut sim = Sim::new(7, 2);
    let (a, a_box, b, b_box) = pair(&mut sim, 80, 1);
    sim.factory.stock(a_box, BATTERY, 1);
    route(&mut sim, a, b);
    run(&mut sim, 10);
    assert_eq!(
        (sim.cargo.list.len(), sim.factory.box_count(b_box, BATTERY)),
        (0, 0),
        "only batteries: nothing to carry"
    );
    sim.factory.stock(a_box, IRON_PLATE, 10);
    run(&mut sim, 10);
    assert_eq!(
        sim.cargo.list.len() + sim.factory.box_count(b_box, IRON_PLATE) as usize,
        0,
        "one battery is not enough"
    );
    sim.factory.stock(a_box, BATTERY, 1);
    run(&mut sim, 30);
    assert_eq!(sim.factory.box_count(b_box, IRON_PLATE), 10, "with two it goes (and takes what there is)");
    // A full destination: the drone stays home.
    sim.factory.stock(b_box, IRON_PLATE, 24 * 64);
    sim.factory.stock(a_box, IRON_PLATE, 100);
    sim.factory.stock(a_box, BATTERY, 4);
    let before = sim.factory.box_count(a_box, BATTERY);
    run(&mut sim, 10);
    assert_eq!(sim.factory.box_count(a_box, BATTERY), before, "no room over there, no launch");
}

#[test]
fn a_route_needs_two_ports_within_the_range_and_can_be_cleared() {
    let mut sim = Sim::new(7, 2);
    let (a, _, b, _) = pair(&mut sim, 250, 1);
    route(&mut sim, a, b);
    assert_eq!(sim.cargo.route_from(a), None, "250 blocks is past the Mk1 range of 200");
    route(&mut sim, a, a);
    assert_eq!(sim.cargo.route_from(a), None, "not to itself");
    route(&mut sim, a, IVec3::new(0, 0, 0));
    assert_eq!(sim.cargo.route_from(a), None, "not to nothing");
    let (c, _) = site(&mut sim, 150, 0);
    route(&mut sim, a, c);
    assert_eq!(sim.cargo.route_from(a), Some(c));
    let far_cell = sim.factory.port_bounds(c).unwrap().1;
    route(&mut sim, a, far_cell);
    assert_eq!(sim.cargo.routes.len(), 1, "any cell of the port will do, and a source has one route");
    sim.apply(P, Action::SetRoute { from: a, to: c, clear: true });
    assert!(sim.cargo.routes.is_empty());
}

#[test]
fn a_port_keeps_one_kind_of_drone() {
    let mut sim = Sim::new(7, 2);
    let (a, _) = site(&mut sim, 0, 1);
    sim.apply(P, Action::Give { item: crate::item::DRONE, count: 1 });
    sim.apply(P, Action::Insert { pos: a, item: crate::item::DRONE });
    let port = &sim.factory.ports()[0];
    assert_eq!((port.couriers, port.home), (1, 0), "a construction drone does not join cargo drones");
}

#[test]
fn breaking_a_port_gives_back_its_cargo_drones_and_ends_its_routes() {
    let mut sim = Sim::new(7, 2);
    let (a, a_box, b, _) = pair(&mut sim, 80, 2);
    sim.factory.stock(a_box, IRON_PLATE, 200);
    sim.factory.stock(a_box, BATTERY, 10);
    route(&mut sim, a, b);
    run(&mut sim, 5);
    assert!(!sim.cargo.list.is_empty());
    let recalled = sim.recall_cargo(a);
    assert_eq!(recalled.map(|s| s.item), Some(CARGO_DRONE));
    assert!(sim.cargo.list.is_empty() && sim.cargo.routes.is_empty());
}

#[test]
fn cargo_round_trips_through_bytes() {
    let mut cargo = Cargo::default();
    cargo.routes.push((IVec3::new(1, 2, 3), IVec3::new(40, 2, -9)));
    let pos = Vec3::new(10.5, 70.0, -3.25);
    let load = Stack { item: IRON_PLATE, count: 17 };
    cargo.list.push(Courier {
        port: IVec3::new(1, 2, 3),
        dest: IVec3::new(40, 2, -9),
        pos,
        prev: pos,
        leg: Leg::Home,
        load,
    });
    let mut w = ByteWriter::default();
    cargo.write_state(&mut w);
    assert_eq!(Cargo::read_state(&mut ByteReader::new(&w.bytes)), Some(cargo));
}
