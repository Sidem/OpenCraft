//! Tests of the coal miner: fuel, self-feeding on coal, no grid, the panel and the bytes.

use super::*;
use crate::block::{COAL_MINER, COAL_ORE, GENERATOR, IRON_ORE, MINER, POLE};
use crate::bytes::{ByteReader, ByteWriter};
use crate::deposits::{DepositKey, Tier};
use crate::factory::{Machine, MinerStatus};
use crate::item::{ItemId, MAX_STACK};
use crate::math::IVec3;
use crate::tests::{nearby_ore, run_until_ready};
use crate::Game;

const UP: IVec3 = IVec3::new(0, 1, 0);

/// A coal miner on the first outcrop of `ore` near spawn, with no fuel: its cell, the ore block under it, the deposit.
fn coal_miner_on(g: &mut Game, ore: u8) -> (IVec3, IVec3, DepositKey) {
    let (p, key) = nearby_ore(g)
        .into_iter()
        .find_map(|p| {
            let key = g.sim.factory.deposits.lookup(&mut g.sim.world, p)?;
            (key.tier == Tier::Outcrop && key.ore == ore).then_some((p, key))
        })
        .expect("an outcrop near spawn");
    let m = p + UP;
    g.sim.factory.place(&mut g.sim.world, COAL_MINER, m, 0, p, 0);
    (m, p, key)
}

fn game() -> Game {
    let mut g = Game::new(2024, 3);
    run_until_ready(&mut g);
    g
}

#[test]
fn without_fuel_it_waits_and_says_so() {
    let mut g = game();
    let (m, _, key) = coal_miner_on(&mut g, IRON_ORE);
    let before = g.sim.factory.deposits.get(&key).unwrap().remaining_units();
    g.skip_time(5.0);
    let miner = g.sim.factory.miner_at(m);
    assert!(miner.is_coal() && miner.status == MinerStatus::NoFuel);
    assert_eq!(g.sim.factory.deposits.get(&key).unwrap().remaining_units(), before, "nothing drawn");
    let panel = g.sim.factory.panel(m).expect("a coal miner has a panel");
    assert!(panel.status.contains("Out of fuel"), "{}", panel.status);
}

#[test]
fn coal_burns_only_while_it_digs_and_runs_out() {
    let mut g = game();
    let (m, _, key) = coal_miner_on(&mut g, IRON_ORE);
    assert_eq!(g.sim.factory.insert(m, COAL_ORE.into(), 2), 2);
    // Two coal are 540 kJ: 54 s at 10 kW, and 0.5 units a second at 40% is 0.2 ore a second.
    g.skip_time(100.0);
    let miner = g.sim.factory.miner_at(m);
    assert_eq!(miner.status, MinerStatus::NoFuel);
    assert!((10..=11).contains(&miner.out.total()), "{} iron ore", miner.out.total());
    assert_eq!(miner.burner.fuel.total(), 0);
    let left = g.sim.factory.deposits.get(&key).unwrap().remaining_units();
    g.skip_time(20.0);
    assert_eq!(g.sim.factory.deposits.get(&key).unwrap().remaining_units(), left, "a cold miner draws nothing");

    // One more coal and it digs again.
    g.sim.factory.insert(m, COAL_ORE.into(), 1);
    g.skip_time(3.0);
    assert_eq!(g.sim.factory.miner_at(m).status, MinerStatus::Running);
}

#[test]
fn a_full_output_burns_nothing() {
    let mut g = game();
    let (m, _, _) = coal_miner_on(&mut g, IRON_ORE);
    g.sim.factory.insert(m, COAL_ORE.into(), MAX_STACK);
    g.skip_time(400.0);
    let miner = g.sim.factory.miner_at(m);
    assert_eq!((miner.status, miner.out.total()), (MinerStatus::OutputFull, MAX_STACK));
    let (kj, fuel) = (miner.burner.stored_kj(), miner.burner.fuel.total());
    g.skip_time(30.0);
    let miner = g.sim.factory.miner_at(m);
    assert_eq!((miner.burner.stored_kj(), miner.burner.fuel.total()), (kj, fuel), "fire only for work");
}

#[test]
fn a_coal_miner_on_coal_keeps_its_own_fire_going() {
    let mut g = game();
    let (m, _, _) = coal_miner_on(&mut g, COAL_ORE);
    g.sim.factory.insert(m, COAL_ORE.into(), 1);
    // 120 s of digging burns 1,200 kJ, four and a half coal; it started with one and dug 24.
    g.skip_time(120.0);
    let miner = g.sim.factory.miner_at(m);
    assert_eq!(miner.status, MinerStatus::Running);
    let kept = miner.out.total() + miner.burner.fuel.total();
    assert!((16..=22).contains(&kept), "{kept} coal left over");
    assert!(miner.burner.fuel.total() <= SELF_FUEL_RESERVE, "it keeps only a small reserve back");
}

#[test]
fn a_hand_fed_stack_is_left_alone() {
    let mut g = game();
    let (m, _, _) = coal_miner_on(&mut g, COAL_ORE);
    g.sim.factory.insert(m, COAL_ORE.into(), 20);
    g.skip_time(30.0);
    let miner = g.sim.factory.miner_at(m);
    assert!(miner.burner.fuel.total() >= 18, "it digs into the output, not the fuel: {}", miner.burner.fuel.total());
    assert!(miner.out.total() >= 5);
}

#[test]
fn it_takes_fuel_only_and_gives_it_back_when_taken_apart() {
    let mut g = game();
    let (m, _, _) = coal_miner_on(&mut g, IRON_ORE);
    assert_eq!(g.sim.factory.insert(m, IRON_ORE.into(), 5), 0, "ore is not fuel");
    assert!(!g.sim.factory.wants(m, IRON_ORE.into()) && g.sim.factory.wants(m, COAL_ORE.into()));
    assert_eq!(g.sim.factory.insert(m, COAL_ORE.into(), 100), MAX_STACK);
    let panel = g.sim.factory.panel(m).unwrap();
    assert_eq!(panel.slots.iter().map(|s| s.0).collect::<Vec<_>>(), [ROLE_FUEL, ROLE_OUTPUT]);
    let back = g.sim.factory.remove(m);
    assert!(back.iter().any(|s| s.item == COAL_ORE.into() && s.count == MAX_STACK), "{back:?}");
}

/// A box of `n` `item` two cells east of `m` and a belt from it into `m`.
fn belt_into(g: &mut Game, m: IVec3, item: ItemId, n: u32) {
    let f = &mut g.sim.factory;
    f.add_storage(m + IVec3::new(2, 0, 0));
    f.stock(m + IVec3::new(2, 0, 0), item, n);
    f.add_belt(m + IVec3::new(1, 0, 0), 3);
}

#[test]
fn a_belt_of_coal_keeps_it_running_and_stops_when_the_stack_is_full() {
    let mut g = game();
    let (m, _, _) = coal_miner_on(&mut g, IRON_ORE);
    belt_into(&mut g, m, COAL_ORE.into(), 100);
    g.skip_time(30.0);
    let miner = g.sim.factory.miner_at(m);
    assert_eq!(miner.status, MinerStatus::Running, "fed by the belt, with no hand on it");
    assert!(miner.out.total() >= 4);
    // A stack's worth went in and no more; the rest waits in the box and on the belt.
    assert!(miner.burner.fuel.total() >= MAX_STACK - 2, "{} coal in the fire", miner.burner.fuel.total());
    g.skip_time(30.0);
    let f = &g.sim.factory;
    assert!(f.miner_at(m).burner.fuel.total() <= MAX_STACK);
    let waiting = f.storages[0].buf.total() + f.belts[0].items.iter().map(|i| i.n).sum::<u32>();
    assert!(waiting > 25, "{waiting} coal left outside the miner");
}

#[test]
fn a_belt_gives_it_fuel_only_and_an_electric_miner_nothing() {
    let mut g = game();
    let (m, _, _) = coal_miner_on(&mut g, IRON_ORE);
    belt_into(&mut g, m, IRON_ORE.into(), 10);
    g.skip_time(10.0);
    let miner = g.sim.factory.miner_at(m);
    assert_eq!((miner.burner.fuel.total(), miner.status), (0, MinerStatus::NoFuel), "ore is not fuel");

    let mut g = game();
    let (m, p, _) = coal_miner_on(&mut g, IRON_ORE);
    g.sim.factory.remove(m);
    g.sim.factory.place(&mut g.sim.world, MINER, m, 0, p, 0);
    belt_into(&mut g, m, COAL_ORE.into(), 10);
    g.skip_time(10.0);
    let miner = g.sim.factory.miner_at(m);
    assert_eq!((miner.burner.fuel.total(), miner.out.total()), (0, 0), "a powered miner takes no belt's coal");
    let f = &g.sim.factory;
    let outside = f.storages[0].buf.total() + f.belts[0].items.iter().map(|i| i.n).sum::<u32>();
    assert_eq!(outside, 10, "the coal stayed on the belt and in the box");
}

#[test]
fn it_never_joins_a_grid_and_has_no_upgrade() {
    let mut g = game();
    let (m, _, _) = coal_miner_on(&mut g, IRON_ORE);
    for (b, at) in [(POLE, m + IVec3::new(1, 0, 0)), (GENERATOR, m + IVec3::new(2, 0, 0))] {
        g.sim.factory.place(&mut g.sim.world, b, at, 0, at, 0);
    }
    g.sim.factory.insert(m + IVec3::new(2, 0, 0), COAL_ORE.into(), 10);
    g.skip_time(5.0);
    let f = &g.sim.factory;
    assert!(!f.miner_at(m).wants_power() && !f.is_wired(m), "a pole beside it does not take it");
    assert_eq!((f.tiered_at(m), f.next_upgrade(m)), (None, None));
    assert_eq!(f.panel(m + IVec3::new(1, 0, 0)).map(|p| p.slots.len()), None);
}

#[test]
fn its_bytes_round_trip_and_an_electric_miner_adds_none() {
    let mut g = game();
    let (m, _, _) = coal_miner_on(&mut g, IRON_ORE);
    g.sim.factory.insert(m, COAL_ORE.into(), 7);
    g.skip_time(20.0);
    let miner = g.sim.factory.miner_at(m);
    let mut w = ByteWriter::default();
    miner.write_state(&mut w);
    let mut r = ByteReader::new(&w.bytes);
    let back = Miner::read_state(&mut r).expect("reads back");
    assert!(r.is_done());
    assert_eq!((back.tier, back.burner.energy), (COAL_TIER, miner.burner.energy));
    assert_eq!(back.burner.fuel.slots, miner.burner.fuel.slots);
    assert_eq!(back.out.total(), miner.out.total());

    // An electric miner writes what it always did: nothing follows its tier byte.
    let mut w = ByteWriter::default();
    Miner::new(IVec3::ZERO, 3, None, 2).write_state(&mut w);
    let mut r = ByteReader::new(&w.bytes);
    assert_eq!(Miner::read_state(&mut r).map(|m| m.tier), Some(2));
    assert!(r.is_done());
}
