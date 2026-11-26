use super::*;
use crate::tests::run_until_ready;
use crate::Game;

/// The first tick of day `day` of the year (from 0) at `hour` on the clock, in the world's second year (the first day's
/// small hours come before tick 0).
fn at(day: u64, hour: u64) -> u64 {
    (YEAR_DAYS + day) * DAY_TICKS + DAY_TICKS * hour / 24 - START
}

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
    assert!(sunlight(0) > 0, "a new world starts in daylight");
}

#[test]
fn every_day_the_sun_peaks_at_noon_and_sets_at_night() {
    for day in 0..YEAR_DAYS {
        assert_eq!(sunlight(at(day, 12)), 1000, "noon of day {day}");
        assert_eq!((sunlight(at(day, 0)), sunlight(at(day, 3)), sunlight(at(day, 21))), (0, 0, 0), "night of {day}");
        assert_eq!(sunlight(at(day, 9)), sunlight(at(day, 15)), "symmetric round noon on day {day}");
        assert!((at(day, 0)..at(day + 1, 0)).step_by(97).all(|t| sunlight(t) <= 1000));
    }
    assert_eq!(sunlight(at(3, 10)), sunlight(at(3 + YEAR_DAYS, 10)), "the same every year");
}

#[test]
fn days_are_long_in_summer_short_in_winter_and_even_out_over_the_year() {
    let len = |day| day_length(day) as f64 / DAY_TICKS as f64 * 24.0;
    let longest = (0..YEAR_DAYS).max_by_key(|&d| day_length(d)).unwrap();
    let shortest = (0..YEAR_DAYS).min_by_key(|&d| day_length(d)).unwrap();
    assert!((8..=9).contains(&longest) && (26..=27).contains(&shortest), "{longest}, {shortest}");
    assert!(len(longest) > 15.9 && len(shortest) < 8.1, "{} h and {} h", len(longest), len(shortest));
    assert!((len(0) - 12.0).abs() < 0.5, "spring starts near twelve hours: {}", len(0));
    let daylight: u64 = (0..YEAR_DAYS).map(day_length).sum();
    assert_eq!(daylight, YEAR_DAYS * DAY_TICKS / 2, "a year's daylight is exactly twelve hours a day");
    // Solar panels make what they made in a year of equal days.
    let year: u64 = (0..YEAR_DAYS * DAY_TICKS).map(|t| u64::from(sunlight(t))).sum();
    let len = DAY_TICKS / 2;
    let equal_day: u64 = (0..len).map(|x| 4000 * x * (len - x) / (len * len)).sum();
    let ratio = year as f64 / (equal_day * YEAR_DAYS) as f64;
    assert!((ratio - 1.0).abs() < 0.001, "yearly solar {ratio} of before");
}

#[test]
fn sun_time_puts_sunrise_and_sunset_at_the_same_place_every_season() {
    for day in [0, 8, 17, 26, 35] {
        let rise = at(day, 0) + (DAY_TICKS - day_length(day)) / 2;
        assert!(sunlight(rise - 1) == 0 && sunlight(rise + DAY_TICKS / 100) > 0, "day {day}");
        assert!((sun_time(rise) - 0.25).abs() < 1e-3, "sunrise of day {day}: {}", sun_time(rise));
        assert!((sun_time(at(day, 12)) - 0.5).abs() < 1e-9);
        assert!((sun_time(rise + day_length(day)) - 0.75).abs() < 1e-3);
        assert!(sun_time(at(day, 0)).abs() < 1e-9);
    }
}

#[test]
fn a_world_starts_on_the_first_day_of_spring() {
    assert_eq!((season(0), day_of_season(0)), (Season::Spring, 1));
    assert_eq!((season(at(8, 12)), day_of_season(at(8, 12))), (Season::Spring, 9));
    assert_eq!((season(at(9, 0)), day_of_season(at(9, 0))), (Season::Summer, 1));
    assert_eq!(season(at(18, 0)), Season::Autumn);
    assert_eq!(season(at(35, 23)), Season::Winter);
    assert_eq!((season(at(36, 0)), day_of_season(at(36, 0))), (Season::Spring, 1), "and the year comes round");
    assert!(year_fraction(0) < 0.01 && year_fraction(at(18, 0)) == 0.5);
}

#[test]
fn growth_is_fast_in_spring_slow_in_autumn_and_waits_out_the_winter() {
    let hour = DAY_TICKS / 24;
    let summer = at(12, 12);
    assert_eq!(growth_ticks(summer, 600), summer + 400, "half again as fast in summer");
    assert_eq!(growth_ticks(at(3, 12), 600), at(3, 12) + 300, "spring twice as fast");
    assert_eq!(growth_ticks(at(20, 12), 600), at(20, 12) + 1200, "autumn half as fast");
    assert_eq!(growth_ticks(at(30, 12), 600), at(36, 0) + 300, "nothing grows in winter: it finishes in spring");
    // Late autumn growth runs into winter and finishes in spring.
    let late = at(26, 0) + 23 * hour;
    assert!(growth_ticks(late, 2 * hour) > at(36, 0));
    // Long growth crosses whole years the same way.
    assert_eq!(growth_ticks(summer, 10 * YEAR_DAYS * DAY_TICKS), summer + 10 * YEAR_DAYS * DAY_TICKS);
    // A year's growth takes a year from any day: the seasons add up to the year-round rate.
    for start in (0..YEAR_DAYS).map(|d| at(d, 5)) {
        assert_eq!(growth_ticks(start, YEAR_DAYS * DAY_TICKS), start + YEAR_DAYS * DAY_TICKS);
    }
    let mut last = 0;
    for normal in (0..5 * DAY_TICKS).step_by(1013) {
        let done = growth_ticks(at(17, 22), normal);
        assert!(done >= last, "never sooner for longer growth");
        last = done;
    }
}

#[test]
fn the_time_is_the_cores_and_leaves_the_state_hash_alone() {
    let mut g = Game::new(2024, 2);
    run_until_ready(&mut g);
    let (hash, tick) = (g.sim.state_hash(), g.sim.tick);
    assert!((g.time_of_day() - time_of_day(tick)).abs() < 1e-9);
    assert_eq!((g.season(), g.day_of_season(), g.days_per_season()), (0, 1, 9));
    assert!(g.daylight_share(8) > 0.66 && g.daylight_share(26) < 0.34, "long summer days, short winter ones");
    assert_eq!((0..4).map(|s| g.growth_percent(s)).collect::<Vec<_>>(), [200, 150, 50, 0]);
    assert_eq!(g.sim.state_hash(), hash, "reading the time changes nothing");
    // Saved with the world: a reloaded game shows the same time.
    let loaded = Game::load(&g.save(), 2).unwrap();
    assert_eq!(loaded.time_of_day(), g.time_of_day());
    assert_eq!(loaded.day_number(), g.day_number());
}
