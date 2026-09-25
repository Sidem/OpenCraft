use super::belt::{END_STOP, ITEM_SPACING};
use super::describe::fmt_duration;
use super::*;
use crate::block::IRON_ORE;

const EAST: u8 = 1;
const SOUTH: u8 = 2;

fn run(f: &mut Factory, seconds: f64, mut check: impl FnMut(&Factory)) {
    let mut world = World::new(1, 2);
    let mut events = Vec::new();
    let ticks = (seconds * crate::TICK_RATE as f64) as u64;
    for tick in 0..ticks {
        f.update(&mut world, tick, &mut events);
        check(f);
        events.clear();
    }
}

fn spacing_ok(f: &Factory) {
    for b in &f.belts {
        for w in b.items.windows(2) {
            assert!(w[0].p - w[1].p >= ITEM_SPACING - 1e-4, "items too close on belt at {:?}: {:?}", b.pos, b.items);
        }
        for it in &b.items {
            assert!((0.0..=1.0).contains(&it.p), "item off the belt: {it:?}");
        }
    }
}

fn stocked_box(f: &mut Factory, pos: IVec3, item: BlockId, n: u32) {
    f.add_storage(pos);
    let Some(Slot::Storage(i)) = f.at.get(&pos) else { unreachable!() };
    add_to_slots(&mut f.storages[*i as usize].slots, item, n);
}

#[test]
fn box_to_box_through_a_belt_line() {
    let mut f = Factory::default();
    stocked_box(&mut f, IVec3::new(0, 0, 0), IRON_ORE, 10);
    for x in 1..=5 {
        f.add_belt(IVec3::new(x, 0, 0), EAST);
    }
    f.add_storage(IVec3::new(6, 0, 0));
    run(&mut f, 12.0, spacing_ok);
    assert_eq!(f.storage_count_at(IVec3::new(6, 0, 0), IRON_ORE), 10);
    assert_eq!(f.storage_count_at(IVec3::new(0, 0, 0), IRON_ORE), 0);
}

#[test]
fn blocked_line_backs_up_without_overlap() {
    let mut f = Factory::default();
    stocked_box(&mut f, IVec3::new(0, 0, 0), IRON_ORE, 40);
    for x in 1..=3 {
        f.add_belt(IVec3::new(x, 0, 0), EAST);
    }
    run(&mut f, 20.0, spacing_ok);
    let on_belts: usize = f.belts.iter().map(|b| b.items.len()).sum();
    assert!((7..=9).contains(&on_belts), "{on_belts} items on three full belts");
    assert_eq!(f.storage_count_at(IVec3::new(0, 0, 0), IRON_ORE) as usize, 40 - on_belts);
    let last = f.belt_at(IVec3::new(3, 0, 0));
    assert!(last.items[0].p <= END_STOP + 1e-6);
}

#[test]
fn corner_turns_and_side_joins() {
    let mut f = Factory::default();
    // East, then a corner turning south.
    f.add_belt(IVec3::new(0, 0, 0), EAST);
    f.add_belt(IVec3::new(1, 0, 0), SOUTH);
    // A straight southbound line with a belt joining it from the west side.
    f.add_belt(IVec3::new(5, 0, -1), SOUTH);
    f.add_belt(IVec3::new(5, 0, 0), SOUTH);
    f.add_belt(IVec3::new(4, 0, 0), EAST);
    // Two belts facing each other never connect.
    f.add_belt(IVec3::new(9, 0, 0), EAST);
    f.add_belt(IVec3::new(10, 0, 0), 3);
    f.relink();

    assert_eq!(f.belt_at(IVec3::new(1, 0, 0)).curve_from, Some(3));
    assert!(matches!(f.belt_at(IVec3::new(0, 0, 0)).out, Link::Belt { mid: false, .. }));
    assert_eq!(f.belt_at(IVec3::new(5, 0, 0)).curve_from, None);
    assert!(matches!(f.belt_at(IVec3::new(4, 0, 0)).out, Link::Belt { mid: true, .. }));
    assert!(matches!(f.belt_at(IVec3::new(5, 0, -1)).out, Link::Belt { mid: false, .. }));
    assert_eq!(f.belt_at(IVec3::new(9, 0, 0)).out, Link::None);
    assert_eq!(f.belt_at(IVec3::new(10, 0, 0)).out, Link::None);

    // Items entering the corner start at its west edge and leave through its south edge.
    let corner = f.belt_at(IVec3::new(1, 0, 0));
    assert_eq!(corner.offset(0.0), (-0.5, 0.0));
    assert_eq!(corner.offset(1.0), (0.0, 0.5));
}

#[test]
fn merging_lines_deliver_everything() {
    let mut f = Factory::default();
    stocked_box(&mut f, IVec3::new(0, 0, -3), IRON_ORE, 12);
    for z in -2..=2 {
        f.add_belt(IVec3::new(0, 0, z), SOUTH);
    }
    stocked_box(&mut f, IVec3::new(-3, 0, 0), crate::block::COAL_ORE, 12);
    for x in -2..=-1 {
        f.add_belt(IVec3::new(x, 0, 0), EAST);
    }
    f.add_storage(IVec3::new(0, 0, 3));
    run(&mut f, 40.0, spacing_ok);
    assert_eq!(f.storage_count_at(IVec3::new(0, 0, 3), IRON_ORE), 12);
    assert_eq!(f.storage_count_at(IVec3::new(0, 0, 3), crate::block::COAL_ORE), 12);
}

#[test]
fn removing_a_belt_returns_its_items_and_relinks() {
    let mut f = Factory::default();
    stocked_box(&mut f, IVec3::new(0, 0, 0), IRON_ORE, 20);
    for x in 1..=4 {
        f.add_belt(IVec3::new(x, 0, 0), EAST);
    }
    run(&mut f, 3.0, |_| {});
    let dropped = f.remove(IVec3::new(1, 0, 0));
    assert!(dropped.iter().map(|s| s.count).sum::<u32>() > 0);
    // The box has no belt leading away any more; the rest of the line still drains forward.
    run(&mut f, 1.0, spacing_ok);
    assert_eq!(f.belt_count(), 3);
    assert!(f.belt_at(IVec3::new(2, 0, 0)).out != Link::None);
}

#[test]
fn number_formatting() {
    assert_eq!(fmt_int(0), "0");
    assert_eq!(fmt_int(999), "999");
    assert_eq!(fmt_int(1000), "1,000");
    assert_eq!(fmt_int(6_000_000), "6,000,000");
    assert_eq!(fmt_duration(42.0), "42 s");
    assert_eq!(fmt_duration(125.0), "2 min");
    assert_eq!(fmt_duration(3.0 * 3600.0 + 20.0 * 60.0), "3 h 20 min");
    assert_eq!(fmt_duration(90_000.0), "1 d 1 h");
}

#[test]
fn box_readout_lists_the_largest_stock_first() {
    use crate::block::{COAL_ORE, COPPER_ORE, STONE};
    let mut f = Factory::default();
    let pos = IVec3::new(0, 0, 0);
    stocked_box(&mut f, pos, COAL_ORE, 5);
    let Some(Slot::Storage(i)) = f.at.get(&pos) else { unreachable!() };
    let slots = &mut f.storages[*i as usize].slots;
    add_to_slots(slots, IRON_ORE, 130);
    add_to_slots(slots, COPPER_ORE, 40);
    add_to_slots(slots, STONE, 1);
    let text = f.describe(pos).unwrap();
    // 130 iron fills three stacks of 64.
    assert_eq!(
        text,
        "6 of 24 slots used\n130 Iron Ore, 40 Copper Ore, 5 Coal Ore, ...\nRight-click to take everything"
    );
}
