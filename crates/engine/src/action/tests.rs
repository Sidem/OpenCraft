use super::*;
use crate::block::{BEDROCK, BELT, DIRT, IRON_ORE, STONE, STORAGE};
use crate::inventory::INVENTORY_SLOTS;
use crate::item::MAX_STACK;

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
    assert_eq!(sim.factory.storage_count(), 1);
    assert_eq!(inv(&sim).count(STORAGE.into()), 0);
    assert_eq!(sim.events, vec![SimEvent::BlockPlaced { player: P, pos, block: STORAGE }]);

    sim.events.clear();
    sim.apply(P, Action::BreakBlock { pos });
    assert_eq!(sim.world.block_anywhere(pos), Some(AIR));
    assert_eq!(sim.factory.storage_count(), 0);
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
    sim.queue(0, b, Action::Join);
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
    assert_eq!((sim.factory.storage_count(), sim.factory.belt_count()), (0, 1));
    assert_eq!((inv(&sim).count(STORAGE.into()), sim.player(b).unwrap().inventory.count(BELT.into())), (0, 3));

    // After leaving, B's actions do nothing; joining again starts empty.
    sim.queue(2, b, Action::Leave);
    sim.queue(2, b, Action::Give { item: STONE.into(), count: 1 });
    sim.step();
    assert!(sim.player(b).is_none());
    sim.queue(3, b, Action::Join);
    sim.step();
    assert_eq!(sim.player(b).unwrap().inventory.count(BELT.into()), 0);
    assert_eq!(inv(&sim).selected, 0, "A is untouched");
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
