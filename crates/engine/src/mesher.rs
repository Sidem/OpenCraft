//! Greedy chunk mesher with per-vertex ambient occlusion.
//!
//! Output is one `u32` per vertex, 4 vertices per quad, consumed with a shared quad index buffer:
//!
//! ```text
//! bits  0..6   x  (0..=32, chunk-local)
//! bits  6..12  y
//! bits 12..18  z
//! bits 18..21  face  (+X, -X, +Y, -Y, +Z, -Z; 6 and 7 are a plant's two diagonal quads)
//! bits 21..23  ambient occlusion (0 = darkest, 3 = unoccluded)
//! bits 23..32  texture array layer
//! ```
//!
//! UVs are not stored: the shader derives them from the vertex position, so a merged 5x3 quad
//! simply tiles its texture five by three times (texture wrap = REPEAT).
//!
//! Each vertex also gets one light byte: smoothed sky light in the low nibble, block light in the
//! high one, averaged from the light field (`light.rs`) in the cells in front of the face around that
//! corner. They follow the vertices in `verts`, four per `u32` (see [`MeshOutput`]).
//!
//! Quads are merged only where AO and light are constant along the merge direction, so greedy
//! merging never changes how their gradients look. Layers with alternates (ores, leaves:
//! `block::tex::alternates`) get one of four looks per block (`pick_layer`), so those faces merge
//! less. Plants (`block::PLANT`) are two crossed quads, each emitted in both windings, into the
//! cutout list and never merged.

use crate::block::{BlockId, AIR, ALT_TEX, CUTOUT, FACE_TEX, MESHED, OPAQUE, PLANT};
use crate::chunk::{index, Chunk};

const N: usize = 32;
const P: usize = N + 2;
const P2: usize = P * P;
/// The padded volume (the chunk and a one-block border) per axis, and in all, as `light.rs` writes it.
pub const PAD: usize = P;
pub const PAD_VOLUME: usize = P * P2;

#[inline]
const fn pidx(x: usize, y: usize, z: usize) -> usize {
    (y * P + z) * P + x
}

/// Index strides in the padded volume for x, y, z.
const STRIDE: [isize; 3] = [1, P2 as isize, P as isize];

/// Per face: (normal axis, normal sign, u axis, v axis), with u × v = normal so that
/// corners (0,0) (1,0) (1,1) (0,1) wind counter-clockwise seen from outside.
const FACES: [(usize, isize, usize, usize); 6] =
    [(0, 1, 1, 2), (0, -1, 2, 1), (1, 1, 2, 0), (1, -1, 0, 2), (2, 1, 0, 1), (2, -1, 1, 0)];

/// Neighbour slot for chunk offset (dx, dy, dz) ∈ [-1, 1]³ in the array passed to [`Mesher::mesh`].
#[inline]
pub const fn neighbor_index(dx: i32, dy: i32, dz: i32) -> usize {
    ((dx + 1) + (dy + 1) * 3 + (dz + 1) * 9) as usize
}

pub struct MeshOutput {
    /// The vertices, opaque quads first, then cutout quads; then their light bytes in the same
    /// order, packed four per `u32` (little-endian, so the bytes read in order).
    pub verts: Vec<u32>,
    pub opaque_quads: u32,
    pub cutout_quads: u32,
}

#[cfg(test)]
impl MeshOutput {
    pub fn vertex_count(&self) -> usize {
        (self.opaque_quads + self.cutout_quads) as usize * 4
    }

    /// The light byte of vertex `i`.
    pub fn light(&self, i: usize) -> u8 {
        (self.verts[self.vertex_count() + i / 4] >> (i % 4 * 8)) as u8
    }
}

pub struct Mesher {
    pad: Vec<BlockId>,
    mask: Vec<u64>,
    opaque: Vec<u32>,
    cutout: Vec<u32>,
    opaque_light: Vec<u8>,
    cutout_light: Vec<u8>,
}

impl Default for Mesher {
    fn default() -> Self {
        Self::new()
    }
}

/// Maps a padded coordinate (0..34) to (neighbour offset 0..3, local coordinate 0..32).
#[inline]
fn split(p: usize) -> (usize, usize) {
    match p {
        0 => (0, N - 1),
        p if p <= N => (1, p - 1),
        _ => (2, 0),
    }
}

impl Mesher {
    pub fn new() -> Self {
        Self {
            pad: vec![AIR; P * P * P],
            mask: vec![0; N * N],
            opaque: Vec::new(),
            cutout: Vec::new(),
            opaque_light: Vec::new(),
            cutout_light: Vec::new(),
        }
    }

    /// Meshes the centre chunk of a 3×3×3 neighbourhood (see [`neighbor_index`]), lit by `light`
    /// (`PAD_VOLUME` bytes in the padded layout, from `light.rs`; unread when [`Self::is_trivially_empty`]).
    pub fn mesh(&mut self, n: &[&Chunk; 27], light: &[u8]) -> MeshOutput {
        self.opaque.clear();
        self.cutout.clear();
        self.opaque_light.clear();
        self.cutout_light.clear();
        if !Self::is_trivially_empty(n) {
            self.fill_padded(n);
            for face in 0..6 {
                self.mesh_face(face, light);
            }
            self.mesh_plants(light);
        }
        let count = self.opaque.len() + self.cutout.len();
        let mut verts = Vec::with_capacity(count + count.div_ceil(4));
        verts.extend_from_slice(&self.opaque);
        verts.extend_from_slice(&self.cutout);
        let bytes = self.opaque_light.iter().chain(&self.cutout_light).copied().collect::<Vec<u8>>();
        verts.extend(bytes.chunks(4).map(|c| c.iter().rev().fold(0u32, |w, &b| (w << 8) | u32::from(b))));
        MeshOutput { verts, opaque_quads: (self.opaque.len() / 4) as u32, cutout_quads: (self.cutout.len() / 4) as u32 }
    }

    /// Air chunks and solid chunks buried in solid chunks produce no faces; skip the scan.
    pub fn is_trivially_empty(n: &[&Chunk; 27]) -> bool {
        match n[neighbor_index(0, 0, 0)].as_uniform() {
            Some(b) if !MESHED[b as usize] => true,
            Some(b) if OPAQUE[b as usize] => [(1, 0, 0), (-1, 0, 0), (0, 1, 0), (0, -1, 0), (0, 0, 1), (0, 0, -1)]
                .iter()
                .all(|&(x, y, z)| n[neighbor_index(x, y, z)].as_uniform().is_some_and(|u| OPAQUE[u as usize])),
            _ => false,
        }
    }

    fn fill_padded(&mut self, n: &[&Chunk; 27]) {
        let nb = |cx: usize, cy: usize, cz: usize| n[cx + cy * 3 + cz * 9];
        for py in 0..P {
            let (cy, ly) = split(py);
            for pz in 0..P {
                let (cz, lz) = split(pz);
                let row = pidx(0, py, pz);
                self.pad[row] = nb(0, cy, cz).get(N - 1, ly, lz);
                self.pad[row + P - 1] = nb(2, cy, cz).get(0, ly, lz);
                let mid = &mut self.pad[row + 1..row + 1 + N];
                let c = nb(1, cy, cz);
                match c.dense() {
                    Some(d) => {
                        let s = index(0, ly, lz);
                        mid.copy_from_slice(&d[s..s + N]);
                    }
                    None => mid.fill(c.get(0, 0, 0)),
                }
            }
        }
    }

    fn mesh_face(&mut self, face: usize, light: &[u8]) {
        let (d, sign, u, v) = FACES[face];
        let sn = STRIDE[d] * sign;
        let (su, sv) = (STRIDE[u], STRIDE[v]);
        let plane_offset = usize::from(sign > 0);
        let pad = &self.pad;
        let mask = &mut self.mask;

        for slice in 0..N {
            let mut any = false;
            for bv in 0..N {
                for bu in 0..N {
                    let mut c = [0usize; 3];
                    c[d] = slice + 1;
                    c[u] = bu + 1;
                    c[v] = bv + 1;
                    let p = pidx(c[0], c[1], c[2]);
                    let b = pad[p];
                    let mut key = 0u64;
                    if MESHED[b as usize] {
                        let q = (p as isize + sn) as usize;
                        if !OPAQUE[pad[q] as usize] {
                            // Corners (0,0) (1,0) (1,1) (0,1) of the face, each AO | light << 2.
                            let mut corners = 0u64;
                            for (i, (du, dv)) in [(-su, -sv), (su, -sv), (su, sv), (-su, sv)].into_iter().enumerate() {
                                let k = corner(pad, light, q, du, dv);
                                corners |= u64::from(k) << (i * CORNER_BITS);
                            }
                            let layer = u64::from(pick_layer(b, face, c));
                            key = KEY_FLAG
                                | (u64::from(CUTOUT[b as usize]) << KEY_CUTOUT)
                                | (layer << KEY_LAYER)
                                | corners;
                            any = true;
                        }
                    }
                    mask[bu + bv * N] = key;
                }
            }
            if !any {
                continue;
            }

            let plane = (slice + plane_offset) as u32;
            for bv in 0..N {
                let mut bu = 0;
                while bu < N {
                    let key = mask[bu + bv * N];
                    if key == 0 {
                        bu += 1;
                        continue;
                    }
                    let [c00, c10, c11, c01] = corners_of(key);
                    let can_u = c00 == c10 && c01 == c11;
                    let can_v = c00 == c01 && c10 == c11;

                    let mut w = 1;
                    if can_u {
                        while bu + w < N && mask[bu + w + bv * N] == key {
                            w += 1;
                        }
                    }
                    let mut h = 1;
                    if can_v {
                        'grow: while bv + h < N {
                            for k in 0..w {
                                if mask[bu + k + (bv + h) * N] != key {
                                    break 'grow;
                                }
                            }
                            h += 1;
                        }
                    }
                    for dv in 0..h {
                        mask[bu + (bv + dv) * N..bu + w + (bv + dv) * N].fill(0);
                    }

                    let out = if key & (1 << KEY_CUTOUT) != 0 {
                        (&mut self.cutout, &mut self.cutout_light)
                    } else {
                        (&mut self.opaque, &mut self.opaque_light)
                    };
                    emit_quad(out, face, (d, u, v), plane, (bu as u32, bv as u32), (w as u32, h as u32), key);
                    bu += w;
                }
            }
        }
    }
}

/// The texture layer for face `face` of block `b` at padded position `c`: its own layer, or for a
/// layer with alternates one of the four, the same for every face of the block. Chosen from the
/// chunk-local position, so the pattern repeats only every chunk.
#[inline]
fn pick_layer(b: BlockId, face: usize, c: [usize; 3]) -> u16 {
    let base = FACE_TEX[b as usize][face];
    let alt = ALT_TEX[b as usize][face];
    if alt == 0 {
        return base;
    }
    let h = (c[0] as u32).wrapping_mul(0x9E37_79B1)
        ^ (c[1] as u32).wrapping_mul(0x85EB_CA77)
        ^ (c[2] as u32).wrapping_mul(0xC2B2_AE3D);
    match ((h ^ (h >> 15)).wrapping_mul(0x2C1B_3C6D) >> 30) as u16 {
        0 => base,
        k => alt + k - 1,
    }
}

impl Mesher {
    /// Two diagonal quads per plant block (faces 6 and 7), each in both windings, unshaded by AO.
    fn mesh_plants(&mut self, light: &[u8]) {
        for y in 0..N {
            for z in 0..N {
                for x in 0..N {
                    let p = pidx(x + 1, y + 1, z + 1);
                    let b = self.pad[p];
                    if !PLANT[b as usize] {
                        continue;
                    }
                    self.cutout_light.extend_from_slice(&[light[p]; 16]);
                    let (x, y, z) = (x as u32, y as u32, z as u32);
                    let layer = FACE_TEX[b as usize][0] as u32;
                    let v = |dx: u32, dy: u32, dz: u32, face: u32| {
                        (x + dx) | ((y + dy) << 6) | ((z + dz) << 12) | (face << 18) | (3 << 21) | (layer << 23)
                    };
                    let a = [v(0, 0, 0, 6), v(1, 0, 1, 6), v(1, 1, 1, 6), v(0, 1, 0, 6)];
                    let b = [v(1, 0, 0, 7), v(0, 0, 1, 7), v(0, 1, 1, 7), v(1, 1, 0, 7)];
                    for q in [a, b] {
                        self.cutout.extend_from_slice(&q);
                        self.cutout.extend_from_slice(&[q[3], q[2], q[1], q[0]]);
                    }
                }
            }
        }
    }
}

/// Classic voxel AO: two occluding sides fully darken the corner regardless of the diagonal.
#[inline]
fn ao(side1: bool, side2: bool, corner: bool) -> u32 {
    if side1 && side2 {
        0
    } else {
        3 - (side1 as u32 + side2 as u32 + corner as u32)
    }
}

/// The merge key: 4 corners of `CORNER_BITS` (AO 2 bits, light 8 bits), then layer, cutout, flag.
const CORNER_BITS: usize = 10;
const KEY_LAYER: usize = 4 * CORNER_BITS;
const KEY_CUTOUT: usize = KEY_LAYER + 9;
const KEY_FLAG: u64 = 1 << 63;

fn corners_of(key: u64) -> [u32; 4] {
    std::array::from_fn(|i| (key >> (i * CORNER_BITS)) as u32 & ((1 << CORNER_BITS) - 1))
}

/// AO | light << 2 for the face corner towards (`du`, `dv`) of air cell `q`. The light is the
/// average of the non-opaque cells among `q`, its two sides and the diagonal between them (the
/// diagonal only when a side is open), sky and block light each rounded to a nibble.
#[inline]
fn corner(pad: &[BlockId], light: &[u8], q: usize, du: isize, dv: isize) -> u32 {
    let at = |off: isize| (q as isize + off) as usize;
    let (side1, side2, diag) = (at(du), at(dv), at(du + dv));
    let (o1, o2, od) = (OPAQUE[pad[side1] as usize], OPAQUE[pad[side2] as usize], OPAQUE[pad[diag] as usize]);
    let (mut sky, mut block, mut n) = (0u32, 0u32, 0u32);
    for (cell, open) in [(q, true), (side1, !o1), (side2, !o2), (diag, !(od || (o1 && o2)))] {
        if open {
            sky += u32::from(light[cell] & 15);
            block += u32::from(light[cell] >> 4);
            n += 1;
        }
    }
    let l = ((sky + n / 2) / n) | (((block + n / 2) / n) << 4);
    ao(o1, o2, od) | (l << 2)
}

fn emit_quad(
    (out, lights): (&mut Vec<u32>, &mut Vec<u8>),
    face: usize,
    (d, u, v): (usize, usize, usize),
    plane: u32,
    (bu, bv): (u32, u32),
    (w, h): (u32, u32),
    key: u64,
) {
    let layer = (key >> KEY_LAYER) as u32 & 0x1FF;
    let corner = corners_of(key);
    let corners = [(bu, bv), (bu + w, bv), (bu + w, bv + h), (bu, bv + h)];
    let mut vs = [0u32; 4];
    let mut ls = [0u8; 4];
    let mut bright = [0u32; 4];
    for i in 0..4 {
        let mut c = [0u32; 3];
        c[d] = plane;
        c[u] = corners[i].0;
        c[v] = corners[i].1;
        let (a, l) = (corner[i] & 3, (corner[i] >> 2) as u8);
        vs[i] = c[0] | (c[1] << 6) | (c[2] << 12) | ((face as u32) << 18) | (a << 21) | (layer << 23);
        ls[i] = l;
        bright[i] = a * 16 + u32::from((l & 15).max(l >> 4));
    }
    // Split the quad along the brighter diagonal to avoid the anisotropic AO and light seam.
    if bright[0] + bright[2] < bright[1] + bright[3] {
        out.extend_from_slice(&[vs[1], vs[2], vs[3], vs[0]]);
        lights.extend_from_slice(&[ls[1], ls[2], ls[3], ls[0]]);
    } else {
        out.extend_from_slice(&vs);
        lights.extend_from_slice(&ls);
    }
}

#[cfg(test)]
mod tests;
