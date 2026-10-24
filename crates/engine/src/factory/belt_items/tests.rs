use super::super::tests::{place_block, run, stocked_box};
use super::*;
use crate::block::{IRON_ORE, SMELTER, STORAGE};
use crate::bytes::{ByteReader, ByteWriter};
use crate::factory::belt::Belt;
use crate::factory::provenance::Tag;
use crate::factory::router::Router;
use crate::factory::Machine;

const EAST: u8 = 1;

fn v(x: i32, y: i32, z: i32) -> IVec3 {
    IVec3::new(x, y, z)
}

/// Ore from a box at the origin, two belts, a smelter fed coal from above; its ingots wait on the belt after it.
fn smelting_line(ore: u32) -> Factory {
    let mut f = Factory::default();
    stocked_box(&mut f, v(0, 0, 0), IRON_ORE.into(), ore);
    f.add_belt(v(1, 0, 0), EAST);
    f.add_belt(v(2, 0, 0), EAST);
    stocked_box(&mut f, v(3, 0, -2), crate::block::COAL_ORE.into(), 2);
    f.add_belt(v(3, 0, -1), 2);
    place_block(&mut f, SMELTER, v(3, 0, 0));
    f.add_belt(v(4, 0, 0), EAST);
    f
}

fn aim_at(cell: IVec3, index: usize) -> ItemAim {
    ItemAim { cell, index, bounds: [0.0; 6] }
}

#[test]
fn an_item_keeps_the_cell_of_whoever_put_it_on_a_belt_and_counts_ticks_on_belts() {
    let mut f = smelting_line(10);
    run(&mut f, 3.0, |_| {});
    let ore: Vec<_> = f.belts.iter().filter(|b| b.pos.z == 0 && b.pos.x < 3).flat_map(|b| b.items.iter()).collect();
    assert!(!ore.is_empty());
    assert!(ore.iter().all(|it| it.tag.origin() == Some(v(0, 0, 0))), "{ore:?}");
    // Three belts a second apart at most: nothing can have ridden longer than the run.
    assert!(ore.iter().all(|it| it.tag.ticks <= 180), "{ore:?}");

    // Ingots leave the smelter with a fresh tag naming the smelter, not the box the ore came from.
    run(&mut f, 25.0, |_| {});
    let last = f.belts.iter().find(|b| b.pos == v(4, 0, 0)).unwrap();
    assert!(!last.items.is_empty());
    assert!(last.items.iter().all(|it| it.tag.origin() == Some(v(3, 0, 0))), "{:?}", last.items);
    // They wait at the end of the line: the oldest has ridden for most of the run.
    assert!(last.items[0].tag.ticks > 600, "{}", last.items[0].tag.ticks);
}

#[test]
fn a_splitter_passes_the_tag_on() {
    let mut f = Factory::default();
    stocked_box(&mut f, v(0, 0, 0), IRON_ORE.into(), 3);
    f.add_belt(v(1, 0, 0), EAST);
    place_block(&mut f, crate::block::SPLITTER, v(2, 0, 0));
    f.add_belt(v(3, 0, 0), EAST);
    run(&mut f, 8.0, |_| {});
    let past = f.belts.iter().find(|b| b.pos == v(3, 0, 0)).unwrap();
    assert!(!past.items.is_empty());
    assert!(past.items.iter().all(|it| it.tag.origin() == Some(v(0, 0, 0)) && it.tag.ticks > 60), "{:?}", past.items);
}

#[test]
fn the_readout_says_how_long_where_from_and_how() {
    let mut f = smelting_line(300);
    run(&mut f, 30.0, |_| {});
    let ingot = f.belt_item_report(aim_at(v(4, 0, 0), 0)).unwrap();
    assert_eq!(ingot.0, crate::item::IRON_INGOT);
    let smelter = block::def(SMELTER).name;
    assert!(ingot.1.contains(" on belts\n"), "{}", ingot.1);
    assert!(ingot.1.contains(&format!("Made in a {smelter}")), "{}", ingot.1);
    assert!(ingot.1.contains("from Iron Ore ×1"), "{}", ingot.1);
    assert!(ingot.1.ends_with("1 west from here"), "{}", ingot.1);

    let ore = f.belt_item_report(aim_at(v(1, 0, 0), 0)).unwrap();
    assert!(ore.1.contains(&format!("Taken out of a {}", block::def(STORAGE).name)), "{}", ore.1);
    assert!(ore.1.ends_with("1 west from here"), "{}", ore.1);

    // The maker is gone, or never known.
    f.remove(v(3, 0, 0));
    let orphan = f.belt_item_report(aim_at(v(4, 0, 0), 0)).unwrap();
    assert!(orphan.1.contains("since removed · 1 west"), "{}", orphan.1);
    f.belts.iter_mut().find(|b| b.pos == v(4, 0, 0)).unwrap().items[0].tag = Tag::UNKNOWN;
    assert!(f.belt_item_report(aim_at(v(4, 0, 0), 0)).unwrap().1.contains("Origin unknown"));
}

#[test]
fn an_item_that_is_gone_has_no_report() {
    let mut f = Factory::default();
    f.add_belt(v(0, 5, 0), EAST);
    let aim = ItemAim { cell: v(0, 5, 0), index: 0, bounds: [0.0; 6] };
    assert!(f.belt_item_report(aim).is_none(), "an empty belt");
    assert!(f.belt_item_report(ItemAim { cell: v(9, 9, 9), ..aim }).is_none(), "no belt at all");
}

#[test]
fn the_crosshair_picks_the_item_under_it_and_a_tall_item_counts_as_part_of_its_belt() {
    let mut f = Factory::default();
    f.add_belt(v(0, 5, 0), EAST);
    let belt = &mut f.belts[0];
    assert!(belt.accept(IRON_ORE.into(), 1, Tag::made_at(v(0, 0, 0)), false, 0.5));
    assert!(belt.accept(IRON_ORE.into(), 1, Tag::made_at(v(7, 0, 0)), false, 0.3));
    // p is 0.5 for the first and 0.15 for the second (spacing): x = 0.5 and 0.15 in the cell.
    let down = Vec3::new(0.0, -1.0, 0.0);
    let hit = f.belt_item_under(v(0, 5, 0), Vec3::new(0.5, 7.0, 0.5), down).expect("the first item");
    assert_eq!(hit.index, 0);
    let second = f.belt_item_under(v(0, 5, 0), Vec3::new(0.15, 7.0, 0.5), down).expect("the second item");
    assert_eq!(second.index, 1);
    assert!(f.belt_item_under(v(0, 5, 0), Vec3::new(0.32, 7.0, 0.5), down).is_none(), "between them");
    assert!(f.belt_item_under(v(0, 5, 0), Vec3::new(0.5, 7.0, 0.95), down).is_none(), "beside them");
    assert!(f.belt_item_under(v(3, 5, 0), Vec3::new(3.5, 7.0, 0.5), down).is_none(), "no belt there");

    // Sideways at the item's upper half, above the rails: only the item stops the ray.
    let side = Vec3::new(0.0, 0.0, 1.0);
    let at = Vec3::new(0.5, 5.0 + f64::from(hit.bounds[1] + hit.bounds[4] * 0.8), -1.0);
    assert!(f.hit_boxes(v(0, 5, 0)).unwrap().enter(at - v(0, 5, 0).as_vec3(), side).is_none(), "the frame misses");
    assert!(f.aim_boxes(v(0, 5, 0)).unwrap().enter(at - v(0, 5, 0).as_vec3(), side).is_some(), "the item is hit");
    // Other machines aim as they hit.
    place_block(&mut f, crate::block::POLE, v(2, 5, 0));
    assert_eq!(f.aim_boxes(v(2, 5, 0)).map(|b| b.bounds()), f.hit_boxes(v(2, 5, 0)).map(|b| b.bounds()));
}

#[test]
fn a_tag_round_trips_and_older_saves_have_none() {
    let tag = Tag { ticks: 123_456, from: v(-40, 17, 9) };
    let mut w = ByteWriter::default();
    tag.write_state(&mut w);
    assert_eq!(w.bytes.len(), 16);
    assert_eq!(Tag::read_state(&mut ByteReader::new(&w.bytes)), Some(tag));
    assert_eq!(Tag::read_state(&mut ByteReader::new(&w.bytes[..15])), None, "cut short");

    let mut old = ByteReader::new(&[]);
    old.version = 55;
    assert_eq!(Tag::read_state(&mut old), Some(Tag::UNKNOWN), "nothing is read before version 56");
    assert_eq!(Tag::UNKNOWN.origin(), None);
    assert_eq!(Tag { ticks: 59, ..Tag::UNKNOWN }.seconds(), 0);
    assert_eq!(Tag { ticks: 120, ..Tag::UNKNOWN }.seconds(), 2);
}

#[test]
fn offsets_read_in_plain_words() {
    assert_eq!(offset_words(v(0, 0, 0)), "right here");
    assert_eq!(offset_words(v(12, 3, 0)), "12 east, 3 up from here");
    assert_eq!(offset_words(v(-1, -2, -3)), "1 west, 3 north, 2 down from here");
}

#[test]
fn belts_and_routers_from_version_55_load_with_unknown_tags() {
    // Version 55's belt: position, facing, shape, tier, then item, uses and progress for each item.
    let mut w = ByteWriter::default();
    w.ivec3(v(1, 2, 3));
    w.u8(1);
    w.u8(0);
    w.u8(0);
    w.count(1);
    w.item(IRON_ORE.into());
    w.u32(1);
    w.f32(0.25);
    let mut r = ByteReader::new(&w.bytes);
    r.version = 55;
    let belt = Belt::read_state(&mut r).expect("an old belt loads");
    assert_eq!((belt.items.len(), belt.items[0].tag), (1, Tag::UNKNOWN));
    assert!(r.bytes(1).is_none(), "nothing left over");

    // And a router: position, facing, kind, filter, held item, its uses, round robin.
    let mut w = ByteWriter::default();
    w.ivec3(v(4, 5, 6));
    w.u8(0);
    w.bool(false);
    w.item(ItemId::NONE);
    w.item(IRON_ORE.into());
    w.u32(1);
    w.u8(2);
    let mut r = ByteReader::new(&w.bytes);
    r.version = 55;
    let router = Router::read_state(&mut r).expect("an old router loads");
    assert_eq!((router.held_n, router.held_tag), (1, Tag::UNKNOWN));
    assert!(r.bytes(1).is_none(), "nothing left over");
}
