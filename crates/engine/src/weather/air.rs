//! The atmosphere (core state): a grid of air cells over the world, each with what a weather station measures — the
//! air's temperature, its water vapour (and so its relative humidity and dew point), the cloud water it carries, the
//! pressure, and the wind that the pressure drives. Every `STEP_TICKS` the grid moves on (`step`):
//!
//! 1. Pressure systems: highs and lows grow and fill (a slow, wrapping noise that the steering wind carries east, the
//!    synoptic stir no cheap model makes by itself), deepened where warm air rises and where rain releases heat.
//! 2. Wind from the pressure: along the isobars (lows turn anticlockwise), turned a little into the lows by friction,
//!    plus the steering westerly. Not stored: a function of the pressure.
//! 3. The wind carries heat, vapour and cloud (semi-Lagrangian: each cell takes what was upwind), so warm and cold air
//!    meet in fronts and wet air reaches the land.
//! 4. The ground pulls the air towards its climate's temperature for the season and evaporates water into it (the sea
//!    most); air rising into lows cools; where it saturates, vapour condenses into cloud (warming the air); thick cloud
//!    rains out; cloud in dry air evaporates.
//!
//! The grid wraps every `PERIOD` blocks each way (an endless world reads it modulo), centred on the spawn, whose ground
//! the cells sample. Places read it through `reading.rs`, which adds their own climate, height and hour.
//!
//! Invariants: whole-number maths only, so every peer computes the same; nothing reads the loaded chunks, the camera or
//! the frame (it depends on the seed, the generator and the tick alone, so a forecast is the same model run ahead). The
//! cell's ground is derived from the generator and never saved. To tune: the constants below; `tests.rs` pins the
//! climate they make (pressure range, how often it rains, storms, fronts moving east) and the cost of a step.

use super::physics::{capacity, qsat, Lattice};
use super::Climate;
use crate::bytes::{ByteReader, ByteWriter};
use crate::daytime::DAY_TICKS;
use crate::worldgen::{WorldGen, SEA_LEVEL};

/// Cells along each side of the grid, and blocks along each side of a cell.
pub const N: usize = 32;
pub const CELL: i32 = 384;
/// The grid repeats every this many blocks each way.
pub const PERIOD: i32 = N as i32 * CELL;
/// Ticks between steps: ten in-game minutes.
pub const STEP_TICKS: u64 = DAY_TICKS / 144;
/// Sea-level pressure with no weather, pascals.
pub const P0: i32 = 101_325;
/// The steering westerly, cm/s.
pub const STEER: i32 = 500;
/// How far (1/65536 cell) a wind of 1 cm/s carries the air in a step, ×1000: 1 m/s is 7.5 blocks an in-game hour.
pub const CARRY: i64 = 2133;

/// Pressure systems: how deep the big ones (4 lattice points round the grid) and the small ones (8) get, and how long one
/// lasts.
const BIG_SYSTEMS: i64 = 2_600;
const SMALL_SYSTEMS: i64 = 1_100;
const SYSTEM_TICKS: i64 = DAY_TICKS as i64 * 2;
/// Steps for the pressure, the temperature and the vapour to come most of the way to what drives them.
const PRESSURE_STEPS: i32 = 60;
const TEMP_STEPS: i32 = 120;
const EVAP_STEPS: i32 = 120;
/// Wind (cm/s) per Pa of pressure difference across a cell, along the isobars and across them into the lows.
const GEO: i32 = 3;
const FRICTION: i32 = 1;
/// How far air is lifted, as the cooling it gets (hundredths of °C): rising into a low (per Pa of the pressure's
/// curvature, ×1000), riding up over colder air at a warm front (per cm/s of wind times hundredths of a degree warmer a
/// cell upwind, ×1,000,000), boiling up as showers where it is colder than its ground (per hundredth of a degree, ×1000), and climbing a
/// slope (per block the wind climbs a cell, at the steering wind's speed). Each of the last three is at most `MAX_PART`:
/// they tip damp air over, so a windward slope or a coast is wetter, not always raining. At most `MAX_LIFT` in all.
const LIFT: i32 = 2_500;
const FRONT_LIFT: i64 = 1_000;
const SHOWER_LIFT: i32 = 400;
const SLOPE_LIFT: i64 = 12;
const MAX_PART: i32 = 300;
const MAX_LIFT: i32 = 900;
/// Cloud water (mg/kg in mild air, less in cold: `physics::capacity`) above which it rains, and the share of the excess
/// that falls each step (thousandths).
const CLOUD_RAINS: i32 = 220;
const RAIN_SHARE: i32 = 400;

/// The ground under a cell: its climate, how wet it keeps the air (relative humidity, thousandths), and how its land
/// rises across it (blocks, east and south).
#[derive(Clone, Copy, Debug, PartialEq)]
struct Ground {
    climate: Climate,
    damp: i32,
    slope: (i32, i32),
}

/// The air over the world. Empty until its first step (`ensure`), which sets it at rest for its climate.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Atmosphere {
    /// The tick of the last step.
    pub(crate) tick: u64,
    /// Per cell, row by row (z then x): temperature (hundredths of °C, the day's mean at sea level), vapour and cloud
    /// water (mg/kg), sea-level pressure (Pa), and the rain that fell in the last step (mg/kg).
    pub(crate) temp: Vec<i32>,
    pub(crate) vapour: Vec<i32>,
    pub(crate) cloud: Vec<i32>,
    pub(crate) pressure: Vec<i32>,
    pub(crate) rain: Vec<i32>,
    /// Derived, not saved: the wind east and south (cm/s) from the pressure, the temperature each cell's ground pulls it
    /// to now (hundredths of °C), and the ground from the generator.
    pub(crate) east: Vec<i32>,
    pub(crate) south: Vec<i32>,
    pub(crate) rest: Vec<i32>,
    ground: Vec<Ground>,
}

impl Atmosphere {
    /// Steps the air to `tick` if a step falls due (call every tick; a world with no air yet starts at rest).
    pub fn advance(&mut self, gen: &WorldGen, tick: u64) {
        if self.temp.is_empty() {
            self.start(gen, tick);
        }
        self.ensure(gen);
        while self.tick + STEP_TICKS <= tick {
            self.step(gen.seed(), self.tick + STEP_TICKS);
        }
    }

    /// Whether the air has started (an empty one reads as calm).
    pub fn started(&self) -> bool {
        !self.temp.is_empty() && !self.ground.is_empty()
    }

    /// The sea-level temperature the ground pulls cell `i` towards at `tick` (hundredths of °C).
    fn rest_temp(&self, i: usize, tick: u64) -> i32 {
        self.ground[i].climate.mean_temperature(tick) * 10
    }

    /// The air at rest at `tick`: each cell at its climate's temperature, as damp as its ground keeps it, no cloud.
    fn start(&mut self, gen: &WorldGen, tick: u64) {
        self.ground.clear();
        self.tick = tick / STEP_TICKS * STEP_TICKS;
        self.ensure(gen);
        self.temp = self.rest.clone();
        self.vapour = (0..N * N).map(|i| qsat(self.temp[i]) * self.ground[i].damp / 1000).collect();
        self.cloud = vec![0; N * N];
        self.rain = vec![0; N * N];
        self.pressure = systems(gen.seed(), self.tick).iter().map(|s| P0 + s).collect();
        self.winds();
    }

    /// Samples each cell's ground from the generator, once (it is not saved), and the derived fields that need it.
    fn ensure(&mut self, gen: &WorldGen) {
        if self.ground.len() == N * N {
            return;
        }
        let q = CELL / 4;
        let mut land = vec![0; N * N];
        let wet: Vec<i32> = (0..N * N)
            .map(|i| {
                let (x, z) = centre(i);
                let heights = [(-q, -q), (q, -q), (-q, q), (q, q)].map(|(dx, dz)| gen.height_at(x + dx, z + dz));
                land[i] = heights.iter().map(|h| (h - SEA_LEVEL).max(0)).sum::<i32>() / 4;
                heights.iter().filter(|&&h| h < SEA_LEVEL).count() as i32 * 250
            })
            .collect();
        self.ground = (0..N * N)
            .map(|i| {
                let (x, z) = centre(i);
                let climate = Climate::at(gen, x, z);
                Ground { climate, damp: climate.damp(wet[i]), slope: gradient(&land, i) }
            })
            .collect();
        self.rest = (0..N * N).map(|i| self.rest_temp(i, self.tick)).collect();
        if !self.pressure.is_empty() {
            self.winds();
        }
    }

    /// One step, ending at `tick`.
    fn step(&mut self, seed: u32, tick: u64) {
        // 1-3: the wind (from the pressure at the last step's end) carries the air.
        let (temp, vapour, cloud) = (self.carried(&self.temp), self.carried(&self.vapour), self.carried(&self.cloud));
        // Lows and highs travel with the wind too (a low the rain deepened moves on rather than raining in place).
        let (old_pressure, old_temp, pull) = (self.carried(&self.pressure), self.temp.clone(), systems(seed, tick));
        for i in 0..N * N {
            let ground = self.ground[i];
            let rest = ground.climate.mean_temperature(tick) * 10;
            self.rest[i] = rest;
            let (mut t, mut q, mut c) = (temp[i], vapour[i], cloud[i]);
            // 4: the ground warms or cools the air and evaporates into it.
            t += (rest - t) / TEMP_STEPS;
            q += (qsat(t) * ground.damp / 1000 - q) / EVAP_STEPS;
            // Lifted air cools; where it saturates it condenses.
            let lift = self.lift(i, &old_pressure, &old_temp, rest - t);
            let saturated = qsat(t - lift);
            if q > saturated {
                let made = (q - saturated) / 2;
                (q, c) = (q - made, c + made);
            } else {
                let dried = c.min((saturated - q) / 2);
                (q, c) = (q + dried, c - dried);
            }
            let rain = (c - CLOUD_RAINS * capacity(t) / 1000).max(0) * RAIN_SHARE / 1000;
            c -= rain;
            // 1: the pressure follows the systems.
            let p = old_pressure[i] + (P0 + pull[i] - old_pressure[i]) / PRESSURE_STEPS;
            (self.temp[i], self.vapour[i], self.cloud[i], self.rain[i], self.pressure[i]) = (t, q.max(0), c, rain, p);
        }
        smooth(&mut self.temp);
        smooth(&mut self.vapour);
        self.winds();
        self.tick = tick;
    }

    /// How far cell `i`'s air is lifted this step, as cooling (hundredths of °C): into a low, over a warm front, in
    /// showers where it is `colder` than its ground, and up the slope the wind blows against.
    fn lift(&self, i: usize, pressure: &[i32], temp: &[i32], colder: i32) -> i32 {
        let (east, south) = (i64::from(self.east[i]), i64::from(self.south[i]));
        let low = curvature(pressure, i) * LIFT / 1000;
        let (gx, gz) = gradient(temp, i);
        let front = (-(east * i64::from(gx) + south * i64::from(gz)) * FRONT_LIFT / 1_000_000) as i32;
        let shower = colder * SHOWER_LIFT / 1000;
        let (sx, sz) = self.ground[i].slope;
        let slope = ((east * i64::from(sx) + south * i64::from(sz)) * SLOPE_LIFT / i64::from(STEER)) as i32;
        let part = |v: i32| v.clamp(0, MAX_PART);
        (low.max(0) + part(front) + part(shower) + part(slope)).min(MAX_LIFT)
    }
    /// The wind in every cell from the pressure (cm/s east and south).
    fn winds(&mut self) {
        let (mut east, mut south) = (vec![0; N * N], vec![0; N * N]);
        for i in 0..N * N {
            let (gx, gz) = gradient(&self.pressure, i);
            // Along the isobars with the low on the left (north is -z), and across them into the low.
            east[i] = STEER + GEO * gz - FRICTION * gx;
            south[i] = -GEO * gx - FRICTION * gz;
        }
        (self.east, self.south) = (east, south);
    }

    /// `field` carried a step by the wind: each cell takes the value upwind of it.
    fn carried(&self, field: &[i32]) -> Vec<i32> {
        (0..N * N)
            .map(|i| {
                let (x, z) = ((i % N) as i64, (i / N) as i64);
                let fx = (x << 16) - i64::from(self.east[i]) * CARRY / 1000;
                let fz = (z << 16) - i64::from(self.south[i]) * CARRY / 1000;
                sample(field, fx, fz)
            })
            .collect()
    }

    pub fn write_state(&self, w: &mut ByteWriter) {
        w.u64(self.tick);
        w.count(self.temp.len());
        for field in [&self.temp, &self.vapour, &self.cloud, &self.pressure, &self.rain] {
            for &v in field {
                w.u32(v as u32);
            }
        }
    }

    /// The air as saved; the ground and the wind follow on the next `advance`.
    pub fn read_state(r: &mut ByteReader) -> Option<Atmosphere> {
        let tick = r.u64()?;
        let n = r.count()?;
        if n != 0 && n != N * N {
            return None;
        }
        let mut fields = Vec::with_capacity(5);
        for _ in 0..5 {
            fields.push((0..n).map(|_| r.u32().map(|v| v as i32)).collect::<Option<Vec<i32>>>()?);
        }
        let [temp, vapour, cloud, pressure, rain]: [Vec<i32>; 5] = fields.try_into().ok()?;
        Some(Atmosphere { tick, temp, vapour, cloud, pressure, rain, ..Default::default() })
    }
}

/// The world (x, z) at the middle of cell `i`.
pub(crate) fn centre(i: usize) -> (i32, i32) {
    let half = N as i32 / 2;
    (((i % N) as i32 - half) * CELL + CELL / 2, ((i / N) as i32 - half) * CELL + CELL / 2)
}

/// Where world (x, z) falls on the grid, in 1/65536 cell from cell 0's middle.
pub(crate) fn grid_of(x: i32, z: i32) -> (i64, i64) {
    let along = |v: i32| (i64::from((v + PERIOD / 2).rem_euclid(PERIOD)) << 16) / i64::from(CELL) - (1 << 15);
    (along(x), along(z))
}

/// `field` at grid position (`fx`, `fz`) in 1/65536 cell, bilinear, wrapping.
pub(crate) fn sample(field: &[i32], fx: i64, fz: i64) -> i32 {
    let (x0, z0) = (fx >> 16, fz >> 16);
    let (tx, tz) = (fx & 0xFFFF, fz & 0xFFFF);
    let at = |x: i64, z: i64| i64::from(field[(z.rem_euclid(N as i64) as usize) * N + x.rem_euclid(N as i64) as usize]);
    let top = at(x0, z0) * (65536 - tx) + at(x0 + 1, z0) * tx;
    let bottom = at(x0, z0 + 1) * (65536 - tx) + at(x0 + 1, z0 + 1) * tx;
    ((top * (65536 - tz) + bottom * tz) >> 32) as i32
}

/// The pressure systems' pull on every cell at `tick` (Pa from `P0`): wrapping noise drifting east with the steering
/// wind, big systems and small.
fn systems(seed: u32, tick: u64) -> Vec<i32> {
    let steps = (tick / STEP_TICKS) as i64;
    // How far the steering wind has carried the pattern, in thousandths of a cell.
    let drift = steps * i64::from(STEER) * CARRY / 65_536;
    let big = Lattice::<4>::new(seed ^ 0x51ED_0000, tick as i64, SYSTEM_TICKS);
    let small = Lattice::<8>::new(seed ^ 0x51ED_0001, tick as i64, SYSTEM_TICKS);
    (0..N * N)
        .map(|i| {
            let (x, z) = ((i % N) as i64 * 1000 + 500 - drift, (i / N) as i64 * 1000 + 500);
            let big = (big.at(x / (N as i64 / 4), z / (N as i64 / 4)) - 500) * BIG_SYSTEMS / 500;
            let small = (small.at(x / (N as i64 / 8), z / (N as i64 / 8)) - 500) * SMALL_SYSTEMS / 500;
            (big + small) as i32
        })
        .collect()
}
/// The pressure's difference across cell `i` (Pa, east and south).
fn gradient(p: &[i32], i: usize) -> (i32, i32) {
    let (x, z) = (i % N, i / N);
    let at = |x: usize, z: usize| p[(z % N) * N + x % N];
    ((at(x + 1, z) - at(x + N - 1, z)) / 2, (at(x, z + 1) - at(x, z + N - 1)) / 2)
}

/// How much higher the pressure is round cell `i` than in it (Pa): positive in a low, where air converges and rises.
fn curvature(p: &[i32], i: usize) -> i32 {
    let (x, z) = (i % N, i / N);
    let at = |x: usize, z: usize| p[(z % N) * N + x % N];
    at(x + 1, z) + at(x + N - 1, z) + at(x, z + 1) + at(x, z + N - 1) - 4 * p[i]
}

/// Mixes a little of each cell's neighbours into it, so the grid stays smooth.
fn smooth(field: &mut [i32]) {
    let old = field.to_vec();
    for (i, v) in field.iter_mut().enumerate() {
        *v += curvature(&old, i) / 16;
    }
}
