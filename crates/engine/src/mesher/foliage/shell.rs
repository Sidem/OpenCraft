//! One canopy vertex, finished: from its smoothed place in its dual cell to the three layers drawn there (the leafy
//! shell, the sparser fringe outside it, the dark hull inside), with its normal, light, shade and clumps.
//! Invariant: everything here is read from the grid around the vertex and the world cell it is in, so two chunks
//! finish a shared vertex alike. To change the canopy's look: the constants below.

use super::{gidx, Foliage, AXES, R};
use crate::block::{BlockId, OPAQUE};
use crate::mesher::shaped::{mean_light, Vert, KIND_CANOPY, KIND_FRINGE};
use crate::mesher::P;

/// How far behind the leafy shell its dark hull lies, in blocks; and how dark that hull is (occlusion out of 7).
const INSET: f32 = 0.28;
const HULL_SHADE: u32 = 2;
/// Clumps of leaves: the shell swells and sinks along its normal by up to this much, in lumps `CLUMP` blocks across,
/// fixed to the world so a canopy keeps its lumps and two chunks agree on them.
const CLUMP: i32 = 2;
const CLUMP_DEPTH: f32 = 0.8;
/// A second, sparser leafy layer this far outside the shell: the canopy's ragged edge and its depth (shell texturing).
const FRINGE: f32 = 0.3;

impl Foliage {
    /// The vertex for dual cell `c` (grid cells) as seen by the chunk: on its shell, its fringe and its hull.
    #[inline(never)]
    pub(super) fn shell(&self, pad: &[BlockId], light: &[u8], c: [i32; 3]) -> [Vert; 3] {
        let i = self.vert[gidx(c[0], c[1], c[2])] as usize - 1;
        let o = self.off[i];
        let n = self.normal(c, o);
        // The padded index of a corner (the grid is `REACH - 1` cells wider than the padded volume on each side).
        let p = |k: usize| {
            let at = |a: usize, bit: usize| (c[a] - R + 1) as usize + (k >> bit & 1);
            (at(1, 1) * P + at(2, 2)) * P + at(0, 0)
        };
        let open = (0..8).map(p).filter(|&q| !OPAQUE[pad[q] as usize]).map(|q| light[q]);
        let l = mean_light(open).unwrap_or(0);
        // Fuller boxes around: deeper in the canopy, darker.
        let w = |k: usize| -> f32 { (0..3).map(|a| if k >> a & 1 == 1 { o[a] } else { 1.0 - o[a] }).product() };
        let full: f32 = (0..8)
            .map(|k| {
                let q = [c[0] + (k & 1) as i32, c[1] + (k >> 1 & 1) as i32, c[2] + (k >> 2 & 1) as i32];
                w(k) * f32::from(self.blur[gidx(q[0], q[1], q[2])]) / 27.0
            })
            .sum();
        let occlusion = (7.0 * (2.0 * (1.0 - full)).clamp(0.0, 1.0)).round() as u32;
        let cell = [c[0] - R, c[1] - R, c[2] - R];
        let world: [i32; 3] = std::array::from_fn(|k| self.origin[k] + cell[k]);
        let (here, grad) = clump(world, o);
        let bulge = (here - 0.45) * CLUMP_DEPTH;
        // The lumps tilt the normal too, so they show in the light and not only at the edge.
        let slope = grad.map(|g| g * CLUMP_DEPTH);
        let along = slope[0] * n[0] + slope[1] * n[1] + slope[2] * n[2];
        let tilted: [f32; 3] = std::array::from_fn(|k| n[k] - (slope[k] - along * n[k]));
        let len = (tilted[0] * tilted[0] + tilted[1] * tilted[1] + tilted[2] * tilted[2]).sqrt();
        let outer = Vert {
            cell,
            off: std::array::from_fn(|k| o[k] + 0.5 + n[k] * bulge),
            normal: tilted.map(|v| v / len),
            u: 0.0,
            kind: KIND_CANOPY,
            light: l,
            layer: 0,
            // Between the lumps, a little shade.
            occlusion: ((occlusion as f32 * (0.55 + 0.9 * here)).round() as u32).min(7),
        };
        let inner = Vert {
            off: std::array::from_fn(|k| o[k] + 0.5 + n[k] * (bulge * 0.5 - INSET)),
            occlusion: occlusion * HULL_SHADE / 7,
            ..outer
        };
        let fringe = Vert {
            off: std::array::from_fn(|k| o[k] + 0.5 + n[k] * (bulge + FRINGE)),
            kind: KIND_FRINGE,
            occlusion: (outer.occlusion + 1).min(7),
            ..outer
        };
        [outer, fringe, inner]
    }

    /// The softened field's slope at a vertex: from the box counts' central differences at its cell's corners.
    #[inline(never)]
    fn normal(&self, c: [i32; 3], o: [f32; 3]) -> [f32; 3] {
        let mut g = [0f32; 3];
        for k in 0..8usize {
            let corner = [c[0] + (k & 1) as i32, c[1] + (k >> 1 & 1) as i32, c[2] + (k >> 2 & 1) as i32];
            let w: f32 = (0..3).map(|a| if k >> a & 1 == 1 { o[a] } else { 1.0 - o[a] }).product();
            for (a, axis) in AXES.iter().enumerate() {
                let at = |s: i32| {
                    f32::from(
                        self.blur[gidx(corner[0] + axis[0] * s, corner[1] + axis[1] * s, corner[2] + axis[2] * s)],
                    )
                };
                g[a] -= w * (at(1) - at(-1));
            }
        }
        let len = (g[0] * g[0] + g[1] * g[1] + g[2] * g[2]).sqrt();
        if len < 1e-4 {
            [0.0, 1.0, 0.0]
        } else {
            g.map(|v| v / len)
        }
    }
}

/// Smooth value noise in 0..1 at the point `off` (in blocks, about 0..1) of world cell `cell`, `CLUMP` blocks a lattice
/// cell, and its slope per block. The lattice cell and the fraction are worked out apart, so a far point is as exact
/// as a near one.
#[inline(never)]
fn clump(cell: [i32; 3], off: [f32; 3]) -> (f32, [f32; 3]) {
    let mut base = [0i32; 3];
    let (mut t, mut dt) = ([0f32; 3], [0f32; 3]);
    for k in 0..3 {
        base[k] = cell[k].div_euclid(CLUMP);
        let f = (cell[k].rem_euclid(CLUMP) as f32 + 0.5 + off[k]) / CLUMP as f32;
        let whole = f.floor();
        base[k] += whole as i32;
        let s = f - whole;
        t[k] = s * s * (3.0 - 2.0 * s);
        dt[k] = 6.0 * s * (1.0 - s) / CLUMP as f32;
    }
    let hash = |x: i32, y: i32, z: i32| {
        let h = (x as u32).wrapping_mul(0x8DA6_B343)
            ^ (y as u32).wrapping_mul(0xD816_3841)
            ^ (z as u32).wrapping_mul(0xCB1A_B31F);
        let h = (h ^ (h >> 15)).wrapping_mul(0x2C1B_3C6D);
        (h >> 8) as f32 / (1u32 << 24) as f32
    };
    let (mut v, mut g) = (0.0, [0f32; 3]);
    for k in 0..8usize {
        let h = hash(base[0] + (k & 1) as i32, base[1] + (k >> 1 & 1) as i32, base[2] + (k >> 2 & 1) as i32);
        let f: [f32; 3] = std::array::from_fn(|a| if k >> a & 1 == 1 { t[a] } else { 1.0 - t[a] });
        let sign = |a: usize| if k >> a & 1 == 1 { 1.0 } else { -1.0 };
        v += h * f[0] * f[1] * f[2];
        g[0] += h * sign(0) * dt[0] * f[1] * f[2];
        g[1] += h * f[0] * sign(1) * dt[1] * f[2];
        g[2] += h * f[0] * f[1] * sign(2) * dt[2];
    }
    (v, g)
}
