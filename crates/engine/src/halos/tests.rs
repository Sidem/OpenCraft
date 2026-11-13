//! Tests of the halos round light-giving blocks: a lamp gets one where it stands, it goes when the lamp does.

use super::*;
use crate::block::{AIR, LAMP};
use crate::math::IVec3;
use crate::tests::run_until_ready;

/// Steps frames and streaming until the remeshed chunks are in (`run_until_ready` stops once the area is ready).
fn settle(g: &mut Game) {
    for _ in 0..4 {
        g.update(1.0 / 60.0);
        g.begin_work();
        while g.work_step() {}
        while g.next_event() != 0 {}
    }
}

#[test]
fn a_lamp_gets_a_halo_where_it_stands_and_loses_it_when_broken() {
    let mut g = Game::new(2024, 3);
    run_until_ready(&mut g);
    let y = g.sim.world.generator().height_at(3, 3);
    let lamp = IVec3::new(3, y + 3, 3);
    let lamps_drawn = |g: &Game| g.halos.chunks_exact(HALO_FLOATS).filter(|h| h[3] == look(LAMP).1).count();
    settle(&mut g);
    let before = lamps_drawn(&g);
    g.sim.world.set_block_anywhere(lamp, LAMP);
    settle(&mut g);
    assert_eq!(lamps_drawn(&g), before + 1, "a halo for the new lamp");
    let eye = g.render_eye;
    let h = g.halos.chunks_exact(HALO_FLOATS).find(|h| h[3] == look(LAMP).1).expect("drawn");
    let centre = Vec3::new(f64::from(h[0]), f64::from(h[1]), f64::from(h[2])) + eye;
    assert!((centre - (lamp.as_vec3() + Vec3::new(0.5, 0.5, 0.5))).length() < 1e-3, "at the lamp's centre: {centre:?}");
    g.sim.world.set_block_anywhere(lamp, AIR);
    settle(&mut g);
    assert_eq!(lamps_drawn(&g), before, "gone with the lamp");
}
