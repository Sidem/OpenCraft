//! Leaves as cards: each leaf block of the chunk becomes a few cutout cards of leaves in the shaped list
//! (`shaped.rs`), not a cube and not one smooth mass. Cards overlap their neighbours, stray a little from the grid
//! and lie as the species' leaves do (`style.rs`: clumps facing out, flat sprays, hanging curtains, arching fronds),
//! so a crown has a ragged edge and gaps, and the branches inside show through them. Their light comes from the
//! crown's shape, not the card's: the normal points out of the crown (the open cells around the leaf), so a crown is
//! lit as one rounded volume, and a leaf deep inside it is shaded. Cards are drawn from both sides.
//!
//! Invariants: a leaf block's cards depend only on the blocks within 2 of it (`tree_grid.rs`) and its world cell
//! (their scatter is a hash of it), so they are the same whichever chunk was meshed before; each chunk draws its own
//! leaves' cards and no others. To give a leaf block its own look: a row in `style.rs`.

use super::shaped::{pack, Shaped, Vert, KIND_CARD};
use super::tree_grid::{TreeGrid, STEP_Y, STEP_Z};
use super::{pidx, N};
use crate::block::{is_leaves, BlockId, ALT_TEX, FACE_TEX};
use crate::math::hash3;

mod style;
pub use style::{style_of, Lie, Style};

const SEED: u32 = 0x1EAF_CA2D;
/// How far round a leaf the crown's shape is read, in blocks.
const WINDOW: i32 = 2;
const FACES: [[i32; 3]; 6] = [[1, 0, 0], [-1, 0, 0], [0, 1, 0], [0, -1, 0], [0, 0, 1], [0, 0, -1]];

/// Every leaf block of the chunk as cards, into `out`'s cutout list. `origin` is the chunk's lowest world block.
pub fn mesh_leaves(g: &TreeGrid, light: &[u8], origin: [i32; 3], out: &mut Shaped) {
    for y in 0..N as i32 {
        for z in 0..N as i32 {
            for x in 0..N as i32 {
                let b = g.at(x, y, z);
                if is_leaves(b) {
                    leaf(g, light, origin, [x, y, z], b, out);
                }
            }
        }
    }
}

/// What a leaf sees round it: its open faces, the way out of its crown, and how deep in it it is (0..1).
struct Around {
    open: usize,
    out: [f32; 3],
    depth: f32,
}

/// `around` for a leaf in the chunk (so its window lies in the grid), reading the grid's tree mask row by row.
fn around(g: &TreeGrid, c: [i32; 3]) -> Around {
    let centre = TreeGrid::index(c[0], c[1], c[2]);
    let tree = |d: [i32; 3]| {
        let off = d[1] as isize * STEP_Y as isize + d[2] as isize * STEP_Z as isize + d[0] as isize;
        g.tree_at((centre as isize + off) as usize)
    };
    let (mut o, mut full) = ([0f32; 3], 0u32);
    for dy in -WINDOW..=WINDOW {
        for dz in -WINDOW..=WINDOW {
            let row = TreeGrid::index(c[0] - WINDOW, c[1] + dy, c[2] + dz);
            for (k, dx) in (-WINDOW..=WINDOW).enumerate() {
                if g.tree_at(row + k) {
                    full += u32::from(dx != 0 || dy != 0 || dz != 0);
                } else {
                    let d2 = (dx * dx + dy * dy + dz * dz) as f32;
                    o[0] += dx as f32 / d2;
                    o[1] += dy as f32 / d2;
                    o[2] += dz as f32 / d2;
                }
            }
        }
    }
    let len = dot(o, o).sqrt();
    let out = if len < 0.3 { unit([o[0], o[1] + 0.3, o[2]]) } else { o.map(|v| v / len) };
    let side = (2 * WINDOW + 1) as f32;
    Around { open: FACES.iter().filter(|&&d| !tree(d)).count(), out, depth: full as f32 / (side * side * side - 1.0) }
}

#[inline(never)]
fn leaf(g: &TreeGrid, light: &[u8], origin: [i32; 3], c: [i32; 3], b: BlockId, out: &mut Shaped) {
    let a = around(g, c);
    let st = style_of(b);
    let count = if a.open == 0 { st.buried } else { st.cards };
    let w = [origin[0] + c[0], origin[1] + c[1], origin[2] + c[2]];
    let card = Card {
        cell: c,
        light: light[pidx(c[0] as usize + 1, c[1] as usize + 1, c[2] as usize + 1)],
        occlusion: (7.0 - ((a.depth - 0.3).max(0.0) * 9.0)).round().clamp(1.0, 7.0) as u32,
        lit: unit([a.out[0] * 0.75, a.out[1] * 0.75 + 0.25, a.out[2] * 0.75]),
        layer: leaf_layer(b, w),
    };
    for k in 0..count {
        let h = hash3(SEED ^ k.wrapping_mul(0x9E37_79B9), w[0], w[1], w[2]);
        let h2 = hash3(h, w[2], w[0], w[1]);
        if st.lie == Lie::Fronds {
            card.frond(&st, a.out, h, h2, out);
        } else {
            card.flat(&st, a.out, h, h2, out);
        }
    }
}

/// A value in -1..=1 from five bits of `h` (`i` picks which).
fn r(h: u32, i: u32) -> f32 {
    ((h >> (i * 5)) & 31) as f32 / 31.0 * 2.0 - 1.0
}

/// What a leaf block's cards share.
struct Card {
    cell: [i32; 3],
    light: u8,
    occlusion: u32,
    lit: [f32; 3],
    layer: u16,
}

impl Card {
    /// One flat card: clumps and sprays turn round their normal; hanging ones stay upright.
    fn flat(&self, st: &Style, out_dir: [f32; 3], h: u32, h2: u32, out: &mut Shaped) {
        let mut centre = [0.5 + r(h, 0) * st.stray, 0.5 + r(h, 1) * st.stray, 0.5 + r(h, 2) * st.stray];
        let half = st.size * 0.5 * (1.0 + r(h, 3) * 0.15);
        let jit = [r(h2, 0), r(h2, 1), r(h2, 2)];
        let (right, up, hh) = match st.lie {
            Lie::Hanging => {
                let (s, c) = (r(h, 4) * std::f32::consts::PI).sin_cos();
                centre[1] -= 0.3;
                ([c, 0.0, s], [0.0, 1.0, 0.0], half * 1.4)
            }
            lie => {
                let n = if lie == Lie::Sprays {
                    unit([out_dir[0] * st.tilt + jit[0] * 0.2, 1.0, out_dir[2] * st.tilt + jit[2] * 0.2])
                } else {
                    unit(std::array::from_fn(|k| out_dir[k] + jit[k] * st.tilt))
                };
                let (e1, e2) = across(n);
                let (s, c) = (r(h, 4) * std::f32::consts::PI).sin_cos();
                let right = std::array::from_fn(|k| e1[k] * c + e2[k] * s);
                (right, cross(n, right), half)
            }
        };
        let at = |su: f32, sv: f32| std::array::from_fn(|k| centre[k] + right[k] * su * half + up[k] * sv * hh);
        let span = st.size;
        let q = [(0, 0, at(-1.0, -1.0)), (1, 0, at(1.0, -1.0)), (1, 2, at(1.0, 1.0)), (0, 2, at(-1.0, 1.0))];
        out.packed(true, q.map(|(u, v, p)| self.vert(p, u, v, h2, span)), self.layer);
    }

    /// A frond: two strips arching out of the crown and down, narrowing to its tip.
    fn frond(&self, st: &Style, out_dir: [f32; 3], h: u32, h2: u32, out: &mut Shaped) {
        let flat = (out_dir[0] * out_dir[0] + out_dir[2] * out_dir[2]).sqrt();
        let yaw =
            if flat < 0.25 { r(h, 4) * std::f32::consts::PI } else { out_dir[2].atan2(out_dir[0]) + r(h, 4) * 0.5 };
        let (s, c) = yaw.sin_cos();
        let (f, side) = ([c, 0.0, s], [-s, 0.0, c]);
        let len = st.size * (1.0 + r(h, 3) * 0.15);
        let lift = st.tilt * (0.5 + 0.5 * r(h2, 3).abs());
        let centre = [0.5 + r(h, 0) * st.stray, 0.5 + r(h, 1) * st.stray * 0.5, 0.5 + r(h, 2) * st.stray];
        let along = |p: [f32; 3], d: f32, dy: f32| [p[0] + f[0] * d, p[1] + dy, p[2] + f[2] * d];
        let p0 = along(centre, -0.2, 0.0);
        let p1 = along(p0, len * 0.5, len * 0.12 * lift);
        let p2 = along(p1, len * 0.42, -len * 0.5 * (0.4 + lift));
        let wide = |p: [f32; 3], w: f32, sgn: f32| std::array::from_fn(|k| p[k] + side[k] * w * sgn);
        let (w0, w1, w2) = (0.12, len * 0.17, 0.05);
        for (a, b, wa, wb, va, vb) in [(p0, p1, w0, w1, 0, 1), (p1, p2, w1, w2, 1, 2)] {
            let q = [
                (0, va, wide(a, wa, -1.0)),
                (1, va, wide(a, wa, 1.0)),
                (1, vb, wide(b, wb, 1.0)),
                (0, vb, wide(b, wb, -1.0)),
            ];
            out.packed(true, q.map(|(u, v, p)| self.vert(p, u, v, h2, len)), self.layer);
        }
    }

    /// A card vertex: `u` 0 or 1 across, `v` 0, 1 or 2 halves along, `span` the blocks its texture covers.
    fn vert(&self, off: [f32; 3], u: u32, v: u32, h2: u32, span: f32) -> [u32; 3] {
        let bits = u | v << 1 | (h2 >> 20 & 15) << 3 | ((span * 8.0).round() as u32).min(31) << 7;
        let vert = Vert {
            cell: self.cell,
            off,
            normal: self.lit,
            u: bits as f32 / 256.0,
            kind: KIND_CARD,
            occlusion: self.occlusion,
            light: self.light,
            layer: 0,
        };
        pack(&vert)
    }
}

/// The leaf texture of block `b` at world cell `w`: one of its alternates, by a hash of the cell.
fn leaf_layer(b: BlockId, w: [i32; 3]) -> u16 {
    let (base, alt) = (FACE_TEX[b as usize][0], ALT_TEX[b as usize][0]);
    match (alt, hash3(SEED, w[0], w[1], w[2]) >> 30) {
        (0, _) | (_, 0) => base,
        (alt, k) => alt + k as u16 - 1,
    }
}

/// Two unit vectors across `n`.
fn across(n: [f32; 3]) -> ([f32; 3], [f32; 3]) {
    let up = if n[1].abs() < 0.9 { [0.0, 1.0, 0.0] } else { [1.0, 0.0, 0.0] };
    let e1 = unit(cross(up, n));
    (e1, cross(n, e1))
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

#[cfg(test)]
mod tests;
