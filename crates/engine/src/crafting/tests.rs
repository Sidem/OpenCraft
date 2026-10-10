use super::*;
use crate::action::Action;
use crate::block::{BELT, LOG, MINER, PLANKS, SPLITTER, STORAGE};
use crate::bytes::{ByteReader, ByteWriter};
use crate::item::{COPPER_INGOT, COPPER_WIRE, IRON_INGOT, IRON_PLATE, IRON_ROD};

const P: PlayerId = PlayerId(0);

fn recipe(output: ItemId) -> u16 {
    RECIPES.iter().position(|r| r.output == output).expect("a hand recipe") as u16
}

fn count(sim: &Sim, item: ItemId) -> u32 {
    sim.player(P).unwrap().inventory.count(item)
}

fn give(sim: &mut Sim, item: ItemId, n: u32) {
    sim.apply(P, Action::Give { item, count: n });
}

fn queue(sim: &Sim) -> &CraftQueue {
    &sim.player(P).unwrap().crafts
}

fn plan_for(sim: &Sim, output: ItemId, times: u32) -> Option<Plan> {
    plan(&sim.player(P).unwrap().inventory, &sim.factory.research, recipe(output), times)
}

fn ticks(sim: &mut Sim, n: u32) {
    for _ in 0..n {
        sim.step();
    }
}

#[test]
fn a_miner_plans_its_parts_from_ingots_in_running_order() {
    let mut sim = Sim::new(7, 2);
    give(&mut sim, IRON_INGOT, 14);
    give(&mut sim, COPPER_INGOT, 3);
    let p = plan_for(&sim, MINER.into(), 1).unwrap();
    let steps: Vec<(ItemId, u32, bool)> =
        p.steps.iter().map(|s| (RECIPES[s.recipe as usize].output, s.times, s.deliver)).collect();
    let miner: ItemId = MINER.into();
    assert_eq!(steps, [(IRON_PLATE, 5, false), (IRON_ROD, 4, false), (COPPER_WIRE, 3, false), (miner, 1, true)]);
    assert_eq!(p.leaves, [(IRON_INGOT, 14), (COPPER_INGOT, 3)]);
    assert_eq!(p.part_crafts(), 12);
    // One ingot short, and there is no plan.
    let mut short = Sim::new(7, 2);
    give(&mut short, IRON_INGOT, 13);
    give(&mut short, COPPER_INGOT, 3);
    assert!(plan_for(&short, MINER.into(), 1).is_none());
}

#[test]
fn parts_already_held_are_used_before_more_are_made() {
    let mut sim = Sim::new(7, 2);
    give(&mut sim, IRON_PLATE, 3);
    give(&mut sim, IRON_INGOT, 8);
    give(&mut sim, COPPER_WIRE, 6);
    let p = plan_for(&sim, MINER.into(), 1).unwrap();
    // Two more plates (4 ingots) and four rods (4 ingots); the wire and three plates come from the inventory.
    let made: Vec<(ItemId, u32)> = p.steps.iter().map(|s| (RECIPES[s.recipe as usize].output, s.times)).collect();
    assert_eq!(made, [(IRON_PLATE, 2), (IRON_ROD, 4), (MINER.into(), 1)]);
    assert_eq!(p.leaves, [(IRON_PLATE, 3), (IRON_INGOT, 8), (COPPER_WIRE, 6)]);
}

#[test]
fn how_many_can_be_crafted_counts_the_parts_that_can_be_made() {
    let mut sim = Sim::new(7, 2);
    let inv = |s: &Sim| s.player(P).unwrap().inventory.count(IRON_PLATE);
    give(&mut sim, IRON_INGOT, 11);
    let (inventory, research) = (&sim.player(P).unwrap().inventory, &sim.factory.research);
    assert_eq!(max_times(inventory, research, recipe(IRON_PLATE)), 5);
    // A belt is a plate and a rod: three ingots, and there are 11.
    assert_eq!(max_times(inventory, research, recipe(BELT.into())), 3);
    assert_eq!(max_times(inventory, research, recipe(MINER.into())), 0);
    assert_eq!(inv(&sim), 0);
    // Research locks a recipe, however many parts there are.
    give(&mut sim, IRON_PLATE, 4);
    assert!(plan_for(&sim, SPLITTER.into(), 1).is_none());
}

#[test]
fn crafts_take_time_one_after_another_and_deliver_when_done() {
    let mut sim = Sim::new(7, 2);
    give(&mut sim, IRON_INGOT, 4);
    let plate = recipe(IRON_PLATE);
    let each = RECIPES[plate as usize].hand_ticks();
    assert_eq!(each, 150, "2.5 seconds");
    sim.apply(P, Action::Craft { recipe: plate, times: 2 });
    assert_eq!((count(&sim, IRON_INGOT), queue(&sim).orders.len()), (0, 1), "paid for at once, queued");
    ticks(&mut sim, each - 1);
    assert_eq!(count(&sim, IRON_PLATE), 0);
    ticks(&mut sim, 1);
    assert_eq!(count(&sim, IRON_PLATE), 1);
    assert_eq!(queue(&sim).orders[0].permille(1000), 500);
    ticks(&mut sim, each);
    assert_eq!((count(&sim, IRON_PLATE), queue(&sim).orders.len()), (2, 0));
    let crafted =
        sim.events.iter().filter(|e| matches!(e, SimEvent::Crafted { item, .. } if *item == IRON_PLATE)).count();
    assert_eq!(crafted, 2);
}

#[test]
fn handcrafting_research_makes_crafts_shorter() {
    let mut sim = Sim::new(7, 2);
    let first = crate::research::TECHS.iter().position(|t| t.name == "Handcrafting I").unwrap() as u8;
    (0..crate::research::TECHS[first as usize].units).for_each(|_| sim.factory.research.add_unit(first));
    give(&mut sim, IRON_INGOT, 2);
    let plate = recipe(IRON_PLATE);
    let each = craft_ticks(&RECIPES[plate as usize], 1250);
    assert_eq!(each, 120, "2.5 seconds at 125% is 2 seconds");
    sim.apply(P, Action::Craft { recipe: plate, times: 1 });
    assert_eq!(queue(&sim).orders[0].total, each);
    ticks(&mut sim, each - 1);
    assert_eq!(count(&sim, IRON_PLATE), 0);
    ticks(&mut sim, 1);
    assert_eq!(count(&sim, IRON_PLATE), 1);
    assert_eq!(craft_ticks(&RECIPES[plate as usize], u32::MAX), 1, "never shorter than a tick");
}

#[test]
fn a_miner_from_ingots_is_made_part_by_part_and_nothing_is_left_over() {
    let mut sim = Sim::new(7, 2);
    give(&mut sim, IRON_INGOT, 14);
    give(&mut sim, COPPER_INGOT, 3);
    let miner = recipe(MINER.into());
    sim.apply(P, Action::Craft { recipe: miner, times: 1 });
    let total = queue(&sim).orders[0].total;
    let steps = [(IRON_PLATE, 5), (IRON_ROD, 4), (COPPER_WIRE, 3), (MINER.into(), 1)];
    let expected: u32 = steps.iter().map(|&(i, n)| n * RECIPES[recipe(i) as usize].hand_ticks()).sum();
    assert_eq!(total, expected);
    ticks(&mut sim, total - 1);
    assert_eq!(count(&sim, MINER.into()), 0, "not before the last craft ends");
    ticks(&mut sim, 1);
    assert_eq!(count(&sim, MINER.into()), 1);
    // Three wire crafts make six wire, exactly what the miner took: no strays anywhere.
    let inv = &sim.player(P).unwrap().inventory;
    assert_eq!(
        (inv.count(IRON_PLATE), inv.count(IRON_ROD), inv.count(COPPER_WIRE), queue(&sim).orders.len()),
        (0, 0, 0, 0)
    );
}

#[test]
fn a_part_made_in_a_batch_leaves_its_surplus_in_the_inventory() {
    let mut sim = Sim::new(7, 2);
    give(&mut sim, IRON_INGOT, 4);
    give(&mut sim, COPPER_INGOT, 2);
    // A pole needs 2 sticks and 2 wire; here 2 wire is one craft. Screws: a rod makes 4.
    let screw = recipe(crate::item::SCREW);
    sim.apply(P, Action::Craft { recipe: screw, times: 2 });
    ticks(&mut sim, 5_000);
    assert_eq!(count(&sim, crate::item::SCREW), 8);
    // Ask for wire from an ingot: two come out.
    sim.apply(P, Action::Craft { recipe: recipe(COPPER_WIRE), times: 2 });
    ticks(&mut sim, 5_000);
    assert_eq!(count(&sim, COPPER_WIRE), 4);
    assert_eq!(count(&sim, IRON_INGOT), 2);
}

#[test]
fn cancelling_gives_back_everything_even_mid_craft() {
    let mut sim = Sim::new(7, 2);
    give(&mut sim, IRON_INGOT, 14);
    give(&mut sim, COPPER_INGOT, 3);
    let before = (count(&sim, IRON_INGOT), count(&sim, COPPER_INGOT));
    sim.apply(P, Action::Craft { recipe: recipe(MINER.into()), times: 1 });
    ticks(&mut sim, 1000);
    assert!(queue(&sim).orders[0].busy || !queue(&sim).orders[0].held.is_empty());
    sim.apply(P, Action::CancelCraft { order: 0 });
    // The finished plates and rods come back as they are, the rest as ingots.
    let inv = &sim.player(P).unwrap().inventory;
    let ingots_worth = inv.count(IRON_INGOT) + 2 * inv.count(IRON_PLATE) + inv.count(IRON_ROD);
    assert_eq!(ingots_worth, before.0, "iron");
    assert_eq!(inv.count(COPPER_INGOT) + inv.count(COPPER_WIRE).div_ceil(2), before.1, "copper");
    assert!(queue(&sim).orders.is_empty());
}

#[test]
fn the_queue_takes_a_dozen_orders_and_only_what_can_be_paid() {
    let mut sim = Sim::new(7, 2);
    give(&mut sim, IRON_INGOT, 30);
    for _ in 0..MAX_ORDERS + 3 {
        sim.apply(P, Action::Craft { recipe: recipe(IRON_ROD), times: 1 });
    }
    assert_eq!((queue(&sim).orders.len(), count(&sim, IRON_INGOT)), (MAX_ORDERS, 30 - MAX_ORDERS as u32));
    // Asking for more than the ingots pay for queues what they pay for.
    let mut sim = Sim::new(7, 2);
    give(&mut sim, IRON_INGOT, 5);
    sim.apply(P, Action::Craft { recipe: recipe(IRON_PLATE), times: 50 });
    assert_eq!(queue(&sim).orders[0].amount(), 2);
    // With nothing to pay with nothing is queued.
    sim.apply(P, Action::Craft { recipe: recipe(IRON_PLATE), times: 1 });
    assert_eq!(queue(&sim).orders.len(), 1);
}

#[test]
fn a_player_who_leaves_gets_the_queue_back() {
    let mut sim = Sim::new(7, 2);
    sim.apply(PlayerId(1), Action::Join { key: 99 });
    let q = PlayerId(1);
    sim.apply(q, Action::Give { item: IRON_INGOT, count: 4 });
    sim.apply(q, Action::Craft { recipe: recipe(IRON_PLATE), times: 2 });
    sim.apply(q, Action::Leave { pos: crate::math::Vec3::ZERO });
    assert_eq!(sim.away[0].inventory.count(IRON_INGOT), 4);
}

#[test]
fn a_queue_survives_a_save_and_finishes_the_same() {
    let mut sim = Sim::new(7, 2);
    give(&mut sim, IRON_INGOT, 14);
    give(&mut sim, COPPER_INGOT, 3);
    sim.apply(P, Action::Craft { recipe: recipe(MINER.into()), times: 1 });
    ticks(&mut sim, 700);
    let mut w = ByteWriter::default();
    queue(&sim).write_state(&mut w);
    let back = CraftQueue::read_state(&mut ByteReader::new(&w.bytes)).unwrap();
    let mut w2 = ByteWriter::default();
    back.write_state(&mut w2);
    assert_eq!(w.bytes, w2.bytes);
    assert_eq!(back.orders[0].steps, queue(&sim).orders[0].steps);
    // Junk is refused: a recipe that doesn't exist.
    let mut junk = w.bytes.clone();
    junk[5] = 0xff;
    junk[6] = 0xff;
    assert!(CraftQueue::read_state(&mut ByteReader::new(&junk)).is_none());
    ticks(&mut sim, 20_000);
    assert_eq!(count(&sim, MINER.into()), 1);
}

/// The steps of an order, as (output, crafts, delivered) in running order.
fn steps_of(p: &Plan) -> Vec<(ItemId, u32, bool)> {
    p.steps.iter().map(|s| (RECIPES[s.recipe as usize].output, s.times, s.deliver)).collect()
}

#[test]
fn a_storage_box_from_logs_and_ingots_queues_planks_then_plates_then_the_box() {
    let mut sim = Sim::new(7, 2);
    give(&mut sim, LOG.into(), 2);
    give(&mut sim, IRON_INGOT, 4);
    let p = plan_for(&sim, STORAGE.into(), 1).unwrap();
    // 8 planks are two crafts of 4 (2 logs), 2 plates two crafts (4 ingots), then the box.
    assert_eq!(steps_of(&p), [(PLANKS.into(), 2, false), (IRON_PLATE, 2, false), (STORAGE.into(), 1, true)]);
    assert_eq!(p.part_crafts(), 4);
    // Planks already held: only the plates are made first.
    give(&mut sim, PLANKS.into(), 8);
    let p = plan_for(&sim, STORAGE.into(), 1).unwrap();
    assert_eq!(steps_of(&p), [(IRON_PLATE, 2, false), (STORAGE.into(), 1, true)]);
    // Plates held instead: only the planks.
    let mut sim = Sim::new(7, 2);
    give(&mut sim, LOG.into(), 2);
    give(&mut sim, IRON_PLATE, 2);
    let p = plan_for(&sim, STORAGE.into(), 1).unwrap();
    assert_eq!(steps_of(&p), [(PLANKS.into(), 2, false), (STORAGE.into(), 1, true)]);
    // Everything held: just the box.
    give(&mut sim, PLANKS.into(), 8);
    assert_eq!(steps_of(&plan_for(&sim, STORAGE.into(), 1).unwrap()), [(STORAGE.into(), 1, true)]);
}

#[test]
fn the_queue_runs_the_parts_in_order_and_the_box_last() {
    let mut sim = Sim::new(7, 2);
    give(&mut sim, LOG.into(), 2);
    give(&mut sim, IRON_INGOT, 4);
    sim.apply(P, Action::Craft { recipe: recipe(STORAGE.into()), times: 1 });
    let order = &queue(&sim).orders[0];
    assert_eq!(order.steps.len(), 3);
    let crafts = |item: ItemId| RECIPES[recipe(item) as usize].hand_ticks();
    ticks(&mut sim, 2 * crafts(PLANKS.into()));
    assert_eq!((count(&sim, PLANKS.into()), count(&sim, STORAGE.into())), (0, 0), "planks wait in the order");
    assert_eq!(queue(&sim).orders[0].steps[0].recipe, recipe(IRON_PLATE));
    ticks(&mut sim, 2 * crafts(IRON_PLATE) + crafts(STORAGE.into()));
    assert_eq!(count(&sim, STORAGE.into()), 1);
    assert!(queue(&sim).orders.is_empty());
}
