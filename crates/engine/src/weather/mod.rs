//! Weather (ATMOS): a simulated atmosphere, the weather emerging from what a weather station measures. The air is a
//! wrapping grid of cells (`air.rs`, core state, saved) holding temperature, water vapour, cloud water and pressure; the
//! pressure drives the wind, the wind carries heat and vapour, the ground warms, cools and wets the air, rising air
//! condenses into cloud and cloud rains out (`physics.rs` holds the saturation maths). A place reads it as a station
//! would (`reading.rs`: temperature, relative humidity, dew point, pressure, wind, sky), its own climate, hour and height
//! on top (`climate.rs`). The forecast is the same model run ahead on a copy (`forecast.rs`).
//!
//! - The core: `Sim::step` advances the air every `air::STEP_TICKS`; clouds dim solar panels through the sun each cell
//!   lets through ([`SunGrid`], handed to the factory after each step) and rain speeds growth ([`growth_due`]).
//! - The host draws it (`api/weather.rs` → `web/src/render/sky/weather.ts`, `precip.ts`) and the calendar shows the days
//!   ahead. Whole-number maths but the climate field (the f64 noise world generation reads), so every platform agrees.
//!
//! To tune: the constants in `air.rs` (the physics) and `reading.rs` (what the sky shows); `tests.rs` pins the climate.

mod air;
mod climate;
mod forecast;
mod physics;
mod reading;

pub use air::Atmosphere;
pub use climate::Climate;
#[cfg(test)]
pub use forecast::forecast;
pub use forecast::{Ahead, Outlook, FORECAST_FIELDS};
pub use reading::Reading;

use crate::daytime::growth_ticks;
use crate::math::IVec3;
use crate::worldgen::WorldGen;

/// A clear sky's fair-weather cloud: the key art's sky.
pub const FAIR_COVER: u32 = 380;
/// Precipitation (thousandths) from which it is rain (or snow), and a storm.
pub const RAIN_FROM: u32 = 150;
pub const STORM_FROM: u32 = 750;
/// Cloud cover above which a dry sky counts as cloudy.
const CLOUDY_FROM: u32 = 450;
/// The share of sunlight a full gloom takes from a solar panel.
const SOLAR_DIM: u32 = 400;
/// The share of growing time a steady rain saves.
const RAIN_GROWTH: u64 = 500;

/// The sky at a place, in thousandths: cloud cover, how dark and heavy the cloud is, how hard it rains or snows.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct Sky {
    pub cover: u32,
    pub gloom: u32,
    pub precip: u32,
}

/// What a player would call the sky (the calendar's icons; the order is the host's).
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum Kind {
    Clear = 0,
    Cloudy = 1,
    Rain = 2,
    Storm = 3,
}

impl Sky {
    pub fn kind(self) -> Kind {
        if self.precip >= STORM_FROM {
            Kind::Storm
        } else if self.precip >= RAIN_FROM {
            Kind::Rain
        } else if self.cover > CLOUDY_FROM {
            Kind::Cloudy
        } else {
            Kind::Clear
        }
    }

    /// The share of sunlight that reaches the ground, in thousandths.
    pub fn sun_share(self) -> u32 {
        1000 - SOLAR_DIM * self.gloom / 1000
    }
}

/// The sun each air cell lets through to solar panels (thousandths), from its clouds; empty is full sun. The factory
/// holds a copy (`Factory::sun`), refreshed after each step of the air, so panels need no access to the air itself.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct SunGrid(Vec<i32>);

impl SunGrid {
    /// Whether it has not been handed over yet (a fresh or loaded factory).
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// The sun reaching (x, z), in thousandths.
    pub fn share_at(&self, x: i32, z: i32) -> u32 {
        if self.0.is_empty() {
            return 1000;
        }
        let (fx, fz) = air::grid_of(x, z);
        air::sample(&self.0, fx, fz).clamp(0, 1000) as u32
    }
}

impl Atmosphere {
    /// What each cell's clouds let through to solar panels now.
    pub fn sun_grid(&self) -> SunGrid {
        SunGrid(
            (0..self.cloud.len())
                .map(|i| reading::sky_of(self.cloud[i], self.rain[i], self.temp[i]).sun_share() as i32)
                .collect(),
        )
    }
}

/// The tick at which growth that takes `normal` ticks at the year-round rate, started at `tick` at `pos`, is done: the
/// season's pace (`daytime::growth_ticks`), shortened by the rain (not snow) falling there now, up to `RAIN_GROWTH` of
/// the time in the heaviest rain. Grass, sod and saplings use it.
pub fn growth_due(gen: &WorldGen, air: &Atmosphere, pos: IVec3, tick: u64, normal: u64) -> u64 {
    let due = growth_ticks(tick, normal);
    let here = Reading::at(air, gen, pos.x, pos.y, pos.z);
    if here.snow {
        return due;
    }
    due - (due - tick) * RAIN_GROWTH * u64::from(here.sky.precip) / 1_000_000
}

#[cfg(test)]
mod tests;
