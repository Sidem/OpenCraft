//! Foresters in a whole core (`Sim`), on hand-built flat ground far from spawn (no chunk there is ever loaded), placed
//! like a player places them, powered by a coal generator whose coal is counted, logs belted into a box.

use super::*;
use crate::action::Action;
use crate::block::{is_leaves, is_log, AIR};
use crate::block::{BELT, COAL_ORE, DIRT, GENERATOR, GRASS, LEAVES, LOG, POLE, SAPLING, STONE, STORAGE, WATER};
use crate::bytes::{ByteReader, ByteWriter};
use crate::factory::generator::GENERATOR_TIERS;
use crate::item::{ItemId, FORESTER_MK2, FORESTER_MK3};
use crate::recipes::fuel_energy;
use crate::sim::{PlayerId, Sim};
use crate::TICK_RATE;

const P: PlayerId = PlayerId(0);
const O: IVec3 = IVec3::new(-4000, 0, 4000);
const EAST: u8 = 1;
const SOUTH: u8 = 2;
const WEST: u8 = 3;
/// Ground (grass) at y 100; the forester stands on it.
const GROUND: i32 = 100;
const GEN: IVec3 = IVec3::new(-4001, 101, 3997);
const POLE_AT: IVec3 = IVec3::new(-4000, 101, 3997);
const TIERS: [ItemId; 3] = [ItemId::block(FORESTER), FORESTER_MK2, FORESTER_MK3];

fn at(x: i32, y: i32, z: i32) -> IVec3 {
    O + IVec3::new(x, y, z)
}

/// The forester's cell.
fn q() -> IVec3 {
    at(0, GROUND + 1, 0)
}

fn fill(sim: &mut Sim, lo: IVec3, hi: IVec3, b: BlockId) {
    for y in lo.y..=hi.y {
        for z in lo.z..=hi.z {
            for x in lo.x..=hi.x {
                sim.world.set_block_anywhere_later(IVec3::new(x, y, z), b);
            }
        }
    }
}

/// Places `item` at `pos` as a player would; whether it went in.
fn put(sim: &mut Sim, item: ItemId, pos: IVec3, facing: u8) -> bool {
    sim.apply(P, Action::Give { item, count: 1 });
    let slot = sim.players[0].as_ref().unwrap().inventory.slots.iter().position(|s| s.item == item).unwrap();
    sim.apply(P, Action::PlaceBlock { pos, slot: slot as u8, facing, against: pos - IVec3::new(0, 1, 0) });
    sim.world.block_anywhere_or_generate(pos) == item.places().unwrap()
}

/// Flat grass, wide enough for a Mk3's patch, with open sky over it; a belt west into a box, a pole and a generator
/// behind; then a forester of `tier` at `q()` facing east holding `saplings`.
fn lot(seed: u32, saplings: u32, tier: usize) -> Sim {
    let mut sim = bare(seed);
    assert!(put(&mut sim, TIERS[tier], q(), EAST), "it stands on flat ground");
    sim.apply(P, Action::Connect { pole: POLE_AT, to: q() });
    if saplings > 0 {
        assert_eq!(sim.factory.insert(q(), SAPLING.into(), saplings), saplings);
    }
    sim
}

/// The lot with no forester on it yet.
fn bare(seed: u32) -> Sim {
    let mut sim = Sim::new(seed, 2);
    fill(&mut sim, at(-8, GROUND - 6, -14), at(20, GROUND - 1, 14), STONE);
    fill(&mut sim, at(-8, GROUND - 2, -14), at(20, GROUND - 1, 14), DIRT);
    fill(&mut sim, at(-8, GROUND, -14), at(20, GROUND, 14), GRASS);
    fill(&mut sim, at(-8, GROUND + 1, -14), at(20, GROUND + 20, 14), AIR);
    for x in -3..=-1 {
        assert!(put(&mut sim, BELT.into(), at(x, GROUND + 1, 0), WEST));
    }
    assert!(put(&mut sim, STORAGE.into(), at(-4, GROUND + 1, 0), 0));
    assert!(put(&mut sim, POLE.into(), POLE_AT, 0));
    assert!(put(&mut sim, GENERATOR.into(), GEN, 0));
    sim.apply(P, Action::Connect { pole: POLE_AT, to: GEN });
    sim
}

fn forester(sim: &Sim) -> &Processor {
    sim.factory.processors.iter().find(|p| p.spec.block == FORESTER).expect("a forester")
}

fn forest(sim: &Sim) -> &Forest {
    let Extra::Forest(f) = &forester(sim).extra else { unreachable!() };
    f
}

fn patch(sim: &Sim) -> Patch {
    forest(sim).patch(q(), EAST)
}

/// Steps until `done` holds (at most `seconds`).
fn until(sim: &mut Sim, seconds: u32, done: impl Fn(&Sim) -> bool) {
    for _ in 0..seconds * TICK_RATE {
        if done(sim) {
            return;
        }
        sim.step();
    }
    panic!("not within {seconds} s: {}", forester(sim).status_text());
}

/// What a run measured.
#[derive(Default)]
struct Tally {
    /// Logs made in each minute.
    per_minute: Vec<u32>,
    /// Coal put in the generator.
    coal_in: u32,
    /// Plots empty at each minute's end while the forester had no sapling to plant.
    starved: Vec<u32>,
    /// Logs taken out of the box.
    boxed: u32,
    /// Every log the forester took from the world.
    cut: Vec<(IVec3, BlockId)>,
}

/// Runs `minutes`, topping the generator's coal up and emptying the box every minute, counting logs, coal and plots left bare for want of
/// saplings.
fn run(sim: &mut Sim, minutes: u32, t: &mut Tally) {
    for _ in 0..minutes {
        t.coal_in += sim.factory.insert(GEN, COAL_ORE.into(), 64);
        // Someone empties the box, so it never backs the forester up.
        sim.factory.take_contents(at(-4, GROUND + 1, 0), |item, n| {
            t.boxed += if item == LOG.into() { n } else { 0 };
            n
        });
        let mut logs = 0;
        for _ in 0..60 * TICK_RATE {
            sim.step();
            for e in sim.events.drain(..) {
                match e {
                    SimEvent::Produced { item, count } if item == LOG.into() => logs += count,
                    SimEvent::QuarryDug { pos, block } => t.cut.push((pos, block)),
                    _ => {}
                }
            }
        }
        t.per_minute.push(logs);
        let (p, none) = (patch(sim), forester(sim).input.total() == 0);
        let bare =
            (0..p.columns()).filter(|&c| p.is_plot(c) && sim.world.block_anywhere_or_generate(p.column(c)) == AIR);
        t.starved.push(if none { bare.count() as u32 } else { 0 });
    }
}

/// kJ the generator gave: the coal it burned, less the energy it still holds.
fn energy_used(sim: &Sim, t: &Tally) -> f64 {
    let g = &sim.factory.generators[0];
    let burned = t.coal_in - g.fuel.count(COAL_ORE.into());
    f64::from(burned * fuel_energy(COAL_ORE.into()).unwrap()) - f64::from(g.energy) / f64::from(TICK_RATE)
}

/// kJ a log gives burned in the best generator.
fn best_burn() -> f64 {
    let best = GENERATOR_TIERS.iter().map(|t| t.yield_percent).max().unwrap();
    f64::from(fuel_energy(LOG.into()).unwrap() * best) / 100.0
}

#[test]
fn the_patch_lies_in_front_and_each_tier_holds_the_last() {
    let p = Patch { pos: IVec3::new(0, 70, 0), dir: EAST, size: 0 };
    assert_eq!(p.bounds(), (IVec3::new(1, 70, -5), IVec3::new(11, 83, 5)));
    assert_eq!(
        (p.column(0), p.column(10), p.column(11)),
        (IVec3::new(1, 70, -5), IVec3::new(1, 70, 5), IVec3::new(2, 70, 5))
    );
    let plots = |p: Patch| -> Vec<IVec3> { (0..p.columns()).filter(|&c| p.is_plot(c)).map(|c| p.column(c)).collect() };
    for c in plots(p) {
        assert!([3, 6, 9].contains(&c.x) && [-3, 0, 3].contains(&c.z), "plot at {c:?}");
    }
    let sizes: Vec<Patch> = (0..SIZES.len() as u8).map(|size| Patch { size, ..p }).collect();
    assert_eq!(sizes.iter().map(|&p| p.plots()).collect::<Vec<_>>(), [9, 20, 35]);
    for w in sizes.windows(2) {
        assert!((0..w[0].columns()).all(|c| w[1].contains(w[0].column(c))), "a bigger patch holds the smaller");
        assert!(plots(w[0]).iter().all(|c| plots(w[1]).contains(c)), "and keeps its plots");
    }
    assert_eq!(SIZES.len(), FORESTER_SPEC.tiers.len(), "a size per tier");
    let north = Patch { pos: IVec3::ZERO, dir: 0, size: 0 };
    assert_eq!(north.bounds(), (IVec3::new(-5, 0, -11), IVec3::new(5, 13, -1)), "facing north it lies to -z");
}

/// It can only be placed facing flat grass or dirt: a stone, a hole or water in its first patch stops the placing.
#[test]
fn placing_needs_flat_ground_in_front() {
    for (cell, block) in [(at(5, GROUND + 1, -2), STONE), (at(9, GROUND, -4), AIR), (at(3, GROUND, -5), WATER)] {
        let mut sim = bare(7);
        sim.world.set_block_anywhere(cell, block);
        assert!(!put(&mut sim, FORESTER.into(), q(), EAST), "{block} at {cell:?}");
        assert!(put(&mut sim, FORESTER.into(), q(), SOUTH), "facing south the ground is flat");
    }
}

/// Ground spoiled after placing stops it at its next look, says how much and marks where; flattened, it carries on.
#[test]
fn uneven_ground_stops_it_and_says_where() {
    let mut sim = lot(7, 9, 0);
    run(&mut sim, 1, &mut Tally::default());
    let stone = at(4, GROUND + 2, -2);
    sim.world.set_block_anywhere(stone, STONE);
    until(&mut sim, 120, |s| forest(s).job == Job::Blocked);
    assert_eq!(forest(&sim).ground.blocker, Some(stone - IVec3::new(0, 1, 0)));
    assert!(forester(&sim).status_text().contains("Needs flat grass or dirt"), "{}", forester(&sim).status_text());
    sim.world.set_block_anywhere(stone, AIR);
    until(&mut sim, 30, |s| !matches!(forest(s).job, Job::Blocked | Job::Look | Job::Rest));
    assert_eq!(forest(&sim).ground, Survey { size: Some(0), uneven: 0, blocker: None });
}

/// From a bare patch and a few saplings: it plants every plot, fells what grows, replants, and belts the logs away;
/// it takes logs only inside its patch, and its logs cost more energy than they give back burned.
#[test]
fn a_forester_plants_fells_and_replants_onto_a_belt() {
    let mut sim = lot(1337, 9, 0);
    let mut t = Tally::default();
    run(&mut sim, 20, &mut t);
    let logs: u32 = t.per_minute.iter().sum();
    let f = forest(&sim);
    assert!(f.felled >= 9, "felled {}", f.felled);
    assert!(logs >= 40, "made {logs} logs in 20 minutes");
    let boxed = sim.factory.box_slots(at(-4, GROUND + 1, 0)).unwrap();
    let in_box: u32 = boxed.iter().filter(|s| s.item == LOG.into()).map(|s| s.count).sum();
    let home = t.boxed + in_box + forester(&sim).out.total();
    assert!(home <= logs && logs - home <= 12, "every log reached the box, waits, or rides the belt: {home} of {logs}");
    assert!(t.cut.iter().all(|&(c, b)| is_log(b) && patch(&sim).contains(c)), "it only fells inside its patch");
    assert!(forester(&sim).input.total() > 0, "its trees gave it saplings back");
    let kj_per_log = energy_used(&sim, &t) / f64::from(logs);
    assert!(kj_per_log > best_burn(), "a log cost {kj_per_log:.0} kJ, gives {:.0} burned", best_burn());
}

/// A wild tree (not on a plot) next to another: the saw takes it whole, trunk and the leaves only it held, while the
/// neighbour keeps the leaves it holds; its logs come home, and the leaves pay saplings.
#[test]
fn it_fells_a_tree_whole_and_spares_a_neighbours_crown() {
    let mut sim = lot(7, 0, 0);
    let (foot, other) = (at(4, GROUND + 1, 1), at(7, GROUND + 1, 1));
    assert!(!patch(&sim).is_plot_at(4, 1));
    fill(&mut sim, foot + IVec3::new(-2, 3, -2), other + IVec3::new(2, 5, 2), LEAVES);
    fill(&mut sim, foot, foot + IVec3::new(0, 4, 0), LOG);
    fill(&mut sim, other, other + IVec3::new(0, 4, 0), LOG);
    sim.factory.insert(GEN, COAL_ORE.into(), 64);
    let wall = at(12, GROUND + 1, 0);
    sim.world.set_block_anywhere(wall, LOG);
    until(&mut sim, 60, |s| forest(s).job == Job::Fell);
    let f = forest(&sim);
    assert_eq!((f.stem, f.log, f.tree), (5, LOG, (4, 1)));
    for y in 0..5 {
        assert_eq!(sim.world.block_anywhere_or_generate(foot + IVec3::new(0, y, 0)), AIR, "trunk at {y}");
        assert_eq!(sim.world.block_anywhere_or_generate(other + IVec3::new(0, y, 0)), LOG, "the neighbour at {y}");
    }
    // Round the felled trunk, leaves more than 4 steps from the neighbour's logs are gone (no loose crown hangs on);
    // those within 2 stay, held by it.
    for x in -2..=2 {
        for (y, z) in (3..=5).flat_map(|y| (-2..=2).map(move |z| (y, z))) {
            let p = foot + IVec3::new(x, y, z);
            let steps = (p.x - other.x).abs() + (p.z - other.z).abs() + (p.y - (other.y + y.min(4))).abs();
            let leaf = is_leaves(sim.world.block_anywhere_or_generate(p));
            assert!(steps > 2 || leaf, "{p:?} is {steps} from the neighbour and should stay");
            assert!(steps <= 4 || !leaf, "{p:?} is {steps} from the neighbour and should fall");
        }
    }
    // All five logs come home: belted into the box.
    let boxed = |s: &Sim| -> u32 { s.factory.box_slots(at(-4, GROUND + 1, 0)).unwrap().iter().map(|s| s.count).sum() };
    until(&mut sim, 120, |s| boxed(s) == 5);
    assert_eq!(forest(&sim).felled, 1);
    assert_eq!(sim.world.block_anywhere_or_generate(wall), LOG, "outside its patch");
    assert!(forester(&sim).input.total() >= 1, "the leaves gave a sapling");
}

/// With nothing to fell and nothing to plant with, it rests and asks for saplings.
#[test]
fn a_bare_patch_with_no_saplings_asks_for_some() {
    let mut sim = lot(7, 0, 0);
    run(&mut sim, 1, &mut Tally::default());
    assert_eq!(forest(&sim).job, Job::Rest);
    assert_eq!(forester(&sim).status, Status::NoInput);
    assert!(forester(&sim).status_text().contains("no saplings"), "{}", forester(&sim).status_text());
}

#[test]
fn without_power_it_waits_and_says_so() {
    let mut sim = lot(7, 9, 0);
    sim.factory.remove(POLE_AT);
    run(&mut sim, 1, &mut Tally::default());
    assert_eq!(forester(&sim).status, Status::NoPower);
    assert_eq!(forester(&sim).input.total(), 9, "nothing planted");
}

/// An upgrade works a bigger patch, but only as far as the ground is flat; the readout says what stops it.
#[test]
fn an_upgrade_grows_the_patch_where_the_ground_allows() {
    let mut sim = lot(7, 9, 0);
    let rest = |s: &Sim| forest(s).col == 0 && forest(s).job == Job::Look;
    run(&mut sim, 1, &mut Tally::default());
    assert_eq!(forest(&sim).ground.size, Some(0));
    assert!(sim.factory.upgrade(q()));
    until(&mut sim, 300, |s| forest(s).ground.size == Some(1));
    // A stone only a Mk3's patch reaches: it stays at the Mk2's and says why.
    let stone = at(16, GROUND + 1, 10);
    sim.world.set_block_anywhere(stone, STONE);
    assert!(sim.factory.upgrade(q()));
    until(&mut sim, 30, rest);
    sim.step();
    until(&mut sim, 300, rest);
    sim.step();
    assert_eq!(forest(&sim).ground, Survey { size: Some(1), uneven: 1, blocker: Some(stone) });
    assert!(forester(&sim).status_text().contains("flatten more ground"), "{}", forester(&sim).status_text());
    sim.world.set_block_anywhere(stone, AIR);
    until(&mut sim, 300, |s| forest(s).ground.size == Some(2));
}

/// A save made mid-job carries on identically; and a second run of the same world is the same, tick for tick.
#[test]
fn a_save_mid_job_carries_on_identically() {
    let mut sim = lot(2024, 9, 1);
    run(&mut sim, 6, &mut Tally::default());
    assert!(forest(&sim).job.moves(), "{:?}", forest(&sim).job);
    let mut w = ByteWriter::default();
    sim.write_state(&mut w);
    let mut again = Sim::new(2024, 2);
    again.read_state(&mut ByteReader::new(&w.bytes)).expect("reads back");
    for _ in 0..120 * TICK_RATE {
        sim.step();
        again.step();
    }
    assert_eq!(sim.state_hash(), again.state_hash());
    let mut twin = lot(2024, 9, 1);
    run(&mut twin, 6, &mut Tally::default());
    for _ in 0..120 * TICK_RATE {
        twin.step();
    }
    assert_eq!(sim.state_hash(), twin.state_hash());
}

/// One tier's run on `seed` from a bare patch with 9 saplings: logs a minute after `warm` minutes, the coefficient of
/// variation of logs per 5 minutes, kJ drawn per log, and after `warm` the share of plot-minutes bare for want of a
/// sapling (a Mk3 starting with 9 saplings for 35 plots is short of them at first, by design).
fn pace(seed: u32, tier: usize, minutes: u32, warm: usize) -> (f64, f64, f64, f64) {
    let mut sim = lot(seed, 9, tier);
    let mut t = Tally::default();
    run(&mut sim, minutes, &mut t);
    let windows: Vec<f64> = t.per_minute[warm..].chunks_exact(5).map(|w| f64::from(w.iter().sum::<u32>())).collect();
    let mean = windows.iter().sum::<f64>() / windows.len() as f64;
    let sd = (windows.iter().map(|w| (w - mean).powi(2)).sum::<f64>() / (windows.len() - 1) as f64).sqrt();
    let logs: u32 = t.per_minute.iter().sum();
    let plots = f64::from(patch(&sim).plots()) * (minutes as usize - warm) as f64;
    let starved: u32 = t.starved[warm..].iter().sum();
    (mean / 5.0, sd / mean, energy_used(&sim, &t) / f64::from(logs), f64::from(starved) / plots)
}

/// The pace bars (numbers: `cargo test -q slow_steady -- --nocapture`), from a bare patch over 75 minutes on 3 seeds: a
/// Mk1 gives 5 to 8 logs a minute, a Mk2 at least 1.7 times and a Mk3 at least 2.4 times as many; logs per 5 minutes
/// within 25% (coefficient of variation), at most 1% of plot-minutes bare for want of a sapling, and every tier draws
/// more energy per log than a log burns for.
#[test]
fn each_tier_gives_a_slow_steady_supply() {
    let mut means = [0.0; 3];
    for (tier, mean) in means.iter_mut().enumerate() {
        let seeds = [1337, 2024, 7];
        for seed in seeds {
            let (rate, cv, kj, starved) = pace(seed, tier, 75, 15);
            println!(
                "Mk{} seed {seed}: {rate:.2} logs a minute, cv {cv:.3}, {kj:.0} kJ a log, starved {starved:.4}",
                tier + 1
            );
            assert!(cv < 0.25, "Mk{} seed {seed}: cv {cv:.3}", tier + 1);
            assert!(starved <= 0.01, "Mk{} seed {seed}: starved {starved:.4}", tier + 1);
            assert!(kj > best_burn() * 1.1, "Mk{} seed {seed}: {kj:.0} kJ a log", tier + 1);
            *mean += rate / seeds.len() as f64;
        }
        println!("Mk{}: {:.2} logs a minute", tier + 1, *mean);
    }
    assert!((5.0..=8.0).contains(&means[0]), "Mk1 {:.2} a minute", means[0]);
    assert!(means[1] >= 1.7 * means[0] && means[2] >= 2.4 * means[0], "{means:?}");
}

/// The head's records holding a stem `logs` long, fed `fed` of a log through.
fn holding(logs: f64, fed: f64) -> Vec<Vec<f32>> {
    use super::view::{Load, Pose};
    use crate::math::Vec3;
    let head = Vec3::new(3.0, 2.0, 0.0);
    let pose =
        Pose { head, toward: Vec3::new(1.0, 0.0, 0.0), tilt: 1.0, arms: 1.0, saw: 0.0, load: Load::Stem { logs, fed } };
    let mut out = Vec::new();
    super::crane::draw(&mut out, Vec3::ZERO, &pose, 8.0);
    out.chunks_exact(crate::factory::INSTANCE_FLOATS).map(<[f32]>::to_vec).collect()
}

#[test]
fn a_bucked_stem_slides_whole_with_its_bark_and_keeps_a_round_crown() {
    use crate::block::tex;
    let stem = |logs, fed| holding(logs, fed).into_iter().find(|r| r[9] as u16 == tex::LOG_SIDE && r[4] > 0.5).unwrap();
    // Bark texel row at the stem's top end (world-scale v there: slide - length / 2 - 1/2), modulo a tile.
    let top_v = |r: &[f32]| (r[7] - r[5] * 0.5).rem_euclid(1.0);
    let (a, b) = (stem(3.0, 0.2), stem(3.0, 0.6));
    assert_eq!((a[5], b[5]), (3.0, 3.0), "the stem keeps its length while it is fed");
    assert_eq!(a[7], b[7], "and its bark slides with it, not over it");
    let moved = ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt();
    assert!((moved - 0.4).abs() < 1e-4, "it moved {moved}");
    assert_eq!(a[11] as u32 & 1, 1, "bark at world scale");
    let (fed, cut) = (stem(3.0, 1.0), stem(2.0, 0.0));
    assert_eq!(cut[5], 2.0, "a cut takes a whole log off");
    assert!((top_v(&fed) - top_v(&cut)).abs() < 1e-4, "the bark stays put on the wood across the cut");
    let leaves: Vec<_> = holding(3.0, 0.0).into_iter().filter(|r| r[9] as u16 == tex::LEAVES).collect();
    assert!(leaves.len() >= 3, "a crown");
    assert!(leaves.iter().all(|r| r[11] as u32 & crate::factory::ROUND != 0), "the crown is round, not a box");
}
