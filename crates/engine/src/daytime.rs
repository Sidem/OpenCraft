//! Time of day and the calendar, derived from the core tick: a day lasts [`DAY_TICKS`], a new world starts in the
//! morning of the first day of spring, and four seasons of [`DAYS_PER_SEASON`] days make a year. Because the tick is
//! core state, the time and season are saved with the world, identical for every co-op player, and need no state of
//! their own; the world pauses while the game is closed, and so does the year. The host turns them into sky colours,
//! sun and daylight (`web/src/render/sky.ts`).
//!
//! Invariants: days are longest at the end of spring and shortest at the end of autumn, noon is always at 12:00, and
//! a year's daylight adds up to exactly as many ticks as twelve-hour days would (so solar's yearly output is where it
//! was); everything a core rule reads is integer maths, so every platform agrees. Growth follows the season
//! ([`growth_ticks`]). To tune: `DAYS_PER_SEASON`, `DAY_SWING`, `GROWTH_RATE`.

use crate::TICK_RATE;

/// Ticks in one day: 20 minutes.
pub const DAY_TICKS: u64 = 20 * 60 * TICK_RATE as u64;
/// Days in each season.
pub const DAYS_PER_SEASON: u64 = 9;
/// Days in a year: spring, summer, autumn, winter.
pub const YEAR_DAYS: u64 = 4 * DAYS_PER_SEASON;
/// Where tick 0 falls in the day: 7:00.
pub(crate) const START: u64 = DAY_TICKS * 7 / 24;
/// How far daylight swings from twelve hours at the longest and shortest day: four hours.
const DAY_SWING: u64 = DAY_TICKS / 6;
/// Growth per tick in each season, where 2 is the year-round rate it replaced: twice that in spring, half again in summer,
/// half in autumn, none in winter; they add up to the same growth in a year (user, 2026-11-26), so a Forester's yearly
/// output is where it was.
const GROWTH_RATE: [u64; 4] = [4, 3, 1, 0];

/// The seasons, in calendar order (`season` returns the index).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Season {
    Spring = 0,
    Summer = 1,
    Autumn = 2,
    Winter = 3,
}

/// The time of day at `tick` as a fraction of the clock: 0 midnight, 0.5 noon. Sunrise and sunset move with the season
/// ([`sun_time`]).
pub fn time_of_day(tick: u64) -> f64 {
    ((tick + START) % DAY_TICKS) as f64 / DAY_TICKS as f64
}

/// The sun's place in its day at `tick`: 0 midnight, 0.25 sunrise, 0.5 noon, 0.75 sunset, whatever the season. The
/// host draws the sky from it, so the sky turns light exactly when [`sunlight`] starts (presentation only).
pub fn sun_time(tick: u64) -> f64 {
    let x = time_of_day(tick);
    let rise = (DAY_TICKS - day_length(day_index(tick))) as f64 / 2.0 / DAY_TICKS as f64;
    if x < rise {
        0.25 * x / rise
    } else if x < 1.0 - rise {
        0.25 + 0.5 * (x - rise) / (1.0 - 2.0 * rise)
    } else {
        0.75 + 0.25 * (x - (1.0 - rise)) / rise
    }
}

/// Sunlight at `tick` in thousandths: 0 at night, 1000 at noon, a parabola between the day's sunrise and sunset. Integer
/// maths only, so every platform agrees (solar panels: `factory/process/solar.rs`).
pub fn sunlight(tick: u64) -> u32 {
    let len = day_length(day_index(tick));
    let rise = (DAY_TICKS - len) / 2;
    let x = (tick + START) % DAY_TICKS;
    if x < rise || x >= rise + len {
        return 0;
    }
    let x = x - rise;
    (4000 * x * (len - x) / (len * len)) as u32
}

/// Which day `tick` falls on, counting from 1.
pub fn day_number(tick: u64) -> u64 {
    day_index(tick) + 1
}

/// The season at `tick`.
pub fn season(tick: u64) -> Season {
    match day_index(tick) / DAYS_PER_SEASON % 4 {
        0 => Season::Spring,
        1 => Season::Summer,
        2 => Season::Autumn,
        _ => Season::Winter,
    }
}

/// Which day of its season `tick` falls on, counting from 1.
pub fn day_of_season(tick: u64) -> u64 {
    day_index(tick) % DAYS_PER_SEASON + 1
}

/// How far through the year `tick` is, from 0 (the first morning of spring is just past it) to 1 (presentation only).
pub fn year_fraction(tick: u64) -> f64 {
    ((tick + START) % (YEAR_DAYS * DAY_TICKS)) as f64 / (YEAR_DAYS * DAY_TICKS) as f64
}

/// The share of day `day` of the year (from 0, wrapping) the sun is up, 0 to 1: the calendar draws the year's daylight
/// from it (presentation only).
pub fn daylight_share(day: u64) -> f64 {
    day_length(day) as f64 / DAY_TICKS as f64
}

/// How fast things grow in `season` as a percentage of the year-round rate (`GROWTH_RATE`; the calendar shows it).
pub fn growth_percent(season: Season) -> u32 {
    (GROWTH_RATE[season as usize] * 100 / 2) as u32
}

/// The tick at which something that takes `normal` ticks of growth (at the year-round average rate), started at `tick`,
/// is done: growth runs at the season's rate (`GROWTH_RATE`), so a spring sapling grows twice as fast and one planted
/// in autumn waits out the winter. Grass, sod and saplings use it (`sim/timers.rs`).
pub fn growth_ticks(tick: u64, normal: u64) -> u64 {
    const SEASON_TICKS: u64 = DAYS_PER_SEASON * DAY_TICKS;
    let year_work: u64 = GROWTH_RATE.iter().sum::<u64>() * SEASON_TICKS;
    let mut need = normal * 2;
    // Whole years first: each takes the same growth from wherever it starts.
    let mut at = tick + START + need / year_work * YEAR_DAYS * DAY_TICKS;
    need %= year_work;
    if need == 0 {
        return at - START;
    }
    loop {
        let rate = GROWTH_RATE[(at / SEASON_TICKS % 4) as usize];
        let end = (at / SEASON_TICKS + 1) * SEASON_TICKS;
        if rate > 0 && need <= (end - at) * rate {
            return at + need.div_ceil(rate) - START;
        }
        need -= (end - at) * rate;
        at = end;
    }
}

/// Days since the world began, from 0.
fn day_index(tick: u64) -> u64 {
    (tick + START) / DAY_TICKS
}

/// Daylight ticks on day `day`: twelve hours plus `DAY_SWING` times a smooth wave over the year, its crest at the end
/// of spring and its trough at the end of autumn. The wave is two parabolas sampled at the middle of each day, which
/// cancel exactly over a year.
fn day_length(day: u64) -> u64 {
    // Half-days into the year at the day's middle: 1, 3, ... 71.
    let v = (2 * (day % YEAR_DAYS) + 1) as i64;
    let half = YEAR_DAYS as i64;
    let arc = if v < half { v * (half - v) } else { -(v - half) * (2 * half - v) };
    let wave = arc * 4000 / (half * half);
    (DAY_TICKS as i64 / 2 + DAY_SWING as i64 * wave / 1000) as u64
}

#[cfg(test)]
mod tests;
