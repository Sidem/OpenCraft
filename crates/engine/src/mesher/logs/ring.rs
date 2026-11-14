//! The wood's geometry: a stretch of bark between two rings (a cone when they differ), and a flat cut of end grain.
//! A ring's points come from its axis alone (`basis`, the same whichever way the stretch runs), so two stretches that
//! share a ring share its points exactly. To change how round thick wood is: `detail`.

use super::super::shaped::{pack, Shaped, Vert, KIND_END, KIND_SIDE};

/// The most sides a ring has; thinner wood takes every second or third of them (`detail`).
pub const MAX_SIDES: usize = 24;
const C15: f32 = 0.965_925_8;
const S15: f32 = 0.258_819;
const C30: f32 = 0.866_025_4;
const C45: f32 = std::f32::consts::FRAC_1_SQRT_2;
/// (cos, sin) a 24th of a turn apart: a table, not `sin_cos` per vertex.
const TURN: [(f32, f32); MAX_SIDES] = [
    (1.0, 0.0),
    (C15, S15),
    (C30, 0.5),
    (C45, C45),
    (0.5, C30),
    (S15, C15),
    (0.0, 1.0),
    (-S15, C15),
    (-0.5, C30),
    (-C45, C45),
    (-C30, 0.5),
    (-C15, S15),
    (-1.0, 0.0),
    (-C15, -S15),
    (-C30, -0.5),
    (-C45, -C45),
    (-0.5, -C30),
    (-S15, -C15),
    (0.0, -1.0),
    (S15, -C15),
    (0.5, -C30),
    (C45, -C45),
    (C30, -0.5),
    (C15, -S15),
];

/// Sides of wood `r` blocks thick, and how many times its bark tiles round it.
pub fn detail(r: f32) -> (usize, f32) {
    // Few sides: a forest holds thousands of logs, and the smooth normals round them off.
    let sides = if r >= 0.9 {
        24
    } else if r >= 0.5 {
        12
    } else if r >= 0.25 {
        8
    } else if r >= 0.12 {
        6
    } else {
        4
    };
    (sides, (std::f32::consts::TAU * r).round().max(1.0))
}

/// What every vertex of one node shares: its cell (chunk-local), light, shade and textures.
pub struct Ring {
    pub cell: [i32; 3],
    pub light: u8,
    pub occlusion: u32,
    pub side_layer: u16,
    pub end_layer: u16,
}

impl Ring {
    /// Bark from `p0` (radius `r0`) to `p1` (radius `r1`), chunk-local blocks.
    #[inline(never)]
    #[allow(clippy::too_many_arguments)]
    pub fn strip(&self, p0: [f32; 3], r0: f32, p1: [f32; 3], r1: f32, sides: usize, tiles: f32, out: &mut Shaped) {
        let d = [p1[0] - p0[0], p1[1] - p0[1], p1[2] - p0[2]];
        let len = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
        if len < 1e-4 {
            return;
        }
        let (a, axis, e1, e2) = basis(d.map(|v| v / len));
        // Low to high along the axis, so the quads face out.
        let (lo, hi) = if dot(axis, d) > 0.0 { ((p0, r0), (p1, r1)) } else { ((p1, r1), (p0, r0)) };
        let slope = (lo.1 - hi.1) / len;
        let k = 1.0 / (1.0 + slope * slope).sqrt();
        let step = MAX_SIDES / sides;
        let mut ring = [[[0u32; 3]; MAX_SIDES + 1]; 2];
        for (e, (p, r)) in [lo, hi].into_iter().enumerate() {
            for (i, slot) in ring[e].iter_mut().take(sides + 1).enumerate() {
                let (c, s) = TURN[(i * step) % MAX_SIDES];
                let radial: [f32; 3] = std::array::from_fn(|j| e1[j] * c + e2[j] * s);
                let pos = std::array::from_fn(|j| p[j] + radial[j] * r);
                let normal = std::array::from_fn(|j| radial[j] * k + axis[j] * slope * k);
                *slot = self.vert(pos, normal, tiles * i as f32 / sides as f32, KIND_SIDE + a as u32);
            }
        }
        let [low, high] = &ring;
        for i in 0..sides {
            out.packed(false, [low[i], low[i + 1], high[i + 1], high[i]], self.side_layer);
        }
    }

    /// A flat cut at `p` facing `facing` (a unit vector), `r` blocks across: end grain.
    #[inline(never)]
    pub fn cap(&self, p: [f32; 3], facing: [f32; 3], r: f32, sides: usize, out: &mut Shaped) {
        let (a, axis, e1, e2) = basis(facing);
        let outward = dot(axis, facing) > 0.0;
        let step = MAX_SIDES / sides;
        let mut fan = [[0u32; 3]; MAX_SIDES + 1];
        for (i, slot) in fan.iter_mut().take(sides).enumerate() {
            let (c, s) = TURN[(i * step) % MAX_SIDES];
            let pos = std::array::from_fn(|j| p[j] + (e1[j] * c + e2[j] * s) * r);
            *slot = self.vert(pos, facing, 0.0, KIND_END + a as u32);
        }
        fan[sides] = self.vert(p, facing, 0.0, KIND_END + a as u32);
        for i in (0..sides).step_by(2) {
            let [c, r0, r1, r2] = [fan[sides], fan[i], fan[i + 1], fan[(i + 2) % sides]];
            out.packed(false, if outward { [c, r0, r1, r2] } else { [c, r2, r1, r0] }, self.end_layer);
        }
    }

    fn vert(&self, pos: [f32; 3], normal: [f32; 3], u: f32, kind: u32) -> [u32; 3] {
        let off = std::array::from_fn(|j| pos[j] - self.cell[j] as f32);
        pack(&Vert { cell: self.cell, off, normal, u, kind, occlusion: self.occlusion, light: self.light, layer: 0 })
    }
}

/// For a direction: its main axis (0 x, 1 y, 2 z), the direction turned to point along that axis' positive side, and
/// two unit vectors across it with `e1 × e2` along it. The same for a direction and its opposite.
fn basis(d: [f32; 3]) -> (usize, [f32; 3], [f32; 3], [f32; 3]) {
    let a = if d[0].abs() >= d[1].abs() && d[0].abs() >= d[2].abs() {
        0
    } else if d[1].abs() >= d[2].abs() {
        1
    } else {
        2
    };
    let axis = if d[a] < 0.0 { d.map(|v| -v) } else { d };
    let up = if axis[1].abs() < 0.9 { [0.0, 1.0, 0.0] } else { [1.0, 0.0, 0.0] };
    let e1 = unit(cross(up, axis));
    let e2 = cross(axis, e1);
    (a, axis, e1, e2)
}

fn cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]]
}

fn dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn unit(a: [f32; 3]) -> [f32; 3] {
    let l = dot(a, a).sqrt().max(1e-6);
    a.map(|v| v / l)
}
