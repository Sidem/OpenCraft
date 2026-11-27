//! A place's climate as the weather reads it, and its temperature: the climate's warmth + the season + the time of day
//! − the height above the sea, in tenths of a degree Celsius. The year is warmest in the middle of summer and coldest in
//! the middle of winter; the day is warmest at 15:00 and coldest at 03:00. Integer maths after the climate field.
//! To tune: `MILD`, `WARMTH_SPAN`, `YEAR_SWING`, `DAY_SWING`, `LAPSE_BLOCKS`, and `WETNESS` / `DRYNESS` for the rain.

use crate::daytime::{DAY_TICKS, START, YEAR_DAYS};
use crate::worldgen::{WorldGen, SEA_LEVEL};

use super::{storminess, Sky};

/// A neutral climate's yearly mean (tenths of a degree), and how far the climate's warmth (thousandths, about ±600)
/// moves it: tenths per thousand.
const MILD: i64 = 100;
const WARMTH_SPAN: i64 = 400;
/// How far the year and the day swing the temperature each way (tenths).
const YEAR_SWING: i64 = 120;
const DAY_SWING: i64 = 40;
/// A degree colder for every this many blocks above the sea.
const LAPSE_BLOCKS: i64 = 8;
/// How much the climate's moisture wets the weather and its warmth dries it (thousandths of storminess per thousandth).
const WETNESS: i64 = 300;
const DRYNESS: i64 = 150;

/// The climate of a column, in thousandths: its warmth and the shift it gives the storminess.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Climate {
    pub warmth: i32,
    pub wet: i32,
}

impl Climate {
    /// The climate of column (x, z).
    pub fn at(gen: &WorldGen, x: i32, z: i32) -> Climate {
        let (t, m) = gen.climate_at(x, z);
        Climate::of((t * 1000.0) as i32, (m * 1000.0) as i32)
    }

    /// The climate of a warmth and moisture in thousandths.
    pub fn of(warmth: i32, moisture: i32) -> Climate {
        let wet = i64::from(moisture) * WETNESS / 1000 - i64::from(warmth.max(0)) * DRYNESS / 1000;
        Climate { warmth, wet: wet as i32 }
    }

    /// The sky over (x, z) at `tick`, this climate's storminess.
    pub fn sky(self, seed: u32, x: i32, z: i32, tick: u64) -> Sky {
        Sky::from_storminess(storminess(seed, x, z, tick) + i64::from(self.wet))
    }

    /// The share of sunlight that reaches (x, z) at `tick`, in thousandths.
    pub fn sun_share(self, seed: u32, x: i32, z: i32, tick: u64) -> u32 {
        self.sky(seed, x, z, tick).sun_share()
    }

    /// The temperature at height `y` at `tick`, in tenths of a degree Celsius.
    pub fn temperature(self, y: i32, tick: u64) -> i32 {
        const YEAR: u64 = YEAR_DAYS * DAY_TICKS;
        let at = tick + START;
        // The year's wave crests at 3/8 (mid-summer), the day's at 15:00.
        let year = YEAR_SWING * wave((at + YEAR - YEAR / 8) % YEAR, YEAR) / 1000;
        let day = DAY_SWING * wave((at + DAY_TICKS - DAY_TICKS * 3 / 8) % DAY_TICKS, DAY_TICKS) / 1000;
        let height = i64::from((y - SEA_LEVEL).max(0)) * 10 / LAPSE_BLOCKS;
        (MILD + i64::from(self.warmth) * WARMTH_SPAN / 1000 + year + day - height) as i32
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
