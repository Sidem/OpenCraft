//! Bench of the tier marks and product displays (`cargo test --release -q bench_machine_marks -- --ignored
//! --nocapture`): a dense yard of top-tier machines around the camera, every maker with a recipe, so every mark and
//! display is drawn. Prints microseconds and floats for `write_instances`.

use std::time::Instant;

use crate::block::{ASSEMBLER, BELT, CONSTRUCTOR, LAB, MINER, POLE, SMELTER, STORAGE};
use crate::math::{IVec3, Vec3};
use crate::world::World;

use crate::factory::process::spec;
use crate::factory::Factory;

/// Cells per side of the yard (a machine every 3 cells, so 1,600 machines).
const SIDE: i32 = 120;

#[test]
#[ignore]
fn bench_machine_marks() {
    let (mut world, mut f) = (World::new(1, 2), Factory { by_hand: true, ..Factory::default() });
    let y = 100;
    let constructor = (0..2000u16).find(|&i| spec(CONSTRUCTOR).unwrap().recipe(i).is_some()).unwrap();
    let assembler = (0..2000u16).find(|&i| spec(ASSEMBLER).unwrap().recipe(i).is_some()).unwrap();
    let mut n = 0;
    for x in (0..SIDE).step_by(3) {
        for z in (0..SIDE).step_by(3) {
            let pos = IVec3::new(x, y, z);
            let (block, tier) = match n % 8 {
                0 | 1 => (CONSTRUCTOR, 4),
                2 => (ASSEMBLER, 4),
                3 => (SMELTER, 4),
                4 => (STORAGE, 2),
                5 => (LAB, 3),
                6 => (POLE, 3),
                _ => (MINER, 4),
            };
            f.place(&mut world, block, pos, 0, pos - IVec3::new(0, 1, 0), tier);
            match block {
                CONSTRUCTOR => drop(f.set_recipe(pos, Some(constructor))),
                ASSEMBLER => drop(f.set_recipe(pos, Some(assembler))),
                _ => {}
            }
            f.place(&mut world, BELT, pos + IVec3::new(1, 0, 0), 0, pos, 3);
            n += 1;
        }
    }
    let mut events = Vec::new();
    f.update(&mut world, 0, &mut events);
    let eye = Vec3::new(SIDE as f64 / 2.0, y as f64 + 2.0, SIDE as f64 / 2.0);
    let mut out = Vec::new();
    let start = Instant::now();
    for _ in 0..200 {
        out.clear();
        f.write_instances(&mut out, eye, 0.0, 160.0);
    }
    println!("{} machines: write_instances {} µs ({} floats)", n, start.elapsed().as_micros() / 200, out.len());
}
