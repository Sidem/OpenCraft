//! The days ahead at a place, for the calendar (`web/src/ui/calendar.ts`): each whole day, today first, sampled every
//! hour, gives the worst sky that lasts a while, whether its precipitation falls as snow, and its low and high
//! temperatures. A query: it reads the pure weather functions and changes nothing.

use crate::daytime::{DAY_TICKS, START};
use crate::math::IVec3;
use crate::worldgen::WorldGen;

use super::{Climate, RAIN_FROM};

/// Numbers per day in [`forecast`]'s answer.
pub const FORECAST_FIELDS: usize = 4;
/// Hours of a kind of sky (or worse) that make the day that kind, by `Kind`: a third of the day under cloud, two hours
/// of rain or of storm.
const HOURS_FOR: [u32; 4] = [0, 8, 2, 2];

/// `days` days from today at `pos`: per day its `Kind` (as its number: the worst sky that lasts `HOURS_FOR`), 1 if its
/// rain or snow is snow, and the low and high temperatures in tenths of a degree.
pub fn forecast(gen: &WorldGen, pos: IVec3, tick: u64, days: u32) -> Vec<i32> {
    let climate = Climate::at(gen, pos.x, pos.z);
    let midnight = (tick + START) / DAY_TICKS * DAY_TICKS;
    let mut out = Vec::with_capacity(days as usize * FORECAST_FIELDS);
    for d in 0..u64::from(days) {
        let (mut hours, mut snow, mut low, mut high) = ([0u32; 4], false, i32::MAX, i32::MIN);
        for h in 0..24 {
            // The middle of the hour, as a tick (the small hours of the world's first day come before tick 0).
            let at = (midnight + d * DAY_TICKS + (2 * h + 1) * DAY_TICKS / 48).saturating_sub(START);
            let sky = climate.sky(gen.seed(), pos.x, pos.z, at);
            let temp = climate.temperature(pos.y, at);
            hours[..=sky.kind() as usize].iter_mut().for_each(|n| *n += 1);
            snow |= sky.precip >= RAIN_FROM && temp < 0;
            low = low.min(temp);
            high = high.max(temp);
        }
        let kind = (0..4).rev().find(|&k| hours[k] >= HOURS_FOR[k]).unwrap_or(0);
        out.extend([kind as i32, i32::from(snow), low, high]);
    }
    out
}
