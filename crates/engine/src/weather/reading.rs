//! What a weather station at a place reads (`Reading::at`): the atmosphere's cells around it (`air.rs`, blended) plus
//! the place itself. Its temperature is its own climate's for the hour and height (`Climate::temperature`) shifted by the
//! air mass over it (warmer or colder than that air's rest); its relative humidity and dew point follow from the air's
//! vapour, kept at that air's relative humidity at the place's own day's mean temperature, so nights are damp; falling rain moistens the air under it. Clouds come from the air's cloud
//! water and from how close its air is to saturating; rain from what the air rained out; below 0 °C it falls as snow. The
//! wind at the ground is the air's, slowed by friction.
//! A query: reads the air, changes nothing. To tune what the sky shows: the constants below.

use super::air::{grid_of, sample, Atmosphere, P0, STEER};
use super::physics::{capacity, dew_point, qsat, relative_humidity, smooth};
use super::{Climate, Sky, FAIR_COVER, RAIN_FROM};
use crate::worldgen::WorldGen;

/// Relative humidity (thousandths) of the air's lower clouds from which cloud gathers and at which the sky is overcast.
const CLOUD_FROM: i64 = 820;
const CLOUD_FULL: i64 = 1_000;
/// Cloud water (mg/kg in mild air: `sky_of` scales cold air's up) that greys the sky, and that makes it as dark as it gets.
const GREY_FROM: i64 = 60;
const GREY_FULL: i64 = 500;
/// Rain (mg/kg a step) that makes the heaviest precipitation, and how much of it is a light shower's start.
const RAIN_FULL: i64 = 200;
/// How much of the wind aloft blows at a station's height, thousandths (friction with the ground slows it).
const SURFACE_WIND: i32 = 600;
/// How close to saturation (thousandths) rain (from `RAIN_FROM`, drizzle less) brings the air under it as it falls.
const UNDER_RAIN: i32 = 950;
/// How much colder (hundredths of °C) the air is where its clouds form than the air mass's mean at sea level.
const CLOUD_LEVEL: i32 = 400;

/// A place's weather now, in the units a weather station reports.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Reading {
    /// Air temperature and dew point, hundredths of °C.
    pub temp: i32,
    pub dew: i32,
    /// Relative humidity, thousandths.
    pub humidity: i32,
    /// Sea-level pressure, Pa.
    pub pressure: i32,
    /// Wind towards the east and the south, cm/s.
    pub east: i32,
    pub south: i32,
    /// The sky: cloud cover, gloom and precipitation in thousandths.
    pub sky: Sky,
    /// Whether what falls is snow.
    pub snow: bool,
}

impl Reading {
    /// The weather at (`x`, `y`, `z`) by the air's last step; calm air for the climate before the air has started.
    pub fn at(air: &Atmosphere, gen: &WorldGen, x: i32, y: i32, z: i32) -> Reading {
        let climate = Climate::at(gen, x, z);
        if !air.started() {
            return calm(climate, y, air.tick);
        }
        let (fx, fz) = grid_of(x, z);
        let get = |field: &[i32]| sample(field, fx, fz);
        let tick = air.tick;
        let warmer = get(&air.temp) - get(&air.rest);
        let temp = climate.temperature(y, tick) * 10 + warmer;
        let (vapour, cloud) = (get(&air.vapour), get(&air.cloud));
        let Sky { gloom, precip, .. } = sky_of(cloud, get(&air.rain), get(&air.temp));
        let aloft = i64::from(relative_humidity(vapour, get(&air.temp) - CLOUD_LEVEL));
        let vapour = local(vapour, get(&air.temp), climate.day_mean(y, tick) * 10 + warmer);
        // Rain evaporating as it falls moistens the air under it (the air's own vapour is the cloud layer's).
        let wet = (qsat(temp) * UNDER_RAIN / 1000 - vapour).max(0);
        let vapour = vapour + wet * smooth(0, i64::from(RAIN_FROM), i64::from(precip)) as i32 / 1000;
        let thin = smooth(CLOUD_FROM, CLOUD_FULL, aloft).max(smooth(0, GREY_FROM, scaled(cloud, get(&air.temp))));
        let cover = (thin as u32).max(gloom).max(FAIR_COVER);
        Reading {
            temp,
            dew: dew_point(vapour).min(temp),
            humidity: relative_humidity(vapour, temp),
            pressure: get(&air.pressure),
            east: get(&air.east) * SURFACE_WIND / 1000,
            south: get(&air.south) * SURFACE_WIND / 1000,
            sky: Sky { cover, gloom, precip },
            snow: temp < 0,
        }
    }

    /// Wind speed, cm/s.
    pub fn wind(&self) -> i32 {
        (f64::from(self.east).hypot(f64::from(self.south))) as i32
    }

    /// The direction the wind blows from, degrees clockwise from north (0 north, 90 east).
    pub fn wind_from(&self) -> i32 {
        // Blowing towards (east, south) means coming from (-east, -south); north is -z.
        let deg = f64::from(-self.east).atan2(f64::from(self.south)).to_degrees();
        (deg.round() as i32).rem_euclid(360)
    }
}

/// The gloom and precipitation of air at `temp` holding `cloud` mg/kg of cloud water that rained `rain` mg/kg in its last
/// step, judged against what such air can hold (`physics::capacity`; cover is left at fair: it needs the place too).
pub(crate) fn sky_of(cloud: i32, rain: i32, temp: i32) -> Sky {
    let (cloud, rain) = (scaled(cloud, temp), scaled(rain, temp));
    let precip = smooth(0, RAIN_FULL, rain) as u32;
    let gloom = (smooth(GREY_FROM, GREY_FULL, cloud) * 850 / 1000).max(i64::from(precip)) as u32;
    Sky { cover: FAIR_COVER, gloom, precip }
}

/// The vapour (mg/kg) at a place whose day's mean temperature is `mean` under a cell's air holding `vapour` at `temp`:
/// the place keeps the air's relative humidity at its own mean, so a peak or a cold valley in a milder cell is no wetter
/// than the air (it read saturated under a fair sky before), and only the hour moves it: nights are damp.
fn local(vapour: i32, temp: i32, mean: i32) -> i32 {
    (i64::from(vapour) * i64::from(qsat(mean)) / i64::from(qsat(temp).max(1))) as i32
}

/// `amount` of water in air at `temp` as the same share of what mild air holds.
fn scaled(amount: i32, temp: i32) -> i64 {
    i64::from(amount) * 1000 / i64::from(capacity(temp))
}

/// The weather before the air has started: the climate's temperature, fair sky, still air from the west.
fn calm(climate: Climate, y: i32, tick: u64) -> Reading {
    let temp = climate.temperature(y, tick) * 10;
    let vapour = qsat(climate.day_mean(y, tick) * 10) * climate.damp(0) / 1000;
    Reading {
        temp,
        dew: dew_point(vapour).min(temp),
        humidity: relative_humidity(vapour, temp),
        pressure: P0,
        east: STEER * SURFACE_WIND / 1000,
        south: 0,
        sky: Sky { cover: FAIR_COVER, gloom: 0, precip: 0 },
        snow: temp < 0,
    }
}

/// Blocks the steering wind carries the weather east in a day.
#[cfg(test)]
pub(crate) fn drift_per_day() -> i64 {
    use super::air::{CARRY, STEP_TICKS};
    let steps = crate::daytime::DAY_TICKS / STEP_TICKS;
    steps as i64 * i64::from(STEER) * CARRY / 1000 * i64::from(super::air::CELL) / 65_536
}
