use super::*;
use crate::inventory::INVENTORY_SLOTS;
use crate::item::MAX_STACK;

fn dry(_: Vec3) -> bool {
    false
}

fn collector(x: f64, y: f64, z: f64) -> Collector {
    Collector { center: Vec3::new(x, y, z), room: Inventory::default() }
}

#[test]
fn item_falls_and_is_collected() {
    let mut items = Items::default();
    items.spawn(Vec3::new(0.5, 3.0, 0.5), Vec3::ZERO, ItemId(1), 2, 0.0);
    let mut floor = |_x: i32, y: i32, _z: i32| y < 0;
    let loaded = |_p: Vec3| true;
    let mut far = [collector(50.0, 0.0, 50.0)];
    for _ in 0..240 {
        items.update(1.0 / 60.0, &mut far, &mut floor, &dry, &loaded, |_, _, _| {});
    }
    assert!((items.list[0].pos.y - HALF).abs() < 1e-6, "rests on the floor");

    let mut near = [collector(1.5, 0.9, 0.5)];
    let mut got = 0;
    for _ in 0..120 {
        items.update(1.0 / 60.0, &mut near, &mut floor, &dry, &loaded, |_, _, n| got += n);
    }
    assert!(items.list.is_empty());
    assert_eq!(got, 2);
    assert_eq!(near[0].room.slots[0].count, 2);
}

/// Who picks up one stone lying on the floor at the origin: (collector, count) per pickup.
fn pickups(collectors: &mut [Collector]) -> Vec<(usize, u32)> {
    let mut items = Items::default();
    items.spawn(Vec3::new(0.5, 0.2, 0.5), Vec3::ZERO, ItemId(1), 1, 0.0);
    let mut got = Vec::new();
    for _ in 0..120 {
        let mut floor = |_: i32, y: i32, _: i32| y < 0;
        items.update(1.0 / 60.0, collectors, &mut floor, &dry, &|_: Vec3| true, |c, _, n| got.push((c, n)));
    }
    got
}

#[test]
fn the_nearest_player_with_room_gets_the_item() {
    assert_eq!(pickups(&mut [collector(2.0, 0.9, 0.5), collector(1.2, 0.9, 0.5)]), vec![(1, 1)]);

    // A full inventory doesn't attract items; the next player in reach gets them.
    let mut full = collector(1.2, 0.9, 0.5);
    full.room.add(ItemId(2), INVENTORY_SLOTS as u32 * MAX_STACK);
    assert_eq!(pickups(&mut [collector(2.0, 0.9, 0.5), full]), vec![(0, 1)]);
}

#[test]
fn items_float_up_to_the_surface_and_drift() {
    let mut items = Items::default();
    items.spawn(Vec3::new(0.5, 3.0, 0.5), Vec3::ZERO, ItemId(1), 1, 0.0);
    // Water from the floor at y 0 up to its surface at y 10.
    let mut floor = |_: i32, y: i32, _: i32| y < 0;
    let water = |p: Vec3| p.y < 10.0;
    let mut far = [collector(50.0, 0.0, 50.0)];
    let mut heights = Vec::new();
    for _ in 0..60 * 20 {
        items.update(1.0 / 60.0, &mut far, &mut floor, &water, &|_: Vec3| true, |_, _, _| {});
        heights.push(items.list[0].pos.y);
    }
    let bob = &heights[heights.len() - 120..];
    assert!(bob.iter().all(|y| (y - 10.0).abs() < 0.1), "bobs at the surface: {bob:?}");
    let e = &items.list[0];
    let drifted = (e.pos.x - 0.5).hypot(e.pos.z - 0.5);
    assert!(drifted > 1.0 && drifted < 4.0, "drifts slowly: {drifted}");
}
