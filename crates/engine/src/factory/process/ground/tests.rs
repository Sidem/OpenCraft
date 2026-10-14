//! The ground station: it gives 40 TF per science satellite overhead (up to 100), scaled by its power, only while the
//! satellites are overhead, and saves nothing of its own.

use super::*;
use crate::block::{FIBRE_NODE, OPTIMIZER, POLE};
use crate::bytes::{ByteReader, ByteWriter};
use crate::factory::{Factory, Machine};
use crate::item::{COMMS_SATELLITE, SCIENCE_SATELLITE};
use crate::math::IVec3;
use crate::world::World;

const DISH: IVec3 = IVec3::new(0, 100, 0);

/// A station and an optimizer (a data consumer) on one pole and one fibre node. Nothing feeds the pole, so these tests
/// look at what the station offers, not at what its grid is given.
fn plant() -> (Factory, World) {
    let mut world = World::new(1, 2);
    let mut f = Factory::default();
    let mut put = |f: &mut Factory, block, pos: IVec3| {
        f.place(&mut world, block, pos, 0, pos - IVec3::new(0, 1, 0), 0);
    };
    put(&mut f, POLE, IVec3::new(5, 100, 5));
    put(&mut f, FIBRE_NODE, IVec3::new(5, 100, 0));
    put(&mut f, GROUND_STATION, DISH);
    put(&mut f, OPTIMIZER, IVec3::new(8, 100, 0));
    (f, world)
}

fn tick(f: &mut Factory, world: &mut World, from: u64, n: u64) {
    let mut events = Vec::new();
    for t in from..from + n {
        f.update(world, t, &mut events);
    }
}

fn station(f: &Factory) -> &Processor {
    f.processors.iter().find(|p| p.spec.block == GROUND_STATION).unwrap()
}

#[test]
fn with_no_science_satellite_overhead_it_offers_nothing_and_says_so() {
    let (mut f, mut world) = plant();
    f.orbit.launch(COMMS_SATELLITE, DISH, 0);
    tick(&mut f, &mut world, 0, 3);
    assert_eq!((station(&f).store.flow, station(&f).offered_tf()), (0, 0));
    assert!(station(&f).status_text().contains("No science satellite overhead"), "{}", station(&f).status_text());
}

#[test]
fn each_science_satellite_offers_40_tf_up_to_the_cap() {
    let (mut f, mut world) = plant();
    f.orbit.launch(SCIENCE_SATELLITE, DISH, 0);
    tick(&mut f, &mut world, 0, 3);
    assert_eq!(station(&f).offered_tf(), TF_PER_SATELLITE);
    f.orbit.launch(SCIENCE_SATELLITE, DISH, 0);
    f.orbit.launch(SCIENCE_SATELLITE, DISH, 0);
    tick(&mut f, &mut world, 3, 3);
    assert_eq!(station(&f).offered_tf(), MAX_TF, "three would be 120 TF, capped");
}

#[test]
fn it_stops_offering_when_the_satellites_fly_on() {
    let (mut f, mut world) = plant();
    f.orbit.launch(SCIENCE_SATELLITE, DISH, 0);
    tick(&mut f, &mut world, 0, 3);
    assert_eq!(station(&f).offered_tf(), TF_PER_SATELLITE);
    // Twenty minutes later it is some 7000 blocks along its track, outside the 1024-block circle.
    tick(&mut f, &mut world, 60 * 60 * 20 - 1, 2);
    assert_eq!(station(&f).offered_tf(), 0);
}

#[test]
fn a_ground_station_writes_no_state_of_its_own() {
    let (f, _) = plant();
    let mut w = ByteWriter::default();
    station(&f).write_state(&mut w);
    let back = Processor::read_state(&mut ByteReader::new(&w.bytes)).unwrap();
    assert_eq!((back.spec.block, back.store.flow), (GROUND_STATION, 0));
}
