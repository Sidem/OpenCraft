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

fn water_mass(sim: &mut Sim, lo: IVec3, hi: IVec3) -> u32 {
    let mut n = 0u32;
    for y in lo.y..=hi.y {
        for z in lo.z..=hi.z {
            for x in lo.x..=hi.x {
                n += u32::from(crate::block::wet_of(sim.world.block_anywhere_or_generate(IVec3::new(x, y, z))));
            }
        }
    }
    n
}

fn put(sim: &mut Sim, block: BlockId, pos: IVec3, facing: u8) {
    if block == POLE {
        sim.world.set_block_anywhere(pos - IVec3::new(0, 1, 0), STONE); // a pole needs something to stand on
    }
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
/// The cliff's foot: a walled 6 × 6 catchment basin (72 blocks fill it 2 blocks deep with zero puddle evaporation).
const FOOT: (IVec3, IVec3) = (IVec3::new(4012, 85, 4001), IVec3::new(4017, 100, 4006));

/// A walled 5 × 5 × 3 pit of water (top at y 100) in stone, a pump in its bottom corner piped up and
/// east to an outlet at the edge of a cliff whose walled foot is 16 blocks down.
fn pit_over_a_cliff(fuelled: bool) -> Sim {
    let mut sim = Sim::new(1337, 2);
    fill(&mut sim, at(0, 80, 0), at(30, 105, 20), STONE);
    fill(&mut sim, at(0, 101, 0), at(30, 105, 20), AIR);
    fill(&mut sim, at(12, 85, 1), at(17, 100, 6), AIR);
    fill(&mut sim, PIT.0, PIT.1, WATER);
    pipeline(&mut sim, at(2, 98, 2), at(2, 101, 2), at(11, 101, 2), at(0, 101, 3), fuelled);
    sim
}

#[test]
fn a_pump_drains_a_pit_over_a_cliff() {
    let mut sim = pit_over_a_cliff(true);
    assert_eq!(water_mass(&mut sim, PIT.0, PIT.1), 72 * 8, "75 less the pump and two pipes");
    run(&mut sim, 10);
    let pumped = 72 * 8 - water_mass(&mut sim, PIT.0, PIT.1);
    assert!((14 * 8..=18 * 8).contains(&pumped), "about 1.62 a second at 3-block lift (81% pressure): {pumped}/8");

    // A save made mid-drain carries on identically.
    let mut w = ByteWriter::default();
    sim.write_state(&mut w);
    let mut again = Sim::new(1337, 2);
    again.read_state(&mut ByteReader::new(&w.bytes)).expect("reads back");
    for _ in 0..55 * crate::TICK_RATE {
        sim.step();
        again.step();
    }
    assert_eq!(sim.state_hash(), again.state_hash());

    assert_eq!(water_mass(&mut sim, PIT.0, PIT.1), 0, "the pit is empty");
    assert!((PIT.0.y..=PIT.1.y)
        .all(|y| (2..=6).all(|x| !crate::block::LIQUID[sim.world.block_anywhere_or_generate(at(x, y, 4)) as usize])));
    assert_eq!(water_mass(&mut sim, FOOT.0, FOOT.1), 72 * 8, "every eighth landed at the cliff's foot");
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
    let land = water_mass(&mut sim, at(12, SEA_LEVEL + 1, 0), at(60, SEA_LEVEL + 1, 30));
    assert!(land >= 24 * 8, "the pump kept going at 81% pressure: {land}/8 blocks poured on land");
    assert!(count(&mut sim, sea.0, sea.1, WATER) + 1 >= full, "at most the block just taken is missing");
}

#[test]
fn an_outlet_in_a_basin_stops_once_its_hole_is_full() {
    // A walled 4 × 5 basin within `POUR_REACH` of the outlet's front cell: once filled to the spout level, nothing
    // spills over the walls and the outlet reports it is blocked.
    let mut sim = Sim::new(1337, 2);
    fill(&mut sim, at(0, 50, 0), at(60, SEA_LEVEL, 30), STONE);
    fill(&mut sim, at(0, SEA_LEVEL + 1, 0), at(60, SEA_LEVEL + 8, 30), AIR);
    fill(&mut sim, at(1, 55, 1), at(10, SEA_LEVEL, 11), WATER);
    // Walled 4 × 5 basin at x = 31..=34, z = 3..=7, walls at x = 30/35 and z = 2/8.
    fill(&mut sim, at(30, SEA_LEVEL + 1, 2), at(35, SEA_LEVEL + 1, 8), STONE);
    fill(&mut sim, at(31, SEA_LEVEL + 1, 3), at(34, SEA_LEVEL + 1, 7), AIR);
    sim.world.set_block_anywhere(at(30, SEA_LEVEL + 1, 5), AIR);
    pipeline(&mut sim, at(5, 60, 5), at(5, SEA_LEVEL + 1, 5), at(30, SEA_LEVEL + 1, 5), at(5, SEA_LEVEL + 1, 3), true);
    run(&mut sim, 40);
    let pool = (at(31, SEA_LEVEL + 1, 3), at(34, SEA_LEVEL + 1, 7));
    assert_eq!(count(&mut sim, pool.0, pool.1, WATER), 20, "the 4 × 5 basin is full");
    assert_eq!(water_mass(&mut sim, at(12, SEA_LEVEL + 1, 0), at(60, SEA_LEVEL + 8, 30)), 20 * 8, "nothing outside");
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

#[test]
fn pipe_height_reduces_pressure_increases_power_and_stalls_above_max_head() {
    let measure = |lift: i32, tier: u8| {
        let mut sim = Sim::new(1337, 2);
        fill(&mut sim, at(0, 80, 0), at(30, 105, 20), STONE);
        fill(&mut sim, at(0, 101, 0), at(30, 130, 20), AIR);
        fill(&mut sim, at(12, 85, 1), at(29, 100, 19), AIR);
        fill(&mut sim, PIT.0, PIT.1, WATER);
        let pump_pos = at(2, 98, 2);
        let top_y = pump_pos.y + lift;
        pipeline(&mut sim, pump_pos, at(2, top_y, 2), at(11, top_y, 2), at(0, 101, 3), true);
        // Add generators so high lift (up to 400 kW on Mk2) has full grid power.
        for gz in 4..=10 {
            put(&mut sim, GENERATOR, at(0, 101, gz), 0);
            assert_eq!(sim.factory.insert(at(0, 101, gz), COAL_ORE.into(), 64), 64);
        }
        put(&mut sim, POLE, at(1, 101, 7), 0);
        (0..tier).for_each(|_| assert!(sim.factory.upgrade(pump_pos)));
        run(&mut sim, 10);
        let pumped = 72 * 8 - water_mass(&mut sim, PIT.0, PIT.1);
        let pump = &sim.factory.pipework[0];
        let outlet = sim.factory.pipework.iter().find(|p| p.part == Part::Outlet).unwrap();
        (pump.pressure, pump.power_kw(), pump.flow, outlet.flow, pumped)
    };
    let (p_low, kw_low, f_low, _, vol_low) = measure(3, 0);
    let (p_high, kw_high, f_high, _, vol_high) = measure(9, 0);
    assert_eq!((f_low, f_high), (Flow::Working, Flow::Working));
    assert!(p_low > p_high, "higher pipes have lower pressure: {p_low} vs {p_high}");
    assert!(kw_high > kw_low, "higher lift costs more power: {kw_high} vs {kw_low}");
    assert!(vol_low > vol_high, "higher lift pumps less water: {vol_low} vs {vol_high}");

    // Above Mk1 max_head (12): stalls with TooHigh and pumps nothing.
    let (p_over, _, f_over, of_over, vol_over) = measure(14, 0);
    assert_eq!((p_over, f_over, of_over, vol_over), (0, Flow::TooHigh, Flow::TooHigh, 0));

    // Upgrading to Mk2 (max_head 24) restores pressure and flow at lift 14!
    let (p_mk2, _, f_mk2, _, vol_mk2) = measure(14, 1);
    assert_eq!(f_mk2, Flow::Working);
    assert!(p_mk2 > 500 && vol_mk2 > 15 * 8, "Mk2 handles lift 14: p={p_mk2}, vol={vol_mk2}");
}

#[test]
fn outlet_creates_a_live_falling_stream_that_dries_when_pump_stops() {
    let mut sim = pit_over_a_cliff(true);
    run(&mut sim, 3);
    // While pouring, a real conserved falling-water cascade hangs directly in front of the outlet down the cliff.
    let front = at(12, 101, 2);
    assert!(crate::block::flow_level(sim.world.block_anywhere_or_generate(front)).is_some(), "live spout stream");
    assert!(
        crate::block::flow_level(sim.world.block_anywhere_or_generate(at(12, 99, 2))).is_some(),
        "falling waterfall column is continuous"
    );

    // Once the pit runs dry, the outlet stops and the falling waterfall dries back to air.
    run(&mut sim, 55);
    assert_eq!(sim.world.block_anywhere_or_generate(front), AIR, "spout stream dried up");
    assert_eq!(sim.world.block_anywhere_or_generate(at(12, 99, 2)), AIR, "falling column dried up");
}

#[test]
fn rotating_a_placed_outlet_turns_its_spout_and_drains_the_old_stream() {
    let mut sim = pit_over_a_cliff(true);
    let outlet_pos = at(11, 101, 2);
    run(&mut sim, 3);
    assert!(crate::block::flow_level(sim.world.block_anywhere_or_generate(at(12, 101, 2))).is_some());
    assert_eq!(sim.factory.pipework.iter().find(|p| p.pos == outlet_pos).unwrap().facing, EAST);

    // Rotate 90° clockwise: EAST (1) -> SOUTH (2).
    sim.apply(P, Action::Rotate { pos: outlet_pos });
    assert_eq!(sim.factory.pipework.iter().find(|p| p.pos == outlet_pos).unwrap().facing, 2);
    run(&mut sim, 2);
    assert_eq!(sim.world.block_anywhere_or_generate(at(12, 101, 2)), AIR, "old east stream dried up");
}

#[test]
fn an_unreleased_save_89_with_a_level_byte_per_pipe_still_reads() {
    let mut sim = pit_over_a_cliff(true);
    run(&mut sim, 2);
    let mut plain = ByteWriter::default();
    let mut padded = ByteWriter::default();
    for p in &sim.factory.pipework {
        p.write_state(&mut plain);
        p.write_state(&mut padded);
        padded.u8(5); // the stream level that build wrote
    }
    let read = |bytes: &[u8], pad: bool| {
        let mut r = ByteReader::new(bytes);
        (r.version, r.pipe_level_byte) = (89, pad);
        let list: Vec<_> = sim.factory.pipework.iter().map(|_| Pipework::read_state(&mut r).unwrap()).collect();
        assert!(r.is_done());
        list.iter().map(|p| (p.pos, p.part, p.facing, p.progress, p.held, p.tier)).collect::<Vec<_>>()
    };
    assert_eq!(read(&padded.bytes, true), read(&plain.bytes, false));
}
