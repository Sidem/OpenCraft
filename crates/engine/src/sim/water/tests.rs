//! Flowing water on hand-built ground far from spawn: stone boxes filled directly, then changed the way
//! actions change blocks (`change`: the edit plus `block_changed`).

use super::*;
use crate::block::{FLOW_1, FLOW_7, STONE};
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
fn a_broken_dam_falls_into_the_pit_and_spreads_six_cells() {
    let mut sim = pond_above_a_pit();
    change(&mut sim, DAM, AIR);
    settle(&mut sim);
    assert_eq!(sim.block(DAM), flow(7));
    assert_eq!(sim.block(at(6, 100, 15)), flow(6), "the lip");
    for y in 91..100 {
        assert_eq!(sim.block(at(6, y, 15)), flow(6), "falling at y {y}: the lip's level, no stronger");
        assert!(!wet(&mut sim, at(7, y, 15)), "falling water doesn't spread in the air");
    }
    // On the floor: the fall's level where it lands, then one less a cell, so at most 6 cells out.
    for z in 1..=29 {
        for x in 6..=39 {
            let d = (x - 6) + (z - 15i32).abs();
            let expect = match d {
                0 => flow(6),
                1..=6 => flow(7 - d as u8),
                _ => AIR,
            };
            assert_eq!(sim.block(at(x, 90, z)), expect, "({x}, {z})");
        }
    }
    assert_eq!(sim.block(at(12, 90, 15)), FLOW_1);
    assert_eq!(sim.block(at(1, 100, 15)), WATER, "the pond is a source: it stays");
}

/// What the player sees (`mesher/water.rs`): a settled flow thins out in even steps, from 11/16 of a block where it
/// lands (the lip's level 6) to the last cell's 1/16, never a step that skips a height or rises.
#[test]
fn a_settled_flow_thins_in_even_steps_of_height() {
    let mut sim = pond_above_a_pit();
    change(&mut sim, DAM, AIR);
    settle(&mut sim);
    let heights: Vec<u32> = (0..7).map(|d| crate::mesher::water_height(sim.block(at(6 + d, 90, 15)))).collect();
    assert_eq!(heights, [11, 11, 9, 7, 5, 3, 1], "the landing cell, then 6 cells out");
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
    assert_eq!(c.block(at(12, 90, 15)), FLOW_1, "the reloaded core finished the flow");
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

/// Generator version 8 spills each river pool into the next as falling water (`worldgen::falls`): checking every
/// such cell and its neighbours changes nothing, so an edit beside a river never sets it moving.
#[test]
fn generated_falls_are_settled_water() {
    let mut gen = crate::worldgen::WorldGen::new(SEED);
    let mut chunks = Vec::new();
    for z in (-1200..1200).step_by(8) {
        for x in (-1200..1200).step_by(8) {
            let c = IVec3::new(x >> 5, gen.height_at(x, z) >> 5, z >> 5);
            if gen.open_water(x, z) == Some(crate::worldgen::Water::River) && !chunks.contains(&c) && chunks.len() < 40
            {
                chunks.push(c);
            }
        }
    }
    let mut falls = Vec::new();
    for c in chunks {
        let chunk = gen.generate(c);
        for i in 0..crate::chunk::CHUNK_VOLUME {
            let (x, y, z) = (i & 31, i >> 10, (i >> 5) & 31);
            if chunk.get(x, y, z) == FLOW_7 {
                falls.push(IVec3::new(c.x * 32 + x as i32, c.y * 32 + y as i32, c.z * 32 + z as i32));
            }
        }
    }
    assert!(falls.len() > 20, "rivers have falls: {}", falls.len());
    let mut sim = Sim::new(SEED, 2);
    let mut cells: Vec<IVec3> = falls.iter().flat_map(|&p| FACES.map(|f| p + f).into_iter().chain([p])).collect();
    cells.extend(falls.iter().flat_map(|&p| SIDES.map(|s| p + s + UP)));
    let before: Vec<BlockId> = cells.iter().map(|&p| sim.block(p)).collect();
    for &p in &cells {
        sim.water.schedule(sim.tick, p);
    }
    let pending = sim.water.pending_count();
    settle(&mut sim);
    println!("{} falls, {pending} checks", falls.len());
    let after: Vec<BlockId> = cells.iter().map(|&p| sim.block(p)).collect();
    let moved: Vec<_> = cells.iter().zip(before.iter().zip(&after)).filter(|(_, (b, a))| b != a).take(5).collect();
    assert!(moved.is_empty(), "{} falls; changed: {moved:?}", falls.len());
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

#[test]
fn belt_under_a_lake_stays_wet_lake_stays_level_and_items_still_move() {
    use crate::block::{BELT, FAST_BELT, IRON_ORE, STORAGE, WET_SOURCE};
    use crate::item::ItemId;
    use crate::sim::PlayerId;

    // Test both a sea-level lake and a high lake above SEA_LEVEL.
    let mut sim = seaside();
    // High lake at y = 100 (above SEA_LEVEL) walled in stone.
    fill(&mut sim, at(1, 99, 1), at(6, 100, 6), STONE);
    fill(&mut sim, at(2, 100, 2), at(5, 100, 5), WATER);
    fill(&mut sim, at(1, 101, 1), at(6, 102, 6), AIR);

    let sea_belt = at(5, SEA_LEVEL, 5);
    let sea_fast = at(6, SEA_LEVEL, 5);
    let lake_belt = at(3, 100, 3);
    // Facing 1 = +X (East), so sea_belt feeds sea_fast.
    assert!(sim.put_block(PlayerId(0), sea_belt, ItemId::block(BELT), 1, sea_belt - UP));
    assert!(sim.put_block(PlayerId(0), sea_fast, ItemId::block(FAST_BELT), 1, sea_fast - UP));
    assert!(sim.put_block(PlayerId(0), lake_belt, ItemId::block(BELT), 1, lake_belt - UP));

    // Load chunks around the sea belt and settle: once settled, a standing scene remeshes nothing.
    sim.world.update_streaming(sea_belt.as_vec3(), &[]);
    while sim.world.work_step() {}
    settle(&mut sim);
    sim.world.advance_clock(1.0);
    sim.world.begin_work();
    while sim.world.work_step() {}
    sim.world.events.clear();
    for _ in 0..30 {
        sim.step();
        sim.world.advance_clock(1.0 / 60.0);
        sim.world.begin_work();
        assert!(!sim.world.work_step(), "standing scene must remesh nothing");
    }
    assert!(sim.world.events.is_empty(), "no remesh events in a standing waterlogged scene");

    // Both belts stay BELT and full water source; surrounding water stays WATER.
    for p in [sea_belt, sea_fast, lake_belt] {
        assert_eq!(sim.block(p), BELT, "belt kept at {p:?}");
        assert_eq!(sim.world.wet_anywhere(p), WET_SOURCE, "stays source-wet at {p:?}");
    }
    assert!(sim.world.is_water(sea_belt.x, sea_belt.y, sea_belt.z), "bodies still swim in wet belt");
    for z in 1..=10 {
        for x in 1..=10 {
            let p = at(x, SEA_LEVEL, z);
            if p != sea_belt && p != sea_fast {
                assert_eq!(sim.block(p), WATER, "sea stayed level at ({x}, {z})");
            }
        }
    }
    for z in 2..=5 {
        for x in 2..=5 {
            let p = at(x, 100, z);
            if p != lake_belt {
                assert_eq!(sim.block(p), WATER, "high lake stayed level at ({x}, {z})");
            }
        }
    }

    // Items fed onto an underwater belt still move across both wet belts into a box normally.
    let feeder = at(4, SEA_LEVEL, 5);
    let receiver = at(7, SEA_LEVEL, 5);
    assert!(sim.put_block(PlayerId(0), feeder, ItemId::block(STORAGE), 1, feeder - UP));
    assert!(sim.put_block(PlayerId(0), receiver, ItemId::block(STORAGE), 1, receiver - UP));
    sim.factory.stock(feeder, ItemId::block(IRON_ORE), 1);
    for _ in 0..180 {
        sim.step();
    }
    assert_eq!(
        sim.factory.storage_count_at(receiver, ItemId::block(IRON_ORE)),
        1,
        "item crossed both underwater belts into the receiver box"
    );
}

#[test]
fn belt_in_a_flowing_stream_passes_flow_on_to_the_next_cell() {
    use crate::block::BELT;
    use crate::item::ItemId;
    use crate::sim::PlayerId;

    let mut sim = pond_above_a_pit();
    change(&mut sim, DAM, AIR);
    settle(&mut sim);
    // Floor flow from x=6 (level 6) down to x=12 (level 1). Place a belt at x=8 (level 5).
    let mid = at(8, 90, 15);
    assert_eq!(sim.block(mid), flow(5));
    assert!(sim.put_block(PlayerId(0), mid, ItemId::block(BELT), 0, mid - UP));
    assert_eq!(sim.world.wet_anywhere(mid), 5, "placed in flow keeps its flow level");
    settle(&mut sim);
    assert_eq!(sim.block(mid), BELT);
    assert_eq!(sim.world.wet_anywhere(mid), 5, "belt keeps level 5 after settling");
    assert_eq!(sim.block(at(9, 90, 15)), flow(4), "next cell downstream still receives flow 4");
    assert_eq!(sim.block(at(12, 90, 15)), FLOW_1, "stream reaches the full distance past the belt");

    // Closing the dam dries both the stream and the wet belt.
    change(&mut sim, DAM, STONE);
    settle(&mut sim);
    assert_eq!(sim.block(mid), BELT, "belt stays when stream dries");
    assert_eq!(sim.world.wet_anywhere(mid), 0, "wet belt dries when its feed stops");
    assert_eq!(sim.block(at(9, 90, 15)), AIR);
}

#[test]
fn sea_level_trench_with_belts_fills_completely() {
    use crate::block::{BELT, WET_SOURCE};
    use crate::item::ItemId;
    use crate::sim::PlayerId;

    let mut sim = seaside();
    // Dig a dry trench behind a stone plug at x=11, lay dry belts in x=12..=25, then replace the plug with a belt.
    for x in 12..=25 {
        let p = at(x, SEA_LEVEL, 5);
        change(&mut sim, p, AIR);
        assert!(sim.put_block(PlayerId(0), p, ItemId::block(BELT), 0, p - UP));
        assert_eq!(sim.world.wet_anywhere(p), 0, "dry trench belt starts dry at x={x}");
    }
    let plug = at(11, SEA_LEVEL, 5);
    change(&mut sim, plug, AIR);
    assert!(sim.put_block(PlayerId(0), plug, ItemId::block(BELT), 0, plug - UP));
    settle(&mut sim);
    for x in 11..=25 {
        let p = at(x, SEA_LEVEL, 5);
        assert_eq!(sim.block(p), BELT, "belt preserved at x={x}");
        assert_eq!(sim.world.wet_anywhere(p), WET_SOURCE, "sea filled belt at x={x}");
    }
}

#[test]
fn breaking_a_wet_belt_leaves_water_and_dry_placement_is_not_wet() {
    use crate::block::{BELT, PIPE, WET_SOURCE};
    use crate::item::ItemId;
    use crate::sim::PlayerId;

    let mut sim = pond_above_a_pit();
    change(&mut sim, DAM, AIR);
    settle(&mut sim);

    // Dry placement stays dry and breaking it leaves AIR.
    let dry = at(20, 90, 15);
    assert_eq!(sim.block(dry), AIR);
    assert!(sim.put_block(PlayerId(0), dry, ItemId::block(BELT), 0, dry - UP));
    assert_eq!(sim.world.wet_anywhere(dry), 0, "dry ground belt is not wet");
    assert!(sim.dismantle(PlayerId(0), dry, false).is_some());
    assert_eq!(sim.block(dry), AIR);
    assert_eq!(sim.world.wet_anywhere(dry), 0);

    // Belt in source water: wet, and breaking leaves WATER.
    let src = at(2, 100, 15);
    assert_eq!(sim.block(src), WATER);
    assert!(sim.put_block(PlayerId(0), src, ItemId::block(BELT), 0, src - UP));
    assert_eq!(sim.world.wet_anywhere(src), WET_SOURCE);
    assert!(sim.dismantle(PlayerId(0), src, false).is_some());
    assert_eq!(sim.block(src), WATER, "breaking a source-wet belt leaves WATER");
    assert_eq!(sim.world.wet_anywhere(src), 0);

    // Belt in flowing water: wet with flow level, and breaking leaves that flow block.
    let stream = at(8, 90, 15);
    assert_eq!(sim.block(stream), flow(5));
    assert!(sim.put_block(PlayerId(0), stream, ItemId::block(BELT), 0, stream - UP));
    assert_eq!(sim.world.wet_anywhere(stream), 5);
    assert!(sim.dismantle(PlayerId(0), stream, false).is_some());
    assert_eq!(sim.block(stream), flow(5), "breaking a flow-wet belt leaves its flow");
    assert_eq!(sim.world.wet_anywhere(stream), 0);

    // Out-of-scope blocks (pipes, poles, etc.) are not waterlogged.
    let pipe_at = at(3, 100, 15);
    assert_eq!(sim.block(pipe_at), WATER);
    assert!(sim.put_block(PlayerId(0), pipe_at, ItemId::block(PIPE), 0, pipe_at - UP));
    assert_eq!(sim.world.wet_anywhere(pipe_at), 0, "pipes are not waterlogged");
}

#[test]
fn waterlogged_save_reloads_mid_flow_and_cores_agree_whatever_is_loaded() {
    use crate::block::BELT;
    use crate::item::ItemId;
    use crate::sim::PlayerId;

    let setup = |loaded: bool| {
        let mut sim = pond_above_a_pit();
        if loaded {
            sim.world.update_streaming(at(20, 95, 15).as_vec3(), &[]);
            while sim.world.work_step() {}
        }
        // Lay dry belts across the pit floor so the stream must flood through them.
        for x in 7..=11 {
            let p = at(x, 90, 15);
            assert!(sim.put_block(PlayerId(0), p, ItemId::block(BELT), 0, p - UP));
        }
        change(&mut sim, DAM, AIR);
        sim
    };

    let mut a = setup(false);
    let mut b = setup(true);
    let mut mid_tick = 0;
    for t in 0..120 {
        a.step();
        b.step();
        assert_eq!(a.state_hash(), b.state_hash(), "tick {t}");
        if a.world.wet_anywhere(at(7, 90, 15)) > 0 && a.water.pending_count() > 0 {
            mid_tick = t + 1;
            break;
        }
    }
    assert!(mid_tick > 0 && a.water.pending_count() > 0, "stopped mid-flow with first belt wet");

    let mut w = ByteWriter::default();
    a.write_state(&mut w);
    let mut c = Sim::new(SEED, 2);
    c.read_state(&mut ByteReader::new(&w.bytes)).expect("reads back");
    assert_eq!(a.state_hash(), c.state_hash(), "immediate hash equality on reload");

    for t in mid_tick..400 {
        a.step();
        c.step();
        assert_eq!(a.state_hash(), c.state_hash(), "tick {t}");
    }
    assert_eq!(c.world.wet_anywhere(at(8, 90, 15)), 5);
    assert_eq!(c.world.wet_anywhere(at(11, 90, 15)), 2);
    assert_eq!(c.block(at(12, 90, 15)), FLOW_1, "flow passed through all five belts");
}
