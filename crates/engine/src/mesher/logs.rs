//! Logs as round trunks: each log block becomes a `SIDES`-sided bark cylinder in the shaped list (`shaped.rs`) instead
//! of a cube. Its axis follows the logs beside it (`axis_of`): upright unless it only has logs beside it on one
//! horizontal axis, as a dead tree's stub does. An end gets a cap of end grain only where it shows: not against a log
//! running on, not against a cube that hides it. A stub running into a trunk reaches half a block into it, out of
//! sight. An upright log standing on the ground flares out at its foot.
//! Invariants: logs hide no neighbour's faces (`HIDES` in the mesher), so the ground shows around a trunk; light and
//! shade are still those of an opaque block. To give a new log block its own thickness: a row in `radius`.

use super::shaped::{mean_light, pack, Shaped, Vert, HIDES, KIND_END, KIND_SIDE};
use super::{pidx, N, P, STRIDE};
use crate::block::{is_log, BlockId, BIRCH_LOG, FACE_TEX, OPAQUE, SNAG_LOG};

/// Sides of a trunk's cross-section.
pub const SIDES: usize = 12;
/// (cos, sin) round the cross-section, a side apart: a table, not `sin_cos` per vertex.
const H: f32 = 0.866_025_4; // √3 / 2
const RING: [(f32, f32); SIDES] = [
    (1.0, 0.0),
    (H, 0.5),
    (0.5, H),
    (0.0, 1.0),
    (-0.5, H),
    (-H, 0.5),
    (-1.0, 0.0),
    (-H, -0.5),
    (-0.5, -H),
    (0.0, -1.0),
    (0.5, -H),
    (H, -0.5),
];
/// How much wider than the trunk its foot is, and how high the flare reaches.
const FLARE: f32 = 1.35;
const FLARE_HEIGHT: f32 = 0.3;

/// The trunk's radius for each log block, in blocks.
fn radius(b: BlockId) -> f32 {
    match b {
        BIRCH_LOG => 0.3,
        SNAG_LOG => 0.35,
        _ => 0.4,
    }
}

/// The axis (0 x, 1 y, 2 z) the log at padded index `p` runs along.
/// A neighbour outside the padded volume counts as no log.
pub fn axis_of(pad: &[BlockId], p: usize) -> usize {
    let at = [p % P, p / (P * P), p / P % P];
    let log = |axis: usize, s: isize| {
        let c = at[axis] as isize + s;
        (0..P as isize).contains(&c) && is_log(pad[(p as isize + STRIDE[axis] * s) as usize])
    };
    let along = |axis: usize| log(axis, 1) || log(axis, -1);
    if along(1) {
        1
    } else if along(0) {
        0
    } else if along(2) {
        2
    } else {
        1
    }
}

/// Every log in the chunk as cylinders, into `out`'s opaque list.
pub fn mesh_logs(pad: &[BlockId], light: &[u8], out: &mut Shaped) {
    for y in 0..N {
        for z in 0..N {
            for x in 0..N {
                let p = pidx(x + 1, y + 1, z + 1);
                if is_log(pad[p]) {
                    trunk(pad, light, p, [x as i32, y as i32, z as i32], out);
                }
            }
        }
    }
}

/// The two axes across `a`, ordered so that (p × q) = a.
fn across(a: usize) -> (usize, usize) {
    match a {
        0 => (1, 2),
        1 => (2, 0),
        _ => (0, 1),
    }
}

#[inline(never)]
fn trunk(pad: &[BlockId], light: &[u8], p: usize, cell: [i32; 3], out: &mut Shaped) {
    let b = pad[p];
    let a = axis_of(pad, p);
    let (pa, qa) = across(a);
    let at = |axis: usize, sign: isize| (p as isize + STRIDE[axis] * sign) as usize;
    let hides = |i: usize| HIDES[pad[i] as usize];
    // Each end: how far it reaches (0 or 1, or half a block more into a log of another axis) and whether it is capped.
    let end = |sign: isize| {
        let q = at(a, sign);
        if is_log(pad[q]) {
            let on = axis_of(pad, q) == a;
            (if on { 0.0 } else { 0.5 }, false)
        } else {
            (0.0, !hides(q))
        }
    };
    let (lo_ext, lo_cap) = end(-1);
    let (hi_ext, hi_cap) = end(1);
    let r = radius(b);
    let open_side = [(pa, 1), (pa, -1), (qa, 1), (qa, -1)].iter().any(|&(ax, s)| !hides(at(ax, s)));
    let below = pad[at(1, -1)];
    let flare = a == 1 && OPAQUE[below as usize] && !is_log(below);
    // Rings along the axis: (position, radius).
    let all = [(-lo_ext, r * FLARE), (FLARE_HEIGHT, r), (1.0 + hi_ext, r)];
    let one = [(-lo_ext, r), (1.0 + hi_ext, r)];
    let rings: &[(f32, f32)] = if flare { &all } else { &one };
    let tiles = (std::f32::consts::TAU * r).round().max(1.0);
    let side_layer = FACE_TEX[b as usize][0];
    let end_layer = FACE_TEX[b as usize][2];
    let f = Frame {
        cell,
        axes: [a, pa, qa],
        tiles,
        // The four cells beside the log (+p, -p, +q, -q) and their light where open.
        open: [(pa, 1), (pa, -1), (qa, 1), (qa, -1)].map(|(ax, s)| {
            let i = at(ax, s);
            (!OPAQUE[pad[i] as usize]).then(|| light[i])
        }),
        fallback: fallback(pad, light, p),
    };
    // Rings are packed once each, a vertex at a time in `Frame` (opt-level 3 unrolls these loops: whatever their
    // bodies inline is copied into the wasm a dozen times).
    let mut ring = [[[0u32; 3]; SIDES + 1]; 2];
    if open_side {
        for w in rings.windows(2) {
            let slope = (w[0].1 - w[1].1) / (w[1].0 - w[0].0);
            let foot = flare && w[0].0 == rings[0].0;
            for (e, &(t, rad)) in w.iter().enumerate() {
                let occlusion = if foot && e == 0 { 4 } else { 7 };
                for (i, slot) in ring[e].iter_mut().enumerate() {
                    *slot = f.side(t, rad, i, slope, occlusion);
                }
            }
            let [low, high] = &ring;
            for i in 0..SIDES {
                out.packed(false, [low[i], low[i + 1], high[i + 1], high[i]], side_layer);
            }
        }
    }
    for (cap, sign, (t, rad)) in [(lo_cap, -1isize, rings[0]), (hi_cap, 1, rings[rings.len() - 1])] {
        if !cap {
            continue;
        }
        let l = if OPAQUE[pad[at(a, sign)] as usize] { f.fallback } else { light[at(a, sign)] };
        // The rim's vertices, then the centre (radius 0) in the last slot.
        let fan = &mut ring[0];
        for (i, slot) in fan.iter_mut().enumerate() {
            *slot = f.end(t, if i == SIDES { 0.0 } else { rad }, i, sign as f32, l);
        }
        for i in (0..SIDES).step_by(2) {
            let [c, r0, r1, r2] = [fan[SIDES], fan[i], fan[i + 1], fan[(i + 2) % SIDES]];
            // Seen from outside the end, the rim runs counter-clockwise on the high end and clockwise on the low one.
            out.packed(false, if sign > 0 { [c, r0, r1, r2] } else { [c, r2, r1, r0] }, end_layer);
        }
    }
}

/// One log's axes (along, then the two across) and surroundings, for building its vertices.
struct Frame {
    cell: [i32; 3],
    axes: [usize; 3],
    tiles: f32,
    open: [Option<u8>; 4],
    fallback: u8,
}

impl Frame {
    /// Ring vertex `i` at `t` along the axis and `rad` out: its offset, and the cosine and sine of its angle.
    fn point(&self, t: f32, rad: f32, i: usize) -> ([f32; 3], f32, f32) {
        let [a, pa, qa] = self.axes;
        let (c, s) = RING[i % SIDES];
        let mut off = [0.5f32; 3];
        off[a] = t;
        off[pa] += rad * c;
        off[qa] += rad * s;
        (off, c, s)
    }

    /// A side vertex, packed. Its normal leans along the axis by how fast the cone narrows (`slope`); its light is the
    /// mean of the open cells it faces.
    #[inline(never)]
    fn side(&self, t: f32, rad: f32, i: usize, slope: f32, occlusion: u32) -> [u32; 3] {
        let [a, pa, qa] = self.axes;
        let (off, c, s) = self.point(t, rad, i);
        let k = 1.0 / (1.0 + slope * slope).sqrt();
        let mut n = [0f32; 3];
        n[pa] = c * k;
        n[qa] = s * k;
        n[a] = slope * k;
        let facing = [c * k, -c * k, s * k, -s * k];
        let lit = (0..4).filter(|&j| facing[j] > 0.25).filter_map(|j| self.open[j]);
        pack(&Vert {
            cell: self.cell,
            off,
            normal: n,
            u: self.tiles * i as f32 / SIDES as f32,
            kind: KIND_SIDE + a as u32,
            occlusion,
            light: mean_light(lit).unwrap_or(self.fallback),
            layer: 0,
        })
    }

    /// An end vertex facing `sign` along the axis, packed.
    #[inline(never)]
    fn end(&self, t: f32, rad: f32, i: usize, sign: f32, light: u8) -> [u32; 3] {
        let a = self.axes[0];
        let mut normal = [0f32; 3];
        normal[a] = sign;
        let off = self.point(t, rad, i).0;
        pack(&Vert { cell: self.cell, off, normal, u: 0.0, kind: KIND_END + a as u32, occlusion: 7, light, layer: 0 })
    }
}

#[cfg(test)]
mod tests;

/// The brightest open cell beside the log: for the faces whose own neighbour is solid.
fn fallback(pad: &[BlockId], light: &[u8], p: usize) -> u8 {
    let mut best = 0u8;
    for stride in STRIDE {
        for s in [-1isize, 1] {
            let i = (p as isize + stride * s) as usize;
            if !OPAQUE[pad[i] as usize] && (light[i] & 15).max(light[i] >> 4) > (best & 15).max(best >> 4) {
                best = light[i];
            }
        }
    }
    best
}
