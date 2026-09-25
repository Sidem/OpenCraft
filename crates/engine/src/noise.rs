//! Seeded gradient noise (Perlin's improved noise) with fBm helpers.

use crate::math::{lerp, Rng};

pub struct Perlin {
    perm: [u8; 512],
}

#[inline]
fn fade(t: f64) -> f64 {
    t * t * t * (t * (t * 6.0 - 15.0) + 10.0)
}

#[inline]
fn grad2(h: u8, x: f64, y: f64) -> f64 {
    match h & 7 {
        0 => x + y,
        1 => -x + y,
        2 => x - y,
        3 => -x - y,
        4 => x,
        5 => -x,
        6 => y,
        _ => -y,
    }
}

#[inline]
fn grad3(h: u8, x: f64, y: f64, z: f64) -> f64 {
    let h = h & 15;
    let u = if h < 8 { x } else { y };
    let v = if h < 4 {
        y
    } else if h == 12 || h == 14 {
        x
    } else {
        z
    };
    (if h & 1 == 0 { u } else { -u }) + (if h & 2 == 0 { v } else { -v })
}

impl Perlin {
    pub fn new(seed: u64) -> Self {
        let mut p = [0u8; 256];
        for (i, v) in p.iter_mut().enumerate() {
            *v = i as u8;
        }
        let mut rng = Rng::new(seed);
        for i in (1..256).rev() {
            let j = rng.below(i as u32 + 1) as usize;
            p.swap(i, j);
        }
        let mut perm = [0u8; 512];
        for (i, v) in perm.iter_mut().enumerate() {
            *v = p[i & 255];
        }
        Self { perm }
    }

    #[inline]
    fn p(&self, i: usize) -> usize {
        self.perm[i] as usize
    }

    /// Roughly in [-1, 1].
    pub fn noise2(&self, x: f64, y: f64) -> f64 {
        let (x0, y0) = (x.floor(), y.floor());
        let xi = (x0 as i64 & 255) as usize;
        let yi = (y0 as i64 & 255) as usize;
        let (xf, yf) = (x - x0, y - y0);
        let (u, v) = (fade(xf), fade(yf));
        let a = self.p(xi) + yi;
        let b = self.p(xi + 1) + yi;
        let (aa, ab, ba, bb) = (self.perm[a], self.perm[a + 1], self.perm[b], self.perm[b + 1]);
        lerp(
            v,
            lerp(u, grad2(aa, xf, yf), grad2(ba, xf - 1.0, yf)),
            lerp(u, grad2(ab, xf, yf - 1.0), grad2(bb, xf - 1.0, yf - 1.0)),
        )
    }

    /// Roughly in [-1, 1].
    pub fn noise3(&self, x: f64, y: f64, z: f64) -> f64 {
        let (x0, y0, z0) = (x.floor(), y.floor(), z.floor());
        let xi = (x0 as i64 & 255) as usize;
        let yi = (y0 as i64 & 255) as usize;
        let zi = (z0 as i64 & 255) as usize;
        let (xf, yf, zf) = (x - x0, y - y0, z - z0);
        let (u, v, w) = (fade(xf), fade(yf), fade(zf));
        let a = self.p(xi) + yi;
        let aa = self.p(a) + zi;
        let ab = self.p(a + 1) + zi;
        let b = self.p(xi + 1) + yi;
        let ba = self.p(b) + zi;
        let bb = self.p(b + 1) + zi;
        let pm = &self.perm;
        lerp(
            w,
            lerp(
                v,
                lerp(u, grad3(pm[aa], xf, yf, zf), grad3(pm[ba], xf - 1.0, yf, zf)),
                lerp(u, grad3(pm[ab], xf, yf - 1.0, zf), grad3(pm[bb], xf - 1.0, yf - 1.0, zf)),
            ),
            lerp(
                v,
                lerp(u, grad3(pm[aa + 1], xf, yf, zf - 1.0), grad3(pm[ba + 1], xf - 1.0, yf, zf - 1.0)),
                lerp(u, grad3(pm[ab + 1], xf, yf - 1.0, zf - 1.0), grad3(pm[bb + 1], xf - 1.0, yf - 1.0, zf - 1.0)),
            ),
        )
    }

    /// Fractal Brownian motion, normalised by total amplitude.
    pub fn fbm2(&self, x: f64, y: f64, octaves: u32) -> f64 {
        let (mut sum, mut amp, mut freq, mut norm) = (0.0, 1.0, 1.0, 0.0);
        for o in 0..octaves {
            // Offset each octave so they don't all share a lattice point at the origin.
            let off = o as f64 * 17.31;
            sum += amp * self.noise2(x * freq + off, y * freq - off);
            norm += amp;
            amp *= 0.5;
            freq *= 2.0;
        }
        sum / norm
    }
}

#[cfg(test)]
mod tests;
