//! The days ahead at a place, for the calendar (`web/src/ui/calendar.ts`) and the console: the atmosphere run ahead on a
//! copy ([`Ahead`]) and read every in-game hour ([`Outlook`]). Each whole day, today first (from now), gives the worst
//! sky that lasts a while, whether its precipitation falls as snow, and its low and high temperatures. Since the air
//! depends on nothing but the seed, the generator and the tick, the copy's future is the world's: an outlook stays true
//! as time passes and is only extended, never rerun. A query: the real air is not touched.

use super::{Atmosphere, Reading, RAIN_FROM};
use crate::daytime::{DAY_TICKS, START};
use crate::math::IVec3;
use crate::worldgen::WorldGen;

/// Numbers per day in [`Outlook::days`]'s answer.
pub const FORECAST_FIELDS: usize = 4;
/// Hours of a kind of sky (or worse) that make the day that kind, by `Kind`: a third of the day under cloud, two hours
/// of rain or of storm.
const HOURS_FOR: [u32; 4] = [0, 8, 2, 2];
const HOUR: u64 = DAY_TICKS / 24;

/// The air run ahead from a tick on a copy: `hour` runs it an in-game hour on and reads a place there.
#[derive(Clone)]
pub struct Ahead {
    air: Atmosphere,
    pub tick: u64,
}

impl Ahead {
    pub fn new(air: &Atmosphere, tick: u64) -> Ahead {
        Ahead { air: air.clone(), tick }
    }

    pub fn hour(&mut self, gen: &WorldGen, pos: IVec3) -> Reading {
        self.tick += HOUR;
        self.air.advance(gen, self.tick);
        Reading::at(&self.air, gen, pos.x, pos.y, pos.z)
    }
}

/// A place's weather every in-game hour from when it was made on, kept and extended as it is asked further ahead.
pub struct Outlook {
    pub pos: IVec3,
    ahead: Ahead,
    hours: Vec<(u64, Reading)>,
}

impl Outlook {
    pub fn new(air: &Atmosphere, pos: IVec3, tick: u64) -> Outlook {
        Outlook { pos, ahead: Ahead::new(air, tick), hours: Vec::new() }
    }

    /// `days` days from today at `tick`, `now` the weather there now: per day its `Kind` (as its number: the worst sky
    /// that lasts `HOURS_FOR`), 1 if its rain or snow is snow, and the low and high temperatures in tenths of a degree.
    pub fn days(&mut self, gen: &WorldGen, now: Reading, tick: u64, days: u32) -> Vec<i32> {
        if days == 0 {
            return Vec::new();
        }
        let midnight = (tick + START) / DAY_TICKS * DAY_TICKS;
        let end = |d: u64| midnight + (d + 1) * DAY_TICKS - START;
        let last = end(u64::from(days) - 1);
        self.hours.retain(|&(t, _)| t > tick);
        while self.ahead.tick + HOUR <= last {
            self.hours.push((self.ahead.tick + HOUR, self.ahead.hour(gen, self.pos)));
        }
        let mut out = Vec::with_capacity(days as usize * FORECAST_FIELDS);
        let mut hours = self.hours.iter().peekable();
        for d in 0..u64::from(days) {
            let (mut count, mut snow, mut low, mut high) = ([0u32; 4], false, i32::MAX, i32::MIN);
            let mut take = |r: &Reading| {
                count[..=r.sky.kind() as usize].iter_mut().for_each(|n| *n += 1);
                snow |= r.sky.precip >= RAIN_FROM && r.snow;
                (low, high) = (low.min(r.temp / 10), high.max(r.temp / 10));
            };
            if d == 0 {
                take(&now);
            }
            while let Some((_, r)) = hours.next_if(|(t, _)| *t <= end(d)) {
                take(r);
            }
            let kind = (0..4).rev().find(|&k| count[k] >= HOURS_FOR[k]).unwrap_or(0);
            out.extend([kind as i32, i32::from(snow), low, high]);
        }
        out
    }
}

/// `days` days from today at `pos`, from the air as it is at `tick` (a fresh [`Outlook`]).
#[cfg(test)]
pub fn forecast(gen: &WorldGen, air: &Atmosphere, pos: IVec3, tick: u64, days: u32) -> Vec<i32> {
    let now = Reading::at(air, gen, pos.x, pos.y, pos.z);
    Outlook::new(air, pos, tick).days(gen, now, tick, days)
}
