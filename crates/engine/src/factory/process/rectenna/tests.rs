//! The rectenna: it gives what the power satellites over it beam, capped, only what the grid lacks, only while they are
//! overhead, and saves nothing of its own.

use super::*;
use crate::block::{CONSTRUCTOR, POLE};
use crate::bytes::{ByteReader, ByteWriter};
use crate::factory::tests::recipe_for;
use crate::factory::{Factory, Machine};
use crate::item::{COMMS_SATELLITE, IRON_INGOT, IRON_ROD, POWER_SATELLITE};
use crate::math::IVec3;

const FIELD: IVec3 = IVec3::new(0, 100, 0);
const LOAD: IVec3 = IVec3::new(9, 100, 2);

/// A rectenna, a pole and a 15 kW load (a constructor pressing rods).
fn plant() -> (Factory, World) {
    let mut world = World::new(1, 2);
    let mut f = Factory::default();
    let mut put = |f: &mut Factory, block, pos: IVec3, tier| {
        f.place(&mut world, block, pos, 0, pos - IVec3::new(0, 1, 0), tier);
    };
    put(&mut f, POLE, IVec3::new(6, 100, 2), 3);
    put(&mut f, RECTENNA, FIELD, 0);
    put(&mut f, CONSTRUCTOR, LOAD, 0);
    assert!(f.set_recipe(LOAD, Some(recipe_for(IRON_ROD))).is_some());
    f.insert(LOAD, IRON_INGOT, 64);
    (f, world)
}

fn tick(f: &mut Factory, world: &mut World, from: u64, n: u64) {
    let mut events = Vec::new();
    for t in from..from + n {
        f.update(world, t, &mut events);
    }
}

fn field(f: &Factory) -> &Processor {
    f.processors.iter().find(|p| p.energy() == Energy::Rectenna).unwrap()
}

#[test]
fn with_no_power_satellite_overhead_it_gives_nothing_and_says_so() {
    let (mut f, mut world) = plant();
    f.orbit.launch(crate::item::SURVEY_SATELLITE, FIELD, 0);
    f.orbit.launch(COMMS_SATELLITE, FIELD, 0);
    tick(&mut f, &mut world, 0, 3);
    assert_eq!((field(&f).store.flow, f.power.supply[0]), (0, 0), "survey and comms satellites beam nothing");
    assert_eq!(field(&f).status_text(), "No power satellite overhead");
}

#[test]
fn each_satellite_gives_200_kw_up_to_a_megawatt_and_only_what_the_grid_lacks() {
    let (mut f, mut world) = plant();
    f.orbit.launch(POWER_SATELLITE, FIELD, 0);
    tick(&mut f, &mut world, 0, 5);
    assert_eq!(f.power.demand[0], 15);
    assert_eq!((field(&f).store.flow, f.power.supply[0], field(&f).store.given), (200, 15, 15));
    assert_eq!(f.power.capacity[0], 200, "what the sky could give");
    assert!(field(&f).status_text().contains("giving 15 kW"), "{}", field(&f).status_text());
    for _ in 0..7 {
        f.orbit.launch(POWER_SATELLITE, FIELD, 0);
    }
    tick(&mut f, &mut world, 5, 3);
    assert_eq!(field(&f).store.flow, MAX_KW, "eight would be 1600 kW, capped");
}

#[test]
fn it_goes_dark_when_the_satellites_fly_on() {
    let (mut f, mut world) = plant();
    f.orbit.launch(POWER_SATELLITE, FIELD, 0);
    tick(&mut f, &mut world, 0, 3);
    assert_eq!(field(&f).store.flow, 200);
    // Twenty minutes later it is some 7000 blocks along its track, outside the 1024-block circle.
    tick(&mut f, &mut world, 60 * 60 * 20 - 1, 2);
    assert_eq!((field(&f).store.flow, f.power.supply[0]), (0, 0));
}

#[test]
fn a_rectenna_writes_no_state_of_its_own() {
    let (f, _) = plant();
    let mut w = ByteWriter::default();
    field(&f).write_state(&mut w);
    let back = Processor::read_state(&mut ByteReader::new(&w.bytes)).unwrap();
    assert_eq!((back.spec.block, back.store.flow, back.store.charge), (RECTENNA, 0, 0));
}
