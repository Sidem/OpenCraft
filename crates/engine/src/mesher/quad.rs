//! Per-corner and per-quad helpers for the mesher: ambient occlusion, corner light, the greedy merge key
//! and writing a finished quad. Invariant: a key holds everything that must match for two faces to merge.

use crate::block::{BlockId, OPAQUE};
/// Classic voxel AO: two occluding sides fully darken the corner regardless of the diagonal.
#[inline]
fn ao(side1: bool, side2: bool, corner: bool) -> u32 {
    if side1 && side2 {
        0
    } else {
        3 - (side1 as u32 + side2 as u32 + corner as u32)
    }
}

/// The merge key: 4 corners of `CORNER_BITS` (AO 2 bits, light 8 bits, then 2 bits only a liquid corner uses:
/// the high bits of its drop, `water.rs`), then 12-bit layer, cutout, liquid, translucent, flag.
pub(super) const CORNER_BITS: usize = 12;
pub(super) const KEY_LAYER: usize = 4 * CORNER_BITS;
pub(super) const KEY_CUTOUT: usize = KEY_LAYER + 12;
pub(super) const KEY_LIQUID: usize = KEY_CUTOUT + 1;
pub(super) const KEY_TRANSLUCENT: usize = KEY_LIQUID + 1;
pub(super) const KEY_FLAG: u64 = 1 << 63;

pub struct MeshOutput {
    /// The vertices, opaque quads first, then cutout, liquid, translucent; then their
    /// `[light, paint | (layer >> 9) << 5]` byte pairs in the same order, packed two vertices per `u32`
    /// (little-endian, so bytes read in order).
    pub verts: Vec<u32>,
    pub opaque_quads: u32,
    pub cutout_quads: u32,
    pub liquid_quads: u32,
    pub translucent_quads: u32,
    /// The shaped list (`shaped.rs`): opaque quads, then cutout, then translucent, three `u32` a vertex.
    pub shaped: Vec<u32>,
    pub shaped_opaque: u32,
    pub shaped_cutout: u32,
    pub shaped_translucent: u32,
}

#[cfg(test)]
impl MeshOutput {
    pub fn vertex_count(&self) -> usize {
        (self.opaque_quads + self.cutout_quads + self.liquid_quads + self.translucent_quads) as usize * 4
    }

    /// The light byte of vertex `i`.
    pub fn light(&self, i: usize) -> u8 {
        (self.verts[self.vertex_count() + i / 2] >> ((i % 2) * 16)) as u8
    }

    /// The paint palette index (`0..=16`) of vertex `i`.
    pub fn paint(&self, i: usize) -> u8 {
        ((self.verts[self.vertex_count() + i / 2] >> ((i % 2) * 16 + 8)) as u8) & 31
    }

    /// The 12-bit texture array layer (`0..=4095`) of non-liquid vertex `i`.
    pub fn layer(&self, i: usize) -> u16 {
        let lo = (self.verts[i] >> 23) as u16;
        let hi = (((self.verts[self.vertex_count() + i / 2] >> ((i % 2) * 16 + 8)) as u8) >> 5) as u16;
        lo | (hi << 9)
    }
}

pub(super) fn corners_of(key: u64) -> [u32; 4] {
    std::array::from_fn(|i| (key >> (i * CORNER_BITS)) as u32 & ((1 << CORNER_BITS) - 1))
}

/// AO | light << 2 for the face corner towards (`du`, `dv`) of air cell `q`. The light is the
/// average of the non-opaque cells among `q`, its two sides and the diagonal between them (the
/// diagonal only when a side is open), sky and block light each rounded to a nibble.
#[inline]
pub(super) fn corner(pad: &[BlockId], light: &[u8], q: usize, du: isize, dv: isize) -> u32 {
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

pub(super) fn emit_quad(
    (out, lights): (&mut Vec<u32>, &mut Vec<u8>),
    face: usize,
    (d, u, v): (usize, usize, usize),
    plane: u32,
    (bu, bv): (u32, u32),
    (w, h): (u32, u32),
    (key, paint): (u64, u8),
) {
    let layer = (key >> KEY_LAYER) as u32 & 0xFFF; // for a liquid quad: its kind and direction (`water.rs`)
    let lo = layer & 0x1FF;
    let attr = (paint & 31) | (((layer >> 9) as u8) << 5);
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
        let low = lo | (corner[i] >> 10) << 5; // a liquid vertex's layer bits hold its drop's high bits
        vs[i] = c[0] | (c[1] << 6) | (c[2] << 12) | ((face as u32) << 18) | (a << 21) | (low << 23);
        ls[i] = l;
        bright[i] = a * 16 + u32::from((l & 15).max(l >> 4));
    }
    // Split the quad along the brighter diagonal to avoid the anisotropic AO and light seam.
    if bright[0] + bright[2] < bright[1] + bright[3] {
        out.extend_from_slice(&[vs[1], vs[2], vs[3], vs[0]]);
        lights.extend_from_slice(&[ls[1], attr, ls[2], attr, ls[3], attr, ls[0], attr]);
    } else {
        out.extend_from_slice(&vs);
        lights.extend_from_slice(&[ls[0], attr, ls[1], attr, ls[2], attr, ls[3], attr]);
    }
}
