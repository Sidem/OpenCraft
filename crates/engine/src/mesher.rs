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
//! bits 23..32  texture array layer low 9 bits (high 3 bits sit in `a_light.y` bits 5..8, so layers reach 4095;
//!              `block::turn::MASK` and shaped vertices cap terrain layers at 2048)
//! (a liquid vertex holds its surface drop, kind of water, flow direction and ice there instead: `water.rs`)
//! ```
//!
//! UVs are not stored: the shader derives them from the vertex position, so a merged 5x3 quad
//! simply tiles its texture five by three times (texture wrap = REPEAT).
//!
//! Each vertex also gets one light byte: smoothed sky light in the low nibble, block light in the
//! high one, averaged from the light field (`light.rs`) in the cells in front of the face around that
//! corner. They follow the vertices in `verts`, four per `u32` (see [`MeshOutput`]).
//!
//! Quads are merged only where AO and light are constant along the merge direction, so greedy merging never changes
//! how their gradients look. Layers with alternates (ores, leaves: `block::tex::alternates`) get one of four looks per
//! block (`face_layer.rs`, which also greens turf's lit, grown-over faces: `sim/sod.rs`), so those faces merge less.
//! Plants (`block::PLANT`) are two crossed quads into the cutout list, never merged (`plants.rs`; so is the ground
//! cover on natural ground: `cover.rs`). Liquids (`block::LIQUID`) get faces only against cells that are neither
//! opaque nor liquid, unshaded by AO, into a third list the host draws blended after the others. Their surface corners
//! sit below the block's top by a height read from the cells around (`water::corner_drop`), so
//! flows slope smoothly and no face is needed between two liquid cells.
//!
//! Trees are not cubes: logs (branches, `logs.rs`) and leaves (cards, `foliage.rs`), read from the blocks around the
//! chunk (`tree_grid.rs`), go into a fourth, "shaped" list with its own vertex format (`shaped.rs`). Logs hide no
//! faces beside them (`HIDES`).

use crate::block::{is_leaves, is_log, BlockId, AIR, CUTOUT, LIQUID, MESHED, OPAQUE, TRANSLUCENT};
use crate::chunk::{index, Chunk};
use shaped::{HIDES, SHAPED};

mod blocks;
mod cover;
mod face_layer;
mod foliage;
mod logs;
mod plants;
mod quad;
pub mod shaped;
mod tree_grid;
mod water;
pub use cover::{bears_cover, Cover};
use face_layer::face_layer;
pub use foliage::{pack_foliage, pack_ground, season_layers};
pub use quad::MeshOutput;
use quad::{corner, emit_quad, grow, CORNER_BITS, KEY_CUTOUT, KEY_FLAG, KEY_LAYER, KEY_LIQUID, KEY_TRANSLUCENT};
pub use tree_grid::REACH as TREE_REACH;
pub use water::height as water_height;

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

pub struct Mesher {
    pad: Vec<BlockId>,
    /// Shape modifiers for the padded 34³ volume; filled only when `has_shapes` is true (zero work for normal chunks).
    pad_shapes: Vec<u8>,
    has_shapes: bool,
    /// Per-face paint for the centre 32³ chunk; filled only when `has_paint` is true (zero work for unpainted chunks).
    centre_paint: Vec<u32>,
    has_paint: bool,
    /// The centre chunk's bare turf faces (`Chunk::bares`, `sim/sod.rs`), sorted by cell; usually empty.
    centre_bare: Vec<(u16, u8)>,
    mask: Vec<(u64, u8)>,
    opaque: Vec<u32>,
    cutout: Vec<u32>,
    liquid: Vec<u32>,
    translucent: Vec<u32>,
    opaque_light: Vec<u8>,
    cutout_light: Vec<u8>,
    liquid_light: Vec<u8>,
    translucent_light: Vec<u8>,
    shaped: shaped::Shaped,
    trees: tree_grid::TreeGrid,
    /// The world block at the chunk's lowest corner: leaf cards scatter by world cell.
    origin: [i32; 3],
    /// The climate round the chunk (`Cover::warmth`), which water surfaces read for ice; `None` with no cover.
    warmth: Option<[i32; 4]>,
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
            pad_shapes: vec![0; P * P * P],
            has_shapes: false,
            centre_paint: vec![0; N * N * N],
            has_paint: false,
            centre_bare: Vec::new(),
            mask: vec![(0, 0); N * N],
            opaque: Vec::new(),
            cutout: Vec::new(),
            liquid: Vec::new(),
            translucent: Vec::new(),
            opaque_light: Vec::new(),
            cutout_light: Vec::new(),
            liquid_light: Vec::new(),
            translucent_light: Vec::new(),
            shaped: shaped::Shaped::default(),
            trees: tree_grid::TreeGrid::default(),
            origin: [0; 3],
            warmth: None,
        }
    }

    /// The world block at the lowest corner of the chunks meshed next: leaf cards are scattered by world cell.
    pub fn set_origin(&mut self, origin: [i32; 3]) {
        self.origin = origin;
    }

    /// Meshes the centre chunk of a 3×3×3 neighbourhood (see [`neighbor_index`]), lit by `light`
    /// (`PAD_VOLUME` bytes in the padded layout, from `light.rs`; unread when [`Self::is_trivially_empty`]).
    pub fn mesh(&mut self, n: &[&Chunk; 27], light: &[u8], cover: Option<&Cover>) -> MeshOutput {
        self.opaque.clear();
        self.cutout.clear();
        self.liquid.clear();
        self.translucent.clear();
        self.opaque_light.clear();
        self.cutout_light.clear();
        self.liquid_light.clear();
        self.translucent_light.clear();
        self.shaped.clear();
        self.warmth = cover.map(|c| c.warmth);
        if !Self::is_trivially_empty(n) {
            self.fill_padded(n);
            for face in 0..6 {
                self.mesh_face(face, light);
            }
            self.mesh_plants(light);
            if let Some(cover) = cover {
                self.mesh_cover(n, light, cover);
            }
            if n[neighbor_index(0, 0, 0)].has_shapes() {
                let paint = if self.has_paint { Some(self.centre_paint.as_slice()) } else { None };
                blocks::mesh_shaped_blocks(&self.pad, &self.pad_shapes, paint, light, &mut self.shaped);
            }
            let (logs, leaves) = (self.pad.iter().any(|&b| is_log(b)), self.pad.iter().any(|&b| is_leaves(b)));
            if logs || leaves {
                self.trees.fill(n);
                logs::mesh_logs(&self.trees, &self.pad, light, &mut self.shaped);
                foliage::mesh_leaves(&self.trees, light, self.origin, &mut self.shaped);
            }
        }
        let count = self.opaque.len() + self.cutout.len() + self.liquid.len() + self.translucent.len();
        let mut verts = Vec::with_capacity(count + count.div_ceil(2));
        for v in [&self.opaque, &self.cutout, &self.liquid, &self.translucent] {
            verts.extend_from_slice(v);
        }
        let mut bytes = Vec::with_capacity(count * 2);
        for l in [&self.opaque_light, &self.cutout_light, &self.liquid_light, &self.translucent_light] {
            bytes.extend_from_slice(l);
        }
        verts.extend(bytes.chunks(4).map(|c| c.iter().rev().fold(0u32, |w, &b| (w << 8) | u32::from(b))));
        let quads = |v: &Vec<u32>| (v.len() / 4) as u32;
        let shaped_quads = |v: &Vec<u32>| (v.len() / 12) as u32;
        let mut shaped =
            Vec::with_capacity(self.shaped.opaque.len() + self.shaped.cutout.len() + self.shaped.translucent.len());
        shaped.extend_from_slice(&self.shaped.opaque);
        shaped.extend_from_slice(&self.shaped.cutout);
        shaped.extend_from_slice(&self.shaped.translucent);
        MeshOutput {
            verts,
            opaque_quads: quads(&self.opaque),
            cutout_quads: quads(&self.cutout),
            liquid_quads: quads(&self.liquid),
            translucent_quads: quads(&self.translucent),
            shaped,
            shaped_opaque: shaped_quads(&self.shaped.opaque),
            shaped_cutout: shaped_quads(&self.shaped.cutout),
            shaped_translucent: shaped_quads(&self.shaped.translucent),
        }
    }

    /// Air chunks, and solid or liquid chunks enclosed by solid chunks (or liquid ones, for liquid), produce
    /// no faces; skip the scan.
    pub fn is_trivially_empty(n: &[&Chunk; 27]) -> bool {
        match n[neighbor_index(0, 0, 0)].as_uniform() {
            Some(b) if !MESHED[b as usize] => true,
            Some(b) if OPAQUE[b as usize] || LIQUID[b as usize] => {
                let hides = |u: BlockId| HIDES[u as usize] || (LIQUID[b as usize] && LIQUID[u as usize]);
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
        water::overlay_wet(n, &mut self.pad);
        self.has_shapes = blocks::fill_pad_shapes(n, &mut self.pad_shapes);
        let centre = n[neighbor_index(0, 0, 0)];
        self.has_paint = centre.has_paint();
        if self.has_paint {
            self.centre_paint.fill(0);
            for &(idx, packed) in centre.paints() {
                self.centre_paint[idx as usize] = packed;
            }
        }
        self.centre_bare.clear();
        self.centre_bare.extend_from_slice(centre.bares());
    }

    fn mesh_face(&mut self, face: usize, light: &[u8]) {
        let (d, sign, u, v) = FACES[face];
        let sn = STRIDE[d] * sign;
        let (su, sv) = (STRIDE[u], STRIDE[v]);
        let plane_offset = usize::from(sign > 0);
        let pad = &self.pad;
        let pad_shapes = &self.pad_shapes;
        let has_shapes = self.has_shapes;
        let centre_paint = &self.centre_paint;
        let has_paint = self.has_paint;
        let centre_bare = &self.centre_bare;
        let (warmth, y0) = (self.warmth, self.origin[1]);
        let opp_bit = 1u8 << (face ^ 1);
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
                    let mut paint = 0u8;
                    if MESHED[b as usize] && !SHAPED[b as usize] && !(has_shapes && pad_shapes[p] != 0) {
                        let q = (p as isize + sn) as usize;
                        let liquid = LIQUID[b as usize];
                        let translucent = TRANSLUCENT[b as usize];
                        let qs = if has_shapes { pad_shapes[q] } else { 0 };
                        let q_hides = HIDES[pad[q] as usize]
                            && (qs == 0
                                || crate::shape::full_face_mask(qs, crate::shape::Corner::OuterLeft) & opp_bit != 0);
                        if !(q_hides || liquid && LIQUID[pad[q] as usize] || translucent && pad[q] == b && qs == 0) {
                            // A liquid under an open cell: its top face and the top edges of its sides sit lower.
                            let above = pad[p + P2] as usize;
                            let surface = liquid && !LIQUID[above] && !OPAQUE[above];
                            // Corners (0,0) (1,0) (1,1) (0,1) of the face, each AO | light << 2.
                            let mut corners = 0u64;
                            for (i, (du, dv)) in [(-su, -sv), (su, -sv), (su, sv), (-su, sv)].into_iter().enumerate() {
                                let mut k = corner(pad, light, q, du, dv);
                                if liquid {
                                    // The corner's side of the cell on each axis: the normal's, then the face's own.
                                    let mut side = [0isize; 3];
                                    side[d] = sign;
                                    side[u] = if i == 1 || i == 2 { 1 } else { -1 };
                                    side[v] = if i >= 2 { 1 } else { -1 };
                                    let drop = if surface && side[1] > 0 {
                                        water::corner_drop(pad, p, side[0], side[2])
                                    } else {
                                        0
                                    };
                                    k = (k & 0x3FC) | (drop & 3) | (drop >> 2) << 10;
                                }
                                corners |= u64::from(k) << (i * CORNER_BITS);
                            }
                            let layer = if liquid {
                                u64::from(water::look(pad, p) | water::ice(b, face, c, y0, warmth) << 5)
                            } else {
                                u64::from(face_layer(b, face, c, light[q] & 15, centre_bare))
                            };
                            key = KEY_FLAG
                                | (u64::from(CUTOUT[b as usize]) << KEY_CUTOUT)
                                | (u64::from(liquid) << KEY_LIQUID)
                                | (u64::from(translucent) << KEY_TRANSLUCENT)
                                | (layer << KEY_LAYER)
                                | corners;
                            if has_paint {
                                paint =
                                    crate::colour::get_face(centre_paint[index(c[0] - 1, c[1] - 1, c[2] - 1)], face);
                            }
                            any = true;
                        }
                    }
                    mask[bu + bv * N] = (key, paint);
                }
            }
            if !any {
                continue;
            }

            let plane = (slice + plane_offset) as u32;
            for bv in 0..N {
                let mut bu = 0;
                while bu < N {
                    let entry = mask[bu + bv * N];
                    let key = entry.0;
                    if key == 0 {
                        bu += 1;
                        continue;
                    }
                    let (w, h) = grow(mask, (bu, bv), N);
                    let out = if key & (1 << KEY_TRANSLUCENT) != 0 {
                        (&mut self.translucent, &mut self.translucent_light)
                    } else if key & (1 << KEY_LIQUID) != 0 {
                        (&mut self.liquid, &mut self.liquid_light)
                    } else if key & (1 << KEY_CUTOUT) != 0 {
                        (&mut self.cutout, &mut self.cutout_light)
                    } else {
                        (&mut self.opaque, &mut self.opaque_light)
                    };
                    emit_quad(out, face, (d, u, v), plane, (bu as u32, bv as u32), (w as u32, h as u32), entry);
                    bu += w;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests;
