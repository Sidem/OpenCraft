use super::*;
use crate::bytes::{ByteReader, ByteWriter};
use crate::item::{IRON_INGOT, IRON_PLATE};

fn round_trip(rules: &FilterRules) -> Option<FilterRules> {
    let mut w = ByteWriter::default();
    rules.write_state(&mut w);
    let mut r = ByteReader::new(&w.bytes);
    let back = FilterRules::read_state(&mut r);
    assert!(r.is_done());
    back
}

#[test]
fn a_new_filter_passes_everything_straight_on() {
    let rules = FilterRules::default();
    assert!(rules.allows(IRON_PLATE, STRAIGHT as usize));
    assert!(!rules.allows(IRON_PLATE, LEFT as usize) && !rules.allows(IRON_PLATE, RIGHT as usize));
}

#[test]
fn an_item_is_picked_for_one_output_at_a_time() {
    let mut rules = FilterRules::default();
    assert!(rules.pick(IRON_PLATE, LEFT));
    assert!(rules.allows(IRON_PLATE, LEFT as usize) && !rules.allows(IRON_PLATE, STRAIGHT as usize));
    assert!(rules.allows(IRON_INGOT, STRAIGHT as usize), "unpicked items go with everything else");
    assert!(rules.pick(IRON_PLATE, RIGHT));
    assert_eq!((rules.picks[LEFT as usize].len(), rules.picks[RIGHT as usize].clone()), (0, vec![IRON_PLATE]));
    assert!(rules.pick(IRON_PLATE, UNPICKED));
    assert!(rules.picks.iter().all(Vec::is_empty) && rules.allows(IRON_PLATE, STRAIGHT as usize));
    assert!(!rules.pick(ItemId::NONE, LEFT) && !rules.pick(IRON_PLATE, 4));
}

#[test]
fn a_list_holds_at_most_max_picks() {
    let mut rules = FilterRules::default();
    let items: Vec<ItemId> = (1..).map(ItemId).filter(|i| i.is_valid()).take(MAX_PICKS + 1).collect();
    for &i in &items[..MAX_PICKS] {
        assert!(rules.pick(i, LEFT));
    }
    assert!(!rules.pick(items[MAX_PICKS], LEFT), "the list is full");
    assert!(rules.pick(items[0], LEFT), "re-picking an item already there is fine");
}

#[test]
fn everything_else_goes_one_way_and_clears_that_ways_picks() {
    let mut rules = FilterRules::default();
    rules.pick(IRON_PLATE, LEFT);
    rules.pick(IRON_INGOT, RIGHT);
    assert!(rules.set_rest(LEFT));
    assert_eq!(rules.rest, 1 << LEFT);
    assert!(rules.picks[LEFT as usize].is_empty() && rules.allows(IRON_PLATE, LEFT as usize));
    assert!(rules.allows(IRON_INGOT, RIGHT as usize) && !rules.allows(IRON_INGOT, LEFT as usize));
    assert!(!rules.set_rest(3), "only three outputs");
    assert!(rules.describe().contains("Left: everything else"));
}

#[test]
fn an_old_filter_sends_its_item_straight_and_the_rest_to_both_sides() {
    let rules = FilterRules::legacy(IRON_PLATE);
    assert!(rules.allows(IRON_PLATE, STRAIGHT as usize) && !rules.allows(IRON_PLATE, LEFT as usize));
    assert!(rules.allows(IRON_INGOT, LEFT as usize) && rules.allows(IRON_INGOT, RIGHT as usize));
    assert!(!rules.allows(IRON_INGOT, STRAIGHT as usize));
    assert!(rules.describe().contains("everything else (shared)"));
    assert_eq!(FilterRules::legacy(ItemId::NONE).picks, <[Vec<ItemId>; 3]>::default());
    assert_eq!(round_trip(&rules), Some(rules));
}

#[test]
fn a_filter_from_save_version_71_loads_with_its_item_straight_on() {
    use crate::factory::provenance::Tag;
    use crate::factory::router::Router;
    use crate::factory::Machine;
    // The layout before version 72: position, facing, filter flag, the one item, then what it holds.
    let mut w = ByteWriter::default();
    w.ivec3(crate::math::IVec3::new(4, 5, 6));
    w.u8(1);
    w.bool(true);
    w.item(IRON_PLATE);
    w.item(ItemId::NONE);
    w.u32(0);
    w.u8(2);
    Tag::UNKNOWN.write_state(&mut w);
    let mut r = ByteReader::new(&w.bytes);
    r.version = 71;
    let old = Router::read_state(&mut r).expect("an old filter loads");
    assert!(r.is_done());
    assert_eq!(old.rules, FilterRules::legacy(IRON_PLATE));
    // Saved again, it keeps those rules.
    let mut again = ByteWriter::default();
    old.write_state(&mut again);
    let back = Router::read_state(&mut ByteReader::new(&again.bytes)).unwrap();
    assert_eq!(back.rules, old.rules);
}

#[test]
fn reading_refuses_rules_that_break_an_invariant() {
    let mut rules = FilterRules::default();
    rules.pick(IRON_PLATE, LEFT);
    rules.pick(IRON_INGOT, RIGHT);
    assert_eq!(round_trip(&rules), Some(rules.clone()));
    let mut w = ByteWriter::default();
    rules.write_state(&mut w);
    let good = w.bytes;
    let read = |bytes: &[u8]| FilterRules::read_state(&mut ByteReader::new(bytes));
    for bad_rest in [0u8, 8, 255] {
        let mut b = good.clone();
        b[0] = bad_rest;
        assert_eq!(read(&b), None, "rest {bad_rest}");
    }
    // The same item on two lists: copy the left list's item over the right's.
    let mut twice = FilterRules::default();
    twice.picks[LEFT as usize].push(IRON_PLATE);
    twice.picks[RIGHT as usize].push(IRON_PLATE);
    let mut w = ByteWriter::default();
    twice.write_state(&mut w);
    assert_eq!(read(&w.bytes), None);
    let mut long = ByteWriter::default();
    long.u8(1);
    long.u8(MAX_PICKS as u8 + 1);
    assert_eq!(read(&long.bytes), None);
    for cut in 0..good.len() {
        assert_eq!(read(&good[..cut]), None, "cut at {cut}");
    }
}
