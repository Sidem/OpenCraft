//! The datacenter in a bare factory: heat and coolant (driven directly, as the power share is a number), the compute it
//! gives a node-hung grid on a brownout-sized power supply, and the save.

use super::*;
use crate::block::{COAL_ORE, FIBRE_NODE, GENERATOR, POLE};
use crate::bytes::{ByteReader, ByteWriter};
use crate::factory::{Factory, Machine};
use crate::math::IVec3;
use crate::world::World;

const AT: IVec3 = IVec3::new(0, 0, 0);

fn hall() -> Processor {
    Processor::new(AT, &DATACENTER_SPEC, 0)
}

/// Ticks at full power, keeping the tank at `water` units (0: no coolant), and returns the ticks it ran before tripping.
fn run_full(p: &mut Processor, ticks: u32, water: u32) -> u32 {
    for t in 0..ticks {
        p.steam.water = water * COOLANT_UNIT;
        p.run_datacenter(1000);
        if p.status == Status::Overheated {
            return t;
        }
    }
    ticks
}

#[test]
fn the_spec_is_a_4x4x3_hall_with_a_water_inlet_that_draws_3_mw() {
    assert_eq!(DATACENTER_SPEC.footprint.size, [4, 4, 3]);
    assert_eq!(DATACENTER_SPEC.footprint.ports.len(), 1);
    assert_eq!(DATACENTER_SPEC.compute, 100);
    assert_eq!(DATACENTER_SPEC.tiers[0].power, 3_000);
    assert!(hall().datacenter_wants_power());
}

#[test]
fn without_coolant_it_trips_after_about_twenty_seconds_and_stops_drawing() {
    let mut p = hall();
    let ran = run_full(&mut p, 60 * TICK_RATE, 0);
    assert!((20 * TICK_RATE..=22 * TICK_RATE).contains(&ran), "tripped after {ran} ticks");
    assert_eq!(p.status, Status::Overheated);
    assert!(!p.datacenter_wants_power(), "a tripped hall draws nothing");
    assert!(p.datacenter_text().unwrap().starts_with("Overheated and shut down"));
}

#[test]
fn it_restarts_once_the_heat_has_halved() {
    let mut p = hall();
    run_full(&mut p, 60 * TICK_RATE, 0);
    let mut ticks = 0;
    while p.status == Status::Overheated && ticks < 600 * TICK_RATE {
        p.run_datacenter(0);
        ticks += 1;
    }
    assert!(ticks > 0 && p.status != Status::Overheated, "still hot after {ticks} ticks");
    assert!(p.progress <= HEAT_LIMIT / 2);
}

#[test]
fn coolant_keeps_it_cool_and_is_spent_by_the_load() {
    let mut p = hall();
    p.steam.water = 3 * COOLANT_UNIT;
    for _ in 0..10 * TICK_RATE {
        p.run_datacenter(1000);
    }
    assert_eq!(p.steam.water, 2 * COOLANT_UNIT, "ten seconds of full load spend one unit");
    assert_eq!((p.status, p.progress), (Status::Working, 0), "cooled: no heat builds");
    p.steam.water = COOLANT_UNIT;
    for _ in 0..5 * TICK_RATE {
        p.run_datacenter(500);
    }
    assert_eq!(p.steam.water, COOLANT_UNIT - COOLANT_UNIT / 4, "half load for five seconds spends a quarter");
}

#[test]
fn no_power_is_not_an_overheat() {
    let mut p = hall();
    p.run_datacenter(0);
    assert_eq!(p.status, Status::NoPower);
    assert!(p.datacenter_wants_power());
}

fn grid() -> (Factory, World) {
    let mut f = Factory::default();
    let mut world = World::new(1, 2);
    f.place(&mut world, POLE, IVec3::new(6, 0, 2), 0, IVec3::new(6, 0, 2), 0);
    for x in 0..3 {
        let g = IVec3::new(8 + x, 0, 2);
        f.place(&mut world, GENERATOR, g, 0, g, 0);
        f.insert(g, COAL_ORE.into(), 64);
    }
    f.place(&mut world, FIBRE_NODE, IVec3::new(4, 0, 2), 0, IVec3::new(4, 0, 2), 0);
    f.place(&mut world, DATACENTER, AT, 0, AT - IVec3::new(0, 1, 0), 0);
    (f, world)
}

#[test]
fn it_gives_the_data_grid_compute_until_it_overheats() {
    let (mut f, mut world) = grid();
    let mut events = Vec::new();
    for t in 0..2 * TICK_RATE {
        f.update(&mut world, t as u64, &mut events);
    }
    let first = f.data.supply[0];
    assert!(first > 0 && first <= 100, "supplies {first} TF");
    assert_eq!(f.data.process_node[0], Some(0), "hangs on the node");
    // Tripped: nothing supplied.
    let p = f.processors.iter_mut().find(|p| p.spec.block == DATACENTER).unwrap();
    p.status = Status::Overheated;
    p.progress = HEAT_LIMIT;
    f.update(&mut world, 99, &mut events);
    assert_eq!(f.data.supply[0], 0);
}

#[test]
fn heat_and_coolant_survive_a_save() {
    let mut p = hall();
    run_full(&mut p, 60 * TICK_RATE, 0);
    p.steam.water = 77;
    let mut w = ByteWriter::default();
    p.write_state(&mut w);
    let back = Processor::read_state(&mut ByteReader::new(&w.bytes)).unwrap();
    assert_eq!((back.status, back.progress, back.steam.water), (Status::Overheated, p.progress, 77));
    assert_eq!(back.spec.block, DATACENTER);
}

/// Benchmark (`cargo test --release bench_compute -- --ignored --nocapture`): a compute plant on one power grid with
/// 6 datacenters, 8 AI labs, 4 optimizers, 4 swarm hubs, 6 Mk4 drone ports, 14 fibre nodes, 20 coal generators and 4 laser
/// links with a mirror each, 3000 ticks.
#[test]
#[ignore]
fn bench_compute() {
    use crate::block::{AI_LAB, DRONE_PORT, LASER_EMITTER, LASER_MIRROR, LASER_RECEIVER, OPTIMIZER, SWARM_HUB};
    let (mut world, mut f) = (World::new(1, 2), Factory::default());
    let mut put = |f: &mut Factory, block, x: i32, z: i32, facing: u8| {
        let pos = IVec3::new(x, 100, z);
        let tier = if block == POLE || block == DRONE_PORT { 3 } else { 0 };
        f.place(&mut world, block, pos, facing, pos - IVec3::new(0, 1, 0), tier);
    };
    for i in 0..6 {
        put(&mut f, DATACENTER, i * 10, 0, 0);
        put(&mut f, DRONE_PORT, i * 10, 40, 0);
    }
    for i in 0..8 {
        put(&mut f, AI_LAB, i * 8, 16, 0);
    }
    for i in 0..4 {
        put(&mut f, OPTIMIZER, i * 12, 24, 0);
        put(&mut f, SWARM_HUB, i * 12, 32, 0);
        // A link 20 blocks long with a mirror at its far end turning the beam east, into a receiver.
        put(&mut f, LASER_EMITTER, 70 + i * 6, 60, 0);
        put(&mut f, LASER_MIRROR, 70 + i * 6, 40, 0);
        put(&mut f, LASER_RECEIVER, 74 + i * 6, 40, 0);
    }
    for i in 0..14 {
        put(&mut f, FIBRE_NODE, i * 8, 12, 0);
    }
    for i in 0..20 {
        put(&mut f, GENERATOR, i * 3, -8, 0);
    }
    for x in (0..120).step_by(8) {
        for z in (-8..72).step_by(8) {
            put(&mut f, POLE, x + 4, z + 4, 0);
        }
    }
    let mut events = Vec::new();
    let (mut worst, mut at, start) = (std::time::Duration::ZERO, 0, std::time::Instant::now());
    for tick in 0..3000u64 {
        let t = std::time::Instant::now();
        f.update(&mut world, tick, &mut events);
        if tick > 0 && t.elapsed() > worst {
            (worst, at) = (t.elapsed(), tick);
        }
    }
    println!(
        "compute: {} processors, {:?} a tick on average, {worst:?} worst (tick {at})",
        f.processors.len(),
        start.elapsed() / 3000
    );
}
