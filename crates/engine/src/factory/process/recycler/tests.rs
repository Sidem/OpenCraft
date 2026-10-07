//! The recycler: what it takes, what it pays, that a dear item is paid out over time, that it stops when the coins have
//! nowhere to go and starts again, and that what it owes is saved and returned.

use crate::block::{COAL_ORE, DIRT, GENERATOR, POLE};
use crate::bytes::{ByteReader, ByteWriter};
use crate::factory::{Factory, Machine};
use crate::item::{ItemId, COIN, COIN_STACK, IRON_INGOT, IRON_PLATE, MOTOR, STEEL_PICKAXE, STICK};
use crate::math::IVec3;
use crate::recipes::recycling::{coins, millicoins};
use crate::research::TECHS;
use crate::world::World;

use super::*;

/// A recycler at the origin, a pole and two generators (120 kW) with coal, and the Recycling tech done.
fn rig(powered: bool) -> Factory {
    let mut world = World::new(1, 2);
    let mut f = Factory::default();
    let tech = TECHS.iter().position(|t| t.name == "Recycling").unwrap() as u8;
    (0..TECHS[tech as usize].units).for_each(|_| f.research.add_unit(tech));
    f.place(&mut world, RECYCLER, IVec3::ZERO, 0, IVec3::ZERO, 0);
    if powered {
        let pole = IVec3::new(4, 0, 0);
        f.place(&mut world, POLE, pole, 0, pole, 0);
        for dz in 0..2 {
            let gen = pole + IVec3::new(1, 0, dz);
            f.place(&mut world, GENERATOR, gen, 0, gen, 0);
            assert_eq!(f.insert(gen, COAL_ORE.into(), 64), 64);
        }
    }
    f
}

fn run(f: &mut Factory, seconds: u32) {
    let mut world = World::new(1, 2);
    let mut events = Vec::new();
    for tick in 0..seconds * crate::TICK_RATE {
        f.update(&mut world, tick as u64, &mut events);
    }
}

fn recycler(f: &Factory) -> &Processor {
    f.processors.iter().find(|p| p.spec.pick == Pick::Recycle).unwrap()
}

#[test]
fn it_takes_any_item_but_the_coin() {
    let mut f = rig(true);
    assert_eq!(f.insert(IVec3::ZERO, DIRT.into(), 10), 10);
    assert_eq!(f.insert(IVec3::ZERO, IRON_PLATE, 10), 10);
    assert_eq!(f.insert(IVec3::ZERO, COIN, 10), 0, "a coin is worth nothing");
}

#[test]
fn it_destroys_items_and_pays_what_they_are_worth() {
    let mut f = rig(true);
    f.insert(IVec3::ZERO, IRON_INGOT, 4);
    f.insert(IVec3::ZERO, IRON_PLATE, 2);
    run(&mut f, 3);
    let r = recycler(&f);
    assert_eq!((r.input.total(), r.out.count(COIN)), (0, 4 * 2 + 2 * 8));
    assert_eq!(r.power(), 90);
}

#[test]
fn six_hatches_in_and_two_out() {
    let faces = |role| RECYCLER_SPEC.footprint.faces(IVec3::ZERO, 0, role).len();
    assert_eq!((faces(Role::In), faces(Role::Out), faces(Role::Side)), (6, 2, 0));
}

#[test]
fn coins_stack_to_1024_and_a_dear_item_is_paid_over_time() {
    let mut f = rig(true);
    f.insert(IVec3::ZERO, IRON_PLATE, 64);
    run(&mut f, 20);
    let r = recycler(&f);
    assert_eq!(r.out.count(COIN), 64 * 8);
    assert!(r.out.slots.iter().all(|s| s.count <= COIN_STACK));
}

#[test]
fn full_coin_slots_stop_it_and_taking_coins_starts_it_again() {
    let mut f = rig(true);
    assert!(coins(MOTOR) > 50, "a motor pays a lot");
    f.insert(IVec3::ZERO, MOTOR, 64);
    run(&mut f, 60);
    let r = recycler(&f);
    assert_eq!(r.status, Status::OutputFull);
    assert_eq!((r.out.count(COIN), r.owed >= OWED_MAX), (2 * COIN_STACK, true));
    assert!(r.input.total() > 0);
    assert!(r.status_text().starts_with("Output full"), "{}", r.status_text());
    f.take_contents(IVec3::ZERO, |item, n| if item == COIN { n } else { 0 });
    run(&mut f, 60);
    assert_eq!(recycler(&f).input.total(), 0, "every motor destroyed once there was room");
}

#[test]
fn without_power_it_destroys_nothing() {
    let mut f = rig(false);
    f.insert(IVec3::ZERO, IRON_INGOT, 4);
    run(&mut f, 3);
    assert_eq!((recycler(&f).input.total(), recycler(&f).status), (4, Status::NoPower));
}

#[test]
fn what_it_owes_is_saved_and_given_back_when_it_is_broken() {
    let mut p = Processor::new(IVec3::new(1, 2, 3), &RECYCLER_SPEC, 0);
    p.owed = 2_500_700;
    p.input.add(ItemId::block(DIRT), 3);
    let mut w = ByteWriter::default();
    p.write_state(&mut w);
    let mut r = ByteReader::new(&w.bytes);
    let back = Processor::read_state(&mut r).unwrap();
    assert_eq!((back.owed, back.input.total()), (2_500_700, 3));
    assert_eq!(r.u8(), None, "every byte read");
    let coins: Vec<u32> = back.contents().iter().filter(|s| s.item == COIN).map(|s| s.count).collect();
    assert_eq!(coins, [COIN_STACK, COIN_STACK, 452]);
}

#[test]
fn halves_add_up_and_a_tool_goes_in_one_piece() {
    let mut f = rig(true);
    f.insert(IVec3::ZERO, STICK, 3);
    run(&mut f, 3);
    assert_eq!(recycler(&f).out.count(COIN), 1, "three sticks pay 1½ coins: one is paid, the half waits");
    let mut f = rig(true);
    let uses = crate::tools::STEEL_TIER.uses;
    f.insert(IVec3::ZERO, STEEL_PICKAXE, uses);
    run(&mut f, 3);
    let r = recycler(&f);
    assert_eq!(r.input.total(), 0, "a 1500-use pickaxe is one item, destroyed at once");
    assert_eq!(r.out.count(COIN) + r.owed / crate::recipes::recycling::MILLI, uses * millicoins(STEEL_PICKAXE) / 1000);
}

#[test]
fn it_is_unlocked_after_blue_science() {
    let tech = TECHS.iter().find(|t| t.name == "Recycling").unwrap();
    let blue = TECHS.iter().position(|t| t.name == "Blue Science").unwrap() as u8;
    assert_eq!(tech.needs, [blue]);
}
