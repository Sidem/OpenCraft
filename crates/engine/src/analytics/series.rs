//! One graphed quantity over time, kept at two resolutions so an hour is fine-grained and three hours still fit:
//! the last [`FINE_SECS`] seconds one value each, and the last [`COARSE_SECS`] seconds one value per
//! [`COARSE_STEP`] seconds. `window` reads either back, averaged into the points a graph has room for.
//!
//! Invariant: the newest value is the last one pushed; a window longer than the history starts with `NAN`s
//! (the graph leaves those blank). Presentation only: never saved, never read by the core.

/// Seconds at one-second resolution (one hour).
pub const FINE_SECS: u32 = 3600;
/// Seconds at coarse resolution (three hours) and how many seconds each coarse value covers.
pub const COARSE_SECS: u32 = 10_800;
pub const COARSE_STEP: u32 = 10;

/// A fixed-size ring of values, newest last.
struct Ring {
    data: Vec<f32>,
    /// Where the next value goes.
    head: usize,
    len: usize,
}

impl Ring {
    /// `n` zeros as history (for a series that starts late, so its past reads as nothing made).
    fn zeros(cap: usize, n: usize) -> Ring {
        let n = n.min(cap);
        Ring { data: vec![0.0; cap], head: n % cap, len: n }
    }

    fn push(&mut self, v: f32) {
        self.data[self.head] = v;
        self.head = (self.head + 1) % self.data.len();
        self.len = (self.len + 1).min(self.data.len());
    }

    /// The value `age` pushes ago (0 is the newest), if there is that much history.
    fn age(&self, age: usize) -> Option<f32> {
        (age < self.len).then(|| self.data[(self.head + self.data.len() - 1 - age) % self.data.len()])
    }
}

pub struct Series {
    fine: Ring,
    coarse: Ring,
    /// The fine values since the last coarse one: their sum and count.
    acc: f32,
    acc_n: u32,
}

impl Default for Series {
    fn default() -> Series {
        Series::after(0)
    }
}

impl Series {
    /// A series that begins `secs` seconds into the game, with zeros before.
    pub fn after(secs: u32) -> Series {
        Series {
            fine: Ring::zeros(FINE_SECS as usize, secs as usize),
            coarse: Ring::zeros((COARSE_SECS / COARSE_STEP) as usize, (secs / COARSE_STEP) as usize),
            acc: 0.0,
            acc_n: secs % COARSE_STEP,
        }
    }

    /// Adds the value of one more second.
    pub fn push(&mut self, v: f32) {
        self.fine.push(v);
        self.acc += v;
        self.acc_n += 1;
        if self.acc_n >= COARSE_STEP {
            self.coarse.push(self.acc / self.acc_n as f32);
            (self.acc, self.acc_n) = (0.0, 0);
        }
    }

    /// The newest value (0 before any).
    pub fn latest(&self) -> f32 {
        self.fine.age(0).unwrap_or(0.0)
    }

    /// Appends the last `secs` seconds as at most `points` averages, oldest first (`NAN` before the history begins).
    /// Returns how many points it appended: fewer than `points` when the window has fewer values than that.
    pub fn window(&self, secs: u32, points: usize, out: &mut Vec<f32>) -> usize {
        let (ring, step) = if secs <= FINE_SECS { (&self.fine, 1) } else { (&self.coarse, COARSE_STEP) };
        let n = ((secs / step) as usize).max(1);
        let points = points.clamp(1, n);
        for i in 0..points {
            let (from, to) = (i * n / points, (i + 1) * n / points);
            let (mut sum, mut count) = (0.0, 0);
            for j in from..to {
                if let Some(v) = ring.age(n - 1 - j) {
                    sum += v;
                    count += 1;
                }
            }
            out.push(if count == 0 { f32::NAN } else { sum / count as f32 });
        }
        points
    }
}
