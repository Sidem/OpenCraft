//! A place's climate, the ground the weather stands on: its temperature without the weather (the climate's warmth + the
//! season + the time of day − the height above the sea, in tenths of a degree Celsius) and how much water its ground
//! gives the air. The year is warmest in the middle of summer and coldest in the middle of winter; the day is warmest at
//! 15:00 and coldest at 03:00. Integer maths after the climate field. The air's own warm and cold masses come on top
//! (`air.rs`). To tune: `MILD`, `WARMTH_SPAN`, `YEAR_SWING`, `DAY_SWING`, `LAPSE_BLOCKS`, `DAMP` and `SEA_DAMP`.

use crate::daytime::{DAY_TICKS, START, YEAR_DAYS};
use crate::worldgen::{WorldGen, SEA_LEVEL};

/// A neutral climate's yearly mean (tenths of a degree), and how far the climate's warmth (thousandths, about ±600)
/// moves it: tenths per thousand.
const MILD: i64 = 100;
const WARMTH_SPAN: i64 = 400;
/// How far the year and the day swing the temperature each way (tenths).
const YEAR_SWING: i64 = 120;
const DAY_SWING: i64 = 40;
/// A degree colder for every this many blocks above the sea.
const LAPSE_BLOCKS: i64 = 8;
/// The relative humidity (thousandths) a neutral land's ground keeps the air at, how far the climate's moisture
/// (thousandths) moves it, how much its warmth dries it, and what open sea keeps it at.
const DAMP: i32 = 700;
const MOISTURE_DAMP: i32 = 250;
const WARMTH_DRY: i32 = 200;
const SEA_DAMP: i32 = 880;

/// The climate of a column, in thousandths of the generator's climate field (about ±600).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Climate {
    pub warmth: i32,
    pub moisture: i32,
}

impl Climate {
    /// The climate of column (x, z).
    pub fn at(gen: &WorldGen, x: i32, z: i32) -> Climate {
        let (t, m) = gen.climate_at(x, z);
        Climate::of((t * 1000.0) as i32, (m * 1000.0) as i32)
    }

    pub fn of(warmth: i32, moisture: i32) -> Climate {
        Climate { warmth, moisture }
    }

    /// The relative humidity (thousandths) this ground keeps the air at by evaporation, `sea` thousandths of it open
    /// water: wet climates and the sea more, warm dry land less.
    pub fn damp(self, sea: i32) -> i32 {
        let land = DAMP + self.moisture * MOISTURE_DAMP / 1000 - self.warmth.max(0) * WARMTH_DRY / 1000;
        (land.clamp(250, 800) * (1000 - sea) + SEA_DAMP * sea) / 1000
    }

    /// The temperature at height `y` at `tick`, in tenths of a degree Celsius, before the weather's warm or cold air.
    pub fn temperature(self, y: i32, tick: u64) -> i32 {
        let at = tick + START;
        // The day's wave crests at 15:00.
        let day = DAY_SWING * wave((at + DAY_TICKS - DAY_TICKS * 3 / 8) % DAY_TICKS, DAY_TICKS) / 1000;
        (i64::from(self.day_mean(y, tick)) + day) as i32
    }

    /// The day's mean temperature at height `y` at `tick` (tenths of a degree): `temperature` without the hour.
    pub fn day_mean(self, y: i32, tick: u64) -> i32 {
        let height = i64::from((y - SEA_LEVEL).max(0)) * 10 / LAPSE_BLOCKS;
        (i64::from(self.mean_temperature(tick)) - height) as i32
    }

    /// The day's mean temperature at sea level at `tick` (tenths of a degree): the climate and the season, no hour.
    pub fn mean_temperature(self, tick: u64) -> i32 {
        const YEAR: u64 = YEAR_DAYS * DAY_TICKS;
        // The year's wave crests at 3/8 (mid-summer).
        let year = YEAR_SWING * wave((tick + START + YEAR - YEAR / 8) % YEAR, YEAR) / 1000;
        (MILD + i64::from(self.warmth) * WARMTH_SPAN / 1000 + year) as i32
    }
}

/// A smooth wave over `period` in thousandths: a parabola up to 1000 over the first half, down to -1000 over the second.
fn wave(at: u64, period: u64) -> i64 {
    let (p, half) = (at as i64, period as i64 / 2);
    if p < half {
        4000 * p * (half - p) / (half * half)
    } else {
        -4000 * (p - half) * (2 * half - p) / (half * half)
    }
}
