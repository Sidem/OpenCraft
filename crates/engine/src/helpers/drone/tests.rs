use super::*;
use crate::action::Action;
use crate::block::{SMELTER, STORAGE};
use crate::bytes::{ByteReader, ByteWriter};
use crate::inventory::HOTBAR_SLOTS;
use crate::item::{COPPER_WIRE, IRON_INGOT, IRON_PLATE, SCREW, STEEL_INGOT, STEEL_PLATE};
use crate::recipes::RECIPES;
use crate::sim::Sim;
use crate::Game;

const P: PlayerId = PlayerId(0);
/// Far from spawn, so the world there is empty air for the boxes (the core reads it anywhere).
const HOME: IVec3 = IVec3::new(900, 200, -900);
/// A belt's direction toward +x (`factory::DIRS`).
const EAST: u8 = 1;

fn sim_with_drone() -> Sim {
    let mut sim = Sim::new(7, 2);
    sim.apply(P, Action::Give { item: PERSONAL_DRONE, count: 1 });
    sim
}

/// Puts `block` down at `HOME + off` for the player.
fn place(sim: &mut Sim, block: u8, off: IVec3) -> IVec3 {
    let pos = HOME + off;
    sim.apply(P, Action::Give { item: ItemId::block(block), count: 1 });
    let slot = sim.player(P).unwrap().inventory.slots.iter().position(|s| s.item == ItemId::block(block)).unwrap();
    sim.apply(P, Action::PlaceBlock { pos, slot: slot as u8, facing: 0, against: pos });
    pos
}

fn order(sim: &mut Sim, job: Job) {
    sim.apply(P, Action::Drone { order: DroneOrder::Job(job), at: HOME });
}

fn drone(sim: &Sim) -> &Drone {
    &sim.player(P).unwrap().helpers.drone
}

fn pack(sim: &Sim, item: ItemId) -> u32 {
    sim.player(P).unwrap().inventory.count(item)
}

/// Runs until the drone has nothing to do (at most a minute); the notes it gave.
fn until_idle(sim: &mut Sim) -> Vec<Note> {
    let mut notes = Vec::new();
    for _ in 0..60 * TICK_RATE {
        sim.step();
        notes.extend(sim.events.drain(..).filter_map(|e| match e {
            SimEvent::DroneNote { note, .. } => Some(note),
            _ => None,
        }));
        if drone(sim).idle() {
            break;
        }
    }
    notes
}

#[test]
fn it_fetches_from_boxes_first_then_what_machines_made() {
    let mut sim = sim_with_drone();
    let near = place(&mut sim, STORAGE, IVec3::new(4, 0, 0));
    let far = place(&mut sim, STORAGE, IVec3::new(40, 0, 0));
    let smelter = place(&mut sim, SMELTER, IVec3::new(0, 0, 12));
    sim.factory.stock(near, IRON_PLATE, 30);
    sim.factory.stock(far, IRON_PLATE, 100);
    sim.factory.stock_output(smelter, IRON_PLATE, 20);
    order(&mut sim, Job::Fetch { item: IRON_PLATE, count: 140, slot: ANY_SLOT });
    sim.step();
    let trip = drone(&sim).trip.clone().expect("flying");
    let stops: Vec<(IVec3, u32)> = trip.stops.iter().map(|s| (s.at, s.count)).collect();
    assert_eq!(stops, vec![(near, 30), (smelter, 10), (far, 100)], "both boxes, then the machine; flown nearest next");
    assert!(trip.stops.windows(2).all(|w| w[0].tick < w[1].tick) && trip.total > trip.stops[2].tick);
    assert_eq!(until_idle(&mut sim), vec![]);
    assert_eq!(pack(&sim, IRON_PLATE), 140);
    assert_eq!(sim.factory.box_count(far, IRON_PLATE), 0);
    assert_eq!(sim.factory.smelter_at(smelter).out.count(IRON_PLATE), 10, "the machine gave what the boxes lacked");
}

#[test]
fn a_big_fetch_flies_several_trips_and_says_when_it_brought_all_there_was() {
    let mut sim = sim_with_drone();
    let b = place(&mut sim, STORAGE, IVec3::new(6, 0, 0));
    let stack = item::stack_size(COPPER_WIRE);
    sim.factory.stock(b, COPPER_WIRE, stack * (CARGO_STACKS + 2));
    order(&mut sim, Job::Fetch { item: COPPER_WIRE, count: stack * (CARGO_STACKS + 5), slot: ANY_SLOT });
    sim.step();
    assert_eq!(drone(&sim).trip.as_ref().unwrap().stops[0].count, stack * CARGO_STACKS, "one trip's cargo");
    assert_eq!(until_idle(&mut sim), vec![Note::OnlySome]);
    assert_eq!(pack(&sim, COPPER_WIRE), stack * (CARGO_STACKS + 2));
}

#[test]
fn it_says_why_when_it_cannot_fetch() {
    let mut sim = sim_with_drone();
    let b = place(&mut sim, STORAGE, IVec3::new(RANGE + 4, 0, 0));
    sim.factory.stock(b, IRON_PLATE, 10);
    order(&mut sim, Job::Fetch { item: IRON_PLATE, count: 10, slot: ANY_SLOT });
    assert_eq!(until_idle(&mut sim), vec![Note::NoneInReach], "out of reach");
    assert_eq!(pack(&sim, IRON_PLATE), 0);
    // Without the drone in the pack nothing is queued at all.
    sim.players[0].as_mut().unwrap().inventory.remove(PERSONAL_DRONE, 1);
    sim.apply(
        P,
        Action::Drone { order: DroneOrder::Job(Job::Fetch { item: IRON_PLATE, count: 1, slot: ANY_SLOT }), at: b },
    );
    assert!(drone(&sim).idle());
}

#[test]
fn storing_goes_to_the_items_box_else_an_empty_one_never_a_box_of_other_things() {
    let mut sim = sim_with_drone();
    let other = place(&mut sim, STORAGE, IVec3::new(2, 0, 0));
    let empty = place(&mut sim, STORAGE, IVec3::new(9, 0, 0));
    let home_box = place(&mut sim, STORAGE, IVec3::new(20, 0, 0));
    sim.factory.stock(other, COPPER_WIRE, 5);
    sim.factory.stock(home_box, IRON_PLATE, 1);
    sim.apply(P, Action::Give { item: IRON_PLATE, count: 50 });
    sim.apply(P, Action::Give { item: STEEL_PLATE, count: 7 });
    order(&mut sim, Job::Store { item: IRON_PLATE, count: 50, slot: ANY_SLOT });
    order(&mut sim, Job::Store { item: STEEL_PLATE, count: 7, slot: ANY_SLOT });
    sim.step();
    assert_eq!(pack(&sim, IRON_PLATE) + pack(&sim, STEEL_PLATE), 0, "both ride in one trip");
    assert_eq!(until_idle(&mut sim), vec![]);
    assert_eq!(sim.factory.box_count(home_box, IRON_PLATE), 51, "plates to the plates");
    assert_eq!(sim.factory.box_count(empty, STEEL_PLATE), 7, "steel has no box: the empty one");
    assert_eq!(sim.factory.box_count(other, STEEL_PLATE) + sim.factory.box_count(other, IRON_PLATE), 0);
}

#[test]
fn it_never_fills_a_box_on_a_belt_line_nor_an_empty_labelled_box() {
    let mut sim = sim_with_drone();
    let fed = place(&mut sim, STORAGE, IVec3::new(2, 0, 0));
    let feeding = place(&mut sim, STORAGE, IVec3::new(2, 0, 4));
    let labelled = place(&mut sim, STORAGE, IVec3::new(5, 0, -3));
    let free = place(&mut sim, STORAGE, IVec3::new(14, 0, 0));
    let free2 = place(&mut sim, STORAGE, IVec3::new(14, 0, 6));
    sim.factory.add_belt(fed - IVec3::new(1, 0, 0), EAST); // delivers into it
    sim.factory.add_belt(feeding + IVec3::new(1, 0, 0), EAST); // leads away from it
    sim.factory.stock(feeding, IRON_PLATE, 1);
    sim.factory.set_box_label(labelled, 2);
    sim.step();
    sim.apply(P, Action::Give { item: IRON_PLATE, count: 30 });
    sim.apply(P, Action::Give { item: COPPER_WIRE, count: 8 });
    order(&mut sim, Job::Store { item: IRON_PLATE, count: 30, slot: ANY_SLOT });
    order(&mut sim, Job::Store { item: COPPER_WIRE, count: 8, slot: ANY_SLOT });
    assert_eq!(until_idle(&mut sim), vec![]);
    assert_eq!(sim.factory.box_count(free, IRON_PLATE), 30, "past the line boxes, even the one holding plates");
    assert_eq!(sim.factory.box_count(free2, COPPER_WIRE), 8, "one kind to an empty box: the next one");
    for b in [fed, labelled] {
        assert_eq!(sim.factory.box_count(b, IRON_PLATE) + sim.factory.box_count(b, COPPER_WIRE), 0);
    }
    assert_eq!(pack(&sim, IRON_PLATE), 0, "all 30 went to the free box: none to a line");
    // A labelled box is still the home of what it already holds.
    sim.factory.stock(labelled, STEEL_PLATE, 1);
    sim.apply(P, Action::Give { item: STEEL_PLATE, count: 4 });
    order(&mut sim, Job::Store { item: STEEL_PLATE, count: 4, slot: ANY_SLOT });
    until_idle(&mut sim);
    assert_eq!(sim.factory.box_count(labelled, STEEL_PLATE), 5);
}

#[test]
fn it_fetches_from_storage_before_a_box_on_a_line() {
    let mut sim = sim_with_drone();
    let line = place(&mut sim, STORAGE, IVec3::new(3, 0, 0));
    let store = place(&mut sim, STORAGE, IVec3::new(25, 0, 0));
    sim.factory.add_belt(line - IVec3::new(1, 0, 0), EAST);
    sim.factory.stock(line, IRON_PLATE, 50);
    sim.factory.stock(store, IRON_PLATE, 50);
    sim.step();
    order(&mut sim, Job::Fetch { item: IRON_PLATE, count: 60, slot: ANY_SLOT });
    sim.step();
    let stops: Vec<(IVec3, u32)> = drone(&sim).trip.as_ref().unwrap().stops.iter().map(|s| (s.at, s.count)).collect();
    assert_eq!(stops, vec![(line, 10), (store, 50)], "the storage box's 50 first; the tour still flies nearest first");
}

#[test]
fn a_misc_box_takes_what_has_no_other_box_last_several_kinds_a_trip_never_on_a_line() {
    use crate::factory::box_label::MISC;
    let mut sim = sim_with_drone();
    let misc = place(&mut sim, STORAGE, IVec3::new(3, 0, 0));
    let line_misc = place(&mut sim, STORAGE, IVec3::new(2, 0, 4));
    let home = place(&mut sim, STORAGE, IVec3::new(20, 0, 0));
    let free = place(&mut sim, STORAGE, IVec3::new(30, 0, 0));
    sim.factory.add_belt(line_misc - IVec3::new(1, 0, 0), EAST);
    sim.factory.set_box_label(misc, MISC);
    sim.factory.set_box_label(line_misc, MISC);
    sim.factory.stock(home, IRON_PLATE, 1);
    sim.factory.stock(misc, STEEL_PLATE, 1);
    sim.step();
    for (item, n) in [(IRON_PLATE, 10), (COPPER_WIRE, 8), (SCREW, 12), (IRON_INGOT, 5)] {
        sim.apply(P, Action::Give { item, count: n });
        order(&mut sim, Job::Store { item, count: n, slot: ANY_SLOT });
    }
    sim.step();
    let at_misc = drone(&sim).trip.as_ref().unwrap().stops.iter().filter(|s| s.at == misc).count();
    assert_eq!(at_misc, 2, "screws and ingots both go to the misc box on the one trip");
    assert_eq!(until_idle(&mut sim), vec![]);
    let f = &sim.factory;
    assert_eq!(f.box_count(home, IRON_PLATE), 11, "its home first, though the misc box is nearer");
    assert_eq!(f.box_count(free, COPPER_WIRE), 8, "an empty unlabelled box before the misc box");
    assert_eq!((f.box_count(misc, SCREW), f.box_count(misc, IRON_INGOT), f.box_count(misc, STEEL_PLATE)), (12, 5, 1));
    let on_line: u32 = [IRON_PLATE, COPPER_WIRE, SCREW, IRON_INGOT].iter().map(|&i| f.box_count(line_misc, i)).sum();
    assert_eq!(on_line, 0, "a misc label never opens a box on a belt line");
    // A put-away tops up what the misc box already holds, and never dumps a homeless item into it.
    let inv = &mut sim.players[0].as_mut().unwrap().inventory;
    inv.slots[HOTBAR_SLOTS] = Stack { item: SCREW, count: 3 };
    inv.slots[HOTBAR_SLOTS + 1] = Stack { item: STEEL_INGOT, count: 2 };
    order(&mut sim, Job::PutAway);
    until_idle(&mut sim);
    assert_eq!((sim.factory.box_count(misc, SCREW), pack(&sim, STEEL_INGOT)), (15, 2));
}

#[test]
fn a_fill_lands_in_its_slot_and_a_slot_store_takes_only_that_slot() {
    let mut sim = sim_with_drone();
    let b = place(&mut sim, STORAGE, IVec3::new(6, 0, 0));
    sim.factory.stock(b, IRON_PLATE, 200);
    let stack = item::stack_size(IRON_PLATE);
    let inv = &mut sim.players[0].as_mut().unwrap().inventory;
    inv.slots[12] = Stack { item: IRON_PLATE, count: 30 };
    inv.slots[15] = Stack { item: IRON_PLATE, count: 40 };
    order(&mut sim, Job::Fetch { item: IRON_PLATE, count: stack, slot: 20 });
    assert_eq!(until_idle(&mut sim), vec![]);
    let slots = &sim.player(P).unwrap().inventory.slots;
    assert_eq!((slots[20], slots[12].count), (Stack { item: IRON_PLATE, count: stack }, 30), "into slot 20 alone");
    order(&mut sim, Job::Store { item: IRON_PLATE, count: 40, slot: 15 });
    sim.step();
    let slots = &sim.player(P).unwrap().inventory.slots;
    assert_eq!((slots[15].count, slots[12].count, slots[20].count), (0, 30, stack), "only slot 15 left");
    until_idle(&mut sim);
    assert_eq!(sim.factory.box_count(b, IRON_PLATE), 200 - stack + 40);
    // A slot filled by something else meanwhile: the fetch lands in the pack instead, nothing lost.
    order(&mut sim, Job::Fetch { item: IRON_PLATE, count: 10, slot: 25 });
    sim.step();
    sim.players[0].as_mut().unwrap().inventory.slots[25] = Stack { item: COPPER_WIRE, count: 1 };
    until_idle(&mut sim);
    assert_eq!(pack(&sim, IRON_PLATE), 30 + stack + 10);
}

#[test]
fn put_away_leaves_the_hotbar_and_homeless_items_and_takes_the_rest_over_trips() {
    let mut sim = sim_with_drone();
    let plates = place(&mut sim, STORAGE, IVec3::new(5, 0, 0));
    let wires = place(&mut sim, STORAGE, IVec3::new(-5, 0, 3));
    sim.factory.stock(plates, IRON_PLATE, 1);
    sim.factory.stock(wires, COPPER_WIRE, 1);
    let inv = &mut sim.players[0].as_mut().unwrap().inventory;
    inv.slots[1] = Stack { item: IRON_PLATE, count: 40 };
    let stack = item::stack_size(COPPER_WIRE);
    for s in HOTBAR_SLOTS..HOTBAR_SLOTS + CARGO_STACKS as usize + 2 {
        inv.slots[s] = Stack { item: COPPER_WIRE, count: stack };
    }
    inv.slots[HOTBAR_SLOTS + 10] = Stack { item: IRON_PLATE, count: 25 };
    inv.slots[HOTBAR_SLOTS + 11] = Stack { item: STEEL_PLATE, count: 9 };
    order(&mut sim, Job::PutAway);
    assert_eq!(until_idle(&mut sim), vec![]);
    let inv = &sim.player(P).unwrap().inventory;
    assert_eq!(inv.slots[1], Stack { item: IRON_PLATE, count: 40 }, "the hotbar stays");
    assert_eq!((pack(&sim, COPPER_WIRE), pack(&sim, STEEL_PLATE)), (0, 9), "steel has no box of its own");
    assert_eq!(sim.factory.box_count(wires, COPPER_WIRE), 1 + stack * (CARGO_STACKS + 2), "two trips");
    assert_eq!(sim.factory.box_count(plates, IRON_PLATE), 26);
    order(&mut sim, Job::PutAway);
    assert_eq!(until_idle(&mut sim), vec![Note::NothingToPutAway]);
}

#[test]
fn it_fetches_what_a_craft_lacks_then_crafts_it() {
    let mut sim = sim_with_drone();
    let b = place(&mut sim, STORAGE, IVec3::new(7, 0, 0));
    sim.factory.stock(b, IRON_INGOT, 10);
    let screws = RECIPES.iter().position(|r| r.output == SCREW).unwrap() as u16;
    sim.factory.research.complete_all();
    order(&mut sim, Job::Craft { recipe: screws, times: 3 });
    let jobs = drone(&sim).jobs.clone();
    assert_eq!(
        jobs,
        vec![Job::Fetch { item: IRON_INGOT, count: 3, slot: ANY_SLOT }, Job::Craft { recipe: screws, times: 3 }]
    );
    until_idle(&mut sim);
    let crafts = &sim.player(P).unwrap().crafts;
    assert_eq!(crafts.orders.len(), 1, "the screws are on the bench, their rods first");
    assert_eq!(sim.factory.box_count(b, IRON_INGOT), 7);
    // Nothing in reach to make it from: refused with a note, nothing queued.
    order(&mut sim, Job::Craft { recipe: screws, times: 50 });
    assert!(drone(&sim).idle());
    assert!(sim.events.iter().any(|e| matches!(e, SimEvent::DroneNote { note: Note::CantCraft, .. })));
}

#[test]
fn leaving_mid_trip_brings_the_load_back_to_the_pack() {
    let mut sim = sim_with_drone();
    let b = place(&mut sim, STORAGE, IVec3::new(30, 0, 0));
    sim.apply(P, Action::Give { item: IRON_PLATE, count: 20 });
    sim.factory.stock(b, IRON_PLATE, 1);
    order(&mut sim, Job::Store { item: IRON_PLATE, count: 20, slot: ANY_SLOT });
    sim.step();
    assert_eq!(pack(&sim, IRON_PLATE), 0);
    sim.recall_drone(P);
    assert_eq!(pack(&sim, IRON_PLATE), 20);
    assert!(drone(&sim).trip.is_none());
}

#[test]
fn a_drone_saved_mid_trip_finishes_the_same_after_loading() {
    let mut sim = sim_with_drone();
    let b = place(&mut sim, STORAGE, IVec3::new(12, 3, -8));
    sim.factory.stock(b, IRON_PLATE, 90);
    sim.factory.stock(b, COPPER_WIRE, 1);
    sim.apply(P, Action::Give { item: STEEL_PLATE, count: 5 });
    sim.players[0].as_mut().unwrap().inventory.slots[HOTBAR_SLOTS + 4] = Stack { item: COPPER_WIRE, count: 12 };
    order(&mut sim, Job::Fetch { item: IRON_PLATE, count: 80, slot: 20 });
    order(&mut sim, Job::Store { item: STEEL_PLATE, count: 5, slot: ANY_SLOT });
    order(&mut sim, Job::PutAway);
    for _ in 0..20 {
        sim.step();
    }
    let d = drone(&sim).clone();
    assert!(d.trip.is_some() && d.jobs.len() == 2);
    let mut w = ByteWriter::default();
    d.write_state(&mut w);
    let mut r = ByteReader::new(&w.bytes);
    r.version = crate::save::SAVE_VERSION;
    assert_eq!(Drone::read_state(&mut r), Some(d));
    assert!(r.is_done());
}

/// Saved at version 70 by the commit before the drone took orders: a world (seed 2024) whose player carries a
/// personal drone that is half a second into fetching a stack of iron plates from a box holding 100, 20 blocks east.
const V70_SAVE: &[u8] = include_bytes!("v70.ocworld");

#[test]
fn an_errand_from_save_70_loads_as_its_fetch_and_brings_the_stack() {
    assert_eq!(u32::from_le_bytes(V70_SAVE[4..8].try_into().unwrap()), 70);
    let mut g = Game::load(V70_SAVE, 2).expect("loads");
    let d = &g.sim.player(g.local).unwrap().helpers.drone;
    assert_eq!(d.jobs, vec![Job::Fetch { item: IRON_PLATE, count: 64, slot: ANY_SLOT }]);
    g.run_ticks(15 * TICK_RATE);
    let p = g.sim.player(g.local).unwrap();
    assert_eq!((p.inventory.count(IRON_PLATE), p.helpers.drone.idle()), (64, true));
}

#[test]
fn every_order_round_trips_through_bytes() {
    let orders = [
        DroneOrder::Job(Job::Fetch { item: IRON_PLATE, count: 500, slot: ANY_SLOT }),
        DroneOrder::Job(Job::Fetch { item: IRON_PLATE, count: 64, slot: 20 }),
        DroneOrder::Job(Job::Store { item: COPPER_WIRE, count: u32::MAX, slot: 3 }),
        DroneOrder::Job(Job::PutAway),
        DroneOrder::Job(Job::Craft { recipe: 7, times: 3 }),
        DroneOrder::Cancel(4),
        DroneOrder::Follow,
    ];
    for order in orders {
        let a = Action::Drone { order, at: HOME };
        let mut w = ByteWriter::default();
        a.write(&mut w);
        let mut r = ByteReader::new(&w.bytes);
        assert_eq!((Action::read(&mut r), r.is_done()), (Some(a), true));
        for cut in 0..w.bytes.len() {
            assert_eq!(Action::read(&mut ByteReader::new(&w.bytes[..cut])), None);
        }
    }
}

#[test]
fn the_old_fetch_action_reads_as_a_fetch_of_a_stack() {
    let mut w = ByteWriter::default();
    w.u8(38);
    w.item(IRON_PLATE);
    w.ivec3(HOME);
    let read = crate::action::Action::read(&mut ByteReader::new(&w.bytes));
    let job = Job::Fetch { item: IRON_PLATE, count: item::stack_size(IRON_PLATE), slot: ANY_SLOT };
    assert_eq!(read, Some(Action::Drone { order: DroneOrder::Job(job), at: HOME }));
}
