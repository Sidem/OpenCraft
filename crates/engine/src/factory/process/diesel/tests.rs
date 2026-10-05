//! The diesel generator in a bare factory: a Mk1 constructor pressing rods is the steady 15 kW load, kept fed.

use super::*;
use crate::block::{CONSTRUCTOR, POLE};
use crate::bytes::{ByteReader, ByteWriter};
use crate::factory::tests::recipe_for;
use crate::factory::{Factory, Machine};
use crate::item::{IRON_INGOT, IRON_ROD};
use crate::math::IVec3;
use crate::world::World;

const LOAD: IVec3 = IVec3::new(8, 0, 0);
const GEN: IVec3 = IVec3::new(0, 0, -4);

/// A Mk4 pole, a diesel generator and, if `load`, the constructor on one grid.
fn rig(load: bool) -> Factory {
    let mut f = Factory::default();
    let mut world = World::new(1, 2);
    let mut put = |f: &mut Factory, block, pos: IVec3, tier| {
        f.place(&mut world, block, pos, 0, pos - IVec3::new(0, 1, 0), tier);
    };
    put(&mut f, POLE, IVec3::new(4, 0, 4), 3);
    put(&mut f, DIESEL_GENERATOR, GEN, 0);
    if load {
        put(&mut f, CONSTRUCTOR, LOAD, 0);
        assert!(f.set_recipe(LOAD, Some(recipe_for(IRON_ROD))).is_some());
    }
    f
}

fn run(f: &mut Factory, seconds: u32, load: bool) {
    let mut world = World::new(1, 2);
    let mut events = Vec::new();
    for tick in 0..seconds * TICK_RATE {
        f.update(&mut world, tick as u64, &mut events);
        if load && tick % 20 == 0 {
            f.insert(LOAD, IRON_INGOT, 64);
            let crate::factory::Slot::Process(i) = f.at[&LOAD] else { unreachable!() };
            let p = &mut f.processors[i as usize];
            p.out.take(0, p.out.total());
        }
    }
}

fn generator(f: &Factory) -> &Processor {
    f.processors.iter().find(|p| p.spec.block == DIESEL_GENERATOR).unwrap()
}

#[test]
fn it_gives_what_the_grid_lacks_and_hands_the_empty_canister_back() {
    let mut f = rig(true);
    assert_eq!(f.insert(GEN, DIESEL_CANISTER, 2), 2);
    assert_eq!(f.insert(GEN, EMPTY_CANISTER, 1), 0, "only diesel goes in");
    run(&mut f, 10, true);
    let g = generator(&f);
    assert_eq!((g.input.count(DIESEL_CANISTER), g.out.count(EMPTY_CANISTER)), (1, 1), "one lit, its shell out");
    let burned = CANISTER_KJ * TICK_RATE - g.store.charge;
    assert!(burned > 0 && burned <= 15 * 10 * TICK_RATE, "burned {burned} kW·ticks for a 15 kW load");
    assert_eq!(f.power.supply[0], 15);
    assert!(g.status_text().starts_with("Giving 15 of 400 kW"), "{}", g.status_text());
}

#[test]
fn with_no_load_it_burns_nothing_and_stands_by() {
    let mut f = rig(false);
    f.insert(GEN, DIESEL_CANISTER, 2);
    run(&mut f, 5, false);
    let g = generator(&f);
    assert_eq!((g.input.count(DIESEL_CANISTER), g.store.charge), (2, 0));
    assert!(g.status_text().starts_with("Standing by"), "{}", g.status_text());
    assert_eq!(f.power.capacity[0], DIESEL_KW, "it could give 400 kW");
}

#[test]
fn a_full_output_stops_it_lighting_and_an_empty_one_says_so() {
    let mut f = rig(true);
    run(&mut f, 2, true);
    assert!(generator(&f).status_text().starts_with("Out of fuel"));
    f.insert(GEN, DIESEL_CANISTER, 1);
    let crate::factory::Slot::Process(i) = f.at[&GEN] else { unreachable!() };
    f.processors[i as usize].out.add(EMPTY_CANISTER, 16);
    run(&mut f, 2, true);
    let g = generator(&f);
    assert_eq!((g.status, g.input.count(DIESEL_CANISTER), g.store.charge), (Status::OutputFull, 1, 0));
    assert_eq!(f.power.supply[0], 0);
}

#[test]
fn its_charge_survives_a_save() {
    let mut f = rig(false);
    let crate::factory::Slot::Process(i) = f.at[&GEN] else { unreachable!() };
    f.processors[i as usize].store.charge = 123_456;
    let mut w = ByteWriter::default();
    f.processors[i as usize].write_state(&mut w);
    let back = Processor::read_state(&mut ByteReader::new(&w.bytes)).unwrap();
    assert_eq!(back.store.charge, 123_456);
}
