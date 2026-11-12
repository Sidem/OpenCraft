//! The pumpjack: finds its well, fills canisters at the reservoir's pace, stops for want of empties, runs dry and saves;
//! its footprint and ports, the linkage it is drawn with, and a one-block pumpjack from an older save.

use super::*;
use crate::block::{COAL_ORE, GENERATOR, POLE};
use crate::bytes::{ByteReader, ByteWriter};
use crate::deposits::Tier;
use crate::factory::tests::stocked_box;
use crate::factory::{opposite, Machine, DIRS, INSTANCE_FLOATS};
use crate::math::Vec3;
use crate::Game;

/// The pumpjack spot above the first oil vein of a new world: its deposit's centre, on the ground.
fn above_a_vein(world: &World) -> IVec3 {
    let g = world.generator();
    let mut seeded = Vec::new();
    for cz in -32..32 {
        for cx in -32..32 {
            g.seed_deposits(cx, cz, &mut seeded);
            if let Some(d) = seeded.iter().find(|d| d.ore() == OIL_SAND && d.tier() == Tier::Vein) {
                return IVec3::new(d.center.x, g.height_at(d.center.x, d.center.z) + 1, d.center.z);
            }
        }
    }
    panic!("no oil vein within reach");
}

/// A factory with a pumpjack on `spot`, a pole and two generators with coal beside it.
fn rig(world: &mut World, spot: IVec3) -> Factory {
    let mut f = Factory::default();
    let pole = spot + IVec3::new(2, 0, 0);
    f.place(world, POLE, pole, 0, pole, 0);
    // A pumpjack draws 90 kW and a coal generator gives 60.
    for dx in [3, 4] {
        let gen = spot + IVec3::new(dx, 0, 0);
        f.place(world, GENERATOR, gen, 0, gen, 0);
        assert_eq!(f.insert(gen, COAL_ORE.into(), 64), 64);
    }
    f.place(world, PUMPJACK, spot, 0, spot, 0);
    f
}

fn run(f: &mut Factory, world: &mut World, from: u64, seconds: u32) -> u64 {
    let mut events = Vec::new();
    let to = from + seconds as u64 * crate::TICK_RATE as u64;
    for tick in from..to {
        f.update(world, tick, &mut events);
    }
    to
}

fn jack(f: &Factory) -> &Processor {
    f.processors.iter().find(|p| p.spec.block == PUMPJACK).unwrap()
}

#[test]
fn a_pumpjack_drills_down_to_the_oil_below_and_fills_canisters_at_the_veins_pace() {
    let mut world = World::new(1337, 2);
    let spot = above_a_vein(&world);
    let mut f = rig(&mut world, spot);
    let well = jack(&f).pump_state().well.expect("oil under the pumpjack");
    assert_eq!(well.key.ore, OIL_SAND);
    assert!(well.bit.y < spot.y - 10, "the oil is deep: bit at {:?}", well.bit);
    assert_eq!(f.insert(spot, EMPTY_CANISTER, 16), 16);
    run(&mut f, &mut world, 0, 30);
    let p = jack(&f);
    // A vein gives 4 units a second, 0.9 of it kept, 10 to a canister: about 11 in 30 s.
    let full = p.out.count(CRUDE_CANISTER);
    assert!((9..=11).contains(&full), "{full} canisters in 30 s");
    assert_eq!(p.input.count(EMPTY_CANISTER), 16 - full, "every full canister used an empty one");
    assert_eq!(p.status, Status::Working);
    assert!(p.reservoir_line(&f).contains("canisters left"));
}

#[test]
fn it_waits_for_empties_and_stops_drawing_while_a_canister_s_worth_is_held() {
    let mut world = World::new(1337, 2);
    let spot = above_a_vein(&world);
    let mut f = rig(&mut world, spot);
    let key = jack(&f).pump_state().well.unwrap().key;
    run(&mut f, &mut world, 0, 20);
    let p = jack(&f);
    assert_eq!((p.status, p.out.total()), (Status::NoInput, 0));
    let held = p.pump_state().carry;
    assert!((CANISTER_UNITS..CANISTER_UNITS + 1.0).contains(&held), "{held} units kept");
    let left = f.deposits.get(&key).unwrap().remaining_units();
    run(&mut f, &mut world, 1200, 20);
    assert_eq!(f.deposits.get(&key).unwrap().remaining_units(), left, "nothing more is drawn while it waits");
    assert!(!jack(&f).pump_wants_power());
    // Only empty canisters go in.
    let unlocked = f.research.machine_recipes_unlocked();
    assert_eq!(
        f.processors.iter_mut().find(|p| p.spec.block == PUMPJACK).unwrap().insert(CRUDE_CANISTER, 4, &unlocked),
        0
    );
}

#[test]
fn a_pumpjack_with_no_oil_below_says_so_and_the_deposit_is_not_touched() {
    let mut world = World::new(1337, 2);
    let mut f = Factory::default();
    let spot = IVec3::new(5, 80, 5);
    f.place(&mut world, PUMPJACK, spot, 0, spot, 0);
    assert!(jack(&f).pump_state().well.is_none());
    run(&mut f, &mut world, 0, 2);
    assert_eq!(jack(&f).status, Status::NoDeposit);
    assert!(jack(&f).status_text().starts_with("No oil below"));
    assert_eq!(f.deposits.tracked(), 0);
}

#[test]
fn the_well_and_the_oil_kept_survive_a_save() {
    let mut world = World::new(1337, 2);
    let spot = above_a_vein(&world);
    let mut f = rig(&mut world, spot);
    f.insert(spot, EMPTY_CANISTER, 3);
    run(&mut f, &mut world, 0, 5);
    let p = jack(&f);
    let mut w = ByteWriter::default();
    p.write_state(&mut w);
    let mut r = ByteReader::new(&w.bytes);
    let back = Processor::read_state(&mut r).expect("reads back");
    assert_eq!(r.u8(), None, "every byte read");
    assert_eq!(back.pump_state().well.map(|x| (x.key, x.bit)), p.pump_state().well.map(|x| (x.key, x.bit)));
    assert_eq!(
        (back.pump_state().carry, back.status, back.out.count(CRUDE_CANISTER)),
        (p.pump_state().carry, p.status, p.out.count(CRUDE_CANISTER))
    );
}

#[test]
fn a_dry_reservoir_leaves_a_pumpjack_that_says_so_and_asks_for_no_power() {
    let mut world = World::new(1337, 2);
    let spot = above_a_vein(&world);
    let mut f = rig(&mut world, spot);
    let Well { key, bit } = jack(&f).pump_state().well.unwrap();
    // Hand mining takes a block's whole share at once (an hour of pumping would take too long to run).
    for _ in 0..f.deposits.get(&key).unwrap().remaining_blocks {
        f.deposits.hand_mined(&mut world, bit);
    }
    f.insert(spot, EMPTY_CANISTER, 4);
    run(&mut f, &mut world, 0, 2);
    let p = jack(&f);
    assert_eq!(p.status, Status::Exhausted);
    assert_eq!(p.status_text(), "The reservoir is dry");
    assert!(!p.pump_wants_power());
    assert_eq!(p.out.total(), 0);
}

#[test]
fn it_stands_two_by_three_by_three_taking_empties_at_its_back_and_giving_canisters_at_its_front() {
    for dir in 0..4u8 {
        let cells = FOOTPRINT.cells(IVec3::ZERO, dir);
        assert_eq!(cells.len(), 18);
        let outward = |&(c, s): &(IVec3, u8)| cells.contains(&c) && !cells.contains(&(c + DIRS[s as usize]));
        let ins = FOOTPRINT.faces(IVec3::ZERO, dir, Role::In);
        let outs = FOOTPRINT.faces(IVec3::ZERO, dir, Role::Out);
        assert_eq!((ins.len(), outs.len()), (2, 2), "two cells across at each end");
        assert!(ins.iter().all(|f| f.1 == dir && outward(f)), "empties go in at the back: {ins:?}");
        assert!(outs.iter().all(|f| f.1 == opposite(dir) && outward(f)), "canisters leave the front: {outs:?}");
    }
}

#[test]
fn belts_bring_empties_to_its_back_and_take_full_canisters_from_its_front() {
    let mut world = World::new(1337, 2);
    let spot = above_a_vein(&world);
    let mut f = rig(&mut world, spot);
    let cells = jack(&f).cells();
    assert_eq!(cells.len(), 18);
    assert!(cells.iter().all(|c| f.at.contains_key(c)), "every cell is the pumpjack's");
    let (back, s) = FOOTPRINT.faces(spot, 0, Role::In)[0];
    let feed = back + DIRS[s as usize];
    stocked_box(&mut f, feed + DIRS[s as usize], EMPTY_CANISTER, 16);
    f.add_belt(feed, opposite(s));
    let (front, s) = FOOTPRINT.faces(spot, 0, Role::Out)[0];
    let lead = front + DIRS[s as usize];
    f.add_belt(lead, s);
    let end = lead + DIRS[s as usize];
    f.add_storage(end);
    // Only the ports take: a belt at a side or the front brings nothing in.
    let side = cells[2] + DIRS[3];
    assert!(jack(&f).takes_from(back, feed));
    assert!(!jack(&f).takes_from(cells[2], side) && !jack(&f).takes_from(front, lead));
    run(&mut f, &mut world, 0, 30);
    let filled = f.storage_count_at(end, CRUDE_CANISTER);
    assert!((8..=11).contains(&filled), "{filled} canisters reached the box in 30 s");
}

#[test]
fn it_finds_oil_under_any_cell_of_its_base() {
    let mut world = World::new(1337, 2);
    let spot = above_a_vein(&world);
    let mut oil_below =
        |c: IVec3| (1..=MAX_DEPTH).any(|d| world.block_anywhere_or_generate(c - IVec3::new(0, d, 0)) == OIL_SAND);
    // An anchor by the vein's edge: its own column misses the oil, another base cell's finds it.
    let offsets = (1..16).flat_map(|r| [(r, 0), (-r, 0), (0, r), (0, -r)]);
    let anchor = offsets
        .map(|(x, z)| spot + IVec3::new(x, 0, z))
        .find(|&a| !oil_below(a) && FOOTPRINT.cells(a, 0)[..6].iter().any(|&c| oil_below(c)))
        .expect("an edge of the vein");
    let mut f = Factory::default();
    f.place(&mut world, PUMPJACK, anchor, 0, anchor, 0);
    let bit = jack(&f).pump_state().well.expect("oil under its base").bit;
    assert_ne!((bit.x, bit.z), (anchor.x, anchor.z));
    assert!(FOOTPRINT.cells(anchor, 0)[..6].iter().any(|c| (c.x, c.z) == (bit.x, bit.z)));
}

#[test]
fn the_beam_rocks_with_the_crank_through_the_real_linkage() {
    let len = |a: [f64; 2], b: [f64; 2]| (a[0] - b[0]).hypot(a[1] - b[1]);
    let (mut lo, mut hi, mut last) = (f64::MAX, f64::MIN, view::pose(0.0).beam);
    assert!(last.abs() < 1e-9, "level with the pin at the crank's front");
    for k in 0..=720 {
        let p = view::pose(k as f64 / 720.0 * std::f64::consts::TAU);
        assert!((len(p.tail, p.pin) - view::pitman()).abs() < 1e-9, "the pitman arm keeps its length at {k}");
        assert!((len(p.tail, [view::PIVOT[0], view::PIVOT[1]]) - view::TAIL).abs() < 1e-9, "the beam keeps its length");
        assert!((p.beam - last).abs() < 0.005, "the beam moves smoothly: a jump at {k}");
        (lo, hi, last) = (lo.min(p.beam), hi.max(p.beam), p.beam);
    }
    // About 20° each way, as a real unit nods.
    assert!((0.3..0.4).contains(&hi) && (-0.4..-0.3).contains(&lo), "{lo} to {hi} radians");
}

/// Every box `p` draws at `rel`, as its eight corners.
fn corners(p: &Processor) -> Vec<[f32; 3]> {
    let mut out = Vec::new();
    super::super::model::draw(p, &mut out, Vec3::ZERO, 0.0);
    let mut all = Vec::new();
    for r in out.chunks_exact(INSTANCE_FLOATS) {
        let (yaw, pitch) = (r[3], r[12]);
        let (sy, cy, sp, cp) = (yaw.sin(), yaw.cos(), pitch.sin(), pitch.cos());
        let own = [[cy, 0.0, sy], [-sp * sy, cp, sp * cy], [-cp * sy, -sp, cp * cy]];
        for k in 0..8 {
            let sign = |b: usize| if k >> b & 1 == 1 { 0.5 } else { -0.5 };
            let c: [f32; 3] =
                std::array::from_fn(|i| r[i] + (0..3).map(|a| own[a][i] * r[4 + a] * sign(a)).sum::<f32>());
            all.push(c);
        }
    }
    all
}

#[test]
fn its_model_stays_inside_its_cells_through_a_whole_stroke_and_shrinks_into_one_when_compact() {
    for dir in 0..4u8 {
        for compact in [false, true] {
            let mut p = Processor { dir, ..Processor::new(IVec3::ZERO, &PUMPJACK_SPEC, 0) };
            let Extra::Pump(pump) = &mut p.extra else { unreachable!() };
            pump.compact = compact;
            let cells = p.cells();
            for k in 0..24 {
                let Extra::Pump(pump) = &mut p.extra else { unreachable!() };
                pump.crank = k as f64 / 24.0;
                for c in corners(&p) {
                    let inside = cells.iter().any(|cell| {
                        let at = [cell.x, cell.y, cell.z];
                        (0..3).all(|i| (c[i] - at[i] as f32).abs() <= 0.501)
                    });
                    assert!(inside, "facing {dir}, compact {compact}, crank {k}/24: {c:?} sticks out");
                }
            }
        }
    }
}

#[test]
fn the_crank_winds_up_while_it_works_and_stops_where_it_is_when_it_waits() {
    let mut world = World::new(1337, 2);
    let spot = above_a_vein(&world);
    let mut f = rig(&mut world, spot);
    f.insert(spot, EMPTY_CANISTER, 2);
    let t = run(&mut f, &mut world, 0, 4);
    let pace = 1.0 / (CRANK_SECONDS * crate::TICK_RATE as f64);
    let state = |f: &Factory| (jack(f).status, jack(f).pump_state().crank, jack(f).pump_state().spin);
    let (status, _, spin) = state(&f);
    assert_eq!(status, Status::Working);
    assert!((spin - pace).abs() < pace * 0.01, "full pace after a few seconds: {spin} of {pace}");
    // Two canisters take about 6 s; then it waits, eases to a stop, and stays put.
    let t = run(&mut f, &mut world, t, 8);
    let (status, crank, spin) = state(&f);
    assert_eq!((status, spin), (Status::NoInput, 0.0));
    run(&mut f, &mut world, t, 2);
    assert_eq!(state(&f).1, crank, "it stops in place, not back at rest");
}

/// Saved at version 69 by the commit before the pumpjack grew: a pumpjack on the oil vein `above_a_vein` finds (seed
/// 1337), in a straight belt line from east to west, powered by a pole and two coal generators, 9 empties waiting.
const V69_SAVE: &[u8] = include_bytes!("v69.ocworld");

#[test]
fn a_one_block_pumpjack_from_save_69_stays_compact_and_keeps_pumping_through_its_belts() {
    assert_eq!(u32::from_le_bytes(V69_SAVE[4..8].try_into().unwrap()), 69);
    let mut g = Game::load(V69_SAVE, 2).expect("loads");
    let p = jack(&g.sim.factory);
    let (pos, empties) = (p.pos, p.input.count(EMPTY_CANISTER));
    assert!(p.is_compact());
    assert_eq!((p.cells(), p.footprint().is_single()), (vec![pos], true), "one cell, taking from every side");
    assert!(p.takes_from(pos, pos + DIRS[1]), "its belt from the east still feeds it");
    let west = |f: &Factory| f.belts.iter().find(|b| b.pos == pos + DIRS[3]).expect("the belt west").items.len();
    let (on_belt, held) = (west(&g.sim.factory), p.out.count(CRUDE_CANISTER) as usize);
    assert!(on_belt > 0, "the belt west took what it made before");
    g.run_ticks(20 * crate::TICK_RATE);
    let p = jack(&g.sim.factory);
    let made = (empties - p.input.count(EMPTY_CANISTER)) as usize;
    assert!(made >= 5, "it fills canisters: {made} in 20 s");
    let now = west(&g.sim.factory) + p.out.count(CRUDE_CANISTER) as usize;
    assert_eq!(now, on_belt + held + made, "every canister is on its belt or waiting in it");

    // Saved anew it stays compact; placed again it is full size.
    let saved = g.save();
    assert_eq!(u32::from_le_bytes(saved[4..8].try_into().unwrap()), crate::save::SAVE_VERSION);
    let back = Game::load(&saved, 2).expect("loads");
    assert!(jack(&back.sim.factory).is_compact());
    assert_eq!(back.sim.state_hash(), g.sim.state_hash());
    let f = &mut g.sim.factory;
    f.remove(pos);
    f.place(&mut g.sim.world, PUMPJACK, pos, 0, pos, 0);
    assert!(!jack(f).is_compact() && jack(f).cells().len() == 18);
}
