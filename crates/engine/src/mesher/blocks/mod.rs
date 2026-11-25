//! Meshes shaped building blocks (slabs, vertical slabs, auto-corner stairs and 45° pitched slopes/roofs)
//! into the chunk's `Shaped` vertex stream (`mesher/shaped.rs`: opaque, cutout and translucent passes).
//!
//! Uses `KIND_PLANT` (`kind = 7`) explicit 1/60th UVs (`0..60`, where `0, 15, 30, 45, 60` are exact quarters)
//! so textures align upright on walls and run cleanly down pitched roof slopes. Runs only when the centre
//! chunk holds at least one shaped block (zero cost for unshaped chunks).

mod slope;

use crate::block::{BlockId, CUTOUT, FACE_TEX, OPAQUE, TRANSLUCENT};
use crate::chunk::Chunk;
use crate::math::IVec3;
use crate::shape::{self, corner_of, dir_face, full_face_mask, kind, to_local, Corner, Kind};

use super::shaped::{fixed, octahedral, Shaped, KIND_PLANT};
use super::{pidx, N, P, P2};

/// Fills `pad_shapes` from the 27 neighbour chunks' sparse shape lists; returns false (with zero work)
/// when none of the 27 chunks hold any shaped blocks.
pub(super) fn fill_pad_shapes(n: &[&Chunk; 27], pad_shapes: &mut [u8]) -> bool {
    if !n.iter().any(|c| c.has_shapes()) {
        return false;
    }
    pad_shapes.fill(0);
    for cz in 0..3usize {
        for cy in 0..3usize {
            for cx in 0..3usize {
                let chunk = n[cx + cy * 3 + cz * 9];
                if !chunk.has_shapes() {
                    continue;
                }
                for &(idx, s) in chunk.shapes() {
                    let lx = (idx & 31) as isize;
                    let lz = ((idx >> 5) & 31) as isize;
                    let ly = (idx >> 10) as isize;
                    let px = (cx as isize - 1) * N as isize + lx + 1;
                    let py = (cy as isize - 1) * N as isize + ly + 1;
                    let pz = (cz as isize - 1) * N as isize + lz + 1;
                    if (0..P as isize).contains(&px) && (0..P as isize).contains(&py) && (0..P as isize).contains(&pz) {
                        pad_shapes[pidx(px as usize, py as usize, pz as usize)] = s;
                    }
                }
            }
        }
    }
    true
}

/// Emits shaped building blocks in the centre chunk into `out`.
pub(super) fn mesh_shaped_blocks(
    pad: &[BlockId],
    shapes: &[u8],
    centre_paint: Option<&[u32]>,
    light: &[u8],
    out: &mut Shaped,
) {
    for y in 0..N {
        for z in 0..N {
            for x in 0..N {
                let p = pidx(x + 1, y + 1, z + 1);
                let s = shapes[p];
                if s == shape::FULL {
                    continue;
                }
                let b = pad[p];
                let pass = if TRANSLUCENT[b as usize] {
                    Pass::Translucent
                } else if CUTOUT[b as usize] {
                    Pass::Cutout
                } else {
                    Pass::Opaque
                };
                let paint = centre_paint.map_or(0, |cp| cp[crate::chunk::index(x, y, z)]);
                let cell = [x as i32, y as i32, z as i32];
                let corner = corner_of(s, IVec3::new(x as i32 + 1, y as i32 + 1, z as i32 + 1), |q| {
                    if q.x >= 0 && q.x < P as i32 && q.y >= 0 && q.y < P as i32 && q.z >= 0 && q.z < P as i32 {
                        shapes[pidx(q.x as usize, q.y as usize, q.z as usize)]
                    } else {
                        shape::FULL
                    }
                });
                let ctx = Ctx { pad, shapes, light, p, b, paint, cell, pass, out };
                match kind(s) {
                    Kind::Full => {}
                    Kind::Slab { quarters, top } => emit_slab(ctx, quarters, top),
                    Kind::VSlab { quarters, dir } => emit_vslab(ctx, quarters, dir),
                    Kind::Stair { dir, inverted } => emit_stair(ctx, dir, inverted, corner),
                    Kind::Slope { dir, inverted } => slope::emit_slope(ctx, dir, inverted, corner),
                }
            }
        }
    }
}

#[derive(Clone, Copy)]
enum Pass {
    Opaque,
    Cutout,
    Translucent,
}

pub(super) struct Ctx<'a> {
    pub(super) pad: &'a [BlockId],
    pub(super) shapes: &'a [u8],
    pub(super) light: &'a [u8],
    pub(super) p: usize,
    pub(super) b: BlockId,
    paint: u32,
    cell: [i32; 3],
    pass: Pass,
    out: &'a mut Shaped,
}

impl Ctx<'_> {
    /// Whether an exterior face `face` (`0..6`) is completely hidden by the adjacent cell.
    pub(super) fn hidden(&self, face: usize) -> bool {
        let q = neighbor_pidx(self.p, face);
        let nb = self.pad[q];
        let ns = self.shapes[q];
        if matches!(self.pass, Pass::Translucent) && nb == self.b && ns == shape::FULL {
            return true;
        }
        if !OPAQUE[nb as usize] || super::shaped::SHAPED[nb as usize] {
            return false;
        }
        if ns == shape::FULL {
            return true;
        }
        let opp = face ^ 1;
        full_face_mask(ns, Corner::OuterLeft) & (1 << opp) != 0
    }

    pub(super) fn face_light(&self, face: usize, on_boundary: bool) -> u8 {
        if on_boundary {
            self.light[neighbor_pidx(self.p, face)]
        } else {
            self.light[self.p]
        }
    }

    fn push_quad(&mut self, pts: [[f32; 3]; 4], uvs: [[f32; 2]; 4], normal: [f32; 3], face: usize, light: u8) {
        let paint = crate::colour::get_face(self.paint, face);
        let layer = (FACE_TEX[self.b as usize][face] & 0x7FF) | (u16::from(paint) << 11);
        let q = std::array::from_fn(|i| vert(self.cell, pts[i], normal, uvs[i], 7, light));
        match self.pass {
            Pass::Opaque => self.out.packed(false, q, layer),
            Pass::Cutout => self.out.packed(true, q, layer),
            Pass::Translucent => self.out.packed_translucent(q, layer),
        }
    }

    /// Emits a quad specified in canonical `(r, y, b)` space, rotated by `dir`.
    /// When `flip_y` is true, mirrors `y -> 1 - y`, negates `ny` and reverses winding (for inverted shapes).
    #[allow(clippy::too_many_arguments)]
    pub(super) fn quad(
        &mut self,
        dir: u8,
        flip_y: bool,
        ryb: [[f32; 3]; 4],
        uvs: [[f32; 2]; 4],
        n_ryb: [f32; 3],
        tex_face: usize,
        light: u8,
    ) {
        let n = rot_normal(dir, n_ryb[0], if flip_y { -n_ryb[1] } else { n_ryb[1] }, n_ryb[2]);
        let mut pts = ryb.map(|[r, y, b]| {
            let (x, z) = to_local(dir, r, b);
            [x, if flip_y { 1.0 - y } else { y }, z]
        });
        let mut uvs = uvs;
        if flip_y {
            pts.swap(1, 3);
            uvs.swap(1, 3);
        }
        self.push_quad(pts, uvs, n, tex_face, light);
    }

    /// Emits a triangle (encoded as a degenerate quad) in canonical `(r, y, b)` space.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn tri(
        &mut self,
        dir: u8,
        flip_y: bool,
        p: [[f32; 3]; 3],
        uv: [[f32; 2]; 3],
        n_ryb: [f32; 3],
        tex_face: usize,
        light: u8,
    ) {
        self.quad(dir, flip_y, [p[0], p[1], p[2], p[2]], [uv[0], uv[1], uv[2], uv[2]], n_ryb, tex_face, light);
    }

    /// Emits all visible faces of an axis-aligned box `[r0..r1, y0..y1, b0..b1]` in `(r, y, b)` space.
    /// `skip` bitmask (`0:front, 1:back, 2:right, 3:left, 4:top, 5:bottom`) omits internal touching faces.
    #[allow(clippy::too_many_arguments)]
    fn emit_box(&mut self, dir: u8, r0: f32, y0: f32, b0: f32, r1: f32, y1: f32, b1: f32, skip: u8) {
        let f_back = dir_face(dir) as usize;
        let f_front = f_back ^ 1;
        let f_right = dir_face((dir + 1) & 3) as usize;
        let f_left = f_right ^ 1;
        type BoxFace = (u8, bool, usize, [[f32; 3]; 4], [[f32; 2]; 4], [f32; 3]);
        let faces: [BoxFace; 6] = [
            (
                1,
                b0 == 0.0,
                f_front,
                [[r0, y0, b0], [r1, y0, b0], [r1, y1, b0], [r0, y1, b0]],
                [[r0, 1.0 - y0], [r1, 1.0 - y0], [r1, 1.0 - y1], [r0, 1.0 - y1]],
                [0.0, 0.0, -1.0],
            ),
            (
                2,
                b1 == 1.0,
                f_back,
                [[r1, y0, b1], [r0, y0, b1], [r0, y1, b1], [r1, y1, b1]],
                [[1.0 - r1, 1.0 - y0], [1.0 - r0, 1.0 - y0], [1.0 - r0, 1.0 - y1], [1.0 - r1, 1.0 - y1]],
                [0.0, 0.0, 1.0],
            ),
            (
                4,
                r1 == 1.0,
                f_right,
                [[r1, y0, b0], [r1, y0, b1], [r1, y1, b1], [r1, y1, b0]],
                [[b0, 1.0 - y0], [b1, 1.0 - y0], [b1, 1.0 - y1], [b0, 1.0 - y1]],
                [1.0, 0.0, 0.0],
            ),
            (
                8,
                r0 == 0.0,
                f_left,
                [[r0, y0, b1], [r0, y0, b0], [r0, y1, b0], [r0, y1, b1]],
                [[1.0 - b1, 1.0 - y0], [1.0 - b0, 1.0 - y0], [1.0 - b0, 1.0 - y1], [1.0 - b1, 1.0 - y1]],
                [-1.0, 0.0, 0.0],
            ),
            (
                16,
                y1 == 1.0,
                2,
                [[r0, y1, b0], [r1, y1, b0], [r1, y1, b1], [r0, y1, b1]],
                [[r0, 1.0 - b0], [r1, 1.0 - b0], [r1, 1.0 - b1], [r0, 1.0 - b1]],
                [0.0, 1.0, 0.0],
            ),
            (
                32,
                y0 == 0.0,
                3,
                [[r0, y0, b1], [r1, y0, b1], [r1, y0, b0], [r0, y0, b0]],
                [[r0, b1], [r1, b1], [r1, b0], [r0, b0]],
                [0.0, -1.0, 0.0],
            ),
        ];
        for (bit, boundary, face, ryb, uvs, n) in faces {
            if skip & bit == 0 && !(boundary && self.hidden(face)) {
                let l = self.face_light(face, boundary);
                self.quad(dir, false, ryb, uvs, n, face, l);
            }
        }
    }
}

fn emit_slab(mut ctx: Ctx<'_>, quarters: u8, top: bool) {
    let h = f32::from(quarters) * 0.25;
    let (y0, y1) = if top { (1.0 - h, 1.0) } else { (0.0, h) };
    ctx.emit_box(0, 0.0, y0, 0.0, 1.0, y1, 1.0, 0);
}

fn emit_vslab(mut ctx: Ctx<'_>, quarters: u8, dir: u8) {
    let t = f32::from(quarters) * 0.25;
    ctx.emit_box(dir, 0.0, 0.0, 1.0 - t, 1.0, 1.0, 1.0, 0);
}

/// Emits any of the 10 stair variants (5 corners × upright/inverted) as 4 non-overlapping quadrants
/// with zero internal faces (clean even for translucent stained glass stairs).
fn emit_stair(mut ctx: Ctx<'_>, dir: u8, inverted: bool, corner: Corner) {
    let tall = match corner {
        Corner::Straight => [[false, true], [false, true]],
        Corner::OuterLeft => [[false, true], [false, false]],
        Corner::OuterRight => [[false, false], [false, true]],
        Corner::InnerLeft => [[true, true], [false, true]],
        Corner::InnerRight => [[false, true], [true, true]],
    };
    for ir in 0..2 {
        for ib in 0..2 {
            let (r0, b0) = (ir as f32 * 0.5, ib as f32 * 0.5);
            let (r1, b1) = (r0 + 0.5, b0 + 0.5);
            let is_tall = tall[ir][ib];
            let (y0, y1) = match (inverted, is_tall) {
                (false, false) => (0.0, 0.5),
                (true, false) => (0.5, 1.0),
                (_, true) => (0.0, 1.0),
            };
            let skip =
                (u8::from(ib == 1)) | (u8::from(ib == 0) << 1) | (u8::from(ir == 0) << 2) | (u8::from(ir == 1) << 3);
            ctx.emit_box(dir, r0, y0, b0, r1, y1, b1, skip);
            if is_tall {
                let (ry0, ry1) = if inverted { (0.0, 0.5) } else { (0.5, 1.0) };
                if ib == 1 && !tall[ir][0] {
                    ctx.emit_box(dir, r0, ry0, b0, r1, ry1, b1, !1);
                }
                if ib == 0 && !tall[ir][1] {
                    ctx.emit_box(dir, r0, ry0, b0, r1, ry1, b1, !2);
                }
                if ir == 0 && !tall[1][ib] {
                    ctx.emit_box(dir, r0, ry0, b0, r1, ry1, b1, !4);
                }
                if ir == 1 && !tall[0][ib] {
                    ctx.emit_box(dir, r0, ry0, b0, r1, ry1, b1, !8);
                }
            }
        }
    }
}

fn vert(cell: [i32; 3], off: [f32; 3], normal: [f32; 3], uv: [f32; 2], occlusion: u32, light: u8) -> [u32; 3] {
    let [x, y, z] = [fixed(cell[0], off[0]), fixed(cell[1], off[1]), fixed(cell[2], off[2])];
    let u = (uv[0].clamp(0.0, 1.0) * 60.0).round_ties_even() as u32;
    let v = (uv[1].clamp(0.0, 1.0) * 60.0).round_ties_even() as u32;
    let packed_uv = (u & 63) | ((v & 63) << 6);
    [
        x | y << 12 | KIND_PLANT << 24 | occlusion.min(7) << 27,
        z | packed_uv << 12 | u32::from(light) << 24,
        octahedral(normal),
    ]
}

#[inline]
fn neighbor_pidx(p: usize, face: usize) -> usize {
    let stride: isize = match face {
        0 => 1,
        1 => -1,
        2 => P2 as isize,
        3 => -(P2 as isize),
        4 => P as isize,
        _ => -(P as isize),
    };
    (p as isize + stride) as usize
}

#[inline]
fn rot_normal(dir: u8, nr: f32, ny: f32, nb: f32) -> [f32; 3] {
    match dir & 3 {
        0 => [nr, ny, -nb],
        1 => [nb, ny, nr],
        2 => [-nr, ny, nb],
        _ => [-nb, ny, -nr],
    }
}
