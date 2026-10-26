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

/// Loose-item cost: `cargo test --release -q bench_items -- --ignored --nocapture`. Prints microseconds per update of
/// 5,000 items resting in one pile and spread over 64 x 64 blocks, and how many entities are left after 10 s.
#[test]
#[ignore]
fn bench_items() {
    for (name, spread) in [("one pile", 0.0), ("spread", 64.0)] {
        let mut items = Items::default();
        for i in 0..5000u32 {
            let (fx, fz) = (f64::from(i % 71) / 71.0, f64::from(i / 71 % 71) / 71.0);
            items.spawn(Vec3::new(fx * spread, 0.2, fz * spread), Vec3::ZERO, ItemId(1), 1, 0.0);
        }
        let mut floor = |_: i32, y: i32, _: i32| y < 0;
        let mut far = [collector(5000.0, 0.0, 5000.0)];
        let start = std::time::Instant::now();
        for _ in 0..600 {
            items.update(1.0 / 60.0, &mut far, &mut floor, &dry, &|_: Vec3| true, |_, _, _| {});
        }
        let us = start.elapsed().as_micros() as f64 / 600.0;
        let mut out = Vec::new();
        let start = std::time::Instant::now();
        for _ in 0..100 {
            out.clear();
            items.write_instances(&mut out, Vec3::new(0.0, 5.0, 0.0));
        }
        let draw = start.elapsed().as_micros() as f64 / 100.0;
        println!("bench_items {name}: {us:.0} us per update, {draw:.0} us to draw, {} entities left", items.list.len());
    }
}

fn floor_step(items: &mut Items, seconds: f64, loaded: bool) {
    let mut floor = |_: i32, y: i32, _: i32| y < 0;
    let mut far = [collector(5000.0, 0.0, 5000.0)];
    for _ in 0..(seconds * 60.0) as usize {
        items.update(1.0 / 60.0, &mut far, &mut floor, &dry, &move |_: Vec3| loaded, |_, _, _| {});
    }
}

fn total(items: &Items, item: ItemId) -> u32 {
    items.list.iter().filter(|e| e.item == item).map(|e| e.count).sum()
}

#[test]
fn resting_items_of_one_kind_merge_into_stacks() {
    let stone = ItemId(1);
    let mut items = Items::default();
    for _ in 0..100 {
        items.spawn(Vec3::new(0.5, 0.2, 0.5), Vec3::ZERO, stone, 1, 0.0);
    }
    floor_step(&mut items, 2.0, true);
    let mut counts: Vec<u32> = items.list.iter().map(|e| e.count).collect();
    counts.sort_unstable();
    assert_eq!(counts, vec![36, 64], "one full stack and the rest");
}

#[test]
fn merging_keeps_the_rules() {
    let (stone, dirt) = (ItemId(1), ItemId(2));
    let tool = (256..).map(ItemId).find(|&i| crate::tools::tool(i).is_some()).expect("a tool item");
    let mut items = Items::default();
    let mut seed = 12345u32;
    let mut next = |n: u32| {
        seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
        (seed >> 8) % n
    };
    let kinds = [stone, dirt, tool];
    let mut before = [0u32; 3];
    for _ in 0..3000 {
        let k = next(3) as usize;
        let count = if k == 2 { 1 + next(50) } else { 1 + next(item::stack_size(kinds[k])) };
        before[k] += count;
        let pos = Vec3::new(f64::from(next(40)) / 10.0, 0.2, f64::from(next(40)) / 10.0);
        items.spawn(pos, Vec3::ZERO, kinds[k], count, 0.0);
    }
    let tools_before = items.list.iter().filter(|e| e.item == tool).count();
    let others_before = items.list.len() - tools_before;
    floor_step(&mut items, 3.0, true);
    for (k, &item) in kinds.iter().enumerate() {
        assert_eq!(total(&items, item), before[k], "no item is lost or made");
    }
    assert!(items.list.iter().all(|e| e.count <= item::stack_size(e.item) || e.item == tool), "no stack is too big");
    assert_eq!(items.list.iter().filter(|e| e.item == tool).count(), tools_before, "tools never merge");
    let others = items.list.len() - tools_before;
    assert!(others * 10 < others_before * 6, "it did merge: {others} of {others_before}");
}

#[test]
fn only_items_that_lie_still_and_close_together_merge() {
    let stone = ItemId(1);
    let mut items = Items::default();
    items.spawn(Vec3::new(0.5, 0.2, 0.5), Vec3::ZERO, stone, 1, 0.0);
    items.spawn(Vec3::new(20.5, 0.2, 0.5), Vec3::ZERO, stone, 1, 0.0); // far away
    items.spawn(Vec3::new(0.5, 0.2, 0.5), Vec3::ZERO, ItemId(2), 1, 0.0); // another kind
    items.spawn(Vec3::new(0.5, 0.2, 0.5), Vec3::ZERO, stone, 1, 5.0); // still in its pickup delay
    floor_step(&mut items, 2.0, true);
    assert_eq!(items.list.len(), 4);
    items.spawn(Vec3::new(0.5, 30.0, 0.5), Vec3::ZERO, stone, 1, 0.0); // in the air
    floor_step(&mut items, 0.3, true);
    assert_eq!(items.list.len(), 5, "an item in the air is left alone");
    floor_step(&mut items, 6.0, true);
    assert_eq!(
        items.list.len(),
        3,
        "the delayed and the fallen one joined the pile at the origin: {:?}",
        items.list.len()
    );
}

#[test]
fn items_waiting_in_unloaded_chunks_merge_where_they_wait() {
    // An ejector far from every player: one item every tenth of a second for ten minutes.
    let stone = ItemId(1);
    let mut items = Items::default();
    let mut peak = 0;
    for _ in 0..6000 {
        items.spawn(Vec3::new(500.5, 70.2, 500.5), Vec3::new(0.0, 1.0, 0.0), stone, 1, 0.25);
        floor_step(&mut items, 0.1, false);
        peak = peak.max(items.list.len());
    }
    // Everything older than the despawn time is gone; what is alive fits in whole stacks (plus the newest).
    let alive: u32 = total(&items, stone);
    assert!(alive <= 3010, "items despawn after {DESPAWN_SECONDS} s: {alive}");
    let limit = alive.div_ceil(item::stack_size(stone)) as usize + 8;
    assert!(items.list.len() <= limit && peak <= 80, "{} entities, peak {peak}, alive {alive}", items.list.len());
}

#[test]
fn a_merged_pile_is_as_old_as_its_items_on_average() {
    let stone = ItemId(1);
    let mut items = Items::default();
    items.spawn(Vec3::new(0.5, 0.2, 0.5), Vec3::ZERO, stone, 3, 0.0);
    items.list[0].age = 100.0;
    items.spawn(Vec3::new(0.5, 0.2, 0.5), Vec3::ZERO, stone, 1, 0.0);
    items.list[1].age = 0.0;
    floor_step(&mut items, 0.5, true);
    assert_eq!(items.list.len(), 1);
    assert_eq!(items.list[0].count, 4);
    assert!((items.list[0].age - 75.5).abs() < 1.0, "weighted age {}", items.list[0].age);
}
