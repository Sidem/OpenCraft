use super::*;
use crate::block::RAIL;
use crate::bytes::{ByteReader, ByteWriter};
use crate::item::{IRON_INGOT, IRON_PLATE};
use crate::math::Rng;
use crate::world::World;

use super::super::schedule::DROP_ALL;
use super::super::{CAR_MM, LOCO_MM};

fn v(x: i32, y: i32, z: i32) -> IVec3 {
    IVec3::new(x, y, z)
}

/// A train of a locomotive and three wagons on a straight line, or halfway round a quarter bend.
fn train(bend: bool) -> Factory {
    let mut f = Factory::default();
    let nodes = if bend { [(v(0, 0, 0), 0), (v(16, 0, -16), 64)] } else { [(v(0, 0, 0), 64), (v(24, 0, 0), 64)] };
    for (c, yaw) in nodes {
        f.place(&mut World::new(1, 2), RAIL, c, yaw, c, 0);
    }
    f.lay_track(nodes[0].0, nodes[1].0);
    assert!(f.place_train(nodes[0].0));
    for _ in 0..3 {
        assert!(f.couple(nodes[0].0));
    }
    f.trains[0].head = 18_000;
    f
}

fn unit(d: Vec3) -> Vec3 {
    d * (1.0 / d.length())
}

#[test]
fn rays_from_beside_the_track_pick_the_car_they_point_at() {
    let mut rng = Rng::new(7);
    let mut rays = 0;
    for bend in [false, true] {
        let f = train(bend);
        let t = &f.trains[0];
        for car in 0..=t.cars {
            let from = if car == 0 { 0 } else { LOCO_MM + i64::from(car - 1) * CAR_MM };
            let len = if car == 0 { LOCO_MM } else { CAR_MM };
            for _ in 0..6 {
                // A point inside the car, found along the track (not from the car's pose), and a ray at it from aside.
                let mm = from + 300 + (rng.next_f64() * (len - 600) as f64) as i64;
                let on = f.point_back(t, mm).unwrap();
                let along = unit(f.point_back(t, mm - 100).unwrap() - f.point_back(t, mm + 100).unwrap());
                let side = Vec3::new(-along.z, 0.0, along.x) * if rng.next_f64() < 0.5 { 3.0 } else { -3.0 };
                let target = on + Vec3::new(0.0, RAIL_TOP + 0.25 + rng.next_f64() * 0.6, 0.0);
                let origin = target + side + Vec3::new(0.0, rng.range(-0.6, 1.2), 0.0);
                let dir = unit(target - origin);
                let hit = f.car_hit(origin, dir, 6.0).map(|(_, aim)| aim);
                assert_eq!(hit, Some(CarAim { train: t.id, car }), "bend {bend}, {mm} mm back");
                assert_eq!(f.car_hit(origin, dir * -1.0, 6.0), None, "pointing away hits nothing");
                rays += 1;
            }
        }
    }
    assert_eq!(rays, 48);
}

#[test]
fn a_ray_over_the_roof_or_out_of_reach_misses() {
    let f = train(false);
    let t = &f.trains[0];
    let on = f.point_back(t, LOCO_MM + CAR_MM / 2).unwrap();
    let over = on + Vec3::new(0.0, RAIL_TOP + WAGON_TOP + 0.1, 0.0);
    assert_eq!(f.car_hit(over + Vec3::new(0.0, 0.0, 3.0), Vec3::new(0.0, 0.0, -1.0), 6.0), None);
    let inside = on + Vec3::new(0.0, RAIL_TOP + 0.5, 0.0);
    assert!(f.car_hit(inside + Vec3::new(0.0, 0.0, 3.0), Vec3::new(0.0, 0.0, -1.0), 6.0).is_some());
    assert_eq!(f.car_hit(inside + Vec3::new(0.0, 0.0, 9.0), Vec3::new(0.0, 0.0, -1.0), 6.0), None, "too far");
}

#[test]
fn readouts_and_the_panel_say_what_each_wagon_holds() {
    let mut f = train(false);
    let id = f.trains[0].id;
    let slots = f.box_slots_mut(car_box(id, 2)).expect("wagon 2 is a box");
    assert_eq!(slots.len(), CAR_SLOTS);
    assert_eq!(crate::inventory::add_to_slots(slots, IRON_INGOT, 300), 0);
    assert_eq!(crate::inventory::add_to_slots(slots, IRON_PLATE, 40), 0);
    let t = &f.trains[0];
    let second = &t.cargo.slots[CAR_SLOTS..2 * CAR_SLOTS];
    let used = second.iter().filter(|s| !s.is_empty()).count();
    assert!(t.cargo.slots[..CAR_SLOTS].iter().all(|s| s.is_empty()), "the box address is the second wagon");
    let report = f.car_report(CarAim { train: id, car: 2 });
    assert!(report.starts_with(&format!("{used} of {CAR_SLOTS} slots used: 300 ")), "{report}");
    assert!(report.contains(", 40 "), "{report}");
    assert!(f.car_report(CarAim { train: id, car: 1 }).starts_with("Empty"));
    assert!(f.car_report(CarAim { train: id, car: 0 }).starts_with(&format!("3 wagons: {used} of 72 slots used")));
    assert_eq!(f.car_name(CarAim { train: id, car: 2 }), "Wagon 2 of 3");
    // Wagons, next stop, stops, then each wagon's used slots and items.
    assert_eq!(f.train_panel(CarAim { train: id, car: 0 }), vec![3, 0, 0, 0, 0, used as i32, 340, 0, 0]);
    assert!(f.box_slots(car_box(id, 4)).is_none() && f.box_slots(car_box(id, 0)).is_none(), "no such wagon");
    assert!(f.box_slots(car_box(id + 1, 1)).is_none(), "no such train");
}

#[test]
fn a_wagon_heaps_its_main_item_as_high_as_it_is_full() {
    let mut f = train(false);
    let id = f.trains[0].id;
    assert_eq!(Factory::heap(&f.trains[0], 1), None, "empty");
    let slots = f.box_slots_mut(car_box(id, 1)).unwrap();
    crate::inventory::add_to_slots(slots, IRON_PLATE, crate::item::stack_size(IRON_PLATE) * 12);
    crate::inventory::add_to_slots(slots, IRON_INGOT, 1);
    let (layer, fill) = Factory::heap(&f.trains[0], 1).unwrap();
    assert_eq!(layer, crate::item::def(IRON_PLATE).unwrap().tex[0]);
    assert!((fill - 12.0 / 24.0).abs() < 0.01, "{fill}");
    assert_eq!(Factory::heap(&f.trains[0], 0), None, "the locomotive has no hold");
}

#[test]
fn the_panel_drops_stops_and_ids_survive_a_save() {
    let mut f = train(false);
    let id = f.trains[0].id;
    f.trains[0].schedule = vec![v(1, 0, 0), v(2, 0, 0), v(3, 0, 0)];
    f.trains[0].next = 2;
    assert!(f.drop_stop(id, 1));
    assert_eq!(f.trains[0].schedule, vec![v(1, 0, 0), v(3, 0, 0)]);
    assert_eq!(f.trains[0].next, 0, "the next stop is still on the list");
    assert!(!f.drop_stop(id, 5) && !f.drop_stop(id + 1, 0));
    assert!(f.drop_stop(id, DROP_ALL) && f.trains[0].schedule.is_empty());
    let mut w = ByteWriter::default();
    f.write_trains(&mut w);
    let mut back = train(false);
    back.trains.clear();
    back.read_trains(&mut ByteReader::new(&w.bytes)).unwrap();
    assert_eq!(back.trains[0].id, id);
    // An older save has no ids: its trains are numbered.
    let mut old = train(false);
    old.trains.clear();
    let mut r = ByteReader::new(&w.bytes[..w.bytes.len() - 4]);
    r.version = 65;
    old.read_trains(&mut r).unwrap();
    assert_eq!(old.trains[0].id, 1);
}
