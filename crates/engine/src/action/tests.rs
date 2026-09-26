use super::*;
use crate::block::{BEDROCK, BELT, DIRT, IRON_ORE, STONE, STORAGE};
use crate::bytes::{ByteReader, ByteWriter};
use crate::factory::Kind;
use crate::inventory::INVENTORY_SLOTS;
use crate::item::{IRON_PLATE, MAX_STACK};

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
fn two_players_build_and_craft_with_their_own_inventories() {
    let mut sim = Sim::new(7, 2);
    let b = PlayerId(1);
    let belt = RECIPES.iter().position(|r| r.output == BELT.into()).unwrap() as u16;
    sim.queue(0, b, Action::Join { key: 0 });
    sim.queue(0, b, Action::Give { item: IRON_ORE.into(), count: 1 });
    sim.queue(0, b, Action::Give { item: STONE.into(), count: 2 });
    sim.queue(0, b, Action::Craft { recipe: belt, times: 1 });
    sim.queue(0, P, Action::Give { item: STORAGE.into(), count: 1 });
    sim.queue(0, P, Action::Craft { recipe: belt, times: 1 });
    sim.step();
    assert_eq!(sim.events, vec![SimEvent::Crafted { player: b, item: BELT.into(), count: 4 }], "only B can pay");
    let inv_b = &sim.player(b).unwrap().inventory;
    assert_eq!((inv_b.count(BELT.into()), inv_b.count(IRON_ORE.into()), inv_b.count(STONE.into())), (4, 0, 0));
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
    let rods = crate::recipes::MACHINE_RECIPES.iter().position(|r| r.output.0 == IRON_ROD).unwrap() as u16;
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
