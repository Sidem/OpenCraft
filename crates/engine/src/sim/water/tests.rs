//! Mass-conserving cellular water on hand-built ground far from spawn: stone boxes filled directly, then changed
//! the way actions change blocks (`change`: the edit plus `block_changed`).

use super::*;
use crate::block::{flow, FLOW_7, ICE, SPRING, STONE};
use crate::bytes::{ByteReader, ByteWriter};
use crate::daytime::{DAY_TICKS, START, YEAR_DAYS};

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

/// Total conserved water+ice mass (in eighths) inside `lo..=hi`, including waterlogged blocks.
fn total_mass(sim: &mut Sim, lo: IVec3, hi: IVec3) -> u32 {
    let mut sum = 0u32;
    for y in lo.y..=hi.y {
        for z in lo.z..=hi.z {
            for x in lo.x..=hi.x {
                let p = IVec3::new(x, y, z);
                let b = sim.block(p);
                sum += if b == ICE { 8 } else { u32::from(sim.water_mass(p)) };
            }
        }
    }
    sum
}

/// Steps until no water check is pending; returns the ticks taken and the most checks one tick ran.
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

/// Sets the simulation clock to mid-winter noon so cold tests do not evaporate warm puddles mid-settle.
fn set_winter(sim: &mut Sim) {
    let winter = (YEAR_DAYS + 33) * DAY_TICKS + DAY_TICKS / 2 - START;
    sim.tick = winter;
    sim.atmosphere.advance(sim.world.generator(), winter);
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

/// A pond (20 water blocks: x 1..=4, z 13..=17 at y 100) held by stone at x 5, beside a pit (x 6..=39, floor
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
fn a_broken_dam_drains_the_pond_and_conserves_total_mass() {
    let mut sim = pond_above_a_pit();
    set_winter(&mut sim);
    let box_all = (at(1, 90, 1), at(39, 101, 29));
    let before = total_mass(&mut sim, box_all.0, box_all.1);
    assert_eq!(before, 20 * 8, "20 full pond blocks = 160 eighths");
    change(&mut sim, DAM, AIR);
    settle(&mut sim);

    let after = total_mass(&mut sim, box_all.0, box_all.1);
    assert_eq!(after, before, "strictly zero water created or lost");
    let pond_left = total_mass(&mut sim, at(1, 100, 13), at(4, 100, 17));
    let pit_mass = total_mass(&mut sim, at(6, 90, 1), at(39, 90, 29));
    assert!(pond_left < before / 2, "the pond genuinely drained: {pond_left} left of {before}");
    assert!(pit_mass > before / 2, "the drained water landed on the pit floor: {pit_mass}");
    for y in 91..=99 {
        assert_eq!(sim.block(at(6, y, 15)), AIR, "falling column at y {y} emptied once flow settled");
    }
}

/// What the player sees (`mesher/water.rs`): settled finite flow thins out smoothly without rising steps.
#[test]
fn a_settled_flow_thins_in_even_steps_of_height() {
    let mut sim = pond_above_a_pit();
    set_winter(&mut sim);
    change(&mut sim, DAM, AIR);
    settle(&mut sim);
    let heights: Vec<u32> = (0..7).map(|d| crate::mesher::water_height(sim.block(at(6 + d, 90, 15)))).collect();
    assert!(heights[0] >= 3, "water pooled at the landing cell: {heights:?}");
    assert!(heights.windows(2).all(|w| w[0] >= w[1]), "heights monotonically thin out: {heights:?}");
}

#[test]
fn closing_the_dam_stops_the_fall_and_conserves_water() {
    let mut sim = pond_above_a_pit();
    set_winter(&mut sim);
    let box_all = (at(1, 90, 1), at(39, 101, 29));
    let before = total_mass(&mut sim, box_all.0, box_all.1);
    change(&mut sim, DAM, AIR);
    for _ in 0..80 {
        sim.step();
    }
    // Close the dam mid-drain (note: replacing water at DAM removes whatever mass was inside the DAM cell itself).
    let dam_mass = u32::from(sim.water_mass(DAM));
    change(&mut sim, DAM, STONE);
    settle(&mut sim);
    for y in 91..=100 {
        assert!(!wet(&mut sim, at(6, y, 15)), "falling column dried at y={y}");
    }
    let after = total_mass(&mut sim, box_all.0, box_all.1);
    assert_eq!(after + dam_mass, before, "pond + pit mass conserved when dam closed");
    assert!(total_mass(&mut sim, at(1, 100, 13), at(4, 100, 17)) > 0, "water behind dam was retained");
    assert!(total_mass(&mut sim, at(6, 90, 1), at(39, 90, 29)) > 0, "fallen water stayed in the pit");
}

#[test]
fn cores_agree_whatever_is_loaded_and_a_save_reloads_mid_flow() {
    let mut a = pond_above_a_pit();
    let mut b = pond_above_a_pit();
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
    for t in 40..900 {
        a.step();
        c.step();
        assert_eq!(a.state_hash(), c.state_hash(), "tick {t}");
    }
    assert!(total_mass(&mut c, at(6, 90, 1), at(39, 90, 29)) > 0, "the reloaded core finished the flow");
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

#[test]
fn mining_ice_from_a_pond_levels_smoothly_and_conserves_total_mass() {
    let mut sim = Sim::new(SEED, 2);
    set_winter(&mut sim);
    // Walled 5 × 2 pond (10 cells) at y = 100 filled with 8/8 WATER (80 eighths).
    fill(&mut sim, at(0, 99, 0), at(6, 100, 3), STONE);
    fill(&mut sim, at(0, 101, 0), at(6, 102, 3), AIR);
    fill(&mut sim, at(1, 100, 1), at(5, 100, 2), WATER);
    let pond = (at(1, 100, 1), at(5, 100, 2));
    assert_eq!(total_mass(&mut sim, pond.0, pond.1), 80);

    // Freeze one cell and mine it: 72 eighths remain.
    change(&mut sim, at(1, 100, 1), ICE);
    change(&mut sim, at(1, 100, 1), AIR);
    settle(&mut sim);

    assert_eq!(total_mass(&mut sim, pond.0, pond.1), 72, "exact conservation after mining 1 ice block");
    // The mined spot refilled smoothly from neighbours — zero empty dent left behind!
    for z in 1..=2 {
        for x in 1..=5 {
            let m = sim.water_mass(at(x, 100, z));
            assert!((6..=8).contains(&m), "pond leveled smoothly at ({x},{z}): {m}/8");
        }
    }
}

#[test]
fn melted_ice_on_floor_conserves_mass_and_evaporates_warm_puddles() {
    let mut sim = Sim::new(SEED, 2);
    // Mid-summer warm noon so 1/8 puddles evaporate above 0 °C.
    let summer = 11 * DAY_TICKS + DAY_TICKS / 2 - START;
    sim.tick = summer;
    sim.atmosphere.advance(sim.world.generator(), summer);

    // Wide flat stone floor at y = 90.
    fill(&mut sim, at(0, 89, 0), at(20, 89, 20), STONE);
    fill(&mut sim, at(0, 90, 0), at(20, 92, 20), AIR);
    let center = at(10, 90, 10);
    // One melted ICE block = 1 WATER = 8 eighths.
    change(&mut sim, center, WATER);
    settle(&mut sim);

    let floor = (at(1, 90, 1), at(19, 90, 19));
    let spread = total_mass(&mut sim, floor.0, floor.1);
    assert!(spread <= 8, "melted ice never creates water: got {spread}/8");
    assert!(spread >= 6, "spread into thin 1/8 puddles: got {spread}/8");
    // Every remaining wet cell is at most 1/8 (`FLOW_1`).
    for z in 1..=19 {
        for x in 1..=19 {
            assert!(sim.water_mass(at(x, 90, z)) <= 1, "melted block thinned to 1/8 puddles");
        }
    }
    // Over several warm minutes, all 1/8 puddles evaporate cleanly to AIR.
    for _ in 0..800 * crate::TICK_RATE {
        sim.step();
    }
    assert_eq!(total_mass(&mut sim, floor.0, floor.1), 0, "warm 1/8 puddles evaporated cleanly");
}

#[test]
fn refreezing_lowered_pond_borrows_one_eighth_and_conserves_total_mass() {
    let mut sim = Sim::new(SEED, 2);
    set_winter(&mut sim);
    // High cold elevation so freezes_at is true.
    let y = SEA_LEVEL + 80;
    fill(&mut sim, at(0, y - 1, 0), at(5, y, 4), STONE);
    fill(&mut sim, at(0, y + 1, 0), at(5, y + 2, 4), AIR);
    // 3 × 2 pond where every cell is slightly lowered to FLOW_7 (6 × 7 = 42 eighths).
    fill(&mut sim, at(1, y, 1), at(3, y, 2), FLOW_7);
    let pond = (at(1, y, 1), at(3, y, 2));
    assert_eq!(total_mass(&mut sim, pond.0, pond.1), 42);

    // Freeze cells one by one: each FLOW_7 borrows 1/8 from a neighbour so total water+ice stays 42!
    for z in 1..=2 {
        for x in 1..=3 {
            sim.freeze_water(at(x, y, z));
        }
    }
    settle(&mut sim);
    let after = total_mass(&mut sim, pond.0, pond.1);
    assert_eq!(after, 42, "water + ice mass strictly conserved on FLOW_7 refreeze");
    let mut ice_blocks = 0u32;
    for z in 1..=2 {
        for x in 1..=3 {
            ice_blocks += u32::from(sim.block(at(x, y, z)) == ICE);
        }
    }
    assert!(ice_blocks >= 3, "lowered pond froze solid ice sheets by borrowing 1/8: {ice_blocks} ice");
}

#[test]
fn natural_spring_produces_long_downhill_river_and_can_be_capped() {
    let mut sim = Sim::new(SEED, 2);
    set_winter(&mut sim);
    // Build a stepped 1-wide descending stone channel: 5 terraces dropping 1 block every 4 blocks (length 20).
    fill(&mut sim, at(0, 88, 0), at(25, 108, 4), STONE);
    fill(&mut sim, at(0, 106, 0), at(25, 108, 4), AIR);
    for step in 0..5i32 {
        let y = 105 - step;
        let x0 = 2 + step * 4;
        fill(&mut sim, at(x0, y, 2), at(x0 + 3, 108, 2), AIR);
    }
    // Catch basin at the foot (x = 22..24, y = 100).
    fill(&mut sim, at(22, 100, 1), at(24, 108, 3), AIR);

    // Place a natural SPRING at the head of the channel (x = 1, y = 105, z = 2).
    let sp = at(1, 105, 2);
    change(&mut sim, sp, SPRING);
    for _ in 0..600 {
        sim.step();
    }
    // Water flowed all the way down the 5 terraces into the catch basin 21 blocks away!
    let foot_mass = total_mass(&mut sim, at(22, 100, 1), at(24, 100, 3));
    assert!(foot_mass > 0, "spring water flowed down the entire stepped valley into the basin");

    // Cap the spring's open east face with STONE: it stops emitting and the upper terrace drains down to <= 1/8 film!
    let before_cap = sim.water_mass(at(3, 105, 2));
    change(&mut sim, at(2, 105, 2), STONE);
    settle(&mut sim);
    assert!(
        sim.water_mass(at(3, 105, 2)) < before_cap && sim.water_mass(at(3, 105, 2)) <= 1,
        "capped spring stopped feeding the upper terrace"
    );
}

#[test]
fn v15_rivers_and_highlands_start_settled_with_zero_idle_work() {
    let mut sim = Sim::new(SEED, 2);
    let mut gen = crate::worldgen::WorldGen::new(SEED);
    let mut spring_pos = None;
    for cz in -12..=12 {
        for cx in -12..=12 {
            for y_chunk in 1..6 {
                let c = gen.generate(IVec3::new(cx, y_chunk, cz));
                if c.as_uniform().is_some() {
                    continue;
                }
                for i in 0..crate::chunk::CHUNK_VOLUME {
                    let (x, y, z) = (i & 31, i >> 10, (i >> 5) & 31);
                    if c.get(x, y, z) == SPRING {
                        spring_pos = Some(IVec3::new(cx * 32 + x as i32, y_chunk * 32 + y as i32, cz * 32 + z as i32));
                        break;
                    }
                }
                if spring_pos.is_some() {
                    break;
                }
            }
            if spring_pos.is_some() {
                break;
            }
        }
        if spring_pos.is_some() {
            break;
        }
    }
    let sp = spring_pos.expect("v15 world generates natural SPRING blocks");
    sim.world.update_streaming(sp.as_vec3(), &[]);
    while sim.world.work_step() {}
    sim.world.advance_clock(1.0);
    sim.world.begin_work();
    while sim.world.work_step() {}
    sim.world.events.clear();
    for _ in 0..30 {
        sim.step();
        sim.world.advance_clock(1.0 / 60.0);
        sim.world.begin_work();
        assert!(!sim.world.work_step(), "standing near a v15 spring/river remeshes nothing");
    }
    assert_eq!(sim.water.pending_count(), 0);
}

#[test]
fn mining_beside_a_natural_river_spring_never_drains_the_river() {
    let mut sim = Sim::new(2024, 2);
    let mut gen = crate::worldgen::WorldGen::new(2024);
    let mut found = None;
    'search: for cz in -12..=12 {
        for cx in -12..=12 {
            for cy in 2..6 {
                let c = gen.generate(IVec3::new(cx, cy, cz));
                if c.as_uniform().is_some() {
                    continue;
                }
                for i in 0..crate::chunk::CHUNK_VOLUME {
                    let (lx, ly, lz) = (i & 31, i >> 10, (i >> 5) & 31);
                    if c.get(lx, ly, lz) == SPRING {
                        let sp = IVec3::new(cx * 32 + lx as i32, cy * 32 + ly as i32, cz * 32 + lz as i32);
                        if SIDES.iter().any(|&s| sim.block(sp + s) == WATER) {
                            found = Some(sp);
                            break 'search;
                        }
                    }
                }
            }
        }
    }
    let sp = found.expect("v15 world has a natural river spring");
    let box_lo = sp - IVec3::new(14, 6, 14);
    let box_hi = sp + IVec3::new(14, 2, 14);
    let mass_before = total_mass(&mut sim, box_lo, box_hi);
    assert!(mass_before > 80, "headwater pool holds water: {mass_before}/8");

    // Mine solid bank/bed blocks right next to the spring, matching the playtest bug report.
    for s in SIDES {
        let p = sp + s;
        if sim.block(p) != WATER && sim.block(p) != SPRING {
            change(&mut sim, p, AIR);
        }
        let below = p - UP;
        if sim.block(below) != WATER && sim.block(below) != SPRING {
            change(&mut sim, below, AIR);
        }
    }

    let (ticks, _) = settle(&mut sim);
    assert!(ticks < 600, "mining beside a river spring must settle cleanly without a chain reaction ({ticks} ticks)");
    let mass_after = total_mass(&mut sim, box_lo, box_hi);
    assert!(
        mass_after >= mass_before,
        "river around the spring stayed full instead of drying up: {mass_after} vs {mass_before}"
    );
}

#[test]
fn damming_a_steady_waterfall_dries_the_fall_cleanly() {
    let mut sim = Sim::new(SEED, 2);
    set_winter(&mut sim);
    // Upper pool at y=104 (x 1..=3, z 2), open FLOW_7 waterfall at x=4 (y 101..=104) landing into lower pool at y=100.
    fill(&mut sim, at(0, 98, 0), at(10, 106, 4), STONE);
    fill(&mut sim, at(1, 105, 1), at(9, 106, 3), AIR);
    fill(&mut sim, at(1, 104, 2), at(3, 104, 2), WATER);
    fill(&mut sim, at(4, 101, 2), at(4, 104, 2), FLOW_7);
    fill(&mut sim, at(4, 100, 1), at(9, 100, 3), WATER);
    fill(&mut sim, at(5, 101, 1), at(9, 104, 3), AIR);

    // Waking the waterfall while upper pool feeds it leaves it settled in steady state.
    sim.water.schedule(sim.tick, at(4, 104, 2));
    sim.water.schedule(sim.tick, at(4, 103, 2));
    sim.water.schedule(sim.tick, at(3, 104, 2));
    settle(&mut sim);
    assert_eq!(sim.block(at(4, 104, 2)), FLOW_7, "fed waterfall holds steady");
    assert_eq!(sim.block(at(3, 104, 2)), WATER, "upper lip does not drain");

    // Building a dam at the lip (x=3) cuts the feed: the falling column above the lower pool dries to AIR and its
    // mass lands conservatively in the lower pool!
    let before_dam = total_mass(&mut sim, at(4, 100, 1), at(9, 104, 3));
    change(&mut sim, at(3, 104, 2), STONE);
    settle(&mut sim);
    for y in 102..=104 {
        assert_eq!(sim.block(at(4, y, 2)), AIR, "dammed waterfall dried at y={y}");
    }
    assert_eq!(
        total_mass(&mut sim, at(4, 100, 1), at(9, 104, 3)),
        before_dam,
        "fallen waterfall mass landed conservatively in the lower pool"
    );
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

    let mut sim = seaside();
    fill(&mut sim, at(1, 99, 1), at(6, 100, 6), STONE);
    fill(&mut sim, at(2, 100, 2), at(5, 100, 5), WATER);
    fill(&mut sim, at(1, 101, 1), at(6, 102, 6), AIR);

    let sea_belt = at(5, SEA_LEVEL, 5);
    let sea_fast = at(6, SEA_LEVEL, 5);
    let lake_belt = at(3, 100, 3);
    assert!(sim.put_block(PlayerId(0), sea_belt, ItemId::block(BELT), 1, sea_belt - UP));
    assert!(sim.put_block(PlayerId(0), sea_fast, ItemId::block(FAST_BELT), 1, sea_fast - UP));
    assert!(sim.put_block(PlayerId(0), lake_belt, ItemId::block(BELT), 1, lake_belt - UP));

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

    // A 1-wide stone trough fed by a SPRING, with a drop at the end, and a belt in the middle.
    let mut sim = Sim::new(SEED, 2);
    set_winter(&mut sim);
    fill(&mut sim, at(0, 95, 0), at(12, 101, 4), STONE);
    fill(&mut sim, at(1, 100, 2), at(9, 102, 2), AIR);
    fill(&mut sim, at(9, 96, 2), at(11, 102, 2), AIR);
    let mid = at(4, 100, 2);
    assert!(sim.put_block(PlayerId(0), mid, ItemId::block(BELT), 0, mid - UP));
    change(&mut sim, at(1, 100, 2), SPRING);
    for _ in 0..200 {
        sim.step();
    }
    assert_eq!(sim.block(mid), BELT);
    assert!(sim.world.wet_anywhere(mid) > 0, "spring flow wetted the belt");
    assert!(sim.water_mass(at(5, 100, 2)) > 0, "flow passed through the belt to the next cell");

    // Cap the spring: water drains off the drop, lowering the wet belt to dry/film and stopping downstream flow.
    let wet_before = sim.world.wet_anywhere(mid);
    change(&mut sim, at(2, 100, 2), STONE);
    settle(&mut sim);
    assert_eq!(sim.block(mid), BELT, "belt stays when stream stops");
    assert!(sim.world.wet_anywhere(mid) < wet_before, "wet belt drained after spring was capped");
    assert!(sim.water_mass(at(5, 100, 2)) <= 2, "downstream channel drained after spring was capped");
}

#[test]
fn sea_level_trench_with_belts_fills_completely() {
    use crate::block::{BELT, WET_SOURCE};
    use crate::item::ItemId;
    use crate::sim::PlayerId;

    let mut sim = seaside();
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
    set_winter(&mut sim);
    // Dry placement stays dry and breaking it leaves AIR.
    let dry = at(20, 90, 15);
    assert_eq!(sim.block(dry), AIR);
    assert!(sim.put_block(PlayerId(0), dry, ItemId::block(BELT), 0, dry - UP));
    assert_eq!(sim.world.wet_anywhere(dry), 0, "dry ground belt is not wet");
    assert!(sim.dismantle(PlayerId(0), dry, false).is_some());
    assert_eq!(sim.block(dry), AIR);
    assert_eq!(sim.world.wet_anywhere(dry), 0);

    // Belt in full water: wet (8), and breaking leaves WATER.
    let src = at(2, 100, 15);
    assert_eq!(sim.block(src), WATER);
    assert!(sim.put_block(PlayerId(0), src, ItemId::block(BELT), 0, src - UP));
    assert_eq!(sim.world.wet_anywhere(src), WET_SOURCE);
    assert!(sim.dismantle(PlayerId(0), src, false).is_some());
    assert_eq!(sim.block(src), WATER, "breaking a full-wet belt leaves WATER");
    assert_eq!(sim.world.wet_anywhere(src), 0);

    // Belt in partial flow: wet with exact flow level, and breaking leaves that exact flow block.
    let stream = at(10, 90, 15);
    sim.world.set_block_anywhere(stream, flow(5));
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
        set_winter(&mut sim);
        if loaded {
            sim.world.update_streaming(at(20, 95, 15).as_vec3(), &[]);
            while sim.world.work_step() {}
        }
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
    for t in 0..200 {
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

    for t in mid_tick..900 {
        a.step();
        c.step();
        assert_eq!(a.state_hash(), c.state_hash(), "tick {t}");
    }
    assert!(c.world.wet_anywhere(at(8, 90, 15)) > 0, "finite flow wetted belt at x=8");
    assert!(c.world.wet_anywhere(at(11, 90, 15)) > 0, "finite flow passed through all five belts");
}
