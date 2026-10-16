use super::*;
use crate::item::SURVEY_SATELLITE;
use crate::math::IVec3;

#[test]
fn a_burst_draws_a_ring_of_puffs_near_the_eye_and_is_gone_after_its_life() {
    let mut fx = OrbitFx::default();
    fx.burst(Vec3::new(10.0, 0.0, 0.0), 5.0, 1.0);
    let (mut near, mut far, mut late) = (Vec::new(), Vec::new(), Vec::new());
    fx.write(&mut near, Vec3::ZERO, 5.5);
    fx.write(&mut far, Vec3::new(DRAW_RANGE + 50.0, 0.0, 0.0), 5.5);
    assert!(!near.is_empty() && far.is_empty(), "drawn near the eye only");
    fx.write(&mut late, Vec3::ZERO, 5.0 + LIFE + 0.1);
    assert!(late.is_empty() && fx.bursts.is_empty(), "forgotten once over");
    for i in 0..MAX_BURSTS + 5 {
        fx.burst(Vec3::ZERO, i as f64, 1.0);
    }
    assert_eq!(fx.bursts.len(), MAX_BURSTS, "the oldest go first");
}

#[test]
fn a_launch_queues_its_toast_and_a_sound_and_a_landing_only_a_sound() {
    let mut g = Game::new(7, 2);
    g.sim.events.push(SimEvent::Launched { pos: IVec3::new(4, 70, 4), item: SURVEY_SATELLITE });
    g.sim.events.push(SimEvent::PodLanded { pos: IVec3::new(40, 70, 4) });
    g.handle_sim_events();
    assert_eq!(g.next_launch(), SURVEY_SATELLITE.0);
    assert_eq!(g.next_launch(), 0, "one toast");
    assert_eq!(g.orbit_fx.bursts.len(), 2, "a burst each");
}
