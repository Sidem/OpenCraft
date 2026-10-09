//! Bench of a large base (`cargo test --release bench_big_base -- --ignored --nocapture`): the costs the performance
//! review (DEV_PLAN section 4, P1–P5) targets. Prints microseconds; record before/after numbers in the plan.

use std::time::{Duration, Instant};

use crate::block::{BELT, POLE, SMELTER};
use crate::math::{IVec3, Vec3};
use crate::world::World;

use crate::factory::power::Power;
use crate::factory::Factory;

const ROWS: i32 = 100;
const BELTS_PER_ROW: i32 = 300;

fn us(d: Duration) -> u128 {
    d.as_micros()
}

#[test]
#[ignore]
fn bench_big_base() {
    let (mut world, mut f) = (World::new(1, 2), Factory { by_hand: true, ..Factory::default() }); // by_hand: no test-only auto-wiring
    let y = 100;
    // 3000 belts in rows, 300 smelters beside them, a pole every 10 cells.
    for z in 0..ROWS {
        for x in 0..BELTS_PER_ROW {
            let pos = IVec3::new(x, y, z * 4);
            f.place(&mut world, BELT, pos, 1, pos - IVec3::new(0, 1, 0), 0);
        }
        for x in (0..BELTS_PER_ROW).step_by(10) {
            let pos = IVec3::new(x + 3, y, z * 4 + 2);
            f.place(&mut world, SMELTER, pos, 0, pos - IVec3::new(0, 1, 0), 0);
        }
    }
    for x in (0..BELTS_PER_ROW).step_by(10) {
        for z in (0..ROWS).step_by(3) {
            let pos = IVec3::new(x + 5, y, z * 4 + 1);
            f.place(&mut world, POLE, pos, 0, pos - IVec3::new(0, 1, 0), 3);
        }
    }
    let mut events = Vec::new();
    f.update(&mut world, 0, &mut events); // the first relink
    println!("base: {} belts, {} processors, {} poles", f.belts.len(), f.processors.len(), f.poles.len());

    let start = Instant::now();
    for tick in 1..=600u64 {
        f.update(&mut world, tick, &mut events);
    }
    println!("steady tick: {} µs", us(start.elapsed()) / 600);

    // Placing one belt far from any pole, as a player building does.
    let start = Instant::now();
    for i in 0..20 {
        let pos = IVec3::new(i, y, ROWS * 4 + 4);
        f.place(&mut world, BELT, pos, 1, pos - IVec3::new(0, 1, 0), 0);
        f.update(&mut world, 700 + i as u64, &mut events);
    }
    println!("place a belt + tick (relink): {} µs", us(start.elapsed()) / 20);

    let start = Instant::now();
    for _ in 0..5 {
        f.relink();
    }
    println!("whole relink: {} µs", us(start.elapsed()) / 5);
    let start = Instant::now();
    for _ in 0..5 {
        let hooked = f.resolve_hooks();
        let (poles, gens, miners, labs) = (&f.poles, &f.generators, &f.miners, &f.labs);
        let p = Power::rebuild(poles, &hooked, gens, miners, &f.processors, labs, &f.pipework, &f.quarries, &[]);
        std::hint::black_box(p);
    }
    println!("  of which hooks + power rebuild: {} µs", us(start.elapsed()) / 5);

    let mut out = Vec::new();
    let start = Instant::now();
    for _ in 0..200 {
        out.clear();
        f.write_instances(&mut out, Vec3::new(50.0, y as f64, 60.0), 0.0, 160.0);
    }
    println!("write_instances: {} µs ({} floats)", us(start.elapsed()) / 200, out.len());
}
