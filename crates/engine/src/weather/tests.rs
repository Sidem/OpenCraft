use super::air::{Atmosphere, N, P0, STEP_TICKS};
use super::*;
use crate::bytes::{ByteReader, ByteWriter};
use crate::daytime::{growth_ticks, DAY_TICKS};
use crate::worldgen::SEA_LEVEL;

const SEEDS: [u32; 3] = [1, 7, 2024];
const STEPS_A_DAY: u64 = DAY_TICKS / STEP_TICKS;
/// Days the air needs from rest before its weather counts.
const SPIN_UP: u64 = 5;

/// Runs the air of `seed` from tick 0 for `days` days after the spin-up, calling `f` after each step.
fn run(seed: u32, days: u64, mut f: impl FnMut(&Atmosphere, &WorldGen)) {
    let gen = WorldGen::new(seed);
    let mut air = Atmosphere::default();
    for k in 1..=(SPIN_UP + days) * STEPS_A_DAY {
        air.advance(&gen, k * STEP_TICKS);
        if k > SPIN_UP * STEPS_A_DAY {
            f(&air, &gen);
        }
    }
}

fn saved(air: &Atmosphere) -> Vec<u8> {
    let mut w = ByteWriter::default();
    air.write_state(&mut w);
    w.bytes
}

/// What 40 days of air give over every cell of three worlds: the pressure's range (Pa), and the shares of cell-steps
/// that rain and that storm.
#[test]
fn the_weather_looks_like_earths() {
    let (mut low, mut high, mut cells, mut rainy, mut stormy) = (i32::MAX, i32::MIN, 0u64, 0u64, 0u64);
    for seed in SEEDS {
        run(seed, 40, |air, _| {
            for i in 0..N * N {
                (low, high) = (low.min(air.pressure[i]), high.max(air.pressure[i]));
                let sky = reading::sky_of(air.cloud[i], air.rain[i], air.temp[i]);
                cells += 1;
                rainy += u64::from(sky.precip >= RAIN_FROM);
                stormy += u64::from(sky.precip >= STORM_FROM);
            }
        });
    }
    let (rain, storm) = (rainy as f64 / cells as f64, stormy as f64 / cells as f64);
    eprintln!("pressure {low}..{high} Pa, rain {:.1}% of the time, storm {:.2}%", rain * 100.0, storm * 100.0);
    assert!(low >= 95_000 && high <= 105_000, "pressure {low}..{high} Pa: Earth's sea level stays in 950..1050 hPa");
    assert!(high - low >= 3_000, "pressure {low}..{high} Pa: highs and lows at least 30 hPa apart");
    assert!((0.04..0.30).contains(&rain), "it rains {:.1}% of the time", rain * 100.0);
    assert!(storm > 0.002 && storm < rain / 3.0, "storms {:.2}% of the time", storm * 100.0);
}

/// A station's readings stay physical everywhere: humidity in 0..100 %, the dew point at most the temperature, snow
/// exactly below 0 °C, and the air at a place is neither always clear nor always wet.
#[test]
fn readings_stay_physical_and_vary() {
    let places = [(0, 0), (2_000, -1_500), (-5_000, 4_000), (7_700, 300)];
    let mut kinds = [0u32; 4];
    let (mut dry, mut damp) = (1000, 0);
    run(7, 30, |air, gen| {
        for (x, z) in places {
            let r = Reading::at(air, gen, x, SEA_LEVEL + 4, z);
            assert!((0..=1000).contains(&r.humidity), "{r:?}");
            assert!(r.dew <= r.temp, "{r:?}");
            assert_eq!(r.snow, r.temp < 0, "{r:?}");
            assert!((FAIR_COVER..=1000).contains(&r.sky.cover) && r.sky.precip <= 1000, "{r:?}");
            kinds[r.sky.kind() as usize] += 1;
            (dry, damp) = (dry.min(r.humidity), damp.max(r.humidity));
        }
    });
    eprintln!("clear, cloudy, rain, storm: {kinds:?}; humidity {dry}..{damp}");
    let total: u32 = kinds.iter().sum();
    assert!(kinds[0] * 4 > total, "clear skies are common: {kinds:?}");
    assert!(kinds[1] > 0 && kinds[2] > 0, "clouds and rain both come: {kinds:?}");
    assert!(dry < 600 && damp > 900, "humidity swings with the day and the weather: {dry}..{damp}");
}

/// Weather comes from the west: the pressure pattern a day later matches today's moved east.
#[test]
fn weather_moves_east() {
    let mut days = Vec::new();
    let mut k = 0;
    run(2024, 3, |air, _| {
        k += 1;
        if k % STEPS_A_DAY == 0 {
            days.push(air.pressure.clone());
        }
    });
    let anomaly = |p: &[i32]| p.iter().map(|&v| f64::from(v - P0)).collect::<Vec<_>>();
    let (a, b) = (anomaly(&days[0]), anomaly(&days[1]));
    let fit = |s: i64| -> f64 {
        (0..N * N).map(|i| a[i] * b[(i / N) * N + (i as i64 % N as i64 + s).rem_euclid(N as i64) as usize]).sum()
    };
    let best = (-6..=6).max_by(|&x, &y| fit(x).total_cmp(&fit(y))).unwrap();
    let expected = reading::drift_per_day() as f64 / f64::from(air::CELL);
    eprintln!("a day later the pattern fits best {best} cells east (the steering wind alone: {expected:.1})");
    assert!((1..=4).contains(&best), "moved {best} cells");
}

/// The air depends only on the seed, the generator and the tick: stepping every tick or jumping, live or saved and
/// loaded, a forecast's copy or the real thing all come out the same.
#[test]
fn the_air_is_deterministic_and_survives_a_save() {
    let gen = WorldGen::new(1);
    let (mut every, mut jump) = (Atmosphere::default(), Atmosphere::default());
    for t in 0..3 * STEP_TICKS + 7 {
        every.advance(&gen, t);
    }
    jump.advance(&gen, 0);
    jump.advance(&gen, 3 * STEP_TICKS + 6);
    assert_eq!(every, jump);
    let (mid, end) = (3 * STEP_TICKS + 6, 3 * STEP_TICKS + 6 + 40 * (DAY_TICKS / 24));
    let mut back = Atmosphere::read_state(&mut ByteReader::new(&saved(&every))).unwrap();
    let mut ahead = Ahead::new(&every, mid);
    for _ in 0..39 {
        ahead.hour(&gen, IVec3::new(0, 70, 0));
    }
    every.advance(&gen, end - DAY_TICKS / 24);
    back.advance(&gen, end - DAY_TICKS / 24);
    assert_eq!(saved(&back), saved(&every), "a loaded save runs on the same");
    assert_eq!(back, every, "derived fields too");
    let pos = IVec3::new(100, 70, -300);
    assert_eq!(
        ahead.hour(&gen, pos),
        Ahead::new(&every, end - DAY_TICKS / 24).hour(&gen, pos),
        "the forecast is what comes"
    );
    let fresh = Atmosphere::default();
    assert_eq!(saved(&Atmosphere::read_state(&mut ByteReader::new(&saved(&fresh))).unwrap()), saved(&fresh));
}

/// The calendar keeps its outlook and only extends it: as the world runs on, it says what a fresh forecast says.
#[test]
fn a_kept_outlook_agrees_with_a_fresh_forecast() {
    let gen = WorldGen::new(7);
    let pos = IVec3::new(300, SEA_LEVEL + 3, -200);
    let mut air = Atmosphere::default();
    let mut tick = 3 * DAY_TICKS;
    air.advance(&gen, tick);
    let mut kept = Outlook::new(&air, pos, tick);
    for _ in 0..6 {
        let now = Reading::at(&air, &gen, pos.x, pos.y, pos.z);
        assert_eq!(kept.days(&gen, now, tick, 3), forecast(&gen, &air, pos, tick, 3), "at {tick}");
        tick += 7 * DAY_TICKS / 24;
        air.advance(&gen, tick);
    }
}

#[test]
fn rain_speeds_growth_and_snow_does_not() {
    let pos = IVec3::new(0, SEA_LEVEL + 2, 0);
    let normal = DAY_TICKS;
    let (mut quicker, mut plain) = (0, 0);
    run(7, 60, |air, gen| {
        let (r, tick) = (Reading::at(air, gen, pos.x, pos.y, pos.z), air.tick);
        let due = growth_due(gen, air, pos, tick, normal);
        if r.sky.precip > 0 && !r.snow {
            assert!(due <= growth_ticks(tick, normal), "rain at {tick}");
        } else {
            assert_eq!(due, growth_ticks(tick, normal), "{r:?}");
        }
        if r.sky.precip >= RAIN_FROM && !r.snow {
            assert!(due < growth_ticks(tick, normal), "rain at {tick}");
            quicker += 1;
        } else {
            plain += 1;
        }
    });
    assert!(quicker > 0 && plain > quicker, "{quicker} rainy steps of {}", quicker + plain);
}

#[test]
fn clouds_dim_the_sun_grid() {
    let mut darkest = 1000;
    run(1, 20, |air, _| {
        let sun = air.sun_grid();
        for (x, z) in [(0, 0), (3_000, 3_000), (-6_000, 100)] {
            let share = sun.share_at(x, z);
            assert!((1000 - SOLAR_DIM..=1000).contains(&share));
            darkest = darkest.min(share);
        }
    });
    assert!(darkest < 900, "the darkest sky lets {darkest} through");
    assert_eq!(SunGrid::default().share_at(5, 5), 1000, "no air yet: full sun");
}

/// A forecast is cheap enough for the calendar: a step of the whole grid in well under a millisecond, here in a debug
/// build (wasm in release is several times quicker). Prints the cost.
#[test]
fn a_step_is_cheap() {
    let gen = WorldGen::new(1);
    let mut air = Atmosphere::default();
    air.advance(&gen, 0);
    let steps = 6 * STEPS_A_DAY; // the calendar's six days
    let start = std::time::Instant::now();
    air.advance(&gen, steps * STEP_TICKS);
    let each = start.elapsed().as_micros() as u64 / steps;
    eprintln!("{each} µs a step of {} cells; six days ahead {} ms", N * N, each * steps / 1000);
    assert!(each < 1_000, "{each} µs a step");
}

/// A tuning aid, not a check: a station's log at the spawn of seed 7 for ten days, every six hours and every wet hour.
/// `cargo test -q --release --lib station_log -- --ignored --nocapture`.
#[test]
#[ignore]
fn station_log() {
    let mut k = 0;
    run(7, 10, |air, gen| {
        k += 1;
        if k % 6 != 0 {
            return;
        }
        let r = Reading::at(air, gen, 0, SEA_LEVEL + 3, 0);
        let sky = ["clear", "cloudy", "RAIN", "STORM"][r.sky.kind() as usize];
        if k % 36 == 0 || r.sky.precip > 0 {
            eprintln!(
                "d{:>2} {:>2}h {:>5.1}C rh{:>3}% dew{:>5.1} {:>6.1}hPa wind {:>4.1}m/s from {:>3} cover{:>4} precip{:>4} {sky}",
                k / 144, (k % 144) / 6, f64::from(r.temp) / 100.0, r.humidity / 10, f64::from(r.dew) / 100.0,
                f64::from(r.pressure) / 100.0, f64::from(r.wind()) / 100.0, r.wind_from(), r.sky.cover, r.sky.precip
            );
        }
    });
}

/// A peak keeps its air's humidity: high ground, many degrees colder than the low air its cell holds, is no wetter on
/// average than the lowlands and is not pinned at saturation under a fair sky (it read 100 % before `reading::local`).
#[test]
fn a_peak_is_no_wetter_than_the_lowlands() {
    let gen = WorldGen::new(1337);
    let mut peak = (0, 0, 0);
    for x in (-1_536..=1_536).step_by(64) {
        for z in (-1_536..=1_536).step_by(64) {
            let h = gen.height_at(x, z);
            if h > peak.2 {
                peak = (x, z, h);
            }
        }
    }
    assert!(peak.2 >= SEA_LEVEL + 80, "{peak:?}: a peak at least 10 °C colder than the sea's shore");
    let (mut n, mut saturated, mut sum) = (0, 0, [0i64; 2]);
    run(1337, 12, |air, gen| {
        let high = Reading::at(air, gen, peak.0, peak.2 + 1, peak.1);
        let low = Reading::at(air, gen, 0, gen.height_at(0, 0).max(SEA_LEVEL) + 1, 0);
        n += 1;
        saturated += i32::from(high.humidity >= 990 && high.sky.precip == 0);
        sum[0] += i64::from(high.humidity);
        sum[1] += i64::from(low.humidity);
    });
    let (high, low) = (sum[0] / n / 10, sum[1] / n / 10);
    assert!(saturated * 20 < n as i32, "the peak read saturated and dry {saturated} of {n} steps");
    assert!((high - low).abs() <= 15, "mean humidity {high} % on the peak, {low} % below");
}
