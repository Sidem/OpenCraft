use super::*;
use crate::block::{BEDROCK, DIRT, STONE};
use crate::inventory::{INVENTORY_SLOTS, MAX_STACK};

const P: PlayerId = PlayerId(0);

fn inv(sim: &Sim) -> &crate::inventory::Inventory {
    &sim.player(P).inventory
}

#[test]
fn actions_apply_at_their_tick_in_queue_order() {
    let mut sim = Sim::new(7, 2);
    sim.queue(2, P, Action::Give { item: STONE, count: 5 });
    sim.queue(0, P, Action::Give { item: STONE, count: 10 });
    // Pick the stack up with the cursor, then put it down in slot 5: only right in this order.
    sim.queue(0, P, Action::ClickSlot { slot: 0, shift: false });
    sim.queue(0, P, Action::ClickSlot { slot: 5, shift: false });
    sim.step();
    assert_eq!((inv(&sim).slots[0].count, inv(&sim).slots[5].count), (0, 10));
    sim.step();
    assert_eq!(inv(&sim).count(STONE), 10, "the tick 2 action waits");
    sim.step();
    assert_eq!(inv(&sim).count(STONE), 15);

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
    sim.apply(P, Action::Give { item: STORAGE, count: 1 });
    sim.apply(P, Action::PlaceBlock { pos, slot: 0, facing: 0, against: pos - IVec3::new(0, 1, 0) });
    assert_eq!(sim.world.block_anywhere(pos), Some(STORAGE));
    assert_eq!(sim.factory.storage_count(), 1);
    assert_eq!(inv(&sim).count(STORAGE), 0);
    assert_eq!(sim.events, vec![SimEvent::BlockPlaced { player: P, pos, block: STORAGE }]);

    sim.events.clear();
    sim.apply(P, Action::BreakBlock { pos });
    assert_eq!(sim.world.block_anywhere(pos), Some(AIR));
    assert_eq!(sim.factory.storage_count(), 0);
    assert!(matches!(
        sim.events[..],
        [SimEvent::BlockBroken { block: STORAGE, .. }, SimEvent::Dropped { item: STORAGE, count: 1, .. }]
    ));
}

#[test]
fn pickup_that_no_longer_fits_is_thrown_back() {
    let mut sim = Sim::new(7, 2);
    sim.apply(P, Action::Give { item: STONE, count: INVENTORY_SLOTS as u32 * MAX_STACK });
    sim.apply(P, Action::PickUp { item: DIRT, count: 3 });
    assert_eq!(inv(&sim).count(DIRT), 0);
    assert_eq!(sim.events, vec![SimEvent::Thrown { player: P, item: DIRT, count: 3 }]);
}
