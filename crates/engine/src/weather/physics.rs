//! The air's physics in whole numbers: how much water vapour air can hold (`qsat`, Tetens' formula tabled at sea-level
//! pressure), relative humidity and the dew point from it, and the smooth periodic noise that stirs up pressure systems.
//! Pure functions; every platform agrees. Units: temperature in hundredths of a degree Celsius, vapour and cloud water in
//! milligrams per kilogram of air (1 g/kg = 1000), pressure in pascals.

use crate::math::hash3;

/// Saturation specific humidity (mg/kg) from -50 °C to +50 °C, one entry a degree: 622·e/(p − 0.378·e) with Tetens'
/// e = 6.112·exp(17.67·T/(T + 243.5)) hPa and p = 1013.25 hPa.
const QSAT: [i32; 101] = [
    39, 44, 49, 55, 61, 68, 76, 85, 94, 105, 116, 129, 143, 158, 175, 193, 213, 235, 259, 285, 313, 344, 378, 414, 454,
    497, 544, 594, 649, 708, 772, 841, 916, 997, 1084, 1177, 1278, 1386, 1503, 1628, 1762, 1906, 2061, 2227, 2404,
    2595, 2798, 3015, 3248, 3496, 3761, 4043, 4344, 4665, 5007, 5371, 5759, 6171, 6608, 7074, 7568, 8092, 8649, 9239,
    9865, 10528, 11230, 11973, 12760, 13592, 14472, 15402, 16385, 17423, 18519, 19676, 20897, 22185, 23542, 24974,
    26482, 28070, 29743, 31505, 33359, 35310, 37362, 39520, 41790, 44175, 46683, 49317, 52085, 54992, 58045, 61251,
    64616, 68148, 71855, 75746, 79828,
];

/// The most water vapour air at `t` (hundredths of °C) holds, in mg/kg; clamped to the table's -50..50 °C.
pub fn qsat(t: i32) -> i32 {
    let at = (t + 5000).clamp(0, 9999);
    let (i, f) = ((at / 100) as usize, at % 100);
    QSAT[i] + (QSAT[i + 1] - QSAT[i]) * f / 100
}

/// How much water air at `t` can hold next to mild air at 15 °C, thousandths (at least `MIN_CAPACITY`): cold air's light
/// snow is as much weather as warm air's downpour, so amounts of cloud and rain are judged against it.
pub fn capacity(t: i32) -> i32 {
    (qsat(t) * 1000 / qsat(1500)).max(MIN_CAPACITY)
}

const MIN_CAPACITY: i32 = 150;

/// Relative humidity in thousandths (0..=1000) of vapour `q` (mg/kg) in air at `t`.
pub fn relative_humidity(q: i32, t: i32) -> i32 {
    (i64::from(q.max(0)) * 1000 / i64::from(qsat(t).max(1))).min(1000) as i32
}

/// The dew point (hundredths of °C) of vapour `q`: the temperature at which it saturates the air.
pub fn dew_point(q: i32) -> i32 {
    let q = q.max(QSAT[0]);
    let i = QSAT.partition_point(|&s| s <= q).clamp(1, QSAT.len() - 1);
    let (lo, hi) = (QSAT[i - 1], QSAT[i]);
    (i as i32 - 1) * 100 - 5000 + ((q - lo) * 100 / (hi - lo).max(1)).min(100)
}

/// Smooth value noise in thousandths (0..1000) that wraps, at one moment: `CELLS` lattice points around each axis of the
/// torus (a power of two), a lattice step of `span` ticks along time. Built once per step, read per cell (`at`).
pub struct Lattice<const CELLS: usize>([[i64; CELLS]; CELLS]);

impl<const CELLS: usize> Lattice<CELLS> {
    pub fn new(seed: u32, t: i64, span: i64) -> Self {
        let (it, ft) = (t.div_euclid(span), fade(t.rem_euclid(span) * 1000 / span));
        let value =
            |x: usize, z: usize, t: i64| (i64::from(hash3(seed, x as i32, z as i32, t as i32) >> 8) * 1000) >> 24;
        let mut v = [[0; CELLS]; CELLS];
        for (z, row) in v.iter_mut().enumerate() {
            for (x, out) in row.iter_mut().enumerate() {
                *out = lerp(value(x, z, it), value(x, z, it + 1), ft);
            }
        }
        Lattice(v)
    }

    /// The noise at (`x`, `z`) in lattice units × 1000.
    pub fn at(&self, x: i64, z: i64) -> i64 {
        let mask = CELLS as i64 - 1;
        let (ix, iz) = (x.div_euclid(1000), z.div_euclid(1000));
        let (fx, fz) = (fade(x.rem_euclid(1000)), fade(z.rem_euclid(1000)));
        let at = |dx: i64, dz: i64| self.0[((iz + dz) & mask) as usize][((ix + dx) & mask) as usize];
        lerp(lerp(at(0, 0), at(1, 0), fx), lerp(at(0, 1), at(1, 1), fx), fz)
    }
}

/// `smoothstep(lo, hi, v)` in thousandths.
pub fn smooth(lo: i64, hi: i64, v: i64) -> i64 {
    fade(((v - lo) * 1000 / (hi - lo)).clamp(0, 1000))
}

fn fade(f: i64) -> i64 {
    f * f * (3000 - 2 * f) / 1_000_000
}

fn lerp(a: i64, b: i64, t: i64) -> i64 {
    a + (b - a) * t / 1000
}
