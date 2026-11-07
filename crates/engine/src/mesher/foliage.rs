//! Canopies as one rounded mass: the leaf blocks around a chunk become a smooth surface (constrained surface nets) in
//! the shaped list (`shaped.rs`), instead of a cube per block. The leaves (and a log inside them, so no tunnel shows
//! round a trunk) are "inside". One vertex sits in each cell of the dual grid whose eight corners are not all alike,
//! at the mean of where its edges cross a softened field; a few smoothing passes that keep the volume (Taubin's λ|μ)
//! then round it, each vertex staying in its own dual cell so the shape never breaks. One quad faces out across each
//! edge between a leaf and an empty cell. Normals come from the slope of the softened field: soft, outward light.
//!
//! Three layers: a cutout shell textured with the leaves (light shows through its gaps), a sparser fringe `FRINGE`
//! outside it (a ragged edge, and depth as the two shift past each other) and, `INSET` behind it, an opaque darker
//! hull, so through the gaps one sees shaded leaves rather than the sky.
//!
//! Invariants: a chunk writes the quads of the edges whose lower cell is its own, so each quad is written once.
//! Every number is computed from the blocks within `REACH` of its vertex, relative to its cell and in the same order
//! in any chunk, so two chunks meet without a crack. An edit can change the canopy `REACH` blocks away (the world
//! remeshes that far: `world/mod.rs`). The finished vertex (normal, light, clumps, its three layers): `shell.rs`.

use super::shaped::{pack, Shaped};
use super::{pick_layer, N};
use crate::block::{is_leaves, is_log, BlockId, AIR};
use crate::chunk::{index, Chunk};

mod shell;

/// How far, in blocks, the blocks around a chunk shape its canopy.
/// (The smoothing passes, plus the softening's three, plus the chunk's own ring of dual cells.)
pub const REACH: usize = 7;
/// The smoothing passes: λ shrinks, μ swells back (Taubin), so a canopy rounds without shrinking.
const PASSES: [f32; 4] = [0.6, -0.62, 0.6, -0.62];

const G: usize = N + 2 * REACH;
const G2: usize = G * G;
const R: i32 = REACH as i32;
/// Dual cells (by their lowest corner, in grid cells) that are smoothed: the chunk's own, and `PASSES.len()` around.
const DUAL_LO: i32 = R - 1 - PASSES.len() as i32;
const DUAL_HI: i32 = R + N as i32 + PASSES.len() as i32; // exclusive

#[inline]
const fn gidx(x: i32, y: i32, z: i32) -> usize {
    ((y as usize * G) + z as usize) * G + x as usize
}

const AXES: [[i32; 3]; 3] = [[1, 0, 0], [0, 1, 0], [0, 0, 1]];

pub struct Foliage {
    /// The world block at the chunk's lowest corner (`Mesher::origin`): where the clumps are.
    pub origin: [i32; 3],
    /// The grid around the chunk (`REACH` blocks of border): the leaf block an inside cell is textured with, else air.
    leaf: Vec<BlockId>,
    /// 1 inside, 0 outside.
    occ: Vec<u8>,
    /// Inside cells in the 3×3×3 box around each cell (0..=27), and a scratch row for computing it.
    blur: Vec<u8>,
    scratch: Vec<u8>,
    /// Per dual cell: its vertex + 1, or 0.
    vert: Vec<u32>,
    cells: Vec<[i32; 3]>,
    off: Vec<[f32; 3]>,
    next: Vec<[f32; 3]>,
    links: Vec<[u32; 6]>,
    shells: Vec<Option<[[u32; 3]; 3]>>,
    /// Logs in the grid (index, and once worn the leaf block in the top byte).
    logs: Vec<u32>,
}

impl Default for Foliage {
    fn default() -> Self {
        Self {
            origin: [0; 3],
            leaf: vec![AIR; G * G2],
            occ: vec![0; G * G2],
            blur: vec![0; G * G2],
            scratch: vec![0; G * G2],
            vert: vec![0; G * G2],
            cells: Vec::new(),
            off: Vec::new(),
            next: Vec::new(),
            links: Vec::new(),
            shells: Vec::new(),
            logs: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests;

impl Foliage {
    /// Meshes the canopy of the centre chunk of `n` (`super::neighbor_index` order) into `out`. `pad` and `light` are
    /// the mesher's padded blocks and light.
    pub fn mesh(&mut self, n: &[&Chunk; 27], pad: &[BlockId], light: &[u8], out: &mut Shaped) {
        if !pad.iter().any(|&b| is_leaves(b)) {
            return;
        }
        self.fill(n);
        let Some((lo, hi)) = self.inside() else { return };
        self.soften(lo, hi);
        self.place(lo, hi);
        for &step in &PASSES {
            self.smooth(step);
        }
        self.emit(pad, light, (lo, hi), out);
    }

    /// The grid's blocks from the 27 chunks.
    #[inline(never)]
    fn fill(&mut self, n: &[&Chunk; 27]) {
        let split = |g: usize| {
            let v = g as i32 - R;
            ((v.div_euclid(N as i32) + 1) as usize, v.rem_euclid(N as i32) as usize)
        };
        for gy in 0..G {
            let (cy, ly) = split(gy);
            for gz in 0..G {
                let (cz, lz) = split(gz);
                let row = (gy * G + gz) * G;
                for (cx, from, to, lx) in [(0, 0, REACH, N - REACH), (1, REACH, REACH + N, 0), (2, REACH + N, G, 0)] {
                    let c = n[cx + cy * 3 + cz * 9];
                    let dst = &mut self.leaf[row + from..row + to];
                    match c.dense() {
                        Some(d) => {
                            let s = index(lx, ly, lz);
                            dst.copy_from_slice(&d[s..s + (to - from)]);
                        }
                        None => dst.fill(c.get(0, 0, 0)),
                    }
                }
            }
        }
    }

    /// Marks the inside cells (and gives a log among leaves a leaf block to wear); their bounds, if any.
    #[inline(never)]
    fn inside(&mut self) -> Option<([i32; 3], [i32; 3])> {
        let (mut lo, mut hi) = ([G as i32; 3], [-1i32; 3]);
        let mut grow = |i: usize| {
            let (x, y, z) = ((i % G) as i32, (i / G2) as i32, (i / G % G) as i32);
            lo = [lo[0].min(x), lo[1].min(y), lo[2].min(z)];
            hi = [hi[0].max(x), hi[1].max(y), hi[2].max(z)];
        };
        self.logs.clear();
        for (i, (&b, o)) in self.leaf.iter().zip(self.occ.iter_mut()).enumerate() {
            *o = u8::from(is_leaves(b));
            if *o == 1 {
                grow(i);
            } else if is_log(b) {
                self.logs.push(i as u32);
            }
        }
        // A log among leaves wears the first leaf beside it; written back only after every log has looked.
        let steps = [1isize, -1, G as isize, -(G as isize), G2 as isize, -(G2 as isize)];
        let mut worn = std::mem::take(&mut self.logs);
        worn.retain_mut(|i| {
            let p = *i as usize;
            let (x, y, z) = (p % G, p / G2, p / G % G);
            if x == 0 || y == 0 || z == 0 || x == G - 1 || y == G - 1 || z == G - 1 {
                return false;
            }
            let Some(leaf) = steps.iter().map(|&d| self.leaf[(p as isize + d) as usize]).find(|&b| is_leaves(b)) else {
                return false;
            };
            *i = p as u32 | u32::from(leaf) << 24;
            true
        });
        for &w in &worn {
            let p = (w & 0xFF_FFFF) as usize;
            self.leaf[p] = (w >> 24) as BlockId;
            self.occ[p] = 1;
            grow(p);
        }
        self.logs = worn;
        (hi[0] >= 0).then_some((lo, hi))
    }

    /// `blur` = inside cells in the 3×3×3 box around each cell, within the inside cells' bounds and one more.
    #[inline(never)]
    fn soften(&mut self, lo: [i32; 3], hi: [i32; 3]) {
        let lo = lo.map(|v| (v - 1).max(1));
        let hi = hi.map(|v| (v + 1).min(G as i32 - 2));
        self.blur.fill(0);
        self.scratch.fill(0);
        // Three passes of three along x, then z, then y: occ → blur → scratch → blur.
        let pass = |src: &[u8], dst: &mut [u8], d: usize, lo: [i32; 3], hi: [i32; 3]| {
            for y in lo[1] - 1..=hi[1] + 1 {
                for z in lo[2] - 1..=hi[2] + 1 {
                    for x in lo[0] - 1..=hi[0] + 1 {
                        let (x, y, z) = (x.clamp(1, G as i32 - 2), y.clamp(1, G as i32 - 2), z.clamp(1, G as i32 - 2));
                        let i = gidx(x, y, z);
                        dst[i] = src[i - d] + src[i] + src[i + d];
                    }
                }
            }
        };
        pass(&self.occ, &mut self.blur, 1, lo, hi);
        pass(&self.blur, &mut self.scratch, G, lo, hi);
        pass(&self.scratch, &mut self.blur, G2, lo, hi);
    }

    /// The softened field at grid cell `i`: inside cells above one half, outside below, by how full their box is.
    #[inline]
    fn field(&self, i: usize) -> f32 {
        0.5 * f32::from(self.occ[i]) + f32::from(self.blur[i]) / 54.0
    }

    /// One vertex per dual cell whose corners differ, at the mean of its edges' crossings; then who links to whom.
    #[inline(never)]
    fn place(&mut self, lo: [i32; 3], hi: [i32; 3]) {
        for &[x, y, z] in &self.cells {
            self.vert[gidx(x, y, z)] = 0;
        }
        self.cells.clear();
        self.off.clear();
        let from = lo.map(|v| (v - 1).max(DUAL_LO));
        let to = hi.map(|v| (v + 1).min(DUAL_HI));
        for y in from[1]..to[1] {
            for z in from[2]..to[2] {
                for x in from[0]..to[0] {
                    let base = gidx(x, y, z);
                    let corner = |k: usize| base + (k & 1) + (k >> 2 & 1) * G + (k >> 1 & 1) * G2;
                    let first = self.occ[corner(0)];
                    if (1..8).all(|k| self.occ[corner(k)] == first) {
                        continue;
                    }
                    let mut sum = [0f32; 3];
                    let mut count = 0.0;
                    // Corner k sits at (k & 1, k >> 1 & 1, k >> 2 & 1) in (x, y, z); its edges run to k ^ 1, k ^ 2, k ^ 4.
                    for k in 0..8usize {
                        for bit in [1usize, 2, 4] {
                            let j = k ^ bit;
                            if j < k || self.occ[corner(k)] == self.occ[corner(j)] {
                                continue;
                            }
                            let (fk, fj) = (self.field(corner(k)), self.field(corner(j)));
                            let t = (0.5 - fk) / (fj - fk);
                            let at = |c: usize| [(c & 1) as f32, (c >> 1 & 1) as f32, (c >> 2 & 1) as f32];
                            let (a, b) = (at(k), at(j));
                            for (s, (a, b)) in sum.iter_mut().zip(a.iter().zip(b)) {
                                *s += a + (b - a) * t;
                            }
                            count += 1.0;
                        }
                    }
                    self.cells.push([x, y, z]);
                    self.off.push(sum.map(|s| s / count));
                    self.vert[base] = self.cells.len() as u32;
                }
            }
        }
        // Links: the six face neighbours, in a fixed order, where the face between holds a crossing.
        self.links.clear();
        for &c in &self.cells {
            let mut l = [u32::MAX; 6];
            for (a, axis) in AXES.iter().enumerate() {
                for (s, sign) in [1i32, -1].into_iter().enumerate() {
                    let d = [c[0] + axis[0] * sign, c[1] + axis[1] * sign, c[2] + axis[2] * sign];
                    if (0..3).any(|i| d[i] < DUAL_LO || d[i] >= DUAL_HI) {
                        continue;
                    }
                    let v = self.vert[gidx(d[0], d[1], d[2])];
                    // The shared face's four corners: the higher cell's low face.
                    let f = if sign > 0 { d } else { c };
                    let (u, w) = (AXES[(a + 1) % 3], AXES[(a + 2) % 3]);
                    let at = |i: i32, j: i32| {
                        self.occ
                            [gidx(f[0] + u[0] * i + w[0] * j, f[1] + u[1] * i + w[1] * j, f[2] + u[2] * i + w[2] * j)]
                    };
                    let mixed = !(at(0, 0) == at(1, 0) && at(0, 0) == at(0, 1) && at(0, 0) == at(1, 1));
                    if v != 0 && mixed {
                        l[a * 2 + s] = v - 1;
                    }
                }
            }
            self.links.push(l);
        }
    }

    /// One smoothing pass: each vertex moves `step` of the way to its linked neighbours' mean, kept in its dual cell.
    #[inline(never)]
    fn smooth(&mut self, step: f32) {
        self.next.clear();
        for (i, l) in self.links.iter().enumerate() {
            let c = self.cells[i];
            let o = self.off[i];
            let (mut sum, mut n) = ([0f32; 3], 0.0);
            for &j in l {
                if j == u32::MAX {
                    continue;
                }
                let (cj, oj) = (self.cells[j as usize], self.off[j as usize]);
                for k in 0..3 {
                    sum[k] += oj[k] + (cj[k] - c[k]) as f32;
                }
                n += 1.0;
            }
            if n == 0.0 {
                self.next.push(o);
                continue;
            }
            self.next.push(std::array::from_fn(|k| (o[k] + step * (sum[k] / n - o[k])).clamp(0.02, 0.98)));
        }
        std::mem::swap(&mut self.off, &mut self.next);
    }

    /// A quad across every edge between an inside and an outside cell whose lower cell is the chunk's.
    #[inline(never)]
    fn emit(&mut self, pad: &[BlockId], light: &[u8], (lo, hi): ([i32; 3], [i32; 3]), out: &mut Shaped) {
        // Each vertex the chunk's quads use, worked out once (dual cells R - 1 .. R + N on every axis).
        let mut shells = std::mem::take(&mut self.shells);
        shells.clear();
        for &c in &self.cells {
            let own = c.iter().all(|&v| (R - 1..R + N as i32).contains(&v));
            shells.push(own.then(|| self.shell(pad, light, c).map(|v| pack(&v))));
        }
        let at = |c: [i32; 3]| shells[self.vert[gidx(c[0], c[1], c[2])] as usize - 1].expect("an owned vertex");
        // An edge with a side inside starts at most one cell before the inside cells' bounds.
        let from = lo.map(|v| (v - 1).max(R));
        let to = hi.map(|v| (v + 1).min(R + N as i32));
        for y in from[1]..to[1] {
            for z in from[2]..to[2] {
                for x in from[0]..to[0] {
                    let g = [x, y, z];
                    let here = self.occ[gidx(x, y, z)];
                    for a in 0..3 {
                        let e = AXES[a];
                        let there = self.occ[gidx(x + e[0], y + e[1], z + e[2])];
                        if here == there {
                            continue;
                        }
                        let (u, v) = (AXES[(a + 1) % 3], AXES[(a + 2) % 3]);
                        let d = |i: i32, j: i32| {
                            [g[0] - u[0] * i - v[0] * j, g[1] - u[1] * i - v[1] * j, g[2] - u[2] * i - v[2] * j]
                        };
                        // (u, v, a) is right-handed, so d(1,1) → d(0,1) → d(0,0) → d(1,0) runs counter-clockwise seen from +a.
                        let ring = [d(1, 1), d(0, 1), d(0, 0), d(1, 0)];
                        let (inside, face) =
                            if here == 1 { (g, a * 2) } else { ([x + e[0], y + e[1], z + e[2]], a * 2 + 1) };
                        let b = self.leaf[gidx(inside[0], inside[1], inside[2])];
                        let layer = pick_layer(b, face, inside.map(|v| (v - R + 1) as usize));
                        let mut shell = ring.map(at);
                        if here == 0 {
                            shell.reverse();
                        }
                        for (i, cutout) in [(0, true), (1, true), (2, false)] {
                            out.packed(cutout, shell.map(|s| s[i]), layer);
                        }
                    }
                }
            }
        }
        self.shells = shells;
    }
}
