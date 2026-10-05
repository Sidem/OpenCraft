//! Pumps, pipes and outlets in a whole core (`Sim`), on hand-built stone far from spawn, placed by
//! actions like a player places them.

use super::*;
use crate::action::Action;
use crate::block::{BlockId, AIR, COAL_ORE, GENERATOR, POLE, STONE, WATER};
use crate::bytes::{ByteReader, ByteWriter};
use crate::sim::{PlayerId, Sim};
use crate::worldgen::SEA_LEVEL;

const P: PlayerId = PlayerId(0);
const O: IVec3 = IVec3::new(4000, 0, 4000);
const EAST: u8 = 1;

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

fn count(sim: &mut Sim, lo: IVec3, hi: IVec3, b: BlockId) -> u32 {
    let mut n = 0;
    for y in lo.y..=hi.y {
        for z in lo.z..=hi.z {
            for x in lo.x..=hi.x {
                n += u32::from(sim.world.block_anywhere_or_generate(IVec3::new(x, y, z)) == b);
            }
        }
    }
    n
}

fn put(sim: &mut Sim, block: BlockId, pos: IVec3, facing: u8) {
    sim.apply(P, Action::Give { item: block.into(), count: 1 });
    sim.apply(P, Action::PlaceBlock { pos, slot: 0, facing, against: pos - IVec3::new(0, 1, 0) });
    assert_eq!(sim.world.block_anywhere_or_generate(pos), block);
}

/// A pump at `pump`, a pipe up to `top` and east along it to an outlet facing east at `outlet`; a pole
/// and a generator (with coal if `fuelled`) beside `pole`.
fn pipeline(sim: &mut Sim, pump: IVec3, top: IVec3, outlet: IVec3, pole: IVec3, fuelled: bool) {
    put(sim, PUMP, pump, 0);
    for y in pump.y + 1..=top.y {
        put(sim, PIPE, IVec3::new(pump.x, y, pump.z), 0);
    }
    for x in top.x + 1..outlet.x {
        put(sim, PIPE, IVec3::new(x, top.y, top.z), 0);
    }
    put(sim, OUTLET, outlet, EAST);
    put(sim, POLE, pole, 0);
    let gen = pole + IVec3::new(0, 0, -1);
    put(sim, GENERATOR, gen, 0);
    if fuelled {
        assert_eq!(sim.factory.insert(gen, COAL_ORE.into(), 64), 64);
    }
}

fn run(sim: &mut Sim, seconds: u32) {
    for _ in 0..seconds * crate::TICK_RATE {
        sim.step();
    }
}

const PIT: (IVec3, IVec3) = (IVec3::new(4002, 98, 4002), IVec3::new(4006, 100, 4006));
/// The cliff's foot: poured water piles up in the few columns within reach of the outlet's front.
const FOOT: (IVec3, IVec3) = (IVec3::new(4012, 85, 4001), IVec3::new(4029, 100, 4019));

/// A walled 5 × 5 × 3 pit of water (top at y 100) in stone, a pump in its bottom corner piped up and
/// east to an outlet at the edge of a cliff whose foot is 16 blocks down.
fn pit_over_a_cliff(fuelled: bool) -> Sim {
    let mut sim = Sim::new(1337, 2);
    fill(&mut sim, at(0, 80, 0), at(30, 105, 20), STONE);
    fill(&mut sim, at(0, 101, 0), at(30, 105, 20), AIR);
    fill(&mut sim, at(12, 85, 1), at(29, 100, 19), AIR);
    fill(&mut sim, PIT.0, PIT.1, WATER);
    pipeline(&mut sim, at(2, 98, 2), at(2, 101, 2), at(11, 101, 2), at(0, 101, 3), fuelled);
    sim
}

#[test]
fn a_pump_drains_a_pit_over_a_cliff() {
    let mut sim = pit_over_a_cliff(true);
    assert_eq!(count(&mut sim, PIT.0, PIT.1, WATER), 72, "75 less the pump and two pipes");
    run(&mut sim, 10);
    let pumped = 72 - count(&mut sim, PIT.0, PIT.1, WATER);
    assert!((17..=21).contains(&pumped), "about 2 a second: {pumped}");

    // A save made mid-drain carries on identically.
    let mut w = ByteWriter::default();
    sim.write_state(&mut w);
    let mut again = Sim::new(1337, 2);
    again.read_state(&mut ByteReader::new(&w.bytes)).expect("reads back");
    for _ in 0..40 * crate::TICK_RATE {
        sim.step();
        again.step();
    }
    assert_eq!(sim.state_hash(), again.state_hash());

    assert_eq!(count(&mut sim, PIT.0, PIT.1, WATER), 0, "the pit is empty");
    assert!((PIT.0.y..=PIT.1.y)
        .all(|y| (2..=6).all(|x| !crate::block::LIQUID[sim.world.block_anywhere_or_generate(at(x, y, 4)) as usize])));
    assert_eq!(count(&mut sim, FOOT.0, FOOT.1, WATER), 72, "every block landed at the cliff's foot");
    let pump = &sim.factory.pipework[0];
    assert_eq!((pump.part, pump.flow, pump.held), (Part::Pump, Flow::NoWater, 0));
}

#[test]
fn an_unpowered_pump_stops() {
    let mut sim = pit_over_a_cliff(false);
    run(&mut sim, 5);
    assert_eq!(count(&mut sim, PIT.0, PIT.1, WATER), 72);
    assert_eq!(sim.factory.pipework[0].flow, Flow::NoPower);
}

#[test]
fn a_pump_in_the_sea_runs_but_the_level_never_drops() {
    let mut sim = Sim::new(1337, 2);
    let sea = (at(1, 55, 1), at(10, SEA_LEVEL, 11));
    fill(&mut sim, at(0, 50, 0), at(60, SEA_LEVEL, 30), STONE);
    fill(&mut sim, at(0, SEA_LEVEL + 1, 0), at(60, SEA_LEVEL + 8, 30), AIR);
    fill(&mut sim, sea.0, sea.1, WATER);
    let top = at(5, SEA_LEVEL + 1, 5);
    // The outlet stands far enough inland that what it pours can't flow back into the sea.
    pipeline(&mut sim, at(5, 60, 5), top, at(30, SEA_LEVEL + 1, 5), at(5, SEA_LEVEL + 1, 3), true);
    let full = count(&mut sim, sea.0, sea.1, WATER);
    run(&mut sim, 20);
    let land = count(&mut sim, at(20, SEA_LEVEL + 1, 0), at(60, SEA_LEVEL + 1, 30), WATER);
    assert!(land >= 35, "the pump kept going: {land} blocks poured on land");
    assert!(count(&mut sim, sea.0, sea.1, WATER) + 1 >= full, "at most the block just taken is missing");
}

#[test]
fn an_outlet_on_open_ground_stops_once_its_hole_is_full() {
    // The sea-pump setup, run long enough to pour far more than fits: the pool stays within reach of
    // the outlet's front cell, no source rises above it, and the outlet reports it is blocked.
    let mut sim = Sim::new(1337, 2);
    fill(&mut sim, at(0, 50, 0), at(60, SEA_LEVEL, 30), STONE);
    fill(&mut sim, at(0, SEA_LEVEL + 1, 0), at(60, SEA_LEVEL + 8, 30), AIR);
    fill(&mut sim, at(1, 55, 1), at(10, SEA_LEVEL, 11), WATER);
    pipeline(&mut sim, at(5, 60, 5), at(5, SEA_LEVEL + 1, 5), at(30, SEA_LEVEL + 1, 5), at(5, SEA_LEVEL + 1, 3), true);
    run(&mut sim, 90);
    let front = at(31, SEA_LEVEL + 1, 5);
    let reach = crate::factory::pumping::POUR_REACH;
    let pool = (front - IVec3::new(reach, 0, reach), front + IVec3::new(reach, 0, reach));
    // The 7 × 7 patch around the front, less the two pipes and the outlet standing in it.
    assert_eq!(count(&mut sim, pool.0, pool.1, WATER), 46, "the patch is full");
    assert_eq!(count(&mut sim, at(12, SEA_LEVEL + 1, 0), at(60, SEA_LEVEL + 8, 30), WATER), 46, "nothing else");
    let outlet = sim.factory.pipework.iter().find(|p| p.part == Part::Outlet).unwrap();
    assert_eq!(outlet.flow, Flow::Blocked);
}

#[test]
fn an_outlet_will_not_start_a_source_in_dry_air_below_sea_level() {
    // The cliff of the first test, but its foot is below the sea: a lone source there would flood
    // everything connected, so the outlet waits instead.
    let mut sim = pit_over_a_cliff(true);
    fill(&mut sim, at(0, 50, 0), at(30, 79, 20), STONE);
    fill(&mut sim, at(12, 57, 1), at(29, 84, 19), AIR);
    run(&mut sim, 20);
    assert_eq!(count(&mut sim, at(12, 50, 1), at(29, 100, 19), WATER), 0);
    let outlet = sim.factory.pipework.iter().find(|p| p.part == Part::Outlet).unwrap();
    assert_eq!(outlet.flow, Flow::Blocked);
}

#[test]
fn pumps_lift_two_four_and_six_blocks_a_second_and_hold_as_many_for_more_power() {
    // A walled pit of water and a pump in its corner, but no outlet: the pump fills and stops.
    let held_after = |tier: u8, ticks: u32| {
        let mut sim = Sim::new(1337, 2);
        fill(&mut sim, at(0, 80, 0), at(30, 105, 20), STONE);
        fill(&mut sim, at(0, 101, 0), at(30, 105, 20), AIR);
        fill(&mut sim, PIT.0, PIT.1, WATER);
        put(&mut sim, PUMP, at(2, 98, 2), 0);
        put(&mut sim, POLE, at(0, 101, 3), 0);
        put(&mut sim, GENERATOR, at(0, 101, 2), 0);
        assert_eq!(sim.factory.insert(at(0, 101, 2), COAL_ORE.into(), 64), 64);
        (0..tier).for_each(|_| assert!(sim.factory.upgrade(at(2, 98, 2))));
        (0..ticks).for_each(|_| sim.step());
        let pump = &sim.factory.pipework[0];
        (pump.held, pump.pump_stats().hold, sim.factory.power.demand[0], pump.flow)
    };
    // Every 30, 15 or 10 ticks one more; 33 ticks in, that is 1, 2 or 3.
    assert_eq!(held_after(0, 33), (1, 2, 5, Flow::Working));
    assert_eq!(held_after(1, 33), (2, 4, 10, Flow::Working));
    assert_eq!(held_after(2, 33), (3, 6, 20, Flow::Working));
    // A second later they are all full.
    assert_eq!(held_after(0, 66), (2, 2, 0, Flow::Full));
    assert_eq!(held_after(1, 66), (4, 4, 0, Flow::Full));
    assert_eq!(held_after(2, 66), (6, 6, 0, Flow::Full));
}
