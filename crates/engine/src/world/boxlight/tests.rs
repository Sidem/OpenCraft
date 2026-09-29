use super::*;
use crate::block::{LAMP, STONE};
use crate::math::Vec3;

fn streamed_world() -> (World, IVec3) {
    let mut w = World::new(5, 2);
    let ground = w.generator().height_at(0, 0);
    w.update_streaming(Vec3::new(0.5, ground as f64 + 1.0, 0.5), &[]);
    loop {
        w.begin_work();
        if !w.work_step() {
            break;
        }
    }
    (w, IVec3::new(0, ground + 3, 0))
}

fn blocks_of(light: u8) -> (u8, u8) {
    (light & 15, light >> 4)
}

#[test]
fn open_air_is_daylit_and_a_lamp_lights_it_at_once() {
    let (mut w, air) = streamed_world();
    assert_eq!(blocks_of(w.light_at(air)), (15, 0));

    // The lit chunk is cached; the edit drops it, so the lamp shows up on the next look.
    assert!(w.set_block(air + IVec3::new(2, 0, 0), LAMP));
    assert_eq!(blocks_of(w.light_at(air)).1, 15, "two blocks from a lamp is full block light");
    let far = air + IVec3::new(0, 0, 25);
    assert!(blocks_of(w.light_at(far)).1 < 15, "and it fades with distance");

    // A roof over the cell takes the sky light away.
    for (dx, dz) in [(0, 0), (1, 0), (-1, 0), (0, 1), (0, -1)] {
        w.set_block(air + IVec3::new(dx, 1, dz), STONE);
    }
    assert!(blocks_of(w.light_at(air)).0 < 15);
}

#[test]
fn an_unlit_place_reads_as_plain_daylight() {
    let (mut w, air) = streamed_world();
    let unloaded = air + IVec3::new(4000, 0, 0);
    assert_eq!(w.light_at(unloaded), DAYLIGHT);
    assert_eq!(w.light_at(IVec3::new(0, -5, 0)), DAYLIGHT);
    assert_eq!(w.light_at(IVec3::new(0, WORLD_HEIGHT + 5, 0)), DAYLIGHT);
}
