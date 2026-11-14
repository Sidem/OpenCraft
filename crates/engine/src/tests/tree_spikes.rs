//! What a tree growing and a tree falling cost the frame (`bench_tree_spikes`, ignored; a bench, not a check):
//!   cargo test --release -q bench_tree_spikes -- --ignored --nocapture
//! Loads seed 1337 around spawn, then times host frames (`update`, then budgeted streaming work, as `web/src/main.ts`
//! does) while (1) a planted sapling of each kind grows, and (2) a grown tree of each kind is felled with a steel axe
//! and falls, plus the host's `take_fallen`; and the slowest single remesh. The check beside it guards the cause of
//! the old spikes: a tree's hundreds of edits never remesh in the tick that makes them.

use std::time::Instant;

use super::run_until_ready;
use crate::action::Action;
use crate::block::{self, AIR, DIRT, GRASS, STONE};
use crate::item::STEEL_AXE;
use crate::math::IVec3;
use crate::worldgen::{grow, height_of, KINDS};
use crate::Game;

const DT: f64 = 1.0 / 60.0;

/// The host's work budget a frame (`WORK_BUDGET_MS` in `web/src/main.ts`).
const WORK_BUDGET_MS: f64 = 6.0;

thread_local! {
    /// The slowest single `work_step` (one chunk remeshed or generated) since it was last taken.
    static STEP: std::cell::Cell<f64> = const { std::cell::Cell::new(0.0) };
}

fn slowest_step() -> f64 {
    STEP.with(|m| m.replace(0.0))
}

/// One host frame: `update`, then streaming work (remeshing) until the budget is spent, then the mesh events
/// drained. A single remesh can't be split, so a frame can run past the budget.
fn frame_ms(g: &mut Game) -> f64 {
    let t = Instant::now();
    g.update(DT);
    let w = Instant::now();
    g.begin_work();
    loop {
        let s = Instant::now();
        let more = g.work_step();
        let ms = s.elapsed().as_secs_f64() * 1000.0;
        STEP.with(|m| m.set(m.get().max(ms)));
        if !more || w.elapsed().as_secs_f64() * 1000.0 >= WORK_BUDGET_MS {
            break;
        }
    }
    while g.next_event() != 0 {}
    t.elapsed().as_secs_f64() * 1000.0
}

fn median(mut v: Vec<f64>) -> f64 {
    v.sort_by(f64::total_cmp);
    v[v.len() / 2]
}

/// A flat clearing `r` round `at` (grass at `at.y`), streamed in and meshed.
fn clearing(g: &mut Game, at: IVec3, r: i32) {
    for z in -r..=r {
        for x in -r..=r {
            for y in at.y - 3..at.y + 50 {
                let b = match y - at.y {
                    d if d > 0 => AIR,
                    0 => GRASS,
                    -1 | -2 => DIRT,
                    _ => STONE,
                };
                g.sim.world.set_block_anywhere_later(at + IVec3::new(x, y - at.y, z), b);
            }
        }
    }
    for _ in 0..120 {
        frame_ms(g);
    }
}

#[test]
#[ignore]
fn bench_tree_spikes() {
    let mut g = Game::new(1337, 6);
    run_until_ready(&mut g);
    let spawn = g.body().pos.floor();
    let at = IVec3::new(spawn.x + 12, spawn.y - 1, spawn.z);
    clearing(&mut g, at, 22);
    let quiet = median((0..120).map(|_| frame_ms(&mut g)).collect());
    println!("quiet frame median {quiet:.2} ms");

    // Growing: a sapling of each kind, made due at once.
    for k in KINDS.iter() {
        let foot = at + IVec3::new(0, 1, 0);
        g.sim.world.set_block_anywhere(foot, k.sapling);
        slowest_step();
        let t = Instant::now();
        g.sim.grow_sapling(foot);
        let tick = t.elapsed().as_secs_f64() * 1000.0;
        let after: Vec<f64> = (0..30).map(|_| frame_ms(&mut g)).collect();
        let worst = after.iter().copied().fold(0.0, f64::max);
        println!(
            "grow  {:<14} sim {tick:7.2} ms, worst frame after {worst:7.2} ms, slowest remesh {:5.2} ms",
            block::def(k.log).name,
            slowest_step()
        );
        clearing(&mut g, at, 22);
    }

    // Felling: a grown tree of each kind, swung at with a steel axe until it falls.
    let inv = &mut g.sim.players[0].as_mut().unwrap().inventory;
    inv.add(STEEL_AXE, 64);
    inv.selected = inv.slots.iter().position(|s| s.item == STEEL_AXE).unwrap();
    for (kind, k) in KINDS.iter().enumerate() {
        let world = &mut g.sim.world;
        grow(kind, at, height_of(kind, 3), 1337, |p, b, replace| {
            if replace || world.block_anywhere_or_generate(p) == AIR {
                world.set_block_anywhere_later(p, b);
            }
        });
        for _ in 0..120 {
            frame_ms(&mut g);
        }
        let trunk = (1..4)
            .find_map(|up| {
                let w = &mut g.sim.world;
                crate::sim::trunk(|p| w.block_anywhere_or_generate(p), at + IVec3::new(0, up, 0))
            })
            .expect("a trunk");
        let (mut worst, mut fallen_ms, mut frames, mut chop) = (0.0f64, 0.0f64, 0, 0.0f64);
        let mut last = 0.0;
        while g.sim.felling.marks(trunk.foot).is_some() || frames == 0 {
            g.act(Action::ChopTree { pos: trunk.foot, at: trunk.foot.as_vec3(), from: 0 });
            chop = chop.max(last);
            last = frame_ms(&mut g);
            frames += 1;
            assert!(frames < 2000, "never fell");
        }
        // The fall and the landing.
        slowest_step();
        for _ in 0..120 {
            worst = worst.max(frame_ms(&mut g));
            let t = Instant::now();
            let _ = g.take_fallen();
            fallen_ms = fallen_ms.max(t.elapsed().as_secs_f64() * 1000.0);
        }
        println!(
            "fell  {:<14} {:3} logs: swing {chop:6.2} ms, the fall's frame {last:7.2} ms, worst after {worst:7.2} ms, take_fallen {fallen_ms:6.2} ms, slowest remesh {:5.2} ms",
            block::def(k.log).name,
            trunk.logs,
            slowest_step()
        );
        clearing(&mut g, at, 22);
    }
}

/// A sapling growing and a tree falling remesh nothing in their tick (each block remeshed at once was a freeze of a
/// second or more); the chunks are left to the streaming budget.
#[test]
fn growing_and_felling_leave_remeshing_to_the_budget() {
    let mut g = Game::new(1337, 2);
    run_until_ready(&mut g);
    let spawn = g.body().pos.floor();
    let at = IVec3::new(spawn.x + 8, spawn.y - 1, spawn.z);
    clearing(&mut g, at, 10);
    let foot = at + IVec3::new(0, 1, 0);
    g.sim.world.set_block_anywhere_later(foot, KINDS[0].sapling);
    let meshes = g.sim.world.events.len();
    g.sim.grow_sapling(foot);
    assert!(block::is_log(g.sim.world.block_anywhere_or_generate(foot)), "it grew");
    assert_eq!(g.sim.world.events.len(), meshes, "growing remeshed in its tick");
    assert!(g.sim.world.dirty_count() > 0, "and left its chunks to remesh");

    let inv = &mut g.sim.players[0].as_mut().unwrap().inventory;
    inv.add(STEEL_AXE, 64);
    inv.selected = inv.slots.iter().position(|s| s.item == STEEL_AXE).unwrap();
    let trunk = crate::sim::trunk(|p| g.sim.world.block_anywhere_or_generate(p), foot).expect("a trunk");
    for _ in 0..2000 {
        g.sim.apply(crate::sim::PlayerId(0), Action::ChopTree { pos: trunk.foot, at: foot.as_vec3(), from: 0 });
        assert_eq!(g.sim.world.events.len(), meshes, "felling remeshed in its tick");
        if g.sim.felling.marks(trunk.foot).is_none() {
            break;
        }
    }
    assert_eq!(g.sim.world.block_anywhere_or_generate(foot), AIR, "it fell");
}
