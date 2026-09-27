//! Greedy chunk mesher with per-vertex ambient occlusion.
//!
//! Output is one `u32` per vertex, 4 vertices per quad, consumed with a shared quad index buffer:
//!
//! ```text
//! bits  0..6   x  (0..=32, chunk-local)
//! bits  6..12  y
//! bits 12..18  z
//! bits 18..21  face  (+X, -X, +Y, -Y, +Z, -Z; 6 and 7 are a plant's two diagonal quads)
//! bits 21..23  ambient occlusion (0 = darkest, 3 = unoccluded); for a liquid quad, 1 marks a surface
//!              corner, which the shader lowers by a tenth of a block (the water line)
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
//! cutout list and never merged. Liquids (`block::LIQUID`) get faces only against cells that are neither
//! opaque nor liquid, unshaded by AO, into a third list the host draws blended after the others.

use crate::block::{BlockId, AIR, ALT_TEX, CUTOUT, FACE_TEX, LIQUID, MESHED, OPAQUE, PLANT};
use crate::chunk::{index, Chunk};

mod quad;
use quad::{corner, corners_of, emit_quad, CORNER_BITS, KEY_CUTOUT, KEY_FLAG, KEY_LAYER, KEY_LIQUID};

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
    /// The vertices, opaque quads first, then cutout quads, then liquid quads; then their light bytes in
    /// the same order, packed four per `u32` (little-endian, so the bytes read in order).
    pub verts: Vec<u32>,
    pub opaque_quads: u32,
    pub cutout_quads: u32,
    pub liquid_quads: u32,
}

#[cfg(test)]
impl MeshOutput {
    pub fn vertex_count(&self) -> usize {
        (self.opaque_quads + self.cutout_quads + self.liquid_quads) as usize * 4
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
    liquid: Vec<u32>,
    opaque_light: Vec<u8>,
    cutout_light: Vec<u8>,
    liquid_light: Vec<u8>,
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
            liquid: Vec::new(),
            opaque_light: Vec::new(),
            cutout_light: Vec::new(),
            liquid_light: Vec::new(),
        }
    }

    /// Meshes the centre chunk of a 3×3×3 neighbourhood (see [`neighbor_index`]), lit by `light`
    /// (`PAD_VOLUME` bytes in the padded layout, from `light.rs`; unread when [`Self::is_trivially_empty`]).
    pub fn mesh(&mut self, n: &[&Chunk; 27], light: &[u8]) -> MeshOutput {
        self.opaque.clear();
        self.cutout.clear();
        self.liquid.clear();
        self.opaque_light.clear();
        self.cutout_light.clear();
        self.liquid_light.clear();
        if !Self::is_trivially_empty(n) {
            self.fill_padded(n);
            for face in 0..6 {
                self.mesh_face(face, light);
            }
            self.mesh_plants(light);
        }
        let count = self.opaque.len() + self.cutout.len() + self.liquid.len();
        let mut verts = Vec::with_capacity(count + count.div_ceil(4));
        verts.extend_from_slice(&self.opaque);
        verts.extend_from_slice(&self.cutout);
        verts.extend_from_slice(&self.liquid);
        let bytes =
            self.opaque_light.iter().chain(&self.cutout_light).chain(&self.liquid_light).copied().collect::<Vec<u8>>();
        verts.extend(bytes.chunks(4).map(|c| c.iter().rev().fold(0u32, |w, &b| (w << 8) | u32::from(b))));
        let quads = |v: &Vec<u32>| (v.len() / 4) as u32;
        MeshOutput {
            verts,
            opaque_quads: quads(&self.opaque),
            cutout_quads: quads(&self.cutout),
            liquid_quads: quads(&self.liquid),
        }
    }

    /// Air chunks, and solid or liquid chunks enclosed by solid chunks (or liquid ones, for liquid), produce
    /// no faces; skip the scan.
    pub fn is_trivially_empty(n: &[&Chunk; 27]) -> bool {
        match n[neighbor_index(0, 0, 0)].as_uniform() {
            Some(b) if !MESHED[b as usize] => true,
            Some(b) if OPAQUE[b as usize] || LIQUID[b as usize] => {
                let hides = |u: BlockId| OPAQUE[u as usize] || (LIQUID[b as usize] && LIQUID[u as usize]);
                [(1, 0, 0), (-1, 0, 0), (0, 1, 0), (0, -1, 0), (0, 0, 1), (0, 0, -1)]
                    .iter()
                    .all(|&(x, y, z)| n[neighbor_index(x, y, z)].as_uniform().is_some_and(hides))
            }
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
                        let liquid = LIQUID[b as usize];
                        if !(OPAQUE[pad[q] as usize] || liquid && LIQUID[pad[q] as usize]) {
                            // A liquid under an open cell: its top face and the top edges of its sides sit lower.
                            let above = pad[p + P2] as usize;
                            let surface = liquid && !LIQUID[above] && !OPAQUE[above];
                            // Corners (0,0) (1,0) (1,1) (0,1) of the face, each AO | light << 2.
                            let mut corners = 0u64;
                            for (i, (du, dv)) in [(-su, -sv), (su, -sv), (su, sv), (-su, sv)].into_iter().enumerate() {
                                let mut k = corner(pad, light, q, du, dv);
                                if liquid {
                                    let top = face == 2 || du == P2 as isize || dv == P2 as isize;
                                    k = (k & !3) | u32::from(surface && top);
                                }
                                corners |= u64::from(k) << (i * CORNER_BITS);
                            }
                            let layer = u64::from(pick_layer(b, face, c));
                            key = KEY_FLAG
                                | (u64::from(CUTOUT[b as usize]) << KEY_CUTOUT)
                                | (u64::from(liquid) << KEY_LIQUID)
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

                    let out = if key & (1 << KEY_LIQUID) != 0 {
                        (&mut self.liquid, &mut self.liquid_light)
                    } else if key & (1 << KEY_CUTOUT) != 0 {
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

#[cfg(test)]
mod tests;
