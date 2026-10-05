//! Spaghetti caves: tunnels where two noise fields are both near zero, sampled on a 4-block lattice. `generate`
//! asks `is_cave` per block below the surface; version 3 also skips cells near water (`water.rs`). Changing
//! the numbers of a released version changes its worlds: don't.
//!
//! Version 6 made caves rare: they exist only inside *cave zones* (a slow noise on a 192-block scale, wide
//! enough that a zone holds a cave system rather than one tunnel), and the tunnels are thinner. A chunk whose
//! lattice lies wholly outside the zones skips the two tunnel fields, which also makes generation faster.

use super::WorldGen;
use crate::math::IVec3;

/// How near zero both fields must be, squared (versions 1 to 5, and 6's thinner tunnels).
const TUNNEL: f32 = 0.0045;
const TUNNEL_V6: f32 = 0.0018;
/// Version 6: the zone noise must exceed this for caves to exist (about a quarter of the ground).
const ZONE_MIN: f32 = 0.2;

/// Spaghetti caves: tunnels where two independent noise fields are both near zero.
/// Noise is sampled on a 4-block lattice and trilinearly interpolated (~45x fewer noise calls).
pub(super) struct CaveField {
    a: [f32; 729],
    b: [f32; 729],
    /// Version 6: the zone noise at the lattice's columns (x then z), `None` for the older versions.
    zone: Option<[f32; 81]>,
    limit: f32,
}

impl CaveField {
    /// The cave field of the chunk at `base`; `None` when version 6 puts no cave anywhere in it.
    pub fn new(g: &WorldGen, base: IVec3) -> Option<Self> {
        let zone = (g.version >= 6).then(|| {
            let mut z = [0.0f32; 81];
            for iz in 0..9 {
                for ix in 0..9 {
                    let (x, zz) = ((base.x + ix * 4) as f64, (base.z + iz * 4) as f64);
                    z[(iz * 9 + ix) as usize] = g.cave_zone.noise2(x / 192.0, zz / 192.0) as f32;
                }
            }
            z
        });
        if zone.as_ref().is_some_and(|z| z.iter().all(|&v| v < ZONE_MIN - 0.05)) {
            return None;
        }
        let limit = if g.version >= 6 { TUNNEL_V6 } else { TUNNEL };
        let mut f = CaveField { a: [0.0; 729], b: [0.0; 729], zone, limit };
        for iy in 0..9 {
            for iz in 0..9 {
                for ix in 0..9 {
                    let x = (base.x + ix * 4) as f64;
                    let y = (base.y + iy * 4) as f64;
                    let z = (base.z + iz * 4) as f64;
                    let i = ((iy * 9 + iz) * 9 + ix) as usize;
                    f.a[i] = g.cave_a.noise3(x / 52.0, y / 30.0, z / 52.0) as f32;
                    f.b[i] = g.cave_b.noise3(x / 52.0, y / 30.0, z / 52.0) as f32;
                }
            }
        }
        Some(f)
    }

    #[inline]
    pub fn is_cave(&self, x: usize, y: usize, z: usize) -> bool {
        let (ix, iy, iz) = (x >> 2, y >> 2, z >> 2);
        let (tx, ty, tz) = ((x & 3) as f32 * 0.25, (y & 3) as f32 * 0.25, (z & 3) as f32 * 0.25);
        let l = |a: f32, b: f32, t: f32| a + (b - a) * t;
        if let Some(zone) = &self.zone {
            let at = |dx: usize, dz: usize| zone[(iz + dz) * 9 + ix + dx];
            if l(l(at(0, 0), at(1, 0), tx), l(at(0, 1), at(1, 1), tx), tz) < ZONE_MIN {
                return false;
            }
        }
        let s = |f: &[f32; 729]| {
            let at = |dx: usize, dy: usize, dz: usize| f[((iy + dy) * 9 + iz + dz) * 9 + ix + dx];
            let c00 = l(at(0, 0, 0), at(1, 0, 0), tx);
            let c10 = l(at(0, 1, 0), at(1, 1, 0), tx);
            let c01 = l(at(0, 0, 1), at(1, 0, 1), tx);
            let c11 = l(at(0, 1, 1), at(1, 1, 1), tx);
            l(l(c00, c10, ty), l(c01, c11, ty), tz)
        };
        let (a, b) = (s(&self.a), s(&self.b));
        a * a + b * b < self.limit
    }
}

#[cfg(test)]
mod tests;
