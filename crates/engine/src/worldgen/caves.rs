//! Spaghetti caves for every generator version: tunnels where two noise fields are both near zero,
//! sampled on a 4-block lattice. `generate` asks `is_cave` per block below the surface; version 3 also
//! skips cells near water (`water.rs`). Changing anything here changes released versions: don't.

use super::WorldGen;
use crate::math::IVec3;
/// Spaghetti caves: tunnels where two independent noise fields are both near zero.
/// Noise is sampled on a 4-block lattice and trilinearly interpolated (~45x fewer noise calls).
pub(super) struct CaveField {
    a: [f32; 729],
    b: [f32; 729],
}

impl CaveField {
    pub fn new(g: &WorldGen, base: IVec3) -> Self {
        let mut f = CaveField { a: [0.0; 729], b: [0.0; 729] };
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
        f
    }

    #[inline]
    pub fn is_cave(&self, x: usize, y: usize, z: usize) -> bool {
        let (ix, iy, iz) = (x >> 2, y >> 2, z >> 2);
        let (tx, ty, tz) = ((x & 3) as f32 * 0.25, (y & 3) as f32 * 0.25, (z & 3) as f32 * 0.25);
        let s = |f: &[f32; 729]| {
            let at = |dx: usize, dy: usize, dz: usize| f[((iy + dy) * 9 + iz + dz) * 9 + ix + dx];
            let l = |a: f32, b: f32, t: f32| a + (b - a) * t;
            let c00 = l(at(0, 0, 0), at(1, 0, 0), tx);
            let c10 = l(at(0, 1, 0), at(1, 1, 0), tx);
            let c01 = l(at(0, 0, 1), at(1, 0, 1), tx);
            let c11 = l(at(0, 1, 1), at(1, 1, 1), tx);
            l(l(c00, c10, ty), l(c01, c11, ty), tz)
        };
        let (a, b) = (s(&self.a), s(&self.b));
        a * a + b * b < 0.0045
    }
}
