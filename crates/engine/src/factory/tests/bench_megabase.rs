//! Bench of a spread-out megabase (`cargo test --release -q bench_megabase -- --ignored --nocapture`): 20 outposts 300 to
//! 1,500 blocks from a hub, each 84 chains of box → smelter → constructor → constructor → box (5,040 processors in all),
//! every outpost sending a loaded belt line home on a layer of its own (no crossings) with long-distance poles beside it,
//! all one grid. Prints microseconds for the costs that scale with the whole factory rather than with the view
//! (DEV_PLAN section 2): tick, relink, power rebuild, instances, state bytes and hash; then the same base inside a
//! `Game` for whole frames, saving and the co-op hash. Poles and wires are added directly: `place` relinks per pole.

use std::time::{Duration, Instant};

use super::recipe_for;
use crate::block::{BlockId, BELT, COAL_ORE, CONSTRUCTOR, GENERATOR, IRON_ORE, LONG_POLE, SMELTER};
use crate::bytes::{fnv1a, ByteWriter};
use crate::factory::links::Slot;
use crate::factory::machine::add_to;
use crate::factory::pole::{Pole, LONG_TIER};
use crate::factory::power::Power;
use crate::factory::provenance::Tag;
use crate::factory::wiring::Hook;
use crate::factory::{Factory, DIRS};
use crate::item::{IRON_PLATE, IRON_ROD, SCREW};
use crate::math::{IVec3, Vec3};
use crate::world::World;
use crate::Game;

const OUTPOSTS: i32 = 20;
/// Chains per outpost: `COLUMNS` side by side, `ROWS` deep, each `CHAIN` cells long.
const COLUMNS: i32 = 6;
const ROWS: i32 = 14;
const CHAIN: i32 = 14;
const EAST: u8 = 1;
const LONG_EVERY: usize = 150;
/// The hub's generators and pole, below every outpost layer.
const HUB: IVec3 = IVec3::new(0, 90, -12);
/// View distances in blocks: render distance 8 and 12 chunks.
const RANGES: [f64; 2] = [256.0, 384.0];

fn us(d: Duration) -> u128 {
    d.as_micros()
}

/// Median, 99th percentile and worst of `t`, in µs.
fn spread(t: &mut [Duration]) -> String {
    t.sort();
    let at = |q: f64| us(t[((t.len() - 1) as f64 * q) as usize]);
    format!("p50 {} p99 {} max {} µs", at(0.5), at(0.99), at(1.0))
}

/// Outpost `k`'s corner: 300 + 63k blocks from the hub at the golden angle, on layer `100 + 5k`.
fn outpost(k: i32) -> IVec3 {
    let (r, a) = (300.0 + 63.0 * k as f64, k as f64 * 2.399_963);
    IVec3::new((r * a.cos()) as i32, 100 + 5 * k, (r * a.sin()) as i32)
}

fn pole(f: &mut Factory, pos: IVec3, tier: u8) {
    add_to(&mut f.poles, Pole { pos, tier, wall: 0 }, &mut f.at, Slot::Pole);
}

fn put(f: &mut Factory, world: &mut World, block: BlockId, pos: IVec3) {
    f.place(world, block, pos, EAST, pos - IVec3::new(0, 1, 0), 0);
}

/// The pole in `poles` nearest `pos`.
fn nearest(poles: &[IVec3], pos: IVec3) -> IVec3 {
    let d = |p: &&IVec3| (p.x - pos.x).pow(2) + (p.z - pos.z).pow(2) + (p.y - pos.y).pow(2);
    *poles.iter().min_by_key(d).unwrap()
}

/// Builds the base; returns it with its belt-line length and a free spot beside the farthest outpost.
fn build(world: &mut World) -> (Factory, usize, IVec3) {
    let mut f = Factory { by_hand: true, ..Factory::default() };
    f.research.complete_all();
    pole(&mut f, HUB, LONG_TIER);
    for i in 0..40 {
        let pos = IVec3::new(2 * i - 40, HUB.y, HUB.z - 4);
        put(&mut f, world, GENERATOR, pos);
        f.insert(pos, COAL_ORE.into(), 64);
        f.hooks.push(Hook { pole: HUB, to: pos });
    }
    let mut trunk = 0;
    for k in 0..OUTPOSTS {
        let o = outpost(k);
        let cell = |x: i32, z: i32| IVec3::new(o.x + x, o.y, o.z + z);
        let mut poles = Vec::new();
        for c in 0..COLUMNS {
            for r in 0..ROWS {
                let at = |dx: i32| cell(c * CHAIN + dx, 2 * r);
                f.add_storage(at(0));
                f.stock(at(0), IRON_ORE.into(), 1500);
                for dx in [1, 2, 4, 5, 7, 8, 10, 11] {
                    put(&mut f, world, BELT, at(dx));
                }
                put(&mut f, world, SMELTER, at(3));
                f.insert(at(3), COAL_ORE.into(), 64);
                put(&mut f, world, CONSTRUCTOR, at(6));
                f.set_recipe(at(6), Some(recipe_for(IRON_ROD)));
                put(&mut f, world, CONSTRUCTOR, at(9));
                f.set_recipe(at(9), Some(recipe_for(SCREW)));
                f.add_storage(at(12));
            }
            for r in (0..ROWS).step_by(3) {
                let p = cell(c * CHAIN + 7, 2 * r + 1);
                pole(&mut f, p, 3);
                // Wired up the column and to the column before.
                for q in [p - IVec3::new(0, 0, 6), p - IVec3::new(CHAIN, 0, 0)] {
                    if poles.contains(&q) {
                        f.hooks.push(Hook { pole: q, to: p });
                    }
                }
                poles.push(p);
            }
            for r in 0..ROWS {
                for dx in [6, 9] {
                    let m = cell(c * CHAIN + dx, 2 * r);
                    f.hooks.push(Hook { pole: nearest(&poles, m), to: m });
                }
            }
        }
        for i in 0..12 {
            let g = cell(7 * i, 2 * ROWS + 2);
            put(&mut f, world, GENERATOR, g);
            f.insert(g, COAL_ORE.into(), 64);
            f.hooks.push(Hook { pole: nearest(&poles, g), to: g });
        }
        trunk += belt_line(&mut f, cell(-2, -2), IVec3::new(3 * k, o.y, 0), &poles);
    }
    let far = outpost(OUTPOSTS - 1) + IVec3::new(0, 0, 2 * ROWS + 6);
    (f, trunk, far)
}

/// A loaded belt line from a box of plates at `from` to a box at `to`, x first then z, with a long-distance pole every
/// `LONG_EVERY` cells wired from the outpost's grid to the hub's. Returns its length.
fn belt_line(f: &mut Factory, from: IVec3, to: IVec3, poles: &[IVec3]) -> usize {
    let mut path = Vec::new();
    let mut p = from;
    loop {
        p = p + match p.x != to.x {
            true => IVec3::new((to.x - p.x).signum(), 0, 0),
            false => IVec3::new(0, 0, (to.z - p.z).signum()),
        };
        if p == to {
            break;
        }
        path.push(p);
    }
    f.add_storage(from);
    f.stock(from, IRON_PLATE, 1500);
    f.add_storage(to);
    let mut last = nearest(poles, from);
    for (i, &c) in path.iter().enumerate() {
        let next = path.get(i + 1).copied().unwrap_or(to);
        let dir = DIRS.iter().position(|&d| d == next - c).unwrap() as u8;
        f.add_belt(c, dir);
        let Some(&Slot::Belt(b)) = f.at.get(&c) else { unreachable!() };
        for _ in 0..2 {
            f.belts[b as usize].accept(IRON_PLATE, 1, Tag::made_at(from), false, 1.0);
        }
        if i % LONG_EVERY == 0 || i + 1 == path.len() {
            let side = if dir % 2 == 1 { IVec3::new(0, 0, -3) } else { IVec3::new(-3, 0, 0) };
            let at = c + side;
            let cells = [at, at + IVec3::new(1, 0, 0), at + IVec3::new(0, 0, 1), at + IVec3::new(1, 0, 1)];
            if cells.iter().all(|c| !f.at.contains_key(c)) {
                pole(f, at, LONG_TIER);
                f.hooks.push(Hook { pole: last, to: at });
                last = at;
            }
        }
    }
    f.hooks.push(Hook { pole: last, to: HUB });
    path.len()
}

#[test]
#[ignore]
fn bench_megabase() {
    let mut world = World::new(1, 2);
    let t = Instant::now();
    let (mut f, trunk, far) = build(&mut world);
    let mut events = Vec::new();
    f.update(&mut world, 0, &mut events);
    println!(
        "megabase: {} belts ({trunk} in belt lines), {} processors, {} poles, {} wires, {} grids; built in {} ms",
        f.belts.len(),
        f.processors.len(),
        f.poles.len(),
        f.hooks.len(),
        f.power.supply.len(),
        t.elapsed().as_millis()
    );

    let mut ticks = Vec::new();
    for tick in 1..=3000u64 {
        let t = Instant::now();
        f.update(&mut world, tick, &mut events);
        ticks.push(t.elapsed());
        events.clear();
    }
    let runs: Vec<Duration> = ticks.chunks(600).map(|c| c.iter().sum::<Duration>() / 600).collect();
    let mut runs = runs;
    runs.sort();
    println!("steady tick: {} µs (median of 5 runs of 600); per tick {}", us(runs[2]), spread(&mut ticks));
    let working = f.processors.iter().filter(|p| p.batch.is_some()).count();
    let items: usize = f.belts.iter().map(|b| b.items.len()).sum();
    println!(
        "  {working} processors mid-batch, {items} items on belts, grid supply {} of {} kW",
        f.power.supply.iter().sum::<u32>(),
        f.power.demand.iter().sum::<u32>()
    );

    let start = Instant::now();
    for i in 0..20 {
        let pos = far + IVec3::new(i, 0, 0);
        f.place(&mut world, BELT, pos, EAST, pos - IVec3::new(0, 1, 0), 0);
        f.update(&mut world, 4000 + i as u64, &mut events);
    }
    println!("place a belt at the farthest outpost + tick (relink): {} µs", us(start.elapsed()) / 20);
    // A pole wires itself (`auto_hook`: every pole's free slots, each a scan of the wires) and relinks once more.
    let start = Instant::now();
    for i in 0..5 {
        let pos = far + IVec3::new(4 * i, 0, 4);
        f.place(&mut world, LONG_POLE, pos, EAST, pos - IVec3::new(0, 1, 0), 0);
        f.update(&mut world, 4100 + i as u64, &mut events);
    }
    println!("place a long-distance pole there + tick: {} µs", us(start.elapsed()) / 5);
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

    let o = outpost(10);
    let views = [
        ("hub", Vec3::new(0.5, 102.0, 0.5)),
        ("outpost 10", Vec3::new(o.x as f64 + 42.0, o.y as f64 + 3.0, o.z as f64 + 14.0)),
        ("nothing in view", Vec3::new(0.5, 5000.0, 0.5)),
    ];
    let mut out = Vec::new();
    for (name, eye) in views {
        for range in RANGES {
            let start = Instant::now();
            for _ in 0..50 {
                out.clear();
                f.write_instances(&mut out, eye, 0.0, range);
            }
            println!(
                "write_instances at {name}, range {range}: {} µs ({} floats)",
                us(start.elapsed()) / 50,
                out.len()
            );
        }
    }

    let mut w = ByteWriter::default();
    let start = Instant::now();
    for _ in 0..5 {
        w = ByteWriter::default();
        f.write_state(&mut w);
    }
    let write = us(start.elapsed()) / 5;
    let start = Instant::now();
    std::hint::black_box(fnv1a(&w.bytes));
    println!("factory write_state: {} bytes in {write} µs; fnv1a {} µs", w.bytes.len(), us(start.elapsed()));

    in_a_game(f, o);
}

/// The base inside a fresh `Game`: whole frames at the hub and at an outpost, the save and the co-op hash.
fn in_a_game(mut f: Factory, o: IVec3) {
    let mut g = Game::new(2024, 8);
    crate::tests::run_until_ready(&mut g);
    let (start, bare) = (Instant::now(), g.save().len());
    let bare_save = us(start.elapsed());
    std::mem::swap(&mut g.sim.factory, &mut f);
    for (name, x, y, z) in
        [("hub", 0.5, 104.0, 0.5), ("outpost 10", o.x as f64 + 42.0, o.y as f64 + 4.0, o.z as f64 + 14.0)]
    {
        g.teleport(x, y, z);
        crate::tests::run_until_ready(&mut g);
        // Frames by the tick they ran: each second's (stall scan), each tenth's (analytics sample), the rest.
        let mut frames: [Vec<Duration>; 3] = Default::default();
        for _ in 0..600 {
            g.teleport(x, y, z);
            let (t, tick) = (Instant::now(), g.sim.tick);
            g.update(1.0 / 60.0);
            let bucket = if tick % 60 == 0 { 0 } else { 1 + usize::from(tick % 10 != 0) };
            frames[bucket].push(t.elapsed());
            g.begin_work();
            while g.work_step() {}
            while g.next_event() != 0 {}
        }
        let [second, tenth, rest] = &mut frames;
        println!("Game::update at {name} (1 tick + presentation): {}", spread(rest));
        println!("  on a tenth-second tick {}; on a whole-second tick {}", spread(tenth), spread(second));
    }
    let start = Instant::now();
    let bytes = g.save().len();
    println!(
        "Game::save: {bytes} bytes in {} µs (without the base: {bare} bytes, {bare_save} µs)",
        us(start.elapsed())
    );
    let start = Instant::now();
    std::hint::black_box(g.sim.state_hash());
    println!("Sim::state_hash (co-op, every CHECKSUM_TICKS): {} µs", us(start.elapsed()));
}
