use super::*;
use crate::block::STONE;
use crate::math::IVec3;
use crate::tests::run_until_ready;

#[test]
fn a_flare_stands_on_the_ground_looked_at_burns_a_minute_and_changes_no_core_state() {
    let mut g = Game::new(2024, 3);
    run_until_ready(&mut g);
    let feet = g.body().pos.floor();
    let ground = feet + IVec3::new(0, -1, -6);
    g.sim.world.set_block_anywhere(ground, STONE);
    for h in 0..4 {
        g.sim.world.set_block_anywhere(ground + IVec3::new(0, 1 + h, 0), AIR);
    }
    let at = ground.as_vec3() + Vec3::new(0.5, 1.0, 0.5);
    let to = at - g.body().eye();
    let to = to * (1.0 / to.length());
    (g.body_mut().pitch, g.body_mut().yaw) = (to.y.asin(), to.x.atan2(-to.z));
    let spot = g.flare_spot_at().expect("ground in view");
    assert!((spot - at).length() < 1e-9, "on top of the aimed block: {spot:?}");

    let hash = g.sim.state_hash();
    g.light_flare_at(spot, 1);
    assert_eq!(g.sim.state_hash(), hash, "presentation only");
    assert_eq!(g.flares.list[0].rgb, COLOURS[1], "in the raiser's colour");
    let eye = g.body().eye();
    g.instances.clear();
    g.write_flares(1.0, eye);
    assert!(!g.instances.is_empty(), "a column is drawn");
    g.write_flares(FLARE_SECONDS, eye);
    assert!(g.flares.list.is_empty(), "burnt out after a minute");
}

#[test]
fn a_new_flare_puts_out_the_oldest_and_bad_spots_are_refused() {
    let mut g = Game::new(2024, 3);
    for i in 0..MAX_FLARES + 3 {
        g.light_flare_at(Vec3::new(i as f64, 70.0, 0.0), 0);
    }
    assert_eq!(g.flares.list.len(), MAX_FLARES);
    assert_eq!(g.flares.list[0].foot.x, 3.0, "the three oldest went out");
    g.light_flare_at(Vec3::new(f64::NAN, 0.0, 0.0), 0);
    assert_eq!(g.flares.list.len(), MAX_FLARES);
}
