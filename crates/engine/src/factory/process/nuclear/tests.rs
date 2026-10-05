//! The centrifuge and the reactor in a bare factory: three Mk4 constructors pressing rods are the steady 225 kW load
//! (more than the 100 kW a reactor sheds without coolant), kept fed.

use super::*;
use crate::block::{COAL_ORE, CONSTRUCTOR, GENERATOR, POLE, URANIUM_ORE};
use crate::bytes::{ByteReader, ByteWriter};
use crate::factory::tests::recipe_for;
use crate::factory::{Factory, Machine, Slot};
use crate::item::{IRON_INGOT, IRON_ROD, STEEL_PLATE};
use crate::math::IVec3;
use crate::research::TECHS;
use crate::world::World;

const REACTOR_AT: IVec3 = IVec3::new(0, 0, -6);
const LOADS: [IVec3; 3] = [IVec3::new(8, 0, 0), IVec3::new(8, 0, 2), IVec3::new(8, 0, 4)];
const LOAD_KW: u32 = 225;

/// A Mk4 pole, a reactor and, if `load`, the three constructors on one grid.
fn rig(load: bool) -> Factory {
    let mut f = Factory::default();
    let mut world = World::new(1, 2);
    let mut put = |f: &mut Factory, block, pos: IVec3, tier| {
        f.place(&mut world, block, pos, 0, pos - IVec3::new(0, 1, 0), tier);
    };
    put(&mut f, POLE, IVec3::new(4, 0, 4), 3);
    put(&mut f, REACTOR, REACTOR_AT, 0);
    for at in LOADS.into_iter().filter(|_| load) {
        put(&mut f, CONSTRUCTOR, at, 3);
        assert!(f.set_recipe(at, Some(recipe_for(IRON_ROD))).is_some());
    }
    f
}

fn run(f: &mut Factory, seconds: u32, load: bool) {
    let mut world = World::new(1, 2);
    let mut events = Vec::new();
    for tick in 0..seconds * TICK_RATE {
        f.update(&mut world, tick as u64, &mut events);
        if load && tick % 20 == 0 {
            for at in LOADS {
                f.insert(at, IRON_INGOT, 64);
                let Slot::Process(i) = f.at[&at] else { unreachable!() };
                let p = &mut f.processors[i as usize];
                p.out.take(0, p.out.total());
            }
        }
    }
}

fn reactor_mut(f: &mut Factory) -> &mut Processor {
    f.processors.iter_mut().find(|p| p.spec.block == REACTOR).unwrap()
}

fn reactor(f: &Factory) -> &Processor {
    f.processors.iter().find(|p| p.spec.block == REACTOR).unwrap()
}

#[test]
fn it_takes_only_fuel_cells_and_gives_what_the_grid_lacks() {
    let mut f = rig(true);
    assert_eq!(f.insert(REACTOR_AT, FUEL_CELL, 2), 2);
    assert_eq!(f.insert(REACTOR_AT, DIESEL_CANISTER, 1), 0, "only fuel cells go in");
    reactor_mut(&mut f).steam.water = COOLANT_UNIT;
    run(&mut f, 10, true);
    let r = reactor(&f);
    assert_eq!((r.input.count(FUEL_CELL), r.status), (1, Status::Working), "one cell lit");
    let burned = CELL_CHARGE - r.store.charge;
    assert!(burned > 0 && burned <= LOAD_KW * 10 * TICK_RATE, "burned {burned} kW·ticks");
    assert_eq!(f.power.supply[0], LOAD_KW);
    assert!(r.status_text().starts_with("Giving 225 of 2000 kW"), "{}", r.status_text());
}

#[test]
fn with_no_load_it_burns_nothing_and_stands_by() {
    let mut f = rig(false);
    f.insert(REACTOR_AT, FUEL_CELL, 2);
    run(&mut f, 5, false);
    let r = reactor(&f);
    assert_eq!((r.input.count(FUEL_CELL), r.store.charge), (2, 0));
    assert!(r.status_text().starts_with("Standing by"), "{}", r.status_text());
    assert_eq!(f.power.capacity[0], REACTOR_KW);
}

#[test]
fn coolant_is_spent_as_it_gives_and_keeps_it_cool() {
    let mut f = rig(true);
    f.insert(REACTOR_AT, FUEL_CELL, 1);
    reactor_mut(&mut f).steam.water = COOLANT_UNIT;
    run(&mut f, 10, true);
    let r = reactor(&f);
    assert_eq!(r.progress, 0, "cooled, it never heats");
    let spent = COOLANT_UNIT - r.steam.water;
    assert!(spent > 0 && spent <= LOAD_KW * 10 * TICK_RATE, "spent {spent}");
}

#[test]
fn without_coolant_it_overheats_shuts_down_and_restarts_when_cooled() {
    let mut f = rig(true);
    f.insert(REACTOR_AT, FUEL_CELL, 1);
    reactor_mut(&mut f).progress = HEAT_LIMIT - 1_000;
    run(&mut f, 10, true);
    let r = reactor(&f);
    assert_eq!(r.status, Status::Overheated);
    assert!(r.status_text().starts_with("Overheated"), "{}", r.status_text());
    assert_eq!(f.power.supply[0], 0, "a shut-down reactor gives nothing");
    assert_eq!(f.power.capacity[0], 0);
    reactor_mut(&mut f).steam.water = COOLANT_UNIT * 2;
    run(&mut f, 12, true);
    assert_eq!(reactor(&f).status, Status::Working, "coolant brought the heat down and it restarted");
    assert_eq!(f.power.supply[0], LOAD_KW);
}

#[test]
fn a_small_load_never_heats_it_even_without_coolant() {
    let mut f = rig(false);
    f.insert(REACTOR_AT, FUEL_CELL, 1);
    reactor_mut(&mut f).progress = 5_000;
    run(&mut f, 5, false);
    assert_eq!(reactor(&f).progress, 0, "it sheds heat on its own");
}

#[test]
fn its_charge_heat_and_coolant_survive_a_save() {
    let mut f = rig(false);
    let r = reactor_mut(&mut f);
    (r.store.charge, r.progress, r.steam.water, r.status) = (123_456, 77_000, 9_000, Status::Overheated);
    let mut w = ByteWriter::default();
    reactor(&f).write_state(&mut w);
    let back = Processor::read_state(&mut ByteReader::new(&w.bytes)).unwrap();
    assert_eq!(
        (back.store.charge, back.progress, back.steam.water, back.status),
        (123_456, 77_000, 9_000, Status::Overheated)
    );
}

#[test]
fn the_centrifuge_makes_a_fuel_cell_from_uranium_and_a_steel_plate() {
    let mut world = World::new(1, 2);
    let mut f = Factory::default();
    let tech = TECHS.iter().position(|t| t.name == "Nuclear Power").unwrap() as u8;
    (0..TECHS[tech as usize].units).for_each(|_| f.research.add_unit(tech));
    let at = IVec3::ZERO;
    f.place(&mut world, CENTRIFUGE, at, 0, at, 0);
    let pole = IVec3::new(4, 0, 0);
    f.place(&mut world, POLE, pole, 0, pole, 0);
    for dz in 0..4 {
        let gen = pole + IVec3::new(1, 0, dz);
        f.place(&mut world, GENERATOR, gen, 0, gen, 0);
        assert_eq!(f.insert(gen, COAL_ORE.into(), 64), 64);
    }
    assert_eq!(f.insert(at, URANIUM_ORE.into(), 4), 4);
    assert_eq!(f.insert(at, STEEL_PLATE, 1), 1);
    let mut events = Vec::new();
    for tick in 0..13 * TICK_RATE {
        f.update(&mut world, tick as u64, &mut events);
    }
    let c = &f.processors[0];
    assert_eq!((c.out.count(FUEL_CELL), c.power()), (1, 200));
}
