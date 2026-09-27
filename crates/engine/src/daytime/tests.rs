use super::*;
use crate::tests::run_until_ready;
use crate::Game;

#[test]
fn a_day_starts_in_the_morning_and_wraps() {
    assert!((time_of_day(0) - 7.0 / 24.0).abs() < 1e-9);
    assert_eq!(day_number(0), 1);
    let noon = DAY_TICKS * 5 / 24; // 7:00 + 5 hours
    assert!((time_of_day(noon) - 0.5).abs() < 1e-9);
    let midnight = DAY_TICKS * 17 / 24;
    assert_eq!(time_of_day(midnight), 0.0);
    assert_eq!(day_number(midnight), 2);
    assert_eq!(time_of_day(midnight + DAY_TICKS), 0.0);
}

#[test]
fn the_time_is_the_cores_and_leaves_the_state_hash_alone() {
    let mut g = Game::new(2024, 2);
    run_until_ready(&mut g);
    let (hash, tick) = (g.sim.state_hash(), g.sim.tick);
    assert!((g.time_of_day() - time_of_day(tick)).abs() < 1e-9);
    assert_eq!(g.sim.state_hash(), hash, "reading the time changes nothing");
    // Saved with the world: a reloaded game shows the same time.
    let loaded = Game::load(&g.save(), 2).unwrap();
    assert_eq!(loaded.time_of_day(), g.time_of_day());
    assert_eq!(loaded.day_number(), g.day_number());
}
