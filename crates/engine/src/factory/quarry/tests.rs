//! Quarries in a whole core (`Sim`), on hand-built ground far from spawn, placed and set by actions
//! like a player does it.

use super::*;
use crate::action::Action;
use crate::block::{
    BEDROCK, BELT, COAL_ORE, DIRT, GENERATOR, GRASS, LAMP, LOG, OUTLET, PIPE, POLE, PUMP, STONE, STORAGE, WATER,
};
use crate::bytes::{ByteReader, ByteWriter};
use crate::sim::{PlayerId, Sim};

const P: PlayerId = PlayerId(0);
const O: IVec3 = IVec3::new(-4000, 0, 4000);
const EAST: u8 = 1;
const WEST: u8 = 3;
/// The quarry's cell: on the ground, whose top (grass) is at y 100.
const Q: IVec3 = IVec3::new(-4000, 101, 4000);

fn at(x: i32, y: i32, z: i32) -> IVec3 {
    O + IVec3::new(x, y, z)
}

fn fill(sim: &mut Sim, lo: IVec3, hi: IVec3, b: BlockId) {
    for y in lo.y..=hi.y {
        for z in lo.z..=hi.z {
            for x in lo.x..=hi.x {
                sim.world.set_block_anywhere(IVec3::new(x, y, z), b);
            }
        }
    }
}

fn put(sim: &mut Sim, block: BlockId, pos: IVec3, facing: u8) {
    sim.apply(P, Action::Give { item: block.into(), count: 1 });
    sim.apply(P, Action::PlaceBlock { pos, slot: 0, facing, against: pos - IVec3::new(0, 1, 0) });
    assert_eq!(sim.world.block_anywhere_or_generate(pos), block);
}

fn run(sim: &mut Sim, seconds: u32) {
    for _ in 0..seconds * crate::TICK_RATE {
        sim.step();
    }
}

/// Stone up to y 99 under a layer of grass, air above; a quarry at `Q` facing east set to a 5 × 5 box
/// 8 deep (x 1..5, z -2..2, y 93..101), with a pole and a generator (with coal if `fuelled`) behind it.
fn quarry_on_flat_ground(fuelled: bool) -> Sim {
    let mut sim = Sim::new(1337, 2);
    fill(&mut sim, at(-12, 85, -8), at(30, 99, 8), STONE);
    fill(&mut sim, at(-12, 100, -8), at(30, 100, 8), GRASS);
    fill(&mut sim, at(-12, 101, -8), at(30, 110, 8), AIR);
    put(&mut sim, QUARRY, Q, EAST);
    sim.apply(P, Action::SetQuarry { pos: Q, width: 0, depth: 0, paused: false });
    put(&mut sim, POLE, at(0, 101, -3), 0);
    put(&mut sim, GENERATOR, at(-1, 101, -3), 0);
    if fuelled {
        assert_eq!(sim.factory.insert(at(-1, 101, -3), COAL_ORE.into(), 64), 64);
    }
    sim
}

/// Belts leading west from the quarry into a box.
fn belt_to_box(sim: &mut Sim) -> IVec3 {
    for x in -3..=-1 {
        put(sim, BELT, at(x, 101, 0), WEST);
    }
    put(sim, STORAGE, at(-4, 101, 0), 0);
    at(-4, 101, 0)
}

fn dug_cells(sim: &mut Sim) -> Vec<IVec3> {
    let dug = sim.events.iter().filter_map(|e| match *e {
        SimEvent::QuarryDug { pos, .. } => Some(pos),
        _ => None,
    });
    let dug = dug.collect();
    sim.events.clear();
    dug
}

fn quarry(sim: &Sim) -> &Quarry {
    &sim.factory.quarries[0]
}

#[test]
fn the_box_lies_in_front_and_runs_back_and_forth() {
    let dig = DigBox::new(IVec3::new(0, 70, 0), EAST, 0, 0);
    assert_eq!((dig.width, dig.layers(), dig.cells()), (5, 9, 225));
    assert_eq!(dig.bounds(), (IVec3::new(1, 62, -2), IVec3::new(5, 70, 2)));
    let first: Vec<IVec3> = (0..7).map(|i| dig.cell(i)).collect();
    assert_eq!(first[0], IVec3::new(1, 70, -2), "nearest row first");
    assert_eq!(first[4], IVec3::new(1, 70, 2));
    assert_eq!(first[5], IVec3::new(2, 70, 2), "the next row comes back");
    assert_eq!(first[6], IVec3::new(2, 70, 1));
    assert_eq!(dig.cell(25), IVec3::new(1, 69, -2), "then the next layer down");
    assert_eq!(DigBox::new(IVec3::new(0, 60, 0), 0, 1, 2).layers(), 0, "below sea level: nothing");
    assert_eq!(DigBox::new(IVec3::new(0, 70, 0), 0, 1, 3).bottom, 1, "to bedrock");
}

#[test]
fn a_quarry_digs_its_box_in_order_onto_a_belt() {
    let mut sim = quarry_on_flat_ground(true);
    let store = belt_to_box(&mut sim);
    run(&mut sim, 30);
    let mut dug = dug_cells(&mut sim);
    let done = dug.len();
    assert!((56..=62).contains(&done), "about 2 blocks a second: {done}");

    // A save made mid-dig carries on identically.
    let mut w = ByteWriter::default();
    sim.write_state(&mut w);
    let mut again = Sim::new(1337, 2);
    again.read_state(&mut ByteReader::new(&w.bytes)).expect("reads back");
    for _ in 0..80 * crate::TICK_RATE {
        sim.step();
        again.step();
    }
    assert_eq!(sim.state_hash(), again.state_hash());
    dug.extend(dug_cells(&mut sim));

    // Every ground cell of the box, in the box's order (its top layer is air), and nothing else.
    let dig = quarry(&sim).dig_box();
    let want: Vec<IVec3> = (0..dig.cells()).map(|i| dig.cell(i)).filter(|c| c.y <= 100).collect();
    assert_eq!(want.len(), 200);
    assert_eq!(dug, want);
    assert_eq!(quarry(&sim).status, QuarryStatus::Done);
    assert_eq!(quarry(&sim).dug, 200);
    assert_eq!(sim.world.block_anywhere_or_generate(at(3, 92, 0)), STONE, "it stops at its depth");
    assert_eq!(sim.world.block_anywhere_or_generate(at(6, 100, 0)), GRASS, "and at its sides");

    let slots = sim.factory.box_slots(store).unwrap();
    let count = |item: BlockId| slots.iter().filter(|s| s.item == item.into()).map(|s| s.count).sum::<u32>();
    assert_eq!((count(DIRT), count(STONE)), (25, 175), "the grass came as dirt");
}

#[test]
fn it_skips_ore_bedrock_and_what_is_not_ground() {
    let mut sim = quarry_on_flat_ground(true);
    let keep = [(at(1, 100, -2), LOG), (at(2, 99, 0), LAMP), (at(3, 97, 1), COAL_ORE), (at(4, 95, 2), BEDROCK)];
    for (p, b) in keep {
        sim.world.set_block_anywhere(p, b);
    }
    put(&mut sim, STORAGE, at(0, 101, 1), 0);
    run(&mut sim, 110);
    assert_eq!(quarry(&sim).status, QuarryStatus::Done);
    assert_eq!(quarry(&sim).dug, 196);
    for (p, b) in keep {
        assert_eq!(sim.world.block_anywhere_or_generate(p), b);
    }
}

#[test]
fn it_waits_while_paused_full_or_unpowered_then_carries_on() {
    let mut sim = quarry_on_flat_ground(false);
    run(&mut sim, 3);
    assert_eq!((quarry(&sim).status, quarry(&sim).dug), (QuarryStatus::NoPower, 0));
    assert_eq!(sim.factory.insert(at(-1, 101, -3), COAL_ORE.into(), 64), 64);
    run(&mut sim, 3);
    assert!(quarry(&sim).dug >= 5, "powered, it digs");

    sim.apply(P, Action::SetQuarry { pos: Q, width: 0, depth: 0, paused: true });
    let dug = quarry(&sim).dug;
    run(&mut sim, 3);
    assert_eq!((quarry(&sim).status, quarry(&sim).dug), (QuarryStatus::Paused, dug));
    sim.apply(P, Action::SetQuarry { pos: Q, width: 0, depth: 0, paused: false });

    // Four full stacks of something else leave no room for what it digs.
    for (i, item) in [LOG, LAMP, BEDROCK, COAL_ORE].into_iter().enumerate() {
        sim.factory.quarries[0].out.slots[i] = Stack { item: item.into(), count: 64 };
    }
    let dug = quarry(&sim).dug;
    run(&mut sim, 3);
    assert_eq!((quarry(&sim).status, quarry(&sim).dug), (QuarryStatus::OutputFull, dug));
    sim.apply(P, Action::TakeContents { pos: Q });
    run(&mut sim, 3);
    assert!(quarry(&sim).dug > dug, "emptied, it carries on");
}

#[test]
fn a_pit_that_breaks_into_a_pond_floods_until_a_pump_drains_it() {
    let mut sim = quarry_on_flat_ground(true);
    // A pond whose near edge is the box's far row (x 5), and a canyon to pour it into.
    let pond = (at(5, 98, -2), at(8, 100, 2));
    fill(&mut sim, pond.0, pond.1, WATER);
    fill(&mut sim, at(16, 86, -8), at(30, 100, 8), AIR);
    put(&mut sim, STORAGE, at(0, 101, 1), 0);
    run(&mut sim, 20);
    assert_eq!(quarry(&sim).status, QuarryStatus::Flooded);
    let dug = quarry(&sim).dug;
    assert!((15..=20).contains(&dug), "it stopped in the top layer: {dug}");
    run(&mut sim, 10);
    assert_eq!(quarry(&sim).dug, dug, "and waits");

    // A pump in the pond's far bottom corner, piped up and east to an outlet over the canyon.
    put(&mut sim, PUMP, at(8, 98, 2), 0);
    for y in 99..=101 {
        put(&mut sim, PIPE, at(8, y, 2), 0);
    }
    for x in 9..=15 {
        put(&mut sim, PIPE, at(x, 101, 2), 0);
    }
    put(&mut sim, OUTLET, at(16, 101, 2), EAST);
    put(&mut sim, POLE, at(7, 101, 3), 0);
    run(&mut sim, 60);
    let wet = |sim: &mut Sim| {
        let (lo, hi) = (at(1, 93, -2), at(8, 101, 2));
        let cells = (lo.y..=hi.y).flat_map(|y| (lo.z..=hi.z).flat_map(move |z| (lo.x..=hi.x).map(move |x| (x, y, z))));
        cells.filter(|&(x, y, z)| LIQUID[sim.world.block_anywhere_or_generate(IVec3::new(x, y, z)) as usize]).count()
    };
    assert_eq!(wet(&mut sim), 0, "the pond and the pit are dry");
    assert!(quarry(&sim).dug > dug + 20, "digging again: {}", quarry(&sim).dug);
}
