use super::*;
use crate::daytime::{season, Season, START, YEAR_DAYS};
use crate::worldgen::SEA_LEVEL;

const SEEDS: [u32; 5] = [1, 42, 12_345, 987_654_321, 0xDEAD_BEEF];
/// One sample every 72 minutes of game time.
const STEP: u64 = DAY_TICKS / 20;
const PLAINS: Climate = Climate { warmth: 0, wet: 0 };

/// The tick at `hour` on day `day` of the year (from 0), in the world's second year.
fn at(day: u64, hour: u64) -> u64 {
    (YEAR_DAYS + day) * DAY_TICKS + DAY_TICKS * hour / 24 - START
}

/// Shares of clear, cloudy, rain and storm time over `days` days at a few places, every seed.
fn shares(climate: Climate, days: u64) -> [f64; 4] {
    let mut n = [0u32; 4];
    for seed in SEEDS {
        for (x, z) in [(0, 0), (5_000, -3_000), (-12_000, 800)] {
            for t in (0..days * DAY_TICKS).step_by(STEP as usize) {
                n[climate.sky(seed, x, z, t).kind() as usize] += 1;
            }
        }
    }
    let total: u32 = n.iter().sum();
    n.map(|k| f64::from(k) / f64::from(total))
}

#[test]
fn the_weather_is_a_pure_function_of_seed_place_and_tick() {
    for seed in SEEDS {
        for t in [0, 7, 99_999, 12_345_678, u64::from(u32::MAX)] {
            let a = PLAINS.sky(seed, 300, -40, t);
            let _ = PLAINS.sky(seed, -9_000, 77, t + 5); // no hidden state between calls
            assert_eq!(PLAINS.sky(seed, 300, -40, t), a);
        }
    }
    let gen = WorldGen::new(7);
    let pos = IVec3::new(1_500, 80, -2_200);
    assert_eq!(forecast(&gen, pos, 50_000, 5), forecast(&gen, pos, 50_000, 5));
}

#[test]
fn different_worlds_and_far_places_have_different_weather() {
    let differ = |f: &dyn Fn(u64) -> (i64, i64)| {
        (0..400).map(|k| f(k * DAY_TICKS / 2)).filter(|(a, b)| a.abs_diff(*b) > 50).count()
    };
    let worlds = differ(&|t| (storminess(1, 0, 0, t), storminess(2, 0, 0, t)));
    let places = differ(&|t| (storminess(1, 0, 0, t), storminess(1, 6_000, 4_000, t)));
    assert!(worlds > 250 && places > 250, "{worlds} and {places} of 400 samples differ");
}

#[test]
fn fronts_drift_east_with_the_wind() {
    // A quarter day later, the pattern has moved a quarter of WIND_PER_DAY east: it matches there far better than in place.
    let (mut moved, mut stayed) = (0i64, 0i64);
    let (dt, dx) = (DAY_TICKS / 4, WIND_PER_DAY as i32 / 4);
    for k in 0..300 {
        let (x, t) = (k * 97, k as u64 * 3_001);
        let here = noise_front(x, t);
        moved += (here - noise_front(x + dx, t + dt)).abs();
        stayed += (here - noise_front(x, t + dt)).abs();
    }
    assert!(moved * 3 < stayed * 2, "moved {moved}, stayed {stayed}");
}

fn noise_front(x: i32, t: u64) -> i64 {
    let x = i64::from(x) - t as i64 * WIND_PER_DAY / DAY_TICKS as i64;
    noise3(3, [x, 0, t as i64], [FRONT_BLOCKS, FRONT_BLOCKS, FRONT_TICKS])
}

#[test]
fn plains_are_clear_most_of_the_time_with_real_storms() {
    let [clear, cloudy, rain, storm] = shares(PLAINS, 600);
    assert!(clear > 0.55 && clear < 0.8, "clear {clear}");
    assert!(rain + storm > 0.06 && rain + storm < 0.2, "rain {rain} storm {storm}");
    assert!(storm > 0.005 && storm < rain, "storm {storm} against rain {rain}");
    assert!(cloudy > 0.05, "cloudy {cloudy}");
}

#[test]
fn a_rainy_spell_comes_every_few_days_and_builds_without_jumps() {
    for seed in SEEDS {
        let (mut spells, mut wet) = (0, false);
        let mut prev = PLAINS.sky(seed, 200, 200, 0);
        for t in (0..600 * DAY_TICKS).step_by(720) {
            let s = PLAINS.sky(seed, 200, 200, t);
            // At most about a tenth in 12 seconds of play (the host eases what it draws over a few more).
            let jump = [s.cover.abs_diff(prev.cover), s.gloom.abs_diff(prev.gloom), s.precip.abs_diff(prev.precip)];
            assert!(jump.iter().all(|&j| j < 130), "seed {seed}: jump {jump:?} at tick {t}");
            let now = s.precip >= RAIN_FROM;
            spells += u32::from(now && !wet);
            (wet, prev) = (now, s);
        }
        let gap = 600.0 / f64::from(spells);
        assert!(gap > 2.0 && gap < 14.0, "seed {seed}: a rainy spell every {gap:.1} days");
    }
}

#[test]
fn most_ten_day_stretches_bring_a_gloomy_spell() {
    for seed in SEEDS {
        let windows: Vec<bool> = (0..100u64)
            .map(|w| (w * 10 * DAY_TICKS..(w + 1) * 10 * DAY_TICKS).step_by(STEP as usize))
            .map(|mut ticks| ticks.any(|t| PLAINS.sky(seed, 0, 0, t).gloom > 400))
            .collect();
        let stormy = windows.iter().filter(|&&s| s).count();
        assert!(stormy > 60, "seed {seed}: {stormy} of 100 ten-day stretches have a gloomy spell");
    }
}

#[test]
fn deserts_are_dry_and_wet_lowlands_wetter() {
    let desert = shares(Climate::of(250, -250), 400);
    let plains = shares(PLAINS, 400);
    let lowlands = shares(Climate::of(0, 200), 400);
    let wet = |s: [f64; 4]| s[2] + s[3];
    assert!(wet(desert) < wet(plains) / 2.0, "desert {} against plains {}", wet(desert), wet(plains));
    assert!(desert[0] > plains[0]);
    assert!(wet(lowlands) > wet(plains) * 1.2, "lowlands {} against plains {}", wet(lowlands), wet(plains));
}

#[test]
fn the_clear_sky_keeps_fair_clouds_and_rain_comes_from_heavy_cloud() {
    for s in (-300..1300).step_by(7) {
        let sky = Sky::from_storminess(s);
        assert!(sky.cover >= FAIR_COVER && sky.cover <= 1000 && sky.gloom <= 1000 && sky.precip <= 1000);
        assert!(sky.gloom >= sky.precip && sky.cover >= sky.gloom, "{sky:?}");
        if sky.kind() == Kind::Clear {
            assert!(sky.gloom < 400 && sky.precip < RAIN_FROM);
        }
    }
}

#[test]
fn the_temperature_follows_the_year_the_day_and_the_height() {
    let sea = SEA_LEVEL + 2;
    let t = |c: Climate, y: i32, day: u64, hour: u64| c.temperature(y, at(day, hour));
    // Plains: warm summer afternoons, frost on midwinter nights, mild spring noons.
    assert!(t(PLAINS, sea, 13, 15) > 200, "high summer {}", t(PLAINS, sea, 13, 15));
    assert!(t(PLAINS, sea, 31, 3) < 0, "midwinter night {}", t(PLAINS, sea, 31, 3));
    assert!(t(PLAINS, sea, 20, 12) > 0, "early autumn is not freezing");
    assert!(t(PLAINS, sea, 4, 15) > t(PLAINS, sea, 4, 3), "afternoons beat the small hours");
    assert!(t(PLAINS, sea, 31, 15) < t(PLAINS, sea, 13, 15), "winter is colder than summer");
    // Deserts never freeze; the boreal north does; mountains are colder.
    let desert = Climate::of(250, -250);
    assert!((0..YEAR_DAYS).all(|d| (0..24).all(|h| t(desert, sea, d, h) > 0)), "deserts never freeze");
    assert!(t(Climate::of(-400, 0), sea, 31, 15) < 0, "boreal winter afternoons freeze");
    assert_eq!(t(PLAINS, sea + 80, 13, 15), t(PLAINS, sea, 13, 15) - 100, "ten degrees colder 80 blocks up");
    // Smooth: no step of more than three tenths of a degree in six minutes on the clock.
    let (year, six) = (YEAR_DAYS * DAY_TICKS, DAY_TICKS / 240);
    let mut prev = PLAINS.temperature(sea, 0);
    for tick in (six..2 * year).step_by(six as usize) {
        let now = PLAINS.temperature(sea, tick);
        assert!(now.abs_diff(prev) <= 3, "{prev} to {now} at {tick}");
        prev = now;
    }
}

#[test]
fn rain_becomes_snow_below_freezing() {
    // The forecast calls a wet day snowy exactly when a wet hour of it is below zero.
    let gen = WorldGen::new(42);
    let (mut snowy, mut rainy) = (0, 0);
    for (x, y) in [(0, SEA_LEVEL + 4), (40, SEA_LEVEL + 120)] {
        let pos = IVec3::new(x, y, 0);
        let climate = Climate::at(&gen, x, 0);
        for day in 0..YEAR_DAYS * 2 {
            let f = forecast(&gen, pos, at(day, 0), 1);
            let wet_hours: Vec<u64> = (0..24).map(|h| at(day, 0) + (2 * h + 1) * DAY_TICKS / 48).collect();
            let freezing_wet = wet_hours
                .iter()
                .any(|&t| climate.sky(42, x, 0, t).precip >= RAIN_FROM && climate.temperature(y, t) < 0);
            assert_eq!(f[1] == 1, freezing_wet, "day {day} at height {y}");
            assert!(f[2] <= f[3]);
            if f[0] >= Kind::Rain as i32 {
                if f[1] == 1 {
                    snowy += 1
                } else {
                    rainy += 1
                }
            }
        }
    }
    assert!(snowy > 0 && rainy > 0, "{snowy} snowy and {rainy} rainy days");
}

#[test]
fn the_forecast_names_each_day_by_the_sky_that_lasts() {
    let gen = WorldGen::new(12_345);
    let mut days = [0u32; 4];
    let f = forecast(&gen, IVec3::new(0, SEA_LEVEL + 4, 0), at(0, 9), 200);
    for day in f.chunks_exact(FORECAST_FIELDS) {
        days[day[0] as usize] += 1;
    }
    // A passing cloud does not make a cloudy day: plenty stay clear, and every kind comes up.
    assert!(days[0] > 50 && days.iter().all(|&n| n > 0), "clear, cloudy, rain, storm days: {days:?}");
}

#[test]
fn clouds_dim_solar_a_little() {
    assert_eq!(Sky::from_storminess(0).sun_share(), 1000, "a fair sky gives full sun");
    let storm = Sky::from_storminess(2000);
    assert_eq!(storm.kind(), Kind::Storm);
    assert_eq!(storm.sun_share(), 1000 - SOLAR_DIM);
    let mut least = 1000;
    for t in (0..200 * DAY_TICKS).step_by(STEP as usize) {
        least = least.min(PLAINS.sun_share(5, 0, 0, t));
    }
    assert!((1000 - SOLAR_DIM..800).contains(&least), "the darkest sky gives {least}");
}

#[test]
fn rain_speeds_growth_and_snow_does_not() {
    let gen = WorldGen::new(1337);
    let climate = Climate::at(&gen, 0, 0);
    let pos = IVec3::new(0, SEA_LEVEL + 4, 0);
    let normal = DAY_TICKS / 6; // four hours of growth at the year-round rate
    let (mut faster, mut same) = (0, 0);
    for k in 0..400 {
        let tick = at(0, 0) + k * DAY_TICKS / 4;
        let plain = growth_ticks(tick, normal);
        let due = growth_due(&gen, pos, tick, normal);
        assert!(due <= plain && due > tick, "never slower");
        let span_rain = (0..4).any(|i| climate.sky(1337, 0, 0, tick + (plain - tick) * (2 * i + 1) / 8).precip > 0);
        if due < plain {
            faster += 1;
            assert!(span_rain, "only rain speeds growth");
        } else {
            same += 1;
        }
    }
    assert!(faster > 10 && same > faster, "{faster} faster, {same} the same");
    // High in the mountains a winter's precipitation is all snow, which waters nothing.
    let peak = IVec3::new(0, SEA_LEVEL + 200, 0);
    for k in 0..40 {
        let tick = at(27, 0) + k * DAY_TICKS / 5;
        assert_eq!(season(tick), Season::Winter);
        assert_eq!(growth_due(&gen, peak, tick, normal), growth_ticks(tick, normal), "snow at {tick}");
    }
}
