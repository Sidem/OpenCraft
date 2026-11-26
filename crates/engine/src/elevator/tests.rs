//! The space elevator's core: where an anchor stands, sections as orders, the climbers' draw, waiting with its town,
//! sleeping and waking, and its bytes.

use super::*;
use crate::action::Action;
use crate::block::TOWN_DEPOT;
use crate::item::ELEVATOR_ANCHOR_ITEM;
use crate::research::ids::SPACE_ELEVATOR;
use crate::research::{Ring, Unlock, TECHS};
use crate::settlements::{Town, WORKSHOP};
use crate::sim::PlayerId;

const P: PlayerId = PlayerId(0);
const TOWN: IVec3 = IVec3::new(3000, 120, -3000);
const SITE: IVec3 = IVec3::new(3020, 120, -3000);
const DOWN: IVec3 = IVec3::new(0, 1, 0);

/// A world with one highland workshop of `built` stages at `TOWN` that never runs out of upkeep and builds nothing.
fn town(built: u8) -> Sim {
    let mut sim = Sim::new(2024, 2);
    sim.factory.place(&mut sim.world, TOWN_DEPOT, TOWN, 0, TOWN - DOWN, 0);
    let mut t = Town::new(TOWN, WORKSHOP);
    (t.built, t.planned, t.due) = (built, built, u64::MAX);
    t.delivered = vec![0; t.order().len()];
    sim.settlements.towns.push(t);
    sim.step();
    sim
}

/// Places an anchor from the player's hand at `at`; whether one stands there now.
fn place(sim: &mut Sim, at: IVec3) -> bool {
    sim.apply(P, Action::Give { item: ELEVATOR_ANCHOR_ITEM, count: 1 });
    sim.apply(P, Action::PlaceBlock { pos: at, slot: 0, facing: 0, against: at - DOWN });
    sim.factory.occupies(at)
}

/// A town with an anchor at `SITE` whose open section misses only `left` of each line.
fn climbing(built: u8, left: u32) -> Sim {
    let mut sim = town(MIN_BUILT);
    assert!(place(&mut sim, SITE));
    let e = sim.elevator.as_mut().unwrap();
    e.built = built;
    e.delivered = e.order().iter().map(|l| l.1 - left).collect();
    // As when the section below rose: the climbers' first minute is due a minute on.
    e.due = sim.tick + MINUTE;
    sim.step();
    sim
}

fn elevator(sim: &Sim) -> &Elevator {
    sim.elevator.as_ref().unwrap()
}

/// Puts `n` of every line of `lines` into the anchor; what went in.
fn feed(sim: &mut Sim, lines: &[Line], n: u32) -> u32 {
    lines.iter().map(|&(item, _)| sim.factory.insert(SITE, item, n)).sum()
}

#[test]
fn an_anchor_stands_only_by_a_town_or_a_city_and_once_per_world() {
    let mut sim = town(MIN_BUILT - 1);
    assert_eq!(sim.elevator_site(SITE), Err(Unfit::NoTown), "a village is not enough");
    assert!(!place(&mut sim, SITE));
    assert!(sim.elevator.is_none());

    let mut sim = town(MIN_BUILT);
    let far = TOWN + IVec3::new(TOWN_REACH + 1, 0, 0);
    assert_eq!(sim.elevator_site(far), Err(Unfit::NoTown), "too far from its depot");
    assert_eq!(sim.elevator_site(SITE), Ok(0));
    assert!(place(&mut sim, SITE));
    assert_eq!(elevator(&sim).built, 0);
    let other = SITE + IVec3::new(0, 0, 20);
    assert_eq!(sim.elevator_site(other), Err(Unfit::Taken), "one per world");
    assert!(!place(&mut sim, other));

    // Broken, it sleeps with what it has; it wakes only on its own spot.
    sim.elevator.as_mut().unwrap().built = 2;
    sim.apply(P, Action::BreakBlock { pos: SITE });
    assert!(!sim.factory.occupies(SITE));
    assert!(elevator(&sim).dormant);
    assert_eq!(sim.elevator_site(other), Err(Unfit::Taken));
    assert!(place(&mut sim, SITE));
    assert!(!elevator(&sim).dormant);
    assert_eq!(elevator(&sim).built, 2, "it kept its sections");
}

#[test]
fn a_complete_order_raises_the_tether_a_section() {
    let mut sim = climbing(0, 10);
    assert!(elevator(&sim).wants().iter().all(|&(_, n)| n == 10), "it asks for what the order misses");
    assert_eq!(feed(&mut sim, ORDERS[0], 10), 10 * ORDERS[0].len() as u32);
    sim.step();
    let e = elevator(&sim);
    assert_eq!(e.built, 1);
    assert_eq!(e.delivered, vec![0; ORDERS[1].len()], "the next section's order opens");
    assert!(sim.events.contains(&SimEvent::ElevatorRose { pos: SITE, built: 1 }));
    assert_eq!(sim.factory.depot_held(SITE).unwrap().total(), 0, "all of it went into the section");
    assert_eq!(SECTIONS as i32 * SECTION_HEIGHT, 512, "phase 1 stands 512 blocks tall");
}

#[test]
fn the_climbers_draw_a_minute_of_goods_or_the_order_waits() {
    let mut sim = climbing(1, 5);
    // The first minute is due a minute after the section rose; until then nothing is drawn.
    sim.elevator.as_mut().unwrap().due = sim.tick;
    sim.step();
    assert!(elevator(&sim).stalled, "nothing to draw yet");
    let wants = elevator(&sim).wants();
    assert!(wants.iter().all(|w| DRAW.iter().any(|d| d.0 == w.0)), "stalled, it takes only what ends the stall");
    sim.step(); // the anchor hears what it wants at the start of a tick
    assert_eq!(
        feed(&mut sim, ORDERS[1], 5),
        DRAW.iter().filter(|d| ORDERS[1].iter().any(|l| l.0 == d.0)).count() as u32 * 5
    );

    // A minute of the draw ends the stall; it is used up, and its rocket fuel gives its empty barrels back.
    for &(item, n) in DRAW {
        sim.factory.insert(SITE, item, n);
    }
    sim.step();
    assert!(!elevator(&sim).stalled);
    let fuel = DRAW.iter().find(|d| d.0 == ROCKET_FUEL_BARREL).unwrap().1;
    let (_, side) = sim.factory.depot_perk(SITE).unwrap();
    assert_eq!(side.count(EMPTY_BARREL), fuel);
    assert_eq!(elevator(&sim).due, sim.tick - 1 + MINUTE, "the next minute is due a minute on");
}

#[test]
fn the_elevator_waits_while_its_town_is_stalled() {
    let mut sim = climbing(0, 10);
    sim.settlements.towns[0].stalled = true;
    feed(&mut sim, ORDERS[0], 10);
    (0..5).for_each(|_| sim.step());
    assert_eq!(elevator(&sim).built, 0, "its town is stalled");
    sim.settlements.towns[0].stalled = false;
    sim.step();
    assert_eq!(elevator(&sim).built, 1);
}

#[test]
fn a_finished_phase_draws_nothing_and_asks_for_nothing() {
    let mut sim = climbing(SECTIONS as u8 - 1, 1);
    feed(&mut sim, ORDERS[SECTIONS - 1], 1);
    for &(item, n) in DRAW {
        sim.factory.insert(SITE, item, n);
    }
    sim.step();
    let e = elevator(&sim);
    assert!(e.done());
    assert!(e.draw().is_empty() && e.wants().is_empty() && e.order().is_empty());
}

#[test]
fn the_elevator_survives_a_save() {
    let mut sim = climbing(3, 100);
    sim.elevator.as_mut().unwrap().stalled = true;
    let mut w = ByteWriter::default();
    sim.write_state(&mut w);
    let mut back = Sim::new(2024, 2);
    back.read_state(&mut ByteReader::new(&w.bytes)).expect("loads");
    assert_eq!(back.elevator, sim.elevator);

    // Deliveries that don't fit the section's order don't load.
    let mut e = elevator(&sim).clone();
    e.delivered.push(0);
    let mut w = ByteWriter::default();
    e.write(&mut w);
    assert!(Elevator::read(&mut ByteReader::new(&w.bytes)).is_none());
}

#[test]
fn the_space_elevator_tech_opens_the_anchor_in_the_sky_ring() {
    let tech = &TECHS[SPACE_ELEVATOR as usize];
    assert_eq!(tech.name, "Space Elevator");
    assert_eq!(tech.ring, Ring::Sky);
    assert!(tech.unlocks.contains(&Unlock::Recipe(ELEVATOR_ANCHOR_ITEM)));
}
