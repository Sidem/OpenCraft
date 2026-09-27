//! Flowing water on hand-built ground far from spawn: stone boxes filled directly, then changed the way
//! actions change blocks (`change`: the edit plus `block_changed`).

use super::*;
use crate::block::{FLOW_1, STONE};
use crate::bytes::{ByteReader, ByteWriter};

const SEED: u32 = 1337;
/// The corner every test builds from.
const O: IVec3 = IVec3::new(3000, 0, 3000);

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

fn change(sim: &mut Sim, p: IVec3, b: BlockId) {
    let old = sim.block(p);
    assert!(sim.world.set_block_anywhere(p, b));
    sim.block_changed(p, old);
}

/// Steps until no check is pending; returns the ticks taken and the most checks one tick ran.
fn settle(sim: &mut Sim) -> (u32, usize) {
    let mut most = 0;
    for t in 0..10_000 {
        if sim.water.pending_count() == 0 {
            return (t, most);
        }
        sim.step();
        most = most.max(sim.water.last_run);
    }
    panic!("the water never settled");
}

/// A sea (sources x 1..=10, y 55..=SEA_LEVEL, z 1..=10) walled in stone, beside stone land up to
/// SEA_LEVEL (x 11..=40), with air above.
fn seaside() -> Sim {
    let mut sim = Sim::new(SEED, 2);
    fill(&mut sim, at(0, 50, 0), at(41, SEA_LEVEL, 11), STONE);
    fill(&mut sim, at(0, SEA_LEVEL + 1, 0), at(41, SEA_LEVEL + 8, 11), AIR);
    fill(&mut sim, at(1, 55, 1), at(10, SEA_LEVEL, 10), WATER);
    sim
}

#[test]
fn a_hole_dug_beside_the_sea_fills_with_a_source() {
    let mut sim = seaside();
    let hole = at(11, SEA_LEVEL, 5);
    change(&mut sim, hole, AIR);
    settle(&mut sim);
    assert_eq!(sim.block(hole), WATER);
    assert_eq!(sim.block(hole + IVec3::new(0, 1, 0)), AIR, "the sea doesn't rise");
}

#[test]
fn a_trench_from_the_sea_fills_completely() {
    let mut sim = seaside();
    for x in 11..=30 {
        change(&mut sim, at(x, SEA_LEVEL, 5), AIR);
    }
    let (ticks, _) = settle(&mut sim);
    for x in 11..=30 {
        assert_eq!(sim.block(at(x, SEA_LEVEL, 5)), WATER, "x {x}");
    }
    assert!(ticks < 20 * 2 * WATER_DELAY as u32 + 20, "fills a cell or so per check: {ticks} ticks");
}

/// A pond (sources x 1..=4, z 13..=17 at y 100) held by stone at x 5, beside a pit (x 6..=39, floor
/// at y 89, air up to 105) cut into a stone block. The dam is at x 5, z 15.
pub(crate) fn pond_above_a_pit() -> Sim {
    let mut sim = Sim::new(SEED, 2);
    build_pond_above_a_pit(&mut sim);
    sim
}

/// Builds `pond_above_a_pit` into any core (a co-op test builds it into both).
pub(crate) fn build_pond_above_a_pit(sim: &mut Sim) {
    fill(sim, at(0, 88, 0), at(40, 106, 30), STONE);
    fill(sim, at(1, 101, 1), at(40, 106, 29), AIR);
    fill(sim, at(6, 90, 1), at(39, 100, 29), AIR);
    fill(sim, at(1, 100, 13), at(4, 100, 17), WATER);
}

pub(crate) const DAM: IVec3 = IVec3::new(3005, 100, 3015);

fn wet(sim: &mut Sim, p: IVec3) -> bool {
    LIQUID[sim.block(p) as usize]
}

#[test]
fn a_broken_dam_falls_into_the_pit_and_spreads_seven_cells() {
    let mut sim = pond_above_a_pit();
    change(&mut sim, DAM, AIR);
    settle(&mut sim);
    assert_eq!(sim.block(DAM), flow(7));
    assert_eq!(sim.block(at(6, 100, 15)), flow(6), "the lip");
    for y in 91..100 {
        assert_eq!(sim.block(at(6, y, 15)), FLOW_7, "falling at y {y}");
        assert!(!wet(&mut sim, at(7, y, 15)), "falling water doesn't spread in the air");
    }
    // On the floor: level 8 - distance from where it lands, so at most 7 cells out.
    for z in 1..=29 {
        for x in 6..=39 {
            let d = (x - 6) + (z - 15i32).abs();
            let expect = match d {
                0 => FLOW_7,
                1..=7 => flow(8 - d as u8),
                _ => AIR,
            };
            assert_eq!(sim.block(at(x, 90, z)), expect, "({x}, {z})");
        }
    }
    assert_eq!(sim.block(at(13, 90, 15)), FLOW_1);
    assert_eq!(sim.block(at(1, 100, 15)), WATER, "the pond is a source: it stays");
}

#[test]
fn closing_the_dam_dries_the_flow() {
    let mut sim = pond_above_a_pit();
    change(&mut sim, DAM, AIR);
    settle(&mut sim);
    change(&mut sim, DAM, STONE);
    settle(&mut sim);
    for y in 90..=100 {
        for z in 1..=29 {
            for x in 6..=39 {
                assert!(!wet(&mut sim, at(x, y, z)), "dry at ({x}, {y}, {z})");
            }
        }
    }
    assert_eq!(sim.block(at(4, 100, 15)), WATER);
}

#[test]
fn cores_agree_whatever_is_loaded_and_a_save_reloads_mid_flow() {
    let mut a = pond_above_a_pit();
    let mut b = pond_above_a_pit();
    // b has the ground around the pit loaded (its edits go to loaded chunks); a has nothing loaded.
    b.world.update_streaming(at(20, 95, 15).as_vec3(), &[]);
    while b.world.work_step() {}
    change(&mut a, DAM, AIR);
    change(&mut b, DAM, AIR);
    for t in 0..40 {
        a.step();
        b.step();
        assert_eq!(a.state_hash(), b.state_hash(), "tick {t}");
    }
    assert!(a.water.pending_count() > 0, "mid-flow");

    let mut w = ByteWriter::default();
    a.write_state(&mut w);
    let mut c = Sim::new(SEED, 2);
    c.read_state(&mut ByteReader::new(&w.bytes)).expect("reads back");
    for t in 40..400 {
        a.step();
        c.step();
        assert_eq!(a.state_hash(), c.state_hash(), "tick {t}");
    }
    assert_eq!(c.block(at(13, 90, 15)), FLOW_1, "the reloaded core finished the flow");
}

/// A walled sea beside a dry basin 38 × 39 × 2 at sea level (2,964 cells), opened by one hole.
fn basin() -> Sim {
    let mut sim = Sim::new(SEED, 2);
    fill(&mut sim, at(0, 55, 0), at(50, SEA_LEVEL, 40), STONE);
    fill(&mut sim, at(0, SEA_LEVEL + 1, 0), at(50, SEA_LEVEL + 4, 40), AIR);
    fill(&mut sim, at(1, 56, 1), at(10, SEA_LEVEL, 39), WATER);
    fill(&mut sim, at(12, SEA_LEVEL - 1, 1), at(49, SEA_LEVEL, 39), AIR);
    sim
}

#[test]
fn a_big_flood_keeps_to_the_cap_and_fills_every_cell() {
    let mut sim = basin();
    change(&mut sim, at(11, SEA_LEVEL, 20), AIR);
    let (_, most) = settle(&mut sim);
    assert_eq!(most, MAX_WATER_UPDATES, "the cap holds the rest back");
    for y in SEA_LEVEL - 1..=SEA_LEVEL {
        for z in 1..=39 {
            for x in 12..=49 {
                assert_eq!(sim.block(at(x, y, z)), WATER, "({x}, {y}, {z})");
            }
        }
    }
}

/// `cargo test --release bench_water -- --ignored --nocapture`
#[test]
#[ignore]
fn bench_water() {
    for loaded in [false, true] {
        let mut sim = basin();
        if loaded {
            sim.world.update_streaming(at(25, SEA_LEVEL, 20).as_vec3(), &[]);
            while sim.world.work_step() {}
        }
        change(&mut sim, at(11, SEA_LEVEL, 20), AIR);
        bench_flood(&mut sim, loaded);
    }
}

fn bench_flood(sim: &mut Sim, loaded: bool) {
    let (mut ticks, mut total, mut worst) = (0, 0.0f64, 0.0f64);
    while sim.water.pending_count() > 0 {
        let start = std::time::Instant::now();
        sim.step();
        let ms = start.elapsed().as_secs_f64() * 1000.0;
        (ticks, total, worst) = (ticks + 1, total + ms, worst.max(ms));
    }
    println!("loaded {loaded}: {ticks} ticks, mean {:.3} ms, worst {:.3} ms per tick", total / ticks as f64, worst);
}
