//! A small brush for painting plants at the texture array's full resolution (`TEX_SIZE`, 64): shapes are signed
//! distances in unit space (x right, y up from the ground at 0, both 0 to 1), so a plant is a list of blades, leaves
//! and petals instead of hand-placed texels. A `Pen` stands on one texel; each shape call lowers its distance to the
//! nearest shape and remembers that shape's colour; the last shape drawn wins where it covers the texel (painter's
//! order). `rgba` turns the result into a texel: opaque inside a shape, transparent outside but keeping the plant's
//! own colour so mips show no dark fringe. To add a shape: a method that works out a signed distance and calls `put`.

use super::TEX_SIZE;
use crate::math::{hash3, unit};

pub type Rgb = [f64; 3];
pub type Pt = (f64, f64);

/// Segments a curved blade is cut into when its distance is measured.
const SEGMENTS: usize = 12;

pub struct Pen {
    x: f64,
    y: f64,
    px: i32,
    py: i32,
    sd: f64,
    col: Rgb,
}

/// Random 0 to 1 from a plant's `seed` and the number of the thing being placed: the same every run.
pub fn rand(seed: u32, i: i32) -> f64 {
    unit(hash3(seed, i, 0, 3))
}

pub fn shade(c: Rgb, k: f64) -> Rgb {
    [c[0] * k, c[1] * k, c[2] * k]
}

/// The point at `s` (0 to 1) along the curve a blade follows from `from` to `to`, bent sideways by `bow` towards the tip.
pub fn curve(from: Pt, to: Pt, bow: f64, s: f64) -> Pt {
    let (dx, dy) = (to.0 - from.0, to.1 - from.1);
    let len = (dx * dx + dy * dy).sqrt().max(1e-9);
    (from.0 + dx * s - dy / len * bow * s * s, from.1 + dy * s + dx / len * bow * s * s)
}

impl Pen {
    /// A pen on texel (`px`, `py`) of a `TEX_SIZE` layer; `fallback` is the colour of an empty texel.
    pub fn new(px: i32, py: i32, fallback: Rgb) -> Pen {
        let size = TEX_SIZE as f64;
        Pen {
            x: (f64::from(px) + 0.5) / size,
            y: 1.0 - (f64::from(py) + 0.5) / size,
            px,
            py,
            sd: f64::MAX,
            col: fallback,
        }
    }

    /// Fine noise 0 to 1 at this texel.
    pub fn grain(&self, salt: u32) -> f64 {
        unit(hash3(salt, self.px, self.py, 7))
    }

    /// A tapered blade from `from` to `to`, `width` wide (half-width) at the root, bowed by `bow`; lighter towards the tip
    /// with a darker midrib.
    pub fn blade(&mut self, from: Pt, to: Pt, bow: f64, width: f64, col: Rgb) {
        let reach = width + bow.abs() + 0.01;
        if self.x < from.0.min(to.0) - reach
            || self.x > from.0.max(to.0) + reach
            || self.y < from.1.min(to.1) - reach
            || self.y > from.1.max(to.1) + reach
        {
            return;
        }
        let mut prev = from;
        let mut best = (f64::MAX, 0.0, 0.0);
        for i in 1..=SEGMENTS {
            let s1 = i as f64 / SEGMENTS as f64;
            let q = curve(from, to, bow, s1);
            let (ex, ey) = (q.0 - prev.0, q.1 - prev.1);
            let t =
                (((self.x - prev.0) * ex + (self.y - prev.1) * ey) / (ex * ex + ey * ey).max(1e-12)).clamp(0.0, 1.0);
            let d = (self.x - prev.0 - ex * t).hypot(self.y - prev.1 - ey * t);
            let s = s1 - (1.0 - t) / SEGMENTS as f64;
            let half = width * (1.0 - s).powf(0.6) + 0.002;
            if d - half < best.0 {
                best = (d - half, s, d / half);
            }
            prev = q;
        }
        let vein = if best.2 < 0.2 { 0.88 } else { 1.0 };
        self.put(best.0, shade(col, (0.72 + 0.4 * best.1 + 0.12 * self.grain(11)) * vein));
    }

    /// An ellipse at `c` with radii `rx` and `ry`, its long axis turned `rot` radians from horizontal; lighter in the middle.
    pub fn ellipse(&mut self, c: Pt, rx: f64, ry: f64, rot: f64, col: Rgb) {
        let (s, co) = rot.sin_cos();
        let (dx, dy) = (self.x - c.0, self.y - c.1);
        let (lx, ly) = (dx * co + dy * s, dy * co - dx * s);
        let k = ((lx / rx).powi(2) + (ly / ry).powi(2)).sqrt();
        if k > 1.6 {
            return;
        }
        self.put((k - 1.0) * rx.min(ry), shade(col, 0.8 + 0.22 * (1.0 - k.min(1.0)) + 0.14 * self.grain(13)));
    }

    pub fn disc(&mut self, c: Pt, r: f64, col: Rgb) {
        self.ellipse(c, r, r, 0.0, col);
    }

    /// `n` petals round `c`: ellipses `len` long and `wide` wide whose centres sit `reach` out, the first at angle `turn`.
    #[allow(clippy::too_many_arguments)]
    pub fn petals(&mut self, c: Pt, n: i32, reach: f64, len: f64, wide: f64, turn: f64, col: Rgb) {
        for i in 0..n {
            let a = turn + std::f64::consts::TAU * f64::from(i) / f64::from(n);
            self.ellipse((c.0 + a.cos() * reach, c.1 + a.sin() * reach), len, wide, a, col);
        }
    }

    /// The texel: its colour, opaque where a shape covers it.
    pub fn rgba(&self) -> [u8; 4] {
        let ch = |v: f64| v.round().clamp(0.0, 255.0) as u8;
        [ch(self.col[0]), ch(self.col[1]), ch(self.col[2]), if self.sd <= 0.0 { 255 } else { 0 }]
    }

    fn put(&mut self, sd: f64, col: Rgb) {
        if sd < 0.0 || sd < self.sd {
            self.sd = sd;
            self.col = col;
        }
    }
}
