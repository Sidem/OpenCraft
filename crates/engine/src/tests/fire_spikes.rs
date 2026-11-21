//! What a 1,000-tree forest fire costs the frame (`bench_fire_spikes`, ignored; a bench, not a check):
//!   cargo test --release -q bench_fire_spikes -- --ignored --nocapture
//! Builds a 32×32 forest (1,024 grown trees), lights the centre tree with `MAX_HEAT`, and times native host frames
//! (`update` + 6 ms streaming work budget) and the added core `Sim::run_fire` tick cost.
//! Target (docs/ROADMAP.md FIRE1): worst host frame under 8 ms, added core tick under 0.3 ms p99.

use std::time::Instant;

use super::run_until_ready;
use crate::action::Action;
use crate::block::{AIR, DIRT, GRASS, LEAVES, OAK_TREE_LOG, STONE};
use crate::math::IVec3;
use crate::sim::fire::{CROWN_TICKS, MAX_HEAT, TRUNK_TICKS};
use crate::Game;

const DT: f64 = 1.0 / 60.0;
const WORK_BUDGET_MS: f64 = 6.0;

fn frame_ms(g: &mut Game, step_ms: &mut f64) -> (f64, f64, f64) {
    let t = Instant::now();
    g.update(DT);
    let update_ms = t.elapsed().as_secs_f64() * 1000.0;
    let w = Instant::now();
    g.begin_work();
    let mut slowest_step = 0.0f64;
    let mut prev = 0.0f64;
    while g.work_step() {
        let now = w.elapsed().as_secs_f64() * 1000.0;
        let step = now - prev;
        *step_ms = (*step_ms * 0.995).max(step);
        slowest_step = slowest_step.max(step);
        prev = now;
        // Mirror `web/src/main.ts` line 322: stop before a next remesh would push past the 6 ms work budget.
        if now + *step_ms * 0.5 >= WORK_BUDGET_MS {
            break;
        }
    }
    while g.next_event() != 0 {}
    (t.elapsed().as_secs_f64() * 1000.0, update_ms, slowest_step)
}

fn plant_tree(g: &mut Game, foot: IVec3) {
    for y in 0..6 {
        g.sim.world.set_block_anywhere_later(foot + IVec3::new(0, y, 0), OAK_TREE_LOG);
    }
    for dy in 4..=6 {
        for dz in -1..=1 {
            for dx in -1..=1 {
                let p = foot + IVec3::new(dx, dy, dz);
                if g.sim.world.block_anywhere_or_generate(p) == AIR {
                    g.sim.world.set_block_anywhere_later(p, LEAVES);
                }
            }
        }
    }
}

#[test]
#[ignore]
fn bench_fire_spikes() {
    let mut g = Game::new(1337, 5);
    run_until_ready(&mut g);
    let spawn = g.body().pos.floor();
    let centre = IVec3::new(spawn.x, spawn.y - 1, spawn.z);

    // Flat ground + 32×32 = 1,024 trees spaced 8 blocks apart around spawn (~16 trees per 32×32 chunk, dense forest).
    let half = 130;
    for z in -half..=half {
        for x in -half..=half {
            g.sim.world.set_block_anywhere_later(centre + IVec3::new(x, -2, z), STONE);
            g.sim.world.set_block_anywhere_later(centre + IVec3::new(x, -1, z), DIRT);
            g.sim.world.set_block_anywhere_later(centre + IVec3::new(x, 0, z), GRASS);
            for y in 1..=9 {
                g.sim.world.set_block_anywhere_later(centre + IVec3::new(x, y, z), AIR);
            }
        }
    }
    for gz in -16..16 {
        for gx in -16..16 {
            plant_tree(&mut g, centre + IVec3::new(gx * 8, 1, gz * 8));
        }
    }
    // Drain initial meshing.
    g.begin_work();
    while g.work_step() {}
    while g.next_event() != 0 {}

    // Ignite centre tree with MAX_HEAT and measure 960 frames (16 s of active forest fire covering crown burns + spread).
    g.act(Action::Ignite { pos: centre + IVec3::new(0, 1, 0), heat: MAX_HEAT });
    let mut frames = Vec::with_capacity(960);
    let mut updates = Vec::with_capacity(960);
    let mut slowest_remesh = 0.0f64;
    let mut step_ms = 0.0f64;
    let mut fire_ticks = Vec::with_capacity(960);
    let mut peak_blazes = 0usize;
    let mut peak_fronts = 0usize;
    for _ in 0..960 {
        peak_blazes = peak_blazes.max(g.sim.fire.blazes().len());
        peak_fronts = peak_fronts.max(g.sim.fire.fronts().len());
        let ft = Instant::now();
        g.sim.run_fire();
        fire_ticks.push(ft.elapsed().as_secs_f64() * 1000.0);
        let (fms, ums, sms) = frame_ms(&mut g, &mut step_ms);
        frames.push(fms);
        updates.push(ums);
        slowest_remesh = slowest_remesh.max(sms);
    }
    frames.sort_by(f64::total_cmp);
    updates.sort_by(f64::total_cmp);
    fire_ticks.sort_by(f64::total_cmp);
    let p50 = frames[frames.len() / 2];
    let p99 = frames[frames.len() * 99 / 100];
    let worst = *frames.last().unwrap();
    let up99 = updates[updates.len() * 99 / 100];
    let uworst = *updates.last().unwrap();
    let fp99 = fire_ticks[fire_ticks.len() * 99 / 100];
    let fworst = *fire_ticks.last().unwrap();
    println!(
        "1024-tree fire: peak blazes={peak_blazes}, fronts={peak_fronts} | frame p50={p50:.2}ms p99={p99:.2}ms worst={worst:.2}ms (update p99={up99:.2}ms worst={uworst:.2}ms, slowest_remesh={slowest_remesh:.2}ms) | fire_tick p99={fp99:.3}ms worst={fworst:.3}ms"
    );
    assert!(peak_blazes > 1, "fire spread across the forest");
    assert!(peak_blazes <= crate::sim::fire::MAX_BLAZES, "blaze cap held strictly");
    assert!(peak_fronts <= crate::sim::fire::MAX_FRONT, "front cap held strictly");
    assert!(up99 < 2.0, "update p99 {up99:.2} ms exceeded 2 ms");
    assert!(fp99 < 0.3, "fire tick p99 {fp99:.3} ms exceeded 0.3 ms");
}

/// Guards that burning a tree never remeshes immediately in its tick (leaves remeshing to the streaming budget),
/// and that an idle world with no fire produces zero mesh events.
#[test]
fn burning_tree_leaves_remeshing_to_the_budget_and_idle_world_does_zero_work() {
    let mut g = Game::new(1337, 2);
    run_until_ready(&mut g);
    let meshes_idle = g.sim.world.events.len();
    for _ in 0..60 {
        g.sim.step();
    }
    assert!(g.sim.fire.is_empty());
    assert_eq!(g.sim.world.events.len(), meshes_idle, "idle world with no fire remeshes nothing");

    let spawn = g.body().pos.floor();
    let foot = IVec3::new(spawn.x + 6, spawn.y, spawn.z);
    for z in -3..=3 {
        for x in -3..=3 {
            g.sim.world.set_block_anywhere_later(foot + IVec3::new(x, -1, z), GRASS);
            for y in 0..=8 {
                g.sim.world.set_block_anywhere_later(foot + IVec3::new(x, y, z), AIR);
            }
        }
    }
    plant_tree(&mut g, foot);
    g.begin_work();
    while g.work_step() {}
    while g.next_event() != 0 {}

    let meshes = g.sim.world.events.len();
    g.sim.apply(crate::sim::PlayerId(0), Action::Ignite { pos: foot, heat: 0 });
    for _ in 0..(CROWN_TICKS + TRUNK_TICKS + 60) {
        g.sim.step();
        assert_eq!(g.sim.world.events.len(), meshes, "burning never remeshes synchronously inside Sim::step");
    }
    assert_eq!(g.sim.world.block_anywhere_or_generate(foot), AIR, "tree burned down");
    assert!(g.sim.world.dirty_count() > 0, "dirty chunks left for the streaming budget");
}
