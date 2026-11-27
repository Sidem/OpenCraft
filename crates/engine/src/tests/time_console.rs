//! The console's time tools through `Game` as the host calls them: the time speed runs extra ticks each frame, and
//! `weather_ahead` finds the next sky of a kind that the weather at that moment really is.

use super::*;

#[test]
fn the_time_speed_runs_that_many_ticks_a_frame_and_clamps() {
    let mut g = Game::new(2024, 3);
    let start = g.sim.tick;
    for _ in 0..60 {
        g.update(1.0 / 60.0);
    }
    assert_eq!(g.sim.tick - start, 60, "normal speed: a tick a frame");
    g.set_time_speed(10);
    assert_eq!(g.time_speed(), 10);
    let start = g.sim.tick;
    for _ in 0..60 {
        g.update(1.0 / 60.0);
    }
    assert_eq!(g.sim.tick - start, 600, "ten times as fast");
    // At 144 frames a second the extra ticks still add up.
    g.set_time_speed(2);
    let start = g.sim.tick;
    for _ in 0..144 {
        g.update(1.0 / 144.0);
    }
    assert!((g.sim.tick - start).abs_diff(120) <= 1, "{} ticks", g.sim.tick - start);
    g.set_time_speed(100_000);
    assert_eq!(g.time_speed(), crate::frame::MAX_TIME_SPEED);
    g.set_time_speed(0);
    assert_eq!(g.time_speed(), 1);
}

#[test]
fn what_falls_is_snow_below_zero_however_light() {
    let mut g = Game::new(2024, 3);
    g.update(1.0 / 60.0);
    // Through a year: start the air afresh at each point and run it half a day, read the camera's weather hourly.
    let (mut cold, mut warm) = (0, 0);
    for k in 0..24u64 {
        let tick = k * crate::daytime::YEAR_DAYS * crate::daytime::DAY_TICKS / 24;
        g.sim.atmosphere = Default::default();
        g.sim.atmosphere.advance(g.sim.world.generator(), tick);
        for h in 0..12 {
            g.sim.tick = tick + h * crate::daytime::DAY_TICKS / 24;
            g.sim.atmosphere.advance(g.sim.world.generator(), g.sim.tick);
            let w = g.weather_here();
            assert_eq!(w[3] > 0.5, w[4] < 0.0, "snow flag {} at {} °C", w[3], w[4]);
            if w[4] < 0.0 {
                cold += 1
            } else {
                warm += 1
            }
        }
    }
    assert!(cold > 0 && warm > 0, "a year has both: {cold} cold, {warm} warm");
}

#[test]
fn weather_ahead_lands_on_that_weather() {
    let mut g = Game::new(2024, 3);
    g.update(1.0 / 60.0);
    let kind = |g: &Game| g.weather_here()[5] as u32;
    for want in [2, 0, 1] {
        let s = g.weather_ahead(want);
        assert!(s > 0.0, "kind {want} comes within a year");
        g.skip_time(s);
        g.update(1.0 / 60.0);
        let now = kind(&g);
        assert!(if want == 2 { now >= 2 } else { now == want }, "wanted {want}, the sky is {now}");
    }
}
