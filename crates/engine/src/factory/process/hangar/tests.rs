//! A comms satellite over a drone port multiplies its reach and cargo range, and only while it is overhead.

use crate::block::DRONE_PORT;
use crate::factory::tests::{powered, run};
use crate::factory::Factory;
use crate::item::{COMMS_SATELLITE, SURVEY_SATELLITE};
use crate::math::IVec3;
use crate::world::World;

fn port(f: &Factory) -> (f64, f64) {
    let p = &f.ports()[0];
    (p.reach, p.cargo_range)
}

fn rig() -> Factory {
    let mut f = Factory::default();
    powered(&mut f);
    f.place(&mut World::new(1, 2), DRONE_PORT, IVec3::new(0, 3, 4), 0, IVec3::new(0, 3, 4), 0);
    run(&mut f, 0.1, |_| {});
    f
}

#[test]
fn a_comms_satellite_over_the_pad_reaches_four_times_as_far_and_flies_cargo_twice_as_far() {
    let mut f = rig();
    assert_eq!(port(&f), (32.0, 200.0), "a Mk1 port on its own");
    f.orbit.launch(SURVEY_SATELLITE, IVec3::new(0, 70, 4), 0);
    run(&mut f, 0.1, |_| {});
    assert_eq!(port(&f), (32.0, 200.0), "a survey satellite does not help");
    f.orbit.launch(COMMS_SATELLITE, IVec3::new(0, 70, 4), 0);
    run(&mut f, 0.1, |_| {});
    assert_eq!(port(&f), (128.0, 400.0));
    let text = f.processors.iter().find_map(|p| p.hangar_text()).unwrap();
    assert!(text.contains("reach 128 blocks (comms satellite)"), "{text}");
    // Two thousand blocks of flight later the footprint is elsewhere: the port is on its own again.
    let mut world = World::new(1, 2);
    f.update(&mut world, 20 * 60 * 10, &mut Vec::new());
    assert_eq!(port(&f), (32.0, 200.0));
}
