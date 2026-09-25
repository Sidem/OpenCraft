use super::*;
use crate::inventory::{INVENTORY_SLOTS, MAX_STACK};

fn collector(x: f64, y: f64, z: f64) -> Collector {
    Collector { center: Vec3::new(x, y, z), room: Inventory::default() }
}

#[test]
fn item_falls_and_is_collected() {
    let mut items = Items::default();
    items.spawn(Vec3::new(0.5, 3.0, 0.5), Vec3::ZERO, 1, 2, 0.0);
    let mut floor = |_x: i32, y: i32, _z: i32| y < 0;
    let loaded = |_p: Vec3| true;
    let mut far = [collector(50.0, 0.0, 50.0)];
    for _ in 0..240 {
        items.update(1.0 / 60.0, &mut far, &mut floor, &loaded, |_, _, _| {});
    }
    assert!((items.list[0].pos.y - HALF).abs() < 1e-6, "rests on the floor");

    let mut near = [collector(1.5, 0.9, 0.5)];
    let mut got = 0;
    for _ in 0..120 {
        items.update(1.0 / 60.0, &mut near, &mut floor, &loaded, |_, _, n| got += n);
    }
    assert!(items.list.is_empty());
    assert_eq!(got, 2);
    assert_eq!(near[0].room.slots[0].count, 2);
}

/// Who picks up one stone lying on the floor at the origin: (collector, count) per pickup.
fn pickups(collectors: &mut [Collector]) -> Vec<(usize, u32)> {
    let mut items = Items::default();
    items.spawn(Vec3::new(0.5, 0.2, 0.5), Vec3::ZERO, 1, 1, 0.0);
    let mut got = Vec::new();
    for _ in 0..120 {
        let mut floor = |_: i32, y: i32, _: i32| y < 0;
        items.update(1.0 / 60.0, collectors, &mut floor, &|_: Vec3| true, |c, _, n| got.push((c, n)));
    }
    got
}

#[test]
fn the_nearest_player_with_room_gets_the_item() {
    assert_eq!(pickups(&mut [collector(2.0, 0.9, 0.5), collector(1.2, 0.9, 0.5)]), vec![(1, 1)]);

    // A full inventory doesn't attract items; the next player in reach gets them.
    let mut full = collector(1.2, 0.9, 0.5);
    full.room.add(2, INVENTORY_SLOTS as u32 * MAX_STACK);
    assert_eq!(pickups(&mut [collector(2.0, 0.9, 0.5), full]), vec![(0, 1)]);
}
