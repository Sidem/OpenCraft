//! The cube-face pass of the mesher: for each of the six directions, every slice's visible faces into the greedy mask,
//! then merged into quads.
//!
//! Finding the faces: a padded (y, z) row is one `u64` with a bit per padded x, built once per chunk (`fill_rows`):
//! `cand`, cells that may show a cube face (meshed, not shaped), and `hide`, cells that surely hide a face against them
//! (`HIDES`, no shape). A face can show only where `cand` meets a neighbour not in `hide`: a shift or a row offset per
//! direction. The exact test (shaped neighbours, liquids, glass against glass) runs only on those cells, so a chunk of
//! stone costs a few thousand bit operations instead of a test per cell and face (2026-11-28: 5 ms a ground chunk).
//!
//! Invariants: the result is the same as testing every cell (`untouched_chunks_mesh_as_they_always_did`); `mask` is
//! all zero between slices (`grow` clears what it merges), so only visible cells are written.

use crate::block::{CUTOUT, LIQUID, MESHED, OPAQUE, TRANSLUCENT};
use crate::chunk::index;

use super::shaped::{HIDES, SHAPED};
use super::{corner, emit_quad, face_layer, grow, pidx, water, Mesher, CORNER_BITS, FACES, N, P, P2, STRIDE};
use super::{KEY_CUTOUT, KEY_FLAG, KEY_LAYER, KEY_LIQUID, KEY_TRANSLUCENT};

/// The bits of the meshed centre cells, padded x 1..=32.
const CENTRE: u64 = ((1u64 << N) - 1) << 1;

impl Mesher {
    /// Builds the `cand` and `hide` rows from the padded blocks and shapes (after `fill_padded`).
    pub(super) fn fill_rows(&mut self) {
        for y in 0..P {
            for z in 0..P {
                let row = pidx(0, y, z);
                let (mut cand, mut hide) = (0u64, 0u64);
                for x in 0..P {
                    let b = self.pad[row + x] as usize;
                    let shaped = self.has_shapes && self.pad_shapes[row + x] != 0;
                    cand |= u64::from(MESHED[b] && !SHAPED[b] && !shaped) << x;
                    hide |= u64::from(HIDES[b] && !shaped) << x;
                }
                self.cand[y * P + z] = cand;
                self.hide[y * P + z] = hide;
            }
        }
    }

    /// Meshes the faces of direction `face` (`FACES`), slice by slice.
    pub(super) fn mesh_face(&mut self, face: usize, light: &[u8]) {
        let (d, sign, u, v) = FACES[face];
        self.visible_rows(d, sign);
        let sn = STRIDE[d] * sign;
        let (su, sv) = (STRIDE[u], STRIDE[v]);
        let plane_offset = usize::from(sign > 0);
        let pad = &self.pad;
        let pad_shapes = &self.pad_shapes;
        let has_shapes = self.has_shapes;
        let centre_paint = &self.centre_paint;
        let has_paint = self.has_paint;
        let centre_bare = &self.centre_bare;
        let opp_bit = 1u8 << (face ^ 1);
        let vis = &self.vis;
        let mask = &mut self.mask;

        for slice in 0..N {
            let mut any = false;
            let mut visit = |c: [usize; 3]| {
                let p = pidx(c[0], c[1], c[2]);
                let b = pad[p];
                let q = (p as isize + sn) as usize;
                let liquid = LIQUID[b as usize];
                let translucent = TRANSLUCENT[b as usize];
                let qs = if has_shapes { pad_shapes[q] } else { 0 };
                let q_hides = HIDES[pad[q] as usize]
                    && (qs == 0 || crate::shape::full_face_mask(qs, crate::shape::Corner::OuterLeft) & opp_bit != 0);
                if q_hides || liquid && LIQUID[pad[q] as usize] || translucent && pad[q] == b && qs == 0 {
                    return;
                }
                // A liquid under an open cell: its top face and the top edges of its sides sit lower.
                // Under `ICE`, water stays full height so it meets the ice block seamlessly with no air gap.
                let above = pad[p + P2] as usize;
                let surface = liquid && !LIQUID[above] && !OPAQUE[above] && above != crate::block::ICE as usize;
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
                        let drop =
                            if surface && side[1] > 0 { water::corner_drop(pad, p, side[0], side[2]) } else { 0 };
                        k = (k & 0x3FC) | (drop & 3) | (drop >> 2) << 10;
                    }
                    corners |= u64::from(k) << (i * CORNER_BITS);
                }
                let layer = if liquid {
                    u64::from(water::look(pad, p))
                } else {
                    u64::from(face_layer(b, face, c, light[q] & 15, centre_bare))
                };
                let key = KEY_FLAG
                    | (u64::from(CUTOUT[b as usize]) << KEY_CUTOUT)
                    | (u64::from(liquid) << KEY_LIQUID)
                    | (u64::from(translucent) << KEY_TRANSLUCENT)
                    | (layer << KEY_LAYER)
                    | corners;
                let paint = if has_paint {
                    crate::colour::get_face(centre_paint[index(c[0] - 1, c[1] - 1, c[2] - 1)], face)
                } else {
                    0
                };
                mask[(c[u] - 1) + (c[v] - 1) * N] = (key, paint);
                any = true;
            };
            let s = slice + 1;
            if d == 0 {
                for y in 1..=N {
                    for z in 1..=N {
                        if vis[y * P + z] >> s & 1 != 0 {
                            visit([s, y, z]);
                        }
                    }
                }
            } else {
                for r in 1..=N {
                    let (y, z) = if d == 1 { (s, r) } else { (r, s) };
                    let mut bits = vis[y * P + z];
                    while bits != 0 {
                        visit([bits.trailing_zeros() as usize, y, z]);
                        bits &= bits - 1;
                    }
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

    /// Fills `vis` for direction (`d`, `sign`): per centre row, the cells whose neighbour that way may not hide them.
    fn visible_rows(&mut self, d: usize, sign: isize) {
        for y in 1..=N {
            for z in 1..=N {
                let i = y * P + z;
                let hidden = match (d, sign > 0) {
                    (0, true) => self.hide[i] >> 1,
                    (0, false) => self.hide[i] << 1,
                    (1, true) => self.hide[i + P],
                    (1, false) => self.hide[i - P],
                    (_, true) => self.hide[i + 1],
                    (_, false) => self.hide[i - 1],
                };
                self.vis[i] = self.cand[i] & !hidden & CENTRE;
            }
        }
    }
}
