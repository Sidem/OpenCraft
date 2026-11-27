//! Weather and temperature (WEATHER): pure functions of the world's seed, a place and the core tick. Nothing is stored,
//! saved or synced, so every co-op peer agrees, loaded chunks or not.
//!
//! - Fronts: [`Climate::sky`] gives the cloud cover, gloom and precipitation (thousandths) at (x, z) and a tick from a
//!   storminess made of three slow integer value noises: fronts `FRONT_BLOCKS` across that the wind carries east and
//!   that change over `FRONT_TICKS`, smaller squalls, and wetter and drier weeks for the whole world. The place's
//!   climate (`worldgen/biome.rs` `climate_at`) shifts it: dry deserts, wet lowlands.
//! - Temperature: [`Climate::temperature`] (`climate.rs`), in tenths of a degree; precipitation below 0 °C is snow.
//! - The core reads it in two places only: clouds dim solar panels ([`Climate::sun_share`], `factory/process/solar.rs`)
//!   and rain speeds growth ([`growth_due`], `sim/timers.rs`). All integer maths but the climate field (the same f64
//!   noise world generation reads), so every platform agrees.
//! - The host draws it (`api/weather.rs` → `web/src/render/sky/weather.ts`, `precip.ts`); the calendar shows the days
//!   ahead ([`forecast`]).
//!
//! To tune: the constants below; the tests pin the shares of clear, cloudy and wet time and how fast it changes.

mod climate;
mod forecast;

pub use climate::Climate;
pub use forecast::{forecast, FORECAST_FIELDS};

use crate::daytime::{growth_ticks, DAY_TICKS};
use crate::math::{hash3, IVec3};
use crate::worldgen::WorldGen;

/// Everything here is in thousandths.
const ONE: i64 = 1000;
/// A front's size in blocks and the ticks it takes to change; squalls are smaller and quicker.
const FRONT_BLOCKS: i64 = 768;
const FRONT_TICKS: i64 = DAY_TICKS as i64 * 3;
const SQUALL_BLOCKS: i64 = 320;
const SQUALL_TICKS: i64 = DAY_TICKS as i64 * 9 / 10;
/// Wetter and drier weeks, the same everywhere.
const SPELL_TICKS: i64 = DAY_TICKS as i64 * 9;
/// How far the wind carries the weather east in a day, in blocks.
const WIND_PER_DAY: i64 = 900;
/// The shares of fronts, squalls and spells in the storminess.
const WEIGHTS: [i64; 3] = [660, 160, 180];
/// The storminess over which cloud gathers, gloom darkens it and precipitation falls (smoothstep edges).
const CLOUDING: (i64, i64) = (460, 660);
const DARKENING: (i64, i64) = (570, 710);
const FALLING: (i64, i64) = (650, 820);
/// A clear sky's fair-weather cloud: the key art's sky.
pub const FAIR_COVER: u32 = 380;
/// Precipitation from which it is rain (or snow), and a storm.
pub const RAIN_FROM: u32 = 150;
pub const STORM_FROM: u32 = 750;
/// Cloud cover above which a dry sky counts as cloudy.
const CLOUDY_FROM: u32 = 450;
/// The share of sunlight a full gloom takes from a solar panel.
const SOLAR_DIM: u32 = 400;
/// The share of growing time a steady rain saves, and how many times a growth timer's span is sampled for rain.
const RAIN_GROWTH: u64 = 500;
const GROWTH_SAMPLES: u64 = 4;

/// The sky at a place and tick, in thousandths: cloud cover, how dark and heavy the cloud is, how hard it rains or snows.
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
    /// The sky for a storminess `s` (thousandths, any range).
    pub fn from_storminess(s: i64) -> Sky {
        let precip = smooth(FALLING, s);
        let gloom = (smooth(DARKENING, s) * 850 / 1000).max(precip);
        let cover = smooth(CLOUDING, s).max(gloom).max(FAIR_COVER);
        Sky { cover, gloom, precip }
    }

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

/// How stormy it is at (x, z) at `tick` before the climate shifts it, in thousandths (about 0..1000).
pub fn storminess(seed: u32, x: i32, z: i32, tick: u64) -> i64 {
    let t = tick as i64;
    // The wind carries the pattern east: what is at x now was at x - drift when the pattern was laid.
    let x = i64::from(x) - t * WIND_PER_DAY / DAY_TICKS as i64;
    let z = i64::from(z);
    let front = noise3(seed, [x, z, t], [FRONT_BLOCKS, FRONT_BLOCKS, FRONT_TICKS]);
    let squall = noise3(seed ^ 0x5BD1_E995, [x, z, t], [SQUALL_BLOCKS, SQUALL_BLOCKS, SQUALL_TICKS]);
    let spell = noise3(seed ^ 0x27D4_EB2F, [0, 0, t], [1, 1, SPELL_TICKS]);
    (WEIGHTS[0] * front + WEIGHTS[1] * squall + WEIGHTS[2] * spell) / ONE
}

/// The tick at which growth that takes `normal` ticks at the year-round rate, started at `tick` at `pos`, is done: the
/// season's pace (`daytime::growth_ticks`), shortened by the rain (not snow) that falls there meanwhile, up to
/// `RAIN_GROWTH` of the time under steady rain. Grass, sod and saplings use it.
pub fn growth_due(gen: &WorldGen, pos: IVec3, tick: u64, normal: u64) -> u64 {
    let due = growth_ticks(tick, normal);
    let span = due - tick;
    let climate = Climate::at(gen, pos.x, pos.z);
    let rain: u64 = (0..GROWTH_SAMPLES)
        .map(|k| tick + span * (2 * k + 1) / (2 * GROWTH_SAMPLES))
        .filter(|&at| climate.temperature(pos.y, at) >= 0)
        .map(|at| u64::from(climate.sky(gen.seed(), pos.x, pos.z, at).precip))
        .sum();
    due - span * RAIN_GROWTH * (rain / GROWTH_SAMPLES) / 1_000_000
}

/// Smooth 3-D value noise in thousandths (0..1000): one lattice point per `cell` along each axis.
fn noise3(seed: u32, p: [i64; 3], cell: [i64; 3]) -> i64 {
    let i = [0, 1, 2].map(|k| p[k].div_euclid(cell[k]));
    let f = [0, 1, 2].map(|k| fade(p[k].rem_euclid(cell[k]) * ONE / cell[k]));
    let at = |dx: i64, dy: i64, dz: i64| {
        let h = hash3(seed, (i[0] + dx) as i32, (i[2] + dz) as i32, (i[1] + dy) as i32);
        (i64::from(h >> 8) * ONE) >> 24
    };
    let edge = |dy: i64, dz: i64| lerp(at(0, dy, dz), at(1, dy, dz), f[0]);
    let face = |dz: i64| lerp(edge(0, dz), edge(1, dz), f[1]);
    lerp(face(0), face(1), f[2])
}

/// The smoothstep fade of a fraction in thousandths.
fn fade(f: i64) -> i64 {
    f * f * (3 * ONE - 2 * f) / (ONE * ONE)
}

fn lerp(a: i64, b: i64, t: i64) -> i64 {
    a + (b - a) * t / ONE
}

/// `smoothstep(edge.0, edge.1, s)` in thousandths.
fn smooth(edge: (i64, i64), s: i64) -> u32 {
    let t = ((s - edge.0) * ONE / (edge.1 - edge.0)).clamp(0, ONE);
    fade(t) as u32
}

#[cfg(test)]
mod tests;
