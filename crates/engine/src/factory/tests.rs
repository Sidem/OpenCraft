use super::belt::{END_STOP, ITEM_SPACING};
use super::describe::fmt_duration;
use super::links::Link;
use super::*;
use crate::block::IRON_ORE;

/// Test-only accessors (inherent methods, so every test module in the crate can use them).
impl Factory {
    pub(crate) fn storage_count_at(&self, pos: IVec3, item: ItemId) -> u32 {
        match self.at.get(&pos) {
            Some(Slot::Storage(i)) => self.storages[*i as usize].buf.count(item),
            _ => 0,
        }
    }

    /// Puts `n` of `item` into the box at `pos`.
    pub(crate) fn stock(&mut self, pos: IVec3, item: ItemId, n: u32) {
        let Some(Slot::Storage(i)) = self.at.get(&pos) else { panic!("no box at {pos:?}") };
        self.storages[*i as usize].buf.add(item, n);
    }

    pub(crate) fn smelter_at(&self, pos: IVec3) -> &Smelter {
        match self.at.get(&pos) {
            Some(Slot::Smelter(i)) => &self.smelters[*i as usize],
            _ => panic!("no smelter at {pos:?}"),
        }
    }

    pub(crate) fn constructor_at(&self, pos: IVec3) -> &Constructor {
        match self.at.get(&pos) {
            Some(Slot::Constructor(i)) => &self.constructors[*i as usize],
            _ => panic!("no constructor at {pos:?}"),
        }
    }

    pub(crate) fn belt_at(&self, pos: IVec3) -> &Belt {
        match self.at.get(&pos) {
            Some(Slot::Belt(i)) => &self.belts[*i as usize],
            _ => panic!("no belt at {pos:?}"),
        }
    }

    pub(crate) fn miner_at(&self, pos: IVec3) -> &Miner {
        match self.at.get(&pos) {
            Some(Slot::Miner(i)) => &self.miners[*i as usize],
            _ => panic!("no miner at {pos:?}"),
        }
    }
}

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

fn stocked_box(f: &mut Factory, pos: IVec3, item: ItemId, n: u32) {
    f.add_storage(pos);
    let Some(Slot::Storage(i)) = f.at.get(&pos) else { unreachable!() };
    f.storages[*i as usize].buf.add(item, n);
}

#[test]
fn box_to_box_through_a_belt_line() {
    let mut f = Factory::default();
    stocked_box(&mut f, IVec3::new(0, 0, 0), IRON_ORE.into(), 10);
    for x in 1..=5 {
        f.add_belt(IVec3::new(x, 0, 0), EAST);
    }
    f.add_storage(IVec3::new(6, 0, 0));
    run(&mut f, 12.0, spacing_ok);
    assert_eq!(f.storage_count_at(IVec3::new(6, 0, 0), IRON_ORE.into()), 10);
    assert_eq!(f.storage_count_at(IVec3::new(0, 0, 0), IRON_ORE.into()), 0);
}

#[test]
fn blocked_line_backs_up_without_overlap() {
    let mut f = Factory::default();
    stocked_box(&mut f, IVec3::new(0, 0, 0), IRON_ORE.into(), 40);
    for x in 1..=3 {
        f.add_belt(IVec3::new(x, 0, 0), EAST);
    }
    run(&mut f, 20.0, spacing_ok);
    let on_belts: usize = f.belts.iter().map(|b| b.items.len()).sum();
    assert!((7..=9).contains(&on_belts), "{on_belts} items on three full belts");
    assert_eq!(f.storage_count_at(IVec3::new(0, 0, 0), IRON_ORE.into()) as usize, 40 - on_belts);
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
    stocked_box(&mut f, IVec3::new(0, 0, -3), IRON_ORE.into(), 12);
    for z in -2..=2 {
        f.add_belt(IVec3::new(0, 0, z), SOUTH);
    }
    stocked_box(&mut f, IVec3::new(-3, 0, 0), crate::block::COAL_ORE.into(), 12);
    for x in -2..=-1 {
        f.add_belt(IVec3::new(x, 0, 0), EAST);
    }
    f.add_storage(IVec3::new(0, 0, 3));
    run(&mut f, 40.0, spacing_ok);
    assert_eq!(f.storage_count_at(IVec3::new(0, 0, 3), IRON_ORE.into()), 12);
    assert_eq!(f.storage_count_at(IVec3::new(0, 0, 3), crate::block::COAL_ORE.into()), 12);
}

#[test]
fn removing_a_belt_returns_its_items_and_relinks() {
    let mut f = Factory::default();
    stocked_box(&mut f, IVec3::new(0, 0, 0), IRON_ORE.into(), 20);
    for x in 1..=4 {
        f.add_belt(IVec3::new(x, 0, 0), EAST);
    }
    run(&mut f, 3.0, |_| {});
    let dropped = f.remove(IVec3::new(1, 0, 0));
    assert!(dropped.iter().map(|s| s.count).sum::<u32>() > 0);
    // The box has no belt leading away any more; the rest of the line still drains forward.
    run(&mut f, 1.0, spacing_ok);
    assert_eq!(f.count(Kind::Belt), 3);
    assert!(f.belt_at(IVec3::new(2, 0, 0)).out != Link::None);
}

#[test]
fn machine_table_rows_follow_kind_order() {
    for (i, def) in MACHINES.iter().enumerate().take(Kind::Router as usize + 1) {
        assert_eq!(def.kind as usize, i, "row {i}");
        assert_eq!(machine(def.block).map(|m| m.kind), Some(def.kind));
    }
    assert!(machine(crate::block::STONE).is_none());
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
    stocked_box(&mut f, pos, COAL_ORE.into(), 5);
    let Some(Slot::Storage(i)) = f.at.get(&pos) else { unreachable!() };
    let buf = &mut f.storages[*i as usize].buf;
    buf.add(IRON_ORE.into(), 130);
    buf.add(COPPER_ORE.into(), 40);
    buf.add(STONE.into(), 1);
    let text = f.describe(pos).unwrap();
    // 130 iron fills three stacks of 64.
    assert_eq!(
        text,
        "6 of 24 slots used\n130 Iron Ore, 40 Copper Ore, 5 Coal Ore, ...\nRight-click to take everything"
    );
}

fn smelter(f: &mut Factory, pos: IVec3) {
    f.place(&mut World::new(1, 2), crate::block::SMELTER, pos, 0, pos - IVec3::new(0, 1, 0));
}

fn smelter_mut(f: &mut Factory, pos: IVec3) -> &mut smelter::Smelter {
    let Some(Slot::Smelter(i)) = f.at.get(&pos) else { unreachable!() };
    &mut f.smelters[*i as usize]
}

#[test]
fn smelter_sorts_what_arrives() {
    use crate::block::{COAL_ORE, LOG, STONE};
    let mut f = Factory::default();
    smelter(&mut f, IVec3::ZERO);
    let s = smelter_mut(&mut f, IVec3::ZERO);
    assert!(s.accept(COAL_ORE.into()) && s.accept(COAL_ORE.into()) && s.accept(IRON_ORE.into()));
    assert!(!s.can_accept(STONE.into()) && !s.accept(STONE.into()));
    assert!(!s.can_accept(crate::item::IRON_INGOT), "ingots are not ore");
    assert_eq!((s.input.total(), s.fuel.total()), (1, 2));
    // One slot each: a log waits until the coal has burned.
    assert!(!s.can_accept(LOG.into()));
}

#[test]
fn one_coal_smelts_five_and_a_third_ingots() {
    let mut f = Factory::default();
    smelter(&mut f, IVec3::ZERO);
    let s = smelter_mut(&mut f, IVec3::ZERO);
    s.input.add(IRON_ORE.into(), 10);
    s.accept(crate::block::COAL_ORE.into());
    run(&mut f, 20.0, |_| {});
    let s = f.smelter_at(IVec3::ZERO);
    assert_eq!(s.out.count(crate::item::IRON_INGOT), 5);
    assert_eq!(s.status, SmelterStatus::NoFuel);
    assert_eq!((s.input.total(), s.batch.is_some(), s.progress), (4, true, 30), "a third of the sixth batch");
    // Taking it apart gives back the half-smelted ore.
    let back = f.remove(IVec3::ZERO);
    let ore: u32 = back.iter().filter(|s| s.item == IRON_ORE.into()).map(|s| s.count).sum();
    assert_eq!(ore, 5);
}

#[test]
fn smelter_waits_while_its_output_is_full() {
    let mut f = Factory::default();
    smelter(&mut f, IVec3::ZERO);
    let s = smelter_mut(&mut f, IVec3::ZERO);
    s.out.add(crate::item::IRON_INGOT, crate::item::MAX_STACK);
    s.input.add(IRON_ORE.into(), 3);
    s.accept(crate::block::COAL_ORE.into());
    run(&mut f, 2.0, |_| {});
    let s = f.smelter_at(IVec3::ZERO);
    assert_eq!((s.status, s.input.total(), s.fuel.total()), (SmelterStatus::OutputFull, 3, 1), "nothing used");
    let mut taken = 0;
    assert!(f.take_contents(IVec3::ZERO, |_, n| {
        taken += n;
        n
    }));
    assert_eq!(taken, crate::item::MAX_STACK);
    run(&mut f, 2.0, |_| {});
    assert_eq!(f.smelter_at(IVec3::ZERO).out.count(crate::item::IRON_INGOT), 1);
}

#[test]
fn ore_and_coal_lines_feed_a_smelter_that_fills_a_box() {
    let mut f = Factory::default();
    stocked_box(&mut f, IVec3::new(0, 0, 0), IRON_ORE.into(), 10);
    f.add_belt(IVec3::new(1, 0, 0), EAST);
    f.add_belt(IVec3::new(2, 0, 0), EAST);
    stocked_box(&mut f, IVec3::new(3, 0, -2), crate::block::COAL_ORE.into(), 2);
    f.add_belt(IVec3::new(3, 0, -1), SOUTH);
    smelter(&mut f, IVec3::new(3, 0, 0));
    f.add_belt(IVec3::new(4, 0, 0), EAST);
    f.add_storage(IVec3::new(5, 0, 0));
    run(&mut f, 25.0, spacing_ok);
    assert_eq!(f.storage_count_at(IVec3::new(5, 0, 0), crate::item::IRON_INGOT), 10);
    assert_eq!(f.smelter_at(IVec3::new(3, 0, 0)).status, SmelterStatus::NoOre);
}

/// The index of the machine recipe that makes `item`.
fn recipe_for(item: ItemId) -> u16 {
    crate::recipes::MACHINE_RECIPES.iter().position(|r| r.output.0 == item).unwrap() as u16
}

/// Power for machines near the origin: a pole at (2, 3, 0) and a generator beside it with a stack of
/// coal (once per factory).
fn powered(f: &mut Factory) {
    let (pole, gen) = (IVec3::new(2, 3, 0), IVec3::new(3, 3, 0));
    if !f.at.contains_key(&pole) {
        f.place(&mut World::new(1, 2), crate::block::POLE, pole, 0, pole);
        f.place(&mut World::new(1, 2), crate::block::GENERATOR, gen, 0, gen);
        assert_eq!(f.insert(gen, crate::block::COAL_ORE.into(), 64), 64);
    }
}

fn constructor(f: &mut Factory, pos: IVec3, makes: ItemId) {
    powered(f);
    f.place(&mut World::new(1, 2), crate::block::CONSTRUCTOR, pos, 0, pos - IVec3::new(0, 1, 0));
    assert!(f.set_recipe(pos, Some(recipe_for(makes))).is_some_and(|back| back.is_empty()));
}

#[test]
fn ingots_through_a_constructor_become_plates_in_a_box() {
    use crate::item::{IRON_INGOT, IRON_PLATE};
    let mut f = Factory::default();
    stocked_box(&mut f, IVec3::new(0, 0, 0), IRON_INGOT, 11);
    f.add_belt(IVec3::new(1, 0, 0), EAST);
    constructor(&mut f, IVec3::new(2, 0, 0), IRON_PLATE);
    f.add_belt(IVec3::new(3, 0, 0), EAST);
    f.add_storage(IVec3::new(4, 0, 0));
    // Five plates at 2 s each; the belt brings an ingot every 0.35 s, so the constructor sets the pace.
    run(&mut f, 14.0, spacing_ok);
    assert_eq!(f.storage_count_at(IVec3::new(4, 0, 0), IRON_PLATE), 5);
    let c = f.constructor_at(IVec3::new(2, 0, 0));
    assert_eq!((c.input.total(), c.status), (1, ConstructorStatus::NoInput), "the odd ingot waits");
}

#[test]
fn a_constructor_takes_only_its_recipe_input() {
    use crate::item::{IRON_INGOT, IRON_PLATE, IRON_ROD, SCREW};
    let mut f = Factory::default();
    let p = IVec3::ZERO;
    powered(&mut f);
    f.place(&mut World::new(1, 2), crate::block::CONSTRUCTOR, p, 0, p);
    assert_eq!(f.insert(p, IRON_INGOT, 5), 0, "no recipe, nothing goes in");
    assert_eq!(f.set_recipe(p, Some(0)), None, "a smelter recipe is refused");
    assert!(f.set_recipe(p, Some(recipe_for(IRON_ROD))).is_some());
    assert_eq!(f.set_recipe(p, Some(recipe_for(IRON_ROD))), None, "no change");
    assert_eq!(f.insert(p, IRON_PLATE, 5), 0);
    assert_eq!(f.insert(p, IRON_INGOT, 100), 64, "one stack");
    assert!(!f.wants(p, IRON_INGOT));
    run(&mut f, 3.0, |_| {});
    // One rod done at 2 s, the second batch a half done: switching hands back its ingot too.
    let back = f.set_recipe(p, Some(recipe_for(SCREW))).unwrap();
    let ingots: u32 = back.iter().filter(|s| s.item == IRON_INGOT).map(|s| s.count).sum();
    assert_eq!(ingots, 63);
    let c = f.constructor_at(p);
    assert_eq!((c.out.count(IRON_ROD), c.input.total(), c.busy), (1, 0, false));
}

#[test]
fn screws_come_four_to_a_rod() {
    use crate::item::{IRON_ROD, SCREW};
    let mut f = Factory::default();
    constructor(&mut f, IVec3::ZERO, SCREW);
    assert_eq!(f.insert(IVec3::ZERO, IRON_ROD, 2), 2);
    run(&mut f, 6.5, |_| {});
    assert_eq!(f.constructor_at(IVec3::ZERO).out.count(SCREW), 8);
}

#[test]
fn a_smelter_panel_takes_ore_and_fuel_by_hand() {
    use crate::block::{COAL_ORE, STONE};
    let mut f = Factory::default();
    smelter(&mut f, IVec3::ZERO);
    assert_eq!(f.insert(IVec3::ZERO, COAL_ORE.into(), 10), 10);
    assert_eq!(f.insert(IVec3::ZERO, IRON_ORE.into(), 70), 64);
    assert_eq!(f.insert(IVec3::ZERO, STONE.into(), 5), 0);
    run(&mut f, 1.6, |_| {});
    let p = f.panel(IVec3::ZERO).unwrap();
    assert_eq!(p.slots.iter().map(|s| (s.0, s.1.count)).collect::<Vec<_>>(), [(0, 62), (1, 9), (2, 1)]);
    assert!(p.status.starts_with("Smelting Iron Ingot"), "{}", p.status);
    assert_eq!(p.fire, 7, "8 s of coal, 1.6 s burned");
    assert!(f.panel(IVec3::new(9, 9, 9)).is_none());
}

#[test]
fn constructors_survive_a_save_round_trip() {
    use crate::bytes::{ByteReader, ByteWriter};
    use crate::item::{IRON_INGOT, IRON_PLATE};
    let mut f = Factory::default();
    constructor(&mut f, IVec3::new(2, 0, 0), IRON_PLATE);
    f.insert(IVec3::new(2, 0, 0), IRON_INGOT, 9);
    run(&mut f, 3.0, |_| {});
    let mut w = ByteWriter::default();
    f.write_state(&mut w);
    let mut g = Factory::read_state(&mut World::new(1, 2), &mut ByteReader::new(&w.bytes)).unwrap();
    let mut again = ByteWriter::default();
    g.write_state(&mut again);
    assert!(again.bytes == w.bytes);
    let c = g.constructor_at(IVec3::new(2, 0, 0));
    assert_eq!((c.recipe, c.busy, c.out.count(IRON_PLATE)), (Some(recipe_for(IRON_PLATE)), true, 1));
    run(&mut g, 10.0, |_| {});
    assert_eq!(g.constructor_at(IVec3::new(2, 0, 0)).out.count(IRON_PLATE), 4);
}

fn router(f: &mut Factory, pos: IVec3, dir: u8, filter: bool) {
    powered(f);
    let block = if filter { crate::block::FILTER } else { crate::block::SPLITTER };
    f.place(&mut World::new(1, 2), block, pos, dir, pos);
}

/// A belt from `from` running `dir` for one cell into a box beyond it.
fn belt_to_box(f: &mut Factory, from: IVec3, dir: u8) -> IVec3 {
    f.add_belt(from, dir);
    let chest = from + DIRS[dir as usize];
    f.add_storage(chest);
    chest
}

#[test]
fn a_splitter_shares_items_round_robin_and_skips_a_blocked_output() {
    const NORTH: u8 = 0;
    let mut f = Factory::default();
    stocked_box(&mut f, IVec3::new(0, 0, 0), IRON_ORE.into(), 30);
    f.add_belt(IVec3::new(1, 0, 0), EAST);
    router(&mut f, IVec3::new(2, 0, 0), EAST, false);
    let front = belt_to_box(&mut f, IVec3::new(3, 0, 0), EAST);
    let left = belt_to_box(&mut f, IVec3::new(2, 0, -1), NORTH);
    // The right-hand belt leads nowhere, so it fills up and blocks.
    f.add_belt(IVec3::new(2, 0, 1), SOUTH);
    run(&mut f, 40.0, spacing_ok);
    let stuck = f.belt_at(IVec3::new(2, 0, 1)).items.len() as u32;
    let (a, b) = (f.storage_count_at(front, IRON_ORE.into()), f.storage_count_at(left, IRON_ORE.into()));
    assert!((2..=3).contains(&stuck), "{stuck} waiting on the blocked belt");
    assert_eq!(a + b + stuck, 30, "nothing lost");
    assert!(a.abs_diff(b) <= 1, "{a} and {b} should be even");
}

#[test]
fn a_filter_sends_its_item_straight_on_and_the_rest_aside() {
    use crate::block::COAL_ORE;
    const NORTH: u8 = 0;
    let mut f = Factory::default();
    stocked_box(&mut f, IVec3::new(0, 0, 0), IRON_ORE.into(), 10);
    f.stock(IVec3::new(0, 0, 0), COAL_ORE.into(), 10);
    f.add_belt(IVec3::new(1, 0, 0), EAST);
    router(&mut f, IVec3::new(2, 0, 0), EAST, true);
    f.set_filter(IVec3::new(2, 0, 0), IRON_ORE.into());
    let front = belt_to_box(&mut f, IVec3::new(3, 0, 0), EAST);
    let left = belt_to_box(&mut f, IVec3::new(2, 0, -1), NORTH);
    let right = belt_to_box(&mut f, IVec3::new(2, 0, 1), SOUTH);
    run(&mut f, 10.0, spacing_ok);
    // Mid-run, the routers' bytes read back unchanged.
    let mut w = crate::bytes::ByteWriter::default();
    f.write_state(&mut w);
    let mut g = Factory::read_state(&mut World::new(1, 2), &mut crate::bytes::ByteReader::new(&w.bytes)).unwrap();
    let mut again = crate::bytes::ByteWriter::default();
    g.write_state(&mut again);
    assert!(again.bytes == w.bytes);
    run(&mut g, 20.0, spacing_ok);
    let count = |pos, item: crate::block::BlockId| g.storage_count_at(pos, item.into());
    assert_eq!((count(front, IRON_ORE), count(front, COAL_ORE)), (10, 0));
    assert_eq!((count(left, IRON_ORE), count(right, IRON_ORE)), (0, 0));
    assert_eq!(count(left, COAL_ORE) + count(right, COAL_ORE), 10);
    assert!(count(left, COAL_ORE).abs_diff(count(right, COAL_ORE)) <= 1);
}

#[test]
fn a_filter_with_no_item_sends_everything_aside() {
    let mut f = Factory::default();
    stocked_box(&mut f, IVec3::new(0, 0, 0), IRON_ORE.into(), 6);
    f.add_belt(IVec3::new(1, 0, 0), EAST);
    router(&mut f, IVec3::new(2, 0, 0), EAST, true);
    let front = belt_to_box(&mut f, IVec3::new(3, 0, 0), EAST);
    let right = belt_to_box(&mut f, IVec3::new(2, 0, 1), SOUTH);
    run(&mut f, 15.0, spacing_ok);
    assert_eq!((f.storage_count_at(front, IRON_ORE.into()), f.storage_count_at(right, IRON_ORE.into())), (0, 6));
    assert!(f.panel(IVec3::new(2, 0, 0)).is_some(), "a filter has a panel");
    router(&mut f, IVec3::new(5, 0, 5), EAST, false);
    assert!(f.panel(IVec3::new(5, 0, 5)).is_none(), "a splitter has none");
}

/// Places belt block `block` (any shape) at `pos` facing `dir`.
fn shaped(f: &mut Factory, block: BlockId, pos: IVec3, dir: u8) {
    f.place(&mut World::new(1, 2), block, pos, dir, pos);
}

/// The factory after its bytes are written and read back, checking they read back unchanged.
fn round_trip(f: &Factory) -> Factory {
    let mut w = crate::bytes::ByteWriter::default();
    f.write_state(&mut w);
    let g = Factory::read_state(&mut World::new(1, 2), &mut crate::bytes::ByteReader::new(&w.bytes)).unwrap();
    let mut again = crate::bytes::ByteWriter::default();
    g.write_state(&mut again);
    assert!(again.bytes == w.bytes);
    g
}

#[test]
fn items_climb_a_ramp_and_come_back_down() {
    use crate::block::{RAMP_DOWN, RAMP_UP};
    let mut f = Factory::default();
    stocked_box(&mut f, IVec3::new(0, 0, 0), IRON_ORE.into(), 8);
    f.add_belt(IVec3::new(1, 0, 0), EAST);
    shaped(&mut f, RAMP_UP, IVec3::new(2, 0, 0), EAST);
    f.add_belt(IVec3::new(3, 1, 0), EAST);
    shaped(&mut f, RAMP_DOWN, IVec3::new(4, 0, 0), EAST);
    f.add_storage(IVec3::new(5, 0, 0));
    run(&mut f, 4.0, spacing_ok);
    assert!(matches!(f.belt_at(IVec3::new(2, 0, 0)).out, Link::Belt { mid: false, .. }), "up onto the high belt");
    assert!(matches!(f.belt_at(IVec3::new(3, 1, 0)).out, Link::Belt { mid: false, .. }), "down the ramp");
    let ramp = f.belt_at(IVec3::new(2, 0, 0));
    assert_eq!((ramp.item_at(0.0).1, ramp.item_at(1.0).1), (0.0, 1.0));
    let mut g = round_trip(&f);
    run(&mut g, 15.0, spacing_ok);
    assert_eq!(g.storage_count_at(IVec3::new(5, 0, 0), IRON_ORE.into()), 8);
}

#[test]
fn a_stack_of_lifts_carries_items_up_and_hands_them_on_ahead() {
    use crate::block::LIFT;
    let mut f = Factory::default();
    stocked_box(&mut f, IVec3::new(0, 0, 0), IRON_ORE.into(), 6);
    f.add_belt(IVec3::new(1, 0, 0), EAST);
    for y in 0..3 {
        shaped(&mut f, LIFT, IVec3::new(2, y, 0), EAST);
    }
    f.add_storage(IVec3::new(3, 3, 0));
    run(&mut f, 20.0, spacing_ok);
    assert_eq!(f.storage_count_at(IVec3::new(3, 3, 0), IRON_ORE.into()), 6);
    let (bottom, top) = (f.belt_at(IVec3::new(2, 0, 0)), f.belt_at(IVec3::new(2, 2, 0)));
    assert_eq!((bottom.lift_below, bottom.lift_above, top.lift_below, top.lift_above), (false, true, true, false));
    assert_eq!(top.item_at(1.0), (0.5, 1.0, 0.0), "the top lift hands on ahead and up");
}

#[test]
fn an_underpass_carries_items_under_a_crossing_belt() {
    use crate::block::{COAL_ORE, UNDERPASS_IN, UNDERPASS_OUT};
    let mut f = Factory::default();
    stocked_box(&mut f, IVec3::new(0, 0, 0), IRON_ORE.into(), 6);
    f.add_belt(IVec3::new(1, 0, 0), EAST);
    shaped(&mut f, UNDERPASS_IN, IVec3::new(2, 0, 0), EAST);
    // A coal line running south crosses at x = 3.
    stocked_box(&mut f, IVec3::new(3, 0, -3), COAL_ORE.into(), 6);
    for z in -2..=2 {
        f.add_belt(IVec3::new(3, 0, z), SOUTH);
    }
    f.add_storage(IVec3::new(3, 0, 3));
    shaped(&mut f, UNDERPASS_OUT, IVec3::new(5, 0, 0), EAST);
    f.add_storage(IVec3::new(6, 0, 0));
    // A machine doesn't feed an exit, and the exit's back doesn't take from the line beside it.
    stocked_box(&mut f, IVec3::new(4, 0, 0), COAL_ORE.into(), 3);
    run(&mut f, 3.0, spacing_ok);
    let mut g = round_trip(&f);
    run(&mut g, 20.0, spacing_ok);
    let count = |pos, item: BlockId| g.storage_count_at(pos, item.into());
    assert_eq!((count(IVec3::new(6, 0, 0), IRON_ORE), count(IVec3::new(6, 0, 0), COAL_ORE)), (6, 0));
    assert_eq!((count(IVec3::new(3, 0, 3), COAL_ORE), count(IVec3::new(3, 0, 3), IRON_ORE)), (6, 0));
    assert_eq!(count(IVec3::new(4, 0, 0), COAL_ORE), 3);
}

/// A constructor at `pos` making rods, with `ingots` iron ingots put in.
fn rod_maker(f: &mut Factory, pos: IVec3, ingots: u32) {
    f.place(&mut World::new(1, 2), crate::block::CONSTRUCTOR, pos, 0, pos);
    f.set_recipe(pos, Some(recipe_for(crate::item::IRON_ROD)));
    assert_eq!(f.insert(pos, crate::item::IRON_INGOT, ingots), ingots);
}

fn place_block(f: &mut Factory, block: BlockId, pos: IVec3) {
    f.place(&mut World::new(1, 2), block, pos, 0, pos);
}

#[test]
fn poles_in_range_form_one_grid_and_machines_hang_on_the_nearest() {
    use crate::block::POLE;
    let mut f = Factory::default();
    for x in [0, 10, 20, 31] {
        place_block(&mut f, POLE, IVec3::new(x, 0, 0));
    }
    rod_maker(&mut f, IVec3::new(29, 0, 2), 0);
    rod_maker(&mut f, IVec3::new(50, 0, 0), 0);
    run(&mut f, 0.1, |_| {});
    let p = &f.power;
    assert_eq!(p.pole_grid, [0, 0, 0, 1], "10 blocks apart link; 11 don't");
    assert_eq!(p.wires, [(0, 1), (1, 2)]);
    assert_eq!(p.constructor_pole, [Some(3), None], "nearest pole within reach, or none");
    // Removing the middle pole splits the grid.
    f.remove(IVec3::new(10, 0, 0));
    run(&mut f, 0.1, |_| {});
    assert_eq!(f.power.pole_grid.len(), 3);
    assert!(f.power.wires.is_empty());
}

#[test]
fn a_brownout_slows_every_machine_on_the_grid() {
    use crate::block::{COAL_ORE, GENERATOR, POLE};
    use crate::item::IRON_ROD;
    let rods_after = |makers: i32, seconds: f64| {
        let mut f = Factory::default();
        place_block(&mut f, POLE, IVec3::new(0, 3, 0));
        place_block(&mut f, GENERATOR, IVec3::new(0, 4, 0));
        f.insert(IVec3::new(0, 4, 0), COAL_ORE.into(), 10);
        for x in 0..makers {
            rod_maker(&mut f, IVec3::new(x - 3, 0, 0), 20);
        }
        run(&mut f, seconds, |_| {});
        let speed = f.power.speed(f.power.constructor_pole[0]);
        (f.constructor_at(IVec3::new(-3, 0, 0)).out.count(IRON_ROD), speed, f.power.demand[0])
    };
    // One generator (60 kW) runs four constructors (15 kW each) at full speed: a rod per 2 s.
    assert_eq!(rods_after(4, 8.05), (4, 1000, 60));
    // Eight need 120 kW: half speed, half the rods.
    assert_eq!(rods_after(8, 8.05), (2, 500, 120));
}

#[test]
fn generators_burn_only_what_their_grid_needs() {
    use crate::block::{COAL_ORE, GENERATOR, POLE};
    let mut f = Factory::default();
    place_block(&mut f, POLE, IVec3::new(0, 3, 0));
    for x in [0, 1] {
        place_block(&mut f, GENERATOR, IVec3::new(x, 4, 0));
        f.insert(IVec3::new(x, 4, 0), COAL_ORE.into(), 2);
    }
    run(&mut f, 5.0, |_| {});
    let fuel = |f: &Factory, x| f.generators.iter().find(|g| g.pos.x == x).unwrap().fuel.total();
    assert_eq!((fuel(&f, 0), fuel(&f, 1)), (2, 2), "no demand, no burning");
    // One constructor needs 15 kW: the first generator covers it; the second stays cold.
    rod_maker(&mut f, IVec3::new(0, 0, 0), 30);
    let mut f = round_trip(&f);
    run(&mut f, 17.0, |_| {});
    assert_eq!((fuel(&f, 0), fuel(&f, 1)), (0, 1), "two coal burned in 16 s, then the second lit");
    assert!(f.generators[1].running && !f.generators[0].running);
    run(&mut f, 16.0, |_| {});
    let c = f.constructor_at(IVec3::ZERO);
    assert_eq!((c.status, c.out.count(crate::item::IRON_ROD)), (ConstructorStatus::NoPower, 16), "32 s of fuel");
}

/// Test-only lab accessor.
impl Factory {
    pub(crate) fn lab_at(&self, pos: IVec3) -> &Lab {
        match self.at.get(&pos) {
            Some(Slot::Lab(i)) => &self.labs[*i as usize],
            _ => panic!("no lab at {pos:?}"),
        }
    }
}

/// A lab at `pos` holding `red` red packs.
fn lab_with(f: &mut Factory, pos: IVec3, red: u32) {
    place_block(f, crate::block::LAB, pos);
    assert_eq!(f.insert(pos, crate::item::RED_PACK, red), red);
}

#[test]
fn labs_research_the_chosen_tech_and_never_overshoot() {
    use crate::research::TechState;
    use lab::LabStatus;
    let mut f = Factory::default();
    powered(&mut f);
    let labs = [IVec3::new(0, 0, 0), IVec3::new(1, 0, 0), IVec3::new(3, 0, 0)];
    for pos in labs {
        lab_with(&mut f, pos, 10);
    }
    let packs = |f: &Factory| labs.iter().map(|&p| f.lab_at(p).packs.total()).sum::<u32>();
    run(&mut f, 1.0, |_| {});
    assert_eq!((f.lab_at(labs[0]).status, packs(&f), f.power.demand[0]), (LabStatus::NoResearch, 30, 0));
    // Belt Routing: 10 units of 5 s. Three labs finish three units at a time.
    f.research.set_current(Some(0));
    run(&mut f, 5.05, |_| {});
    assert_eq!((f.research.progress(0), packs(&f), f.power.demand[0]), (3, 24, 30), "three done, three started");
    let mut f = round_trip(&f);
    run(&mut f, 15.0, |f| assert!(f.research.progress(0) <= 10));
    assert_eq!(f.research.state(0), TechState::Done);
    assert_eq!((f.research.current, packs(&f)), (None, 20), "exactly one pack per unit");
    assert!(labs.iter().all(|&p| f.lab_at(p).status == LabStatus::NoResearch && f.lab_at(p).unit.is_none()));
    assert_eq!(f.research.locked_by(crate::block::SPLITTER.into()), None);
}

#[test]
fn a_lab_needs_one_of_each_pack_and_power() {
    use crate::item::{GREEN_PACK, RED_PACK};
    use lab::LabStatus;
    let mut f = Factory::default();
    for (tech, units) in [(0, 10), (2, 30)] {
        (0..units).for_each(|_| f.research.add_unit(tech));
    }
    f.research.set_current(Some(3)); // Underpasses: red and green, 10 s a unit
    powered(&mut f);
    lab_with(&mut f, IVec3::ZERO, 5);
    lab_with(&mut f, IVec3::new(20, 0, 0), 5); // out of reach of the pole
    assert_eq!(f.insert(IVec3::new(20, 0, 0), GREEN_PACK, 5), 5);
    assert!(!f.wants(IVec3::ZERO, crate::item::IRON_PLATE), "packs only");
    run(&mut f, 2.0, |_| {});
    assert_eq!(f.lab_at(IVec3::ZERO).status, LabStatus::NoPacks);
    assert_eq!(f.lab_at(IVec3::new(20, 0, 0)).status, LabStatus::NoPower);
    assert_eq!(f.lab_at(IVec3::new(20, 0, 0)).packs.total(), 10, "nothing used without power");
    // A belt brings the green packs.
    stocked_box(&mut f, IVec3::new(-2, 0, 0), GREEN_PACK, 2);
    f.add_belt(IVec3::new(-1, 0, 0), EAST);
    run(&mut f, 22.0, |_| {});
    let lab = f.lab_at(IVec3::ZERO);
    assert_eq!((f.research.progress(3), lab.packs.count(RED_PACK), lab.packs.count(GREEN_PACK)), (2, 3, 0));
    assert_eq!(lab.status, LabStatus::NoPacks);
    assert_eq!(f.remove(IVec3::ZERO), vec![crate::inventory::Stack { item: RED_PACK, count: 3 }]);
}

#[test]
fn fast_belts_carry_twice_as_much_and_mix_with_slow_ones() {
    use crate::block::FAST_BELT;
    let carried = |fast_cells: &[i32]| {
        let mut f = Factory::default();
        stocked_box(&mut f, IVec3::new(0, 0, 0), IRON_ORE.into(), 64);
        for x in 1..=4 {
            if fast_cells.contains(&x) {
                shaped(&mut f, FAST_BELT, IVec3::new(x, 0, 0), EAST);
            } else {
                f.add_belt(IVec3::new(x, 0, 0), EAST);
            }
        }
        f.add_storage(IVec3::new(5, 0, 0));
        run(&mut f, 5.0, spacing_ok);
        let mut g = round_trip(&f);
        run(&mut g, 5.0, spacing_ok);
        assert_eq!(g.belt_at(IVec3::new(1, 0, 0)).fast, fast_cells.contains(&1), "saved");
        g.storage_count_at(IVec3::new(5, 0, 0), IRON_ORE.into())
    };
    let (slow, fast) = (carried(&[]), carried(&[1, 2, 3, 4]));
    // Items take 4 s (slow) or 2 s (fast) to reach the box, then arrive at up to 2.9 or 5.7 a second.
    assert!(slow >= 15 && fast >= 2 * slow, "slow {slow}, fast {fast} in 10 s");
    // A slow belt in the line limits it to slow throughput (it only arrives sooner), keeping spacing.
    let mixed = carried(&[1, 2, 4]);
    assert!(mixed > slow && mixed + 10 < fast, "mixed {mixed}");
}
