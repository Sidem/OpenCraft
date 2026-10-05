//! The refinery, the cracker, the chemical plant and the electrolyser: water from a pipe, shells through, every stream a full-stop if it backs up.

use crate::block::{COAL_ORE, ELECTROLYSER, GENERATOR, POLE, PUMP};
use crate::bytes::{ByteReader, ByteWriter};
use crate::factory::{Factory, Machine, DIRS};
use crate::item::{
    ItemId, ACID_CANISTER, CRUDE_CANISTER, DIESEL_CANISTER, EMPTY_CANISTER, HEAVY_OIL_CANISTER, HYDROGEN_CANISTER,
    LUBRICANT_CANISTER, NAPHTHA_CANISTER, OXYGEN_CANISTER, PLASTIC, SULFUR,
};
use crate::math::IVec3;
use crate::recipes::{ACID_RECIPE, LUBRICANT_RECIPE, PLASTIC_RECIPE};
use crate::research::TECHS;
use crate::world::World;

use super::super::{spec, Processor, Status};
use super::*;

/// A plant: `block` at the origin, a pole and three generators (180 kW) with coal, Refining researched, and, if
/// `water`, a pump piece on the machine's water inlet holding ten units.
fn plant(block: u8, water: bool) -> Factory {
    let mut world = World::new(1, 2);
    let mut f = Factory::default();
    for name in ["Refining", "Plastics", "Acids and Lubricants", "Electrolysis"] {
        let tech = TECHS.iter().position(|t| t.name == name).unwrap() as u8;
        (0..TECHS[tech as usize].units).for_each(|_| f.research.add_unit(tech));
    }
    f.place(&mut world, block, IVec3::ZERO, 0, IVec3::ZERO, 0);
    let pole = IVec3::new(4, 0, 0);
    f.place(&mut world, POLE, pole, 0, pole, 0);
    for dz in 0..3 {
        let gen = pole + IVec3::new(1, 0, dz);
        f.place(&mut world, GENERATOR, gen, 0, gen, 0);
        assert_eq!(f.insert(gen, COAL_ORE.into(), 64), 64);
    }
    if water {
        let tap = f.processors[0].pipe_ports().into_iter().find(|p| p.2 == Role::Water).expect("a water inlet");
        let at = tap.0 + DIRS[tap.1 as usize];
        f.place(&mut world, PUMP, at, 0, at, 0);
        f.pipework.iter_mut().for_each(|p| p.held = 10);
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

fn machine(f: &Factory) -> &Processor {
    &f.processors[0]
}

fn side(f: &Factory, item: ItemId) -> u32 {
    machine(f).side.count(item)
}

#[test]
fn a_refinery_turns_three_crude_canisters_and_a_unit_of_water_into_four_products() {
    let mut f = plant(REFINERY, true);
    assert_eq!(f.insert(IVec3::ZERO, CRUDE_CANISTER, 6), 6);
    run(&mut f, 7);
    // One batch (6 s): the main product at the front, the rest at the side hatches.
    assert_eq!(machine(&f).out.count(NAPHTHA_CANISTER), 1);
    assert_eq!((side(&f, DIESEL_CANISTER), side(&f, HEAVY_OIL_CANISTER), side(&f, SULFUR)), (1, 1, 1));
    assert_eq!(machine(&f).input.count(CRUDE_CANISTER), 0, "the second batch has begun and took the rest");
    assert_eq!(machine(&f).status, Status::Working);
    assert_eq!(machine(&f).power(), 150);
    run(&mut f, 6);
    assert_eq!(machine(&f).out.count(NAPHTHA_CANISTER), 2);
    assert_eq!(machine(&f).status, Status::NoInput);
}

#[test]
fn without_a_water_pipe_it_waits_and_asks_for_no_power() {
    let mut f = plant(REFINERY, false);
    f.insert(IVec3::ZERO, CRUDE_CANISTER, 3);
    run(&mut f, 3);
    let p = machine(&f);
    assert_eq!((p.status, p.input.count(CRUDE_CANISTER)), (Status::NoWater, 3));
    assert!(p.status_text().starts_with("Out of water"));
    let unlocked = f.research.machine_recipes_unlocked();
    assert!(!p.wants_power(&unlocked, &f.research));
}

#[test]
fn a_stream_nobody_takes_stops_the_refinery_and_the_status_names_it() {
    let mut f = plant(REFINERY, true);
    // Crude keeps coming (a belt would top it up) but no belt takes the byproducts away.
    for _ in 0..8 {
        f.insert(IVec3::ZERO, CRUDE_CANISTER, 15);
        run(&mut f, 12);
    }
    let p = machine(&f);
    assert_eq!(p.status, Status::OutputFull);
    assert!(p.status_text().contains("has nowhere to go"), "{}", p.status_text());
    assert!(p.input.count(CRUDE_CANISTER) > 0, "it stopped with crude left");
    assert!(p.side.total() >= 16);
}

#[test]
fn a_cracker_splits_two_heavy_oil_canisters_into_naphtha_and_diesel() {
    let mut f = plant(CRACKER, true);
    assert_eq!(f.insert(IVec3::ZERO, HEAVY_OIL_CANISTER, 2), 2);
    assert_eq!(f.insert(IVec3::ZERO, CRUDE_CANISTER, 2), 0, "it takes only heavy oil");
    run(&mut f, 5);
    let p = machine(&f);
    assert_eq!((p.out.count(NAPHTHA_CANISTER), side(&f, DIESEL_CANISTER)), (1, 1));
    assert_eq!(p.power(), 90);
}

#[test]
fn nothing_goes_in_before_refining_is_researched_and_the_water_tank_is_saved() {
    let mut f = plant(REFINERY, true);
    let mut fresh = Processor::new(IVec3::ZERO, spec(REFINERY).unwrap(), 0);
    assert!(!fresh.can_accept_fresh(CRUDE_CANISTER), "locked until Refining");
    f.insert(IVec3::ZERO, CRUDE_CANISTER, 3);
    run(&mut f, 2);
    let p = machine(&f);
    assert!(p.steam.water > 0, "it drew water from the pump");
    fresh.steam.water = p.steam.water;
    let mut w = ByteWriter::default();
    p.write_state(&mut w);
    let mut r = ByteReader::new(&w.bytes);
    let back = Processor::read_state(&mut r).expect("reads back");
    assert_eq!(r.u8(), None, "every byte read");
    assert_eq!((back.steam.water, back.status, back.batch), (p.steam.water, p.status, p.batch));
}

fn chosen(block: u8, recipe: u16, water: bool) -> Factory {
    let mut f = plant(block, water);
    assert!(f.set_recipe(IVec3::ZERO, Some(recipe)).is_some());
    f
}

#[test]
fn the_chemical_plant_presses_plastic_from_naphtha_and_coal_and_gives_the_shells_back() {
    let mut f = chosen(CHEMICAL_PLANT, PLASTIC_RECIPE, false);
    assert_eq!(f.insert(IVec3::ZERO, NAPHTHA_CANISTER, 4), 4);
    assert_eq!(f.insert(IVec3::ZERO, COAL_ORE.into(), 2), 2);
    assert_eq!(f.insert(IVec3::ZERO, CRUDE_CANISTER, 2), 0, "only what the recipe uses");
    run(&mut f, 5);
    let p = machine(&f);
    assert_eq!((p.out.count(PLASTIC), side(&f, EMPTY_CANISTER)), (4, 2), "one batch in 4 s");
    assert_eq!(p.power(), 120);
    run(&mut f, 4);
    assert_eq!((machine(&f).out.count(PLASTIC), side(&f, EMPTY_CANISTER)), (8, 4));
}

#[test]
fn acid_needs_water_and_lubricant_does_not() {
    let mut dry = chosen(CHEMICAL_PLANT, ACID_RECIPE, false);
    dry.insert(IVec3::ZERO, SULFUR, 1);
    dry.insert(IVec3::ZERO, EMPTY_CANISTER, 1);
    run(&mut dry, 4);
    assert_eq!((machine(&dry).status, machine(&dry).out.total()), (Status::NoWater, 0));

    let mut f = chosen(CHEMICAL_PLANT, ACID_RECIPE, true);
    f.insert(IVec3::ZERO, SULFUR, 1);
    f.insert(IVec3::ZERO, EMPTY_CANISTER, 1);
    run(&mut f, 4);
    assert_eq!(machine(&f).out.count(ACID_CANISTER), 1);

    let mut oil = chosen(CHEMICAL_PLANT, LUBRICANT_RECIPE, false);
    oil.insert(IVec3::ZERO, HEAVY_OIL_CANISTER, 1);
    oil.insert(IVec3::ZERO, EMPTY_CANISTER, 1);
    run(&mut oil, 4);
    assert_eq!(machine(&oil).out.count(LUBRICANT_CANISTER), 2, "one heavy oil canister makes two of lubricant");
}

#[test]
fn the_electrolyser_splits_water_into_two_hydrogen_and_an_oxygen_canister_and_gives_the_shells_back() {
    let mut dry = plant(ELECTROLYSER, false);
    assert_eq!(dry.insert(IVec3::ZERO, EMPTY_CANISTER, 3), 3);
    run(&mut dry, 3);
    assert_eq!(machine(&dry).status, Status::NoWater);
    let mut f = plant(ELECTROLYSER, true);
    assert_eq!(f.insert(IVec3::ZERO, CRUDE_CANISTER, 3), 0, "only empties go in");
    assert_eq!(f.insert(IVec3::ZERO, EMPTY_CANISTER, 3), 3);
    // 180 kW of the 500 it wants: a 6 s batch takes about 17 s.
    run(&mut f, 25);
    assert_eq!((machine(&f).out.count(HYDROGEN_CANISTER), side(&f, OXYGEN_CANISTER)), (2, 1));
    assert_eq!(machine(&f).power(), 500);
}
