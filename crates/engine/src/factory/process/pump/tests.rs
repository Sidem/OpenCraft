//! The pumpjack: finds its well, fills canisters at the reservoir's pace, stops for want of empties, runs dry and saves.

use super::*;
use crate::block::{COAL_ORE, GENERATOR, POLE};
use crate::bytes::{ByteReader, ByteWriter};
use crate::deposits::Tier;
use crate::factory::Machine;

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
    let well = jack(&f).pump.well.expect("oil under the pumpjack");
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
    let key = jack(&f).pump.well.unwrap().key;
    run(&mut f, &mut world, 0, 20);
    let p = jack(&f);
    assert_eq!((p.status, p.out.total()), (Status::NoInput, 0));
    let held = p.pump.carry;
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
    assert!(jack(&f).pump.well.is_none());
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
    assert_eq!(back.pump.well.map(|x| (x.key, x.bit)), p.pump.well.map(|x| (x.key, x.bit)));
    assert_eq!(
        (back.pump.carry, back.status, back.out.count(CRUDE_CANISTER)),
        (p.pump.carry, p.status, p.out.count(CRUDE_CANISTER))
    );
}

#[test]
fn a_dry_reservoir_leaves_a_pumpjack_that_says_so_and_asks_for_no_power() {
    let mut world = World::new(1337, 2);
    let spot = above_a_vein(&world);
    let mut f = rig(&mut world, spot);
    let Well { key, bit } = jack(&f).pump.well.unwrap();
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
