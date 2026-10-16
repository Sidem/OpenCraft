//! Bench of an orbit plant (`cargo test --release bench_orbit -- --ignored --nocapture`): step 12.5's worst tick with every
//! satellite kind at its cap overhead, the comms uplink rejoining the data grids each tick, rectennas and ground stations
//! sensing their satellites, and pod launchers sending pods that wait on full pads. Record the numbers in the plan.

use std::time::{Duration, Instant};

use crate::block::{
    DRONE_PORT, FIBRE_NODE, GROUND_STATION, LANDING_PAD, LAUNCH_SITE, OPTIMIZER, POD_LAUNCHER, POLE, RECTENNA,
};
use crate::factory::Factory;
use crate::inventory::Stack;
use crate::item::{COMMS_SATELLITE, DROP_POD, IRON_PLATE, POWER_SATELLITE, SCIENCE_SATELLITE, SURVEY_SATELLITE};
use crate::math::IVec3;
use crate::recipes::MACHINE_RECIPES;
use crate::satellites::MAX_PER_KIND;
use crate::world::World;

const TICKS: u64 = 3000;

#[test]
#[ignore]
fn bench_orbit() {
    let (mut world, mut f) = (World::new(1, 2), Factory::default());
    f.research.complete_all();
    let mut put = |f: &mut Factory, block, x: i32, z: i32, facing: u8| {
        let pos = IVec3::new(x, 100, z);
        let tier = if block == POLE || block == DRONE_PORT { 3 } else { 0 };
        f.place(&mut world, block, pos, facing, pos - IVec3::new(0, 1, 0), tier);
    };
    for i in 0..2 {
        put(&mut f, LAUNCH_SITE, i * 10, -20, 0);
    }
    for i in 0..8 {
        put(&mut f, RECTENNA, i * 7, 0, 0);
        put(&mut f, DRONE_PORT, i * 6, 40, 0);
        put(&mut f, POD_LAUNCHER, i * 5, 50, (i % 4) as u8);
        put(&mut f, LANDING_PAD, i * 5, 60, (i % 4) as u8);
    }
    for i in 0..4 {
        put(&mut f, GROUND_STATION, i * 6, 10, 0);
        put(&mut f, OPTIMIZER, i * 12, 24, 0);
    }
    // Fibre nodes in two islands 60 blocks apart, so only the comms uplink joins them.
    for i in 0..15 {
        put(&mut f, FIBRE_NODE, i * 4, 16, 0);
        put(&mut f, FIBRE_NODE, i * 4, 80, 0);
    }
    // Poles in the gaps between the rows (`place` clears a cell first, so a pole on a machine would remove it).
    for x in (3..64).step_by(8) {
        for z in [-10, 6, 20, 32, 45, 55, 66] {
            put(&mut f, POLE, x, z, 0);
        }
    }
    assert_eq!(f.processors.len(), 42, "every machine stands");
    // Every kind at its cap, launched over the plant over the last minute.
    for (k, item) in [SURVEY_SATELLITE, COMMS_SATELLITE, POWER_SATELLITE, SCIENCE_SATELLITE].into_iter().enumerate() {
        for i in 0..MAX_PER_KIND as u64 {
            f.orbit.launch(item, IVec3::new(30, 100, 30), (k as u64 * MAX_PER_KIND as u64 + i) % 60);
        }
    }
    // Launchers stocked with pods and cargo; the pads have no belts, so they fill and later pods wait in the sky.
    let all = [true; MACHINE_RECIPES.len()];
    for p in f.processors.iter_mut().filter(|p| p.spec.block == POD_LAUNCHER) {
        p.insert(DROP_POD, 10, &all);
        (0..12).for_each(|_| _ = p.insert(IRON_PLATE, 64, &all));
    }
    let pads: Vec<(IVec3, u8)> =
        f.processors.iter().filter(|p| p.spec.block == LANDING_PAD).map(|p| (p.pos, p.dir)).collect();
    for i in 0..40 {
        let (to, channel) = pads[i % pads.len()];
        f.pods.send(IVec3::ZERO, to, channel, vec![Stack { item: IRON_PLATE, count: 64 }; 6], 60 + i as u64 * 20);
    }
    let mut events = Vec::new();
    let (mut worst, mut at, start) = (Duration::ZERO, 0, Instant::now());
    for tick in 60..60 + TICKS {
        let t = Instant::now();
        f.update(&mut world, tick, &mut events);
        if tick > 60 && t.elapsed() > worst {
            (worst, at) = (t.elapsed(), tick);
        }
        events.clear();
    }
    println!(
        "orbit: {} processors ({} wired) on {} grids, {} satellites, {} pods in flight at the end; {:?} a tick on average, \
         {worst:?} worst (tick {at})",
        f.processors.len(),
        f.power.process_pole.iter().flatten().count(),
        f.power.supply.len(),
        f.orbit.count(0) + f.orbit.count(1) + f.orbit.count(2) + f.orbit.count(3),
        f.pods.inbound(0) + f.pods.inbound(1) + f.pods.inbound(2) + f.pods.inbound(3),
        start.elapsed() / TICKS as u32
    );
}
