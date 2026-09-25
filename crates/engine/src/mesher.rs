//! Greedy chunk mesher with per-vertex ambient occlusion.
//!
//! Output is one `u32` per vertex, 4 vertices per quad, consumed with a shared quad index buffer:
//!
//! ```text
//! bits  0..6   x  (0..=32, chunk-local)
//! bits  6..12  y
//! bits 12..18  z
//! bits 18..21  face  (+X, -X, +Y, -Y, +Z, -Z)
//! bits 21..23  ambient occlusion (0 = darkest, 3 = unoccluded)
//! bits 23..32  texture array layer
//! ```
//!
//! UVs are not stored: the shader derives them from the vertex position, so a merged 5x3 quad
//! simply tiles its texture five by three times (texture wrap = REPEAT).
//!
//! Quads are merged only where AO is constant along the merge direction, so greedy merging never
//! changes how the AO gradient looks.

use crate::block::{BlockId, AIR, CUTOUT, FACE_TEX, MESHED, OPAQUE};
use crate::chunk::{index, Chunk};

const N: usize = 32;
const P: usize = N + 2;
const P2: usize = P * P;

#[inline]
const fn pidx(x: usize, y: usize, z: usize) -> usize {
    (y * P + z) * P + x
}

/// Index strides in the padded volume for x, y, z.
const STRIDE: [isize; 3] = [1, P2 as isize, P as isize];

/// Per face: (normal axis, normal sign, u axis, v axis), with u × v = normal so that
/// corners (0,0) (1,0) (1,1) (0,1) wind counter-clockwise seen from outside.
const FACES: [(usize, isize, usize, usize); 6] = [
    (0, 1, 1, 2),
    (0, -1, 2, 1),
    (1, 1, 2, 0),
    (1, -1, 0, 2),
    (2, 1, 0, 1),
    (2, -1, 1, 0),
];

/// Neighbour slot for chunk offset (dx, dy, dz) ∈ [-1, 1]³ in the array passed to [`Mesher::mesh`].
#[inline]
pub const fn neighbor_index(dx: i32, dy: i32, dz: i32) -> usize {
    ((dx + 1) + (dy + 1) * 3 + (dz + 1) * 9) as usize
}

pub struct MeshOutput {
    /// Opaque quads first, then cutout quads.
    pub verts: Vec<u32>,
    pub opaque_quads: u32,
    pub cutout_quads: u32,
}

pub struct Mesher {
    pad: Vec<BlockId>,
    mask: Vec<u32>,
    opaque: Vec<u32>,
    cutout: Vec<u32>,
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
        Self { pad: vec![AIR; P * P * P], mask: vec![0; N * N], opaque: Vec::new(), cutout: Vec::new() }
    }

    /// Meshes the centre chunk of a 3×3×3 neighbourhood (see [`neighbor_index`]).
    pub fn mesh(&mut self, n: &[&Chunk; 27]) -> MeshOutput {
        self.opaque.clear();
        self.cutout.clear();
        if !self.is_trivially_empty(n) {
            self.fill_padded(n);
            for face in 0..6 {
                self.mesh_face(face);
            }
        }
        let mut verts = Vec::with_capacity(self.opaque.len() + self.cutout.len());
        verts.extend_from_slice(&self.opaque);
        verts.extend_from_slice(&self.cutout);
        MeshOutput {
            verts,
            opaque_quads: (self.opaque.len() / 4) as u32,
            cutout_quads: (self.cutout.len() / 4) as u32,
        }
    }

    /// Air chunks and solid chunks buried in solid chunks produce no faces; skip the scan.
    fn is_trivially_empty(&self, n: &[&Chunk; 27]) -> bool {
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

    fn mesh_face(&mut self, face: usize) {
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
                    let mut key = 0u32;
                    if MESHED[b as usize] {
                        let q = (p as isize + sn) as usize;
                        if !OPAQUE[pad[q] as usize] {
                            let o = |off: isize| OPAQUE[pad[(q as isize + off) as usize] as usize];
                            let (um, up, vm, vp) = (o(-su), o(su), o(-sv), o(sv));
                            let a00 = ao(um, vm, o(-su - sv));
                            let a10 = ao(up, vm, o(su - sv));
                            let a11 = ao(up, vp, o(su + sv));
                            let a01 = ao(um, vp, o(-su + sv));
                            let layer = FACE_TEX[b as usize][face] as u32;
                            key = (1 << 31)
                                | (u32::from(CUTOUT[b as usize]) << 17)
                                | (layer << 8)
                                | a00
                                | (a10 << 2)
                                | (a11 << 4)
                                | (a01 << 6);
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
                    let (a00, a10, a11, a01) = (key & 3, (key >> 2) & 3, (key >> 4) & 3, (key >> 6) & 3);
                    let can_u = a00 == a10 && a01 == a11;
                    let can_v = a00 == a01 && a10 == a11;

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

                    let out = if key & (1 << 17) != 0 { &mut self.cutout } else { &mut self.opaque };
                    emit_quad(out, face, (d, u, v), plane, (bu as u32, bv as u32), (w as u32, h as u32), key);
                    bu += w;
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

fn emit_quad(
    out: &mut Vec<u32>,
    face: usize,
    (d, u, v): (usize, usize, usize),
    plane: u32,
    (bu, bv): (u32, u32),
    (w, h): (u32, u32),
    key: u32,
) {
    let layer = (key >> 8) & 0x1FF;
    let a = [key & 3, (key >> 2) & 3, (key >> 4) & 3, (key >> 6) & 3];
    let corners = [(bu, bv), (bu + w, bv), (bu + w, bv + h), (bu, bv + h)];
    let mut vs = [0u32; 4];
    for i in 0..4 {
        let mut c = [0u32; 3];
        c[d] = plane;
        c[u] = corners[i].0;
        c[v] = corners[i].1;
        vs[i] = c[0] | (c[1] << 6) | (c[2] << 12) | ((face as u32) << 18) | (a[i] << 21) | (layer << 23);
    }
    // Split the quad along the brighter diagonal to avoid the anisotropic AO seam.
    if a[0] + a[2] < a[1] + a[3] {
        out.extend_from_slice(&[vs[1], vs[2], vs[3], vs[0]]);
    } else {
        out.extend_from_slice(&vs);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::block::{LEAVES, STONE};

    fn mesh_single(setup: impl FnOnce(&mut Chunk)) -> MeshOutput {
        let air = Chunk::uniform(AIR);
        let mut c = Chunk::uniform(AIR);
        setup(&mut c);
        let mut refs = [&air; 27];
        refs[neighbor_index(0, 0, 0)] = &c;
        Mesher::new().mesh(&refs)
    }

    fn decode(v: u32) -> (u32, u32, u32, u32, u32) {
        (v & 63, (v >> 6) & 63, (v >> 12) & 63, (v >> 18) & 7, (v >> 21) & 3)
    }

    #[test]
    fn single_block_has_six_unoccluded_faces() {
        let m = mesh_single(|c| c.set(5, 5, 5, STONE));
        assert_eq!(m.opaque_quads, 6);
        assert_eq!(m.cutout_quads, 0);
        for &v in &m.verts {
            let (x, y, z, _, ao) = decode(v);
            assert!((5..=6).contains(&x) && (5..=6).contains(&y) && (5..=6).contains(&z));
            assert_eq!(ao, 3);
        }
    }

    #[test]
    fn row_of_blocks_is_merged() {
        let m = mesh_single(|c| {
            for x in 0..10 {
                c.set(x, 3, 3, STONE);
            }
        });
        assert_eq!(m.opaque_quads, 6, "a 10x1x1 bar should become 6 quads");
    }

    #[test]
    fn slab_is_merged_into_six_quads() {
        let m = mesh_single(|c| {
            for z in 0..32 {
                for x in 0..32 {
                    c.set(x, 0, z, STONE);
                }
            }
        });
        assert_eq!(m.opaque_quads, 6);
    }

    #[test]
    fn hidden_faces_are_culled() {
        let m = mesh_single(|c| {
            c.set(1, 1, 1, STONE);
            c.set(2, 1, 1, STONE);
        });
        // Two touching cubes: 10 visible faces, merged down to 6.
        assert_eq!(m.opaque_quads, 6);
    }

    #[test]
    fn leaves_go_to_cutout_pass() {
        let m = mesh_single(|c| c.set(0, 0, 0, LEAVES));
        assert_eq!(m.opaque_quads, 0);
        assert_eq!(m.cutout_quads, 6);
        assert_eq!(m.verts.len(), 24);
    }

    #[test]
    fn corner_occlusion_darkens_vertex() {
        // A block with another block diagonally above-adjacent: the top face gets one darker edge.
        let m = mesh_single(|c| {
            c.set(4, 4, 4, STONE);
            c.set(5, 5, 4, STONE);
        });
        let top_face_min_ao = m
            .verts
            .iter()
            .map(|&v| decode(v))
            .filter(|&(_, y, _, f, _)| f == 2 && y == 5)
            .map(|(.., ao)| ao)
            .min()
            .unwrap();
        assert!(top_face_min_ao < 3);
    }

    #[test]
    fn machines_are_left_to_the_host_and_do_not_hide_faces() {
        use crate::block::{BELT, MINER, STORAGE};
        let m = mesh_single(|c| {
            c.set(4, 4, 4, MINER);
            c.set(5, 4, 4, STONE);
            c.set(8, 4, 4, BELT);
        });
        // Only the stone is meshed, with all six faces (the miner beside it hides nothing).
        assert_eq!(m.opaque_quads, 6);
        let boxed = mesh_single(|c| c.set(1, 1, 1, STORAGE));
        assert_eq!(boxed.opaque_quads, 6, "storage boxes are plain cubes");
    }

    #[test]
    fn uniform_solid_neighbourhood_is_empty() {
        let stone = Chunk::uniform(STONE);
        let refs = [&stone; 27];
        let m = Mesher::new().mesh(&refs);
        assert_eq!(m.verts.len(), 0);
    }

    #[test]
    fn faces_on_chunk_border_respect_neighbours() {
        let air = Chunk::uniform(AIR);
        let stone = Chunk::uniform(STONE);
        let mut c = Chunk::uniform(AIR);
        c.set(31, 0, 0, STONE);
        let mut refs = [&air; 27];
        refs[neighbor_index(0, 0, 0)] = &c;
        refs[neighbor_index(1, 0, 0)] = &stone;
        let m = Mesher::new().mesh(&refs);
        // +X face is hidden by the solid neighbour chunk.
        assert!(m.verts.iter().all(|&v| decode(v).3 != 0));
        assert_eq!(m.opaque_quads, 5);
    }
}
