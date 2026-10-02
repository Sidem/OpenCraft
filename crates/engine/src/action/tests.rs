use super::*;
use crate::block::{AIR, BEDROCK, BELT, DIRT, STONE, STORAGE, WATER};
use crate::bytes::{ByteReader, ByteWriter};
use crate::factory::Kind;
use crate::inventory::{HOTBAR_SLOTS, INVENTORY_SLOTS};
use crate::item::{IRON_PLATE, IRON_ROD, MAX_STACK};
use crate::recipes::RECIPES;

const P: PlayerId = PlayerId(0);
/// The box as an item, for matching events.
const BOX_ITEM: ItemId = ItemId::block(STORAGE);

fn inv(sim: &Sim) -> &crate::inventory::Inventory {
    &sim.player(P).unwrap().inventory
}

#[test]
fn actions_apply_at_their_tick_in_queue_order() {
    let mut sim = Sim::new(7, 2);
    sim.queue(2, P, Action::Give { item: STONE.into(), count: 5 });
    sim.queue(0, P, Action::Give { item: STONE.into(), count: 10 });
    // Pick the stack up with the cursor, then put it down in slot 5: only right in this order.
    sim.queue(0, P, Action::ClickSlot { slot: 0, shift: false });
    sim.queue(0, P, Action::ClickSlot { slot: 5, shift: false });
    sim.step();
    assert_eq!((inv(&sim).slots[0].count, inv(&sim).slots[5].count), (0, 10));
    sim.step();
    assert_eq!(inv(&sim).count(STONE.into()), 10, "the tick 2 action waits");
    sim.step();
    assert_eq!(inv(&sim).count(STONE.into()), 15);

    // An action for a tick already run goes into the next one.
    sim.queue(0, P, Action::SelectSlot { slot: 4 });
    sim.step();
    assert_eq!(inv(&sim).selected, 4);
}

#[test]
fn stale_actions_do_nothing() {
    let mut sim = Sim::new(7, 2);
    let air = IVec3::new(0, 200, 0);
    sim.apply(P, Action::BreakBlock { pos: air });
    sim.apply(P, Action::BreakBlock { pos: IVec3::new(0, 0, 0) });
    assert_eq!(sim.world.block_anywhere_or_generate(IVec3::new(0, 0, 0)), BEDROCK, "unbreakable");
    sim.apply(P, Action::PlaceBlock { pos: air, slot: 0, facing: 0, against: air });
    sim.apply(P, Action::DropSelected { count: 1 });
    sim.apply(P, Action::Craft { recipe: 0, times: 3 });
    assert!(sim.events.is_empty(), "{:?}", sim.events);
    assert_eq!(sim.world.block_anywhere_or_generate(air), AIR);
}

#[test]
fn place_and_break_work_where_no_chunk_is_loaded() {
    let mut sim = Sim::new(7, 2);
    let pos = IVec3::new(500, 200, -500);
    sim.apply(P, Action::Give { item: STORAGE.into(), count: 1 });
    sim.apply(P, Action::PlaceBlock { pos, slot: 0, facing: 0, against: pos - IVec3::new(0, 1, 0) });
    assert_eq!(sim.world.block_anywhere(pos), Some(STORAGE));
    assert_eq!(sim.factory.count(Kind::Storage), 1);
    assert_eq!(inv(&sim).count(STORAGE.into()), 0);
    assert_eq!(sim.events, vec![SimEvent::BlockPlaced { player: P, pos, block: STORAGE }]);

    sim.events.clear();
    sim.apply(P, Action::BreakBlock { pos });
    assert_eq!(sim.world.block_anywhere(pos), Some(AIR));
    assert_eq!(sim.factory.count(Kind::Storage), 0);
    assert!(matches!(
        sim.events[..],
        [SimEvent::BlockBroken { block: STORAGE, .. }, SimEvent::Dropped { item: BOX_ITEM, count: 1, .. }]
    ));
}

#[test]
fn a_mk2_places_its_family_block_and_breaks_back_into_a_mk2() {
    use crate::block::{FAST_BELT, MINER, MINER_MK2};
    for (item, block) in [(FAST_BELT, BELT), (MINER_MK2, MINER), (BELT, BELT)] {
        let mut sim = Sim::new(7, 2);
        let pos = IVec3::new(500, 200, -500);
        sim.apply(P, Action::Give { item: item.into(), count: 1 });
        sim.apply(P, Action::PlaceBlock { pos, slot: 0, facing: 0, against: pos - IVec3::new(0, 1, 0) });
        assert_eq!(sim.world.block_anywhere(pos), Some(block));
        assert_eq!(sim.factory.tiered_at(pos), Some((block, (item != BELT) as u8)));
        sim.events.clear();
        sim.apply(P, Action::BreakBlock { pos });
        let dropped = sim.events.iter().find_map(|e| match e {
            SimEvent::Dropped { item, .. } => Some(*item),
            _ => None,
        });
        assert_eq!(dropped, Some(ItemId::block(item)));
    }
}

#[test]
fn kits_upgrade_belts_until_they_run_out_once_research_allows() {
    use crate::item::GREEN_KIT;
    let mut sim = Sim::new(7, 2);
    let at = |x: i32| IVec3::new(500 + x, 200, -500);
    (0..12).for_each(|x| sim.factory.add_belt(at(x), 1));
    sim.apply(P, Action::Give { item: GREEN_KIT, count: 9 });
    let upgrade_all = |sim: &mut Sim| (0..12).for_each(|x| sim.apply(P, Action::Upgrade { pos: at(x) }));
    upgrade_all(&mut sim);
    assert_eq!(inv(&sim).count(GREEN_KIT), 9, "Belt Mk2 isn't researched yet");
    for tech in [0, 2, 4, 5] {
        (0..crate::research::TECHS[tech as usize].units).for_each(|_| sim.factory.research.add_unit(tech));
    }
    upgrade_all(&mut sim);
    let tiers: Vec<u8> = (0..12).map(|x| sim.factory.tiered_at(at(x)).unwrap().1).collect();
    assert_eq!(tiers, [1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0], "9 kits upgrade 9 of 12");
    assert_eq!(inv(&sim).count(GREEN_KIT), 0);
    // Breaking an upgraded belt gives the Mk2 item back, never the kit.
    sim.events.clear();
    sim.apply(P, Action::BreakBlock { pos: at(0) });
    assert!(!sim.events.iter().any(|e| matches!(e, SimEvent::Dropped { item: GREEN_KIT, .. })));
}

#[test]
fn blocks_and_machines_placed_in_water_replace_it() {
    let mut sim = Sim::new(7, 3);
    let (a, b) = (IVec3::new(500, 200, -500), IVec3::new(501, 200, -500));
    for p in [a, b] {
        sim.world.set_block_anywhere(p, WATER);
    }
    sim.apply(P, Action::Give { item: STONE.into(), count: 1 });
    sim.apply(P, Action::PlaceBlock { pos: a, slot: 0, facing: 0, against: a - IVec3::new(0, 1, 0) });
    sim.apply(P, Action::Give { item: STORAGE.into(), count: 1 });
    sim.apply(P, Action::PlaceBlock { pos: b, slot: 0, facing: 0, against: b - IVec3::new(0, 1, 0) });
    assert_eq!((sim.world.block_anywhere(a), sim.world.block_anywhere(b)), (Some(STONE), Some(STORAGE)));
    assert_eq!(sim.factory.count(Kind::Storage), 1);
    assert!(inv(&sim).slots.iter().all(|s| s.is_empty()));
}

#[test]
fn two_players_build_and_craft_with_their_own_inventories() {
    let mut sim = Sim::new(7, 2);
    let b = PlayerId(1);
    let belt = RECIPES.iter().position(|r| r.output == BELT.into()).unwrap() as u16;
    sim.queue(0, b, Action::Join { key: 0 });
    sim.queue(0, b, Action::Give { item: IRON_PLATE, count: 1 });
    sim.queue(0, b, Action::Give { item: IRON_ROD, count: 1 });
    sim.queue(0, b, Action::Craft { recipe: belt, times: 1 });
    sim.queue(0, P, Action::Give { item: STORAGE.into(), count: 1 });
    sim.queue(0, P, Action::Craft { recipe: belt, times: 1 });
    sim.step();
    assert!(sim.events.is_empty(), "a belt takes a moment by hand");
    for _ in 0..RECIPES[belt as usize].hand_ticks() {
        sim.step();
    }
    assert_eq!(sim.events, vec![SimEvent::Crafted { player: b, item: BELT.into(), count: 4 }], "only B can pay");
    let inv_b = &sim.player(b).unwrap().inventory;
    assert_eq!((inv_b.count(BELT.into()), inv_b.count(IRON_PLATE), inv_b.count(IRON_ROD)), (4, 0, 0));
    assert_eq!((inv(&sim).count(BELT.into()), inv(&sim).count(STORAGE.into())), (0, 1));

    // Same tick: A places a box, B places a belt and breaks A's box. The player order decides.
    sim.events.clear();
    let (box_at, belt_at) = (IVec3::new(100, 200, 0), IVec3::new(104, 200, 0));
    sim.queue(1, b, Action::PlaceBlock { pos: belt_at, slot: 0, facing: 1, against: belt_at });
    sim.queue(1, b, Action::BreakBlock { pos: box_at });
    sim.queue(1, P, Action::PlaceBlock { pos: box_at, slot: 0, facing: 0, against: box_at });
    sim.step();
    assert!(
        matches!(
            sim.events[..],
            [
                SimEvent::BlockPlaced { player: P, block: STORAGE, .. },
                SimEvent::BlockPlaced { player: PlayerId(1), block: BELT, .. },
                SimEvent::BlockBroken { player: PlayerId(1), block: STORAGE, .. },
                SimEvent::Dropped { item: BOX_ITEM, count: 1, .. },
            ]
        ),
        "{:?}",
        sim.events
    );
    assert_eq!((sim.world.block_anywhere(box_at), sim.world.block_anywhere(belt_at)), (Some(AIR), Some(BELT)));
    assert_eq!((sim.factory.count(Kind::Storage), sim.factory.count(Kind::Belt)), (0, 1));
    assert_eq!((inv(&sim).count(STORAGE.into()), sim.player(b).unwrap().inventory.count(BELT.into())), (0, 3));

    // After leaving (without a key), B's actions do nothing; joining again starts empty.
    sim.queue(2, b, Action::Leave { pos: Vec3::ZERO });
    sim.queue(2, b, Action::Give { item: STONE.into(), count: 1 });
    sim.step();
    assert!(sim.player(b).is_none());
    sim.queue(3, b, Action::Join { key: 0 });
    sim.step();
    assert_eq!(sim.player(b).unwrap().inventory.count(BELT.into()), 0);
    assert_eq!(inv(&sim).selected, 0, "A is untouched");
}

#[test]
fn a_player_who_leaves_with_a_key_gets_their_things_back() {
    let (b, c, key) = (PlayerId(1), PlayerId(2), 0xfeed_beef);
    let mut sim = Sim::new(7, 2);
    let fresh = sim.state_hash();
    sim.apply(b, Action::Join { key });
    sim.apply(b, Action::Give { item: STONE.into(), count: 9 });
    sim.apply(b, Action::SelectSlot { slot: 4 });
    let at = Vec3::new(10.5, 70.0, -3.25);
    sim.apply(b, Action::Leave { pos: at });
    assert!(sim.player(b).is_none());
    assert_eq!((sim.away.len(), sim.away[0].key, sim.away[0].pos), (1, key, at));
    assert_ne!(sim.state_hash(), fresh, "the away player is core state");

    // Another key starts empty; the same key gets it all back, under any id, once.
    sim.apply(c, Action::Join { key: 7 });
    assert_eq!(sim.player(c).unwrap().inventory.count(STONE.into()), 0);
    sim.apply(c, Action::Leave { pos: at });
    sim.apply(c, Action::Join { key });
    let back = &sim.player(c).unwrap().inventory;
    assert_eq!((back.count(STONE.into()), back.selected), (9, 4));
    assert_eq!(sim.away.iter().map(|a| a.key).collect::<Vec<_>>(), vec![7], "a returned key is no longer away");

    // Joining while here changes nothing. Two players sharing a key leave one record: the last.
    sim.apply(c, Action::Join { key: 99 });
    assert_eq!(sim.player(c).unwrap().key, key);
    sim.apply(b, Action::Join { key });
    sim.apply(b, Action::Leave { pos: Vec3::ZERO });
    sim.apply(c, Action::Leave { pos: at });
    assert_eq!(sim.away.iter().map(|a| (a.key, a.pos)).collect::<Vec<_>>(), vec![(7, at), (key, at)]);
    assert_eq!(sim.away[1].inventory.count(STONE.into()), 9);
}
#[test]
fn pickup_that_no_longer_fits_is_thrown_back() {
    let mut sim = Sim::new(7, 2);
    sim.apply(P, Action::Give { item: STONE.into(), count: INVENTORY_SLOTS as u32 * MAX_STACK });
    sim.apply(P, Action::PickUp { item: DIRT.into(), count: 3 });
    assert_eq!(inv(&sim).count(DIRT.into()), 0);
    assert_eq!(sim.events, vec![SimEvent::Thrown { player: P, item: DIRT.into(), count: 3 }]);
}

#[test]
fn a_box_screen_moves_stacks_both_ways() {
    let mut sim = Sim::new(7, 2);
    let pos = IVec3::new(0, 200, 0);
    sim.factory.add_storage(pos);
    let slots = |sim: &Sim| sim.factory.box_slots(pos).unwrap().to_vec();
    sim.apply(P, Action::Give { item: STONE.into(), count: 30 });
    sim.apply(P, Action::Give { item: DIRT.into(), count: 5 });
    // Shift-click stores the stone; a click picks the dirt up and puts it in box slot 3.
    sim.apply(P, Action::StoreSlot { pos, slot: 0 });
    sim.apply(P, Action::ClickSlot { slot: 1, shift: false });
    sim.apply(P, Action::ClickBox { pos, slot: 3, shift: false });
    assert_eq!((slots(&sim)[0].count, slots(&sim)[3].count, slots(&sim)[3].item), (30, 5, DIRT.into()));
    assert!(inv(&sim).cursor.is_empty() && inv(&sim).count(STONE.into()) == 0);
    // Shift-click on a box slot takes it back.
    sim.apply(P, Action::ClickBox { pos, slot: 0, shift: true });
    assert_eq!((slots(&sim)[0].count, inv(&sim).count(STONE.into())), (0, 30));
    // No box there: nothing happens.
    sim.apply(P, Action::StoreSlot { pos: pos + IVec3::new(1, 0, 0), slot: 0 });
    assert_eq!(inv(&sim).count(STONE.into()), 30);
}

#[test]
fn shift_right_click_moves_every_stack_of_an_item_until_the_target_is_full() {
    let mut sim = Sim::new(7, 2);
    let pos = IVec3::new(0, 200, 0);
    sim.factory.add_storage(pos);
    let box_count = |sim: &Sim, item: ItemId| {
        sim.factory.box_slots(pos).unwrap().iter().filter(|s| s.item == item).map(|s| s.count).sum::<u32>()
    };
    let stone = ItemId::from(STONE);
    sim.apply(P, Action::Give { item: stone, count: 3 * MAX_STACK });
    sim.apply(P, Action::Give { item: DIRT.into(), count: 5 });

    // All three stacks move to the backpack, and the dirt stays where it was.
    sim.apply(P, Action::QuickMoveAll { slot: 1 });
    assert_eq!(inv(&sim).slots[..4].iter().map(|s| s.count).collect::<Vec<_>>(), [0, 0, 0, 5]);
    assert_eq!(inv(&sim).slots[HOTBAR_SLOTS..HOTBAR_SLOTS + 3].iter().map(|s| s.count).sum::<u32>(), 3 * MAX_STACK);
    sim.apply(P, Action::QuickMoveAll { slot: HOTBAR_SLOTS as u8 + 2 });
    assert_eq!(inv(&sim).slots[..3].iter().map(|s| s.count).sum::<u32>(), 3 * MAX_STACK, "and back again");

    // Into the box and out again, all stacks at once.
    sim.apply(P, Action::StoreAll { pos, slot: 0 });
    assert_eq!((inv(&sim).count(stone), box_count(&sim, stone), inv(&sim).count(DIRT.into())), (0, 3 * MAX_STACK, 5));
    sim.apply(P, Action::TakeAll { pos, slot: 0 });
    assert_eq!((inv(&sim).count(stone), box_count(&sim, stone)), (3 * MAX_STACK, 0));

    // A box with room for two stacks takes two; the third stays.
    for s in sim.factory.box_slots_mut(pos).unwrap().iter_mut().skip(2) {
        *s = Stack { item: IRON_PLATE, count: 1 };
    }
    sim.apply(P, Action::StoreAll { pos, slot: 0 });
    assert_eq!((inv(&sim).count(stone), box_count(&sim, stone)), (MAX_STACK, 2 * MAX_STACK));
    // No box, no stack: nothing happens.
    sim.apply(P, Action::StoreAll { pos: pos + IVec3::new(1, 0, 0), slot: 0 });
    sim.apply(P, Action::TakeAll { pos, slot: 30 });
    sim.apply(P, Action::QuickMoveAll { slot: 200 });
    assert_eq!((inv(&sim).count(stone), box_count(&sim, stone)), (MAX_STACK, 2 * MAX_STACK));
}

#[test]
fn sorting_merges_and_orders_a_box_and_the_backpack() {
    let mut sim = Sim::new(7, 2);
    let pos = IVec3::new(0, 200, 0);
    sim.factory.add_storage(pos);
    let box_slots = sim.factory.box_slots_mut(pos).unwrap();
    box_slots[2] = Stack { item: STONE.into(), count: 20 };
    box_slots[5] = Stack { item: DIRT.into(), count: 3 };
    box_slots[9] = Stack { item: STONE.into(), count: 60 };
    sim.apply(P, Action::SortBox { pos });
    let slots = sim.factory.box_slots(pos).unwrap().to_vec();
    let want = [(STONE, 64), (STONE, 16), (DIRT, 3)].map(|(b, count)| Stack { item: b.into(), count });
    assert_eq!(&slots[..3], &want, "a stack of 64, its rest, then the next item");
    assert!(slots[3..].iter().all(Stack::is_empty));
    sim.apply(P, Action::SortBox { pos: pos + IVec3::new(1, 0, 0) }); // no box: nothing happens

    // The backpack sorts; the hotbar keeps its layout.
    sim.apply(P, Action::Give { item: STONE.into(), count: 9 });
    sim.apply(P, Action::Give { item: IRON_PLATE, count: 2 });
    sim.apply(P, Action::Give { item: DIRT.into(), count: 3 });
    for (from, to) in [(1, 20), (2, 30)] {
        sim.apply(P, Action::ClickSlot { slot: from, shift: false });
        sim.apply(P, Action::ClickSlot { slot: to, shift: false });
    }
    sim.apply(P, Action::SortInventory);
    let stacks = &inv(&sim).slots;
    assert_eq!(stacks[0], Stack { item: STONE.into(), count: 9 }, "the hotbar stays");
    assert_eq!(stacks[HOTBAR_SLOTS], Stack { item: DIRT.into(), count: 3 }, "blocks come before items");
    assert_eq!(stacks[HOTBAR_SLOTS + 1], Stack { item: IRON_PLATE, count: 2 });
    assert!(stacks[HOTBAR_SLOTS + 2..].iter().all(Stack::is_empty));
}

#[test]
fn a_powered_line_works_where_no_chunk_is_loaded() {
    use crate::block::{COAL_ORE, CONSTRUCTOR, GENERATOR, POLE};
    use crate::item::{IRON_INGOT, IRON_ROD};
    let mut sim = Sim::new(7, 2);
    let at = |dx| IVec3::new(900 + dx, 200, -900);
    // Each block lands in slot 0, which the previous placement emptied.
    for (i, block) in [GENERATOR, POLE, CONSTRUCTOR].into_iter().enumerate() {
        sim.apply(P, Action::Give { item: block.into(), count: 1 });
        let pos = at(i as i32 * 4);
        sim.apply(P, Action::PlaceBlock { pos, slot: 0, facing: 0, against: pos });
    }
    let rods = crate::recipes::MACHINE_RECIPES.iter().position(|r| r.main().0 == IRON_ROD).unwrap() as u16;
    sim.apply(P, Action::SetRecipe { pos: at(8), recipe: rods });
    sim.apply(P, Action::Give { item: COAL_ORE.into(), count: 1 });
    sim.apply(P, Action::Give { item: IRON_INGOT, count: 3 });
    sim.apply(P, Action::Insert { pos: at(0), item: COAL_ORE.into() });
    sim.apply(P, Action::Insert { pos: at(8), item: IRON_INGOT });
    for _ in 0..crate::TICK_RATE * 7 {
        sim.step();
    }
    assert!(!sim.world.is_loaded(at(0).as_vec3()), "nothing was streamed in");
    let out = sim.factory.panel(at(8)).unwrap().slots[1].1;
    assert_eq!((out.item, out.count), (IRON_ROD, 3), "the pole 4 blocks away powered it");
}

#[test]
fn crafting_a_recipe_research_locks_does_nothing() {
    use crate::block::SPLITTER;
    let mut sim = Sim::new(7, 2);
    sim.apply(P, Action::Join { key: 0 });
    sim.apply(P, Action::Give { item: IRON_PLATE, count: 4 });
    sim.apply(P, Action::Give { item: BELT.into(), count: 4 });
    let splitter = RECIPES.iter().position(|r| r.output == SPLITTER.into()).unwrap() as u16;
    sim.apply(P, Action::Craft { recipe: splitter, times: 1 });
    assert_eq!((inv(&sim).count(SPLITTER.into()), inv(&sim).count(IRON_PLATE)), (0, 4), "Belt Routing isn't done");
    sim.apply(P, Action::SetResearch { tech: 0 });
    assert_eq!(sim.factory.research.current, Some(0));
    (0..crate::research::TECHS[0].units).for_each(|_| sim.factory.research.add_unit(0));
    sim.apply(P, Action::Craft { recipe: splitter, times: 1 });
    for _ in 0..RECIPES[splitter as usize].hand_ticks() {
        sim.step();
    }
    assert_eq!(inv(&sim).count(SPLITTER.into()), 1);
}

/// One of every action, with values that show up if a field is dropped or swapped.
fn samples() -> Vec<Action> {
    let (pos, against) = (IVec3::new(-3, 70, 123_456), IVec3::new(i32::MIN, -1, i32::MAX));
    vec![
        Action::BreakBlock { pos },
        Action::PlaceBlock { pos, slot: 7, facing: 3, against },
        Action::TakeContents { pos },
        Action::SetRecipe { pos, recipe: u16::MAX },
        Action::SetFilter { pos, item: IRON_PLATE },
        Action::SetResearch { tech: 2 },
        Action::Insert { pos, item: STONE.into() },
        Action::Craft { recipe: 4, times: 70_000 },
        Action::ClickSlot { slot: 35, shift: true },
        Action::ClickBox { pos, slot: 23, shift: false },
        Action::StoreSlot { pos, slot: 9 },
        Action::CloseInventory,
        Action::SelectSlot { slot: 8 },
        Action::ScrollSlot { delta: -1 },
        Action::DropSelected { count: 64 },
        Action::PickUp { item: IRON_PLATE, count: 3 },
        Action::Give { item: ItemId::NONE, count: u32::MAX },
        Action::Join { key: u64::MAX - 5 },
        Action::Leave { pos: Vec3::new(-0.5, 1e9, f64::MIN_POSITIVE) },
        Action::Rotate { pos: IVec3::new(-7, 255, 1 << 20) },
        Action::SetQuarry { pos, width: 3, depth: 2, paused: true },
        Action::MarkSite { a: (i32::MIN, 5), b: (-9, i32::MAX), level: 70, job: Job::Flatten },
        Action::RemoveSite { id: u32::MAX - 1 },
        Action::Upgrade { pos },
        Action::SortInventory,
        Action::SortBox { pos },
        Action::CancelCraft { order: 513 },
        Action::Connect { pole: pos, to: against },
        Action::Disconnect { pole: against, to: pos },
        Action::QuickMoveAll { slot: 20 },
        Action::StoreAll { pos, slot: 3 },
        Action::TakeAll { pos, slot: 11 },
    ]
}

fn encode(a: &Action) -> Vec<u8> {
    let mut w = ByteWriter::default();
    a.write(&mut w);
    w.bytes
}

#[test]
fn every_action_round_trips_through_bytes() {
    let all = samples();
    let tags: Vec<u8> = all.iter().map(|a| encode(a)[0]).collect();
    assert_eq!(tags, (0..codec::TAG_COUNT).collect::<Vec<_>>(), "one sample per tag, in tag order");

    let mut w = ByteWriter::default();
    for a in &all {
        a.write(&mut w);
    }
    let mut r = ByteReader::new(&w.bytes);
    for a in &all {
        assert_eq!(Action::read(&mut r), Some(*a));
    }
    assert!(r.is_done());
}

#[test]
fn damaged_action_bytes_fail_cleanly() {
    for a in samples() {
        let bytes = encode(&a);
        for cut in 0..bytes.len() {
            assert_eq!(Action::read(&mut ByteReader::new(&bytes[..cut])), None, "{a:?} cut at {cut}");
        }
    }
    for tag in codec::TAG_COUNT..=u8::MAX {
        assert_eq!(Action::read(&mut ByteReader::new(&[tag, 0, 0, 0, 0, 0, 0, 0, 0])), None);
    }
    let mut bad_bool = encode(&Action::ClickSlot { slot: 1, shift: true });
    bad_bool[2] = 2;
    assert_eq!(Action::read(&mut ByteReader::new(&bad_bool)), None);
    let mut bad_item = encode(&Action::Give { item: STONE.into(), count: 1 });
    bad_item[1..3].copy_from_slice(&60_000u16.to_le_bytes());
    assert_eq!(Action::read(&mut ByteReader::new(&bad_item)), None);
    let mut bad_job = encode(&Action::MarkSite { a: (0, 0), b: (1, 1), level: 70, job: Job::Dig });
    bad_job[21] = 3;
    assert_eq!(Action::read(&mut ByteReader::new(&bad_job)), None);

    // Random bytes: whatever reads must be a real action, and nothing panics.
    let mut rng = crate::math::Rng::new(99);
    for _ in 0..2000 {
        let bytes: Vec<u8> = (0..rng.below(40)).map(|_| rng.next_u32() as u8).collect();
        let mut r = ByteReader::new(&bytes);
        while let Some(a) = Action::read(&mut r) {
            assert_eq!(Action::read(&mut ByteReader::new(&encode(&a))), Some(a));
        }
    }
}
