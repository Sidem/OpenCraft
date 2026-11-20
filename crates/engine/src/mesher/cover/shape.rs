//! 3D botanical geometry for ground cover and ore-indicator plants (`mesher/cover.rs`), emitted into the chunk's
//! shaped cutout list (`mesher/shaped.rs`, `KIND_PLANT`). Each plant archetype gets a compact, low-profile 3D shape
//! matched to its texture layout (`tools/art/plants.py` and `textures/hires.rs`) so flowers face upward without
//! X-cross slicing, ferns arch in rosettes, lichen hugs rock flat, and meadow cover stays clear of the player's view.
//!
//! UV contract (6-bit u and v in 60ths, v = 0 top of tile, v = 60 ground):
//! - Full tile (`tuft`, `bluebell`, `fern`, `reed`, `dry_brush`, `lichen`, `copper_bell`): `0..=60` × `0..=60`.
//! - Split 2×2 tile (`daisy`, `poppy`, `rust_bloom`, `pale_star`, `coal_shrub`): top-left and top-right (`1..29` /
//!   `31..59` × `1..29`) are two upward-facing blossom or foliage sprig variants; bottom-left (`1..29` × `32..60`) is
//!   the stem rooted at bottom-centre; bottom-right (`31..59` × `32..60`) is the basal leaf rosette or second branch.

use super::super::shaped::{pack, Shaped, Vert, KIND_PLANT};
use crate::block::tex;
use std::f32::consts::{FRAC_PI_2, FRAC_PI_3, PI, TAU};

#[derive(Clone, Copy)]
struct Uv {
    u0: u32,
    u1: u32,
    v0: u32,
    v1: u32,
}

const FULL: Uv = Uv { u0: 0, u1: 60, v0: 0, v1: 60 };
const HEAD_A: Uv = Uv { u0: 1, u1: 29, v0: 1, v1: 29 };
const HEAD_B: Uv = Uv { u0: 31, u1: 59, v0: 1, v1: 29 };
const STEM: Uv = Uv { u0: 1, u1: 29, v0: 32, v1: 60 };
const BASAL: Uv = Uv { u0: 31, u1: 59, v0: 32, v1: 60 };

/// Proportions of a multi-blossom wildflower cluster (`daisy`, `poppy`, `rust_bloom`, `pale_star`).
struct BlossomSpec {
    min_count: u32,
    extra_permille: u32,
    h: (f32, f32),
    head: (f32, f32),
    tilt: (f32, f32),
    spread: f32,
    stem_w: f32,
    basal: (f32, f32),
}

const DAISY_SPEC: BlossomSpec = BlossomSpec {
    min_count: 3,
    extra_permille: 450,
    h: (0.22, 0.36),
    head: (0.17, 0.23),
    tilt: (0.16, 0.44),
    spread: 0.23,
    stem_w: 0.19,
    basal: (0.38, 0.16),
};
const POPPY_SPEC: BlossomSpec = BlossomSpec {
    min_count: 2,
    extra_permille: 700,
    h: (0.26, 0.42),
    head: (0.19, 0.26),
    tilt: (0.24, 0.56),
    spread: 0.22,
    stem_w: 0.20,
    basal: (0.38, 0.17),
};
const RUST_SPEC: BlossomSpec = BlossomSpec {
    min_count: 3,
    extra_permille: 300,
    h: (0.25, 0.40),
    head: (0.20, 0.26),
    tilt: (0.18, 0.46),
    spread: 0.23,
    stem_w: 0.20,
    basal: (0.42, 0.18),
};
const PALE_SPEC: BlossomSpec = BlossomSpec {
    min_count: 3,
    extra_permille: 300,
    h: (0.28, 0.44),
    head: (0.19, 0.25),
    tilt: (0.16, 0.42),
    spread: 0.23,
    stem_w: 0.19,
    basal: (0.36, 0.15),
};

/// Appends the 3D shaped quads for one ground-cover plant in chunk cell `cell` using texture `layer`.
pub(super) fn push(out: &mut Shaped, cell: [i32; 3], layer: u16, light: u8, h: u32) {
    let mut b = Builder { out, cell, layer, light, rng: h | 1 };
    match layer {
        tex::GRASS_TUFT => b.tuft(),
        tex::DAISY => b.blossom(&DAISY_SPEC),
        tex::POPPY => b.blossom(&POPPY_SPEC),
        tex::INDICATOR_IRON => b.blossom(&RUST_SPEC),
        tex::INDICATOR_PALE => b.blossom(&PALE_SPEC),
        tex::BLUEBELL => b.arching(0.22, 0.26, 0.38, 0.34, 0.44),
        tex::INDICATOR_COPPER => b.arching(0.24, 0.40, 0.54, 0.36, 0.46),
        tex::FERN => b.fern(),
        tex::REED => b.reed(),
        tex::DRY_BRUSH => b.dry_brush(),
        tex::LICHEN => b.lichen(),
        tex::INDICATOR_COAL => b.coal_shrub(),
        _ => b.tuft(),
    }
}

struct Builder<'a> {
    out: &'a mut Shaped,
    cell: [i32; 3],
    layer: u16,
    light: u8,
    rng: u32,
}

impl Builder<'_> {
    /// Next pseudo-random float in `[0, 1)`.
    fn next(&mut self) -> f32 {
        let mut x = self.rng;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        self.rng = x;
        (x >> 8) as f32 * (1.0 / 16_777_216.0)
    }

    fn range(&mut self, lo: f32, hi: f32) -> f32 {
        lo + (hi - lo) * self.next()
    }

    /// Packs a single shaped plant vertex with UV in 60ths.
    fn vert(&self, off: [f32; 3], normal: [f32; 3], u: u32, v: u32, occlusion: u32) -> [u32; 3] {
        let bits = (u.min(60)) | (v.min(60) << 6);
        pack(&Vert {
            cell: self.cell,
            off,
            normal,
            u: bits as f32 / 256.0,
            kind: KIND_PLANT,
            occlusion,
            light: self.light,
            layer: 0,
        })
    }

    /// Emits a quad (b0, b1, t1, t0) with ground occlusion `occ_b` and top occlusion `occ_t`.
    #[allow(clippy::too_many_arguments)]
    fn quad(&mut self, p: [[f32; 3]; 4], n: [f32; 3], uv: Uv, occ_b: u32, occ_t: u32) {
        let q = [
            self.vert(p[0], n, uv.u0, uv.v1, occ_b),
            self.vert(p[1], n, uv.u1, uv.v1, occ_b),
            self.vert(p[2], n, uv.u1, uv.v0, occ_t),
            self.vert(p[3], n, uv.u0, uv.v0, occ_t),
        ];
        self.out.packed(true, q, self.layer);
    }

    /// Standing or leaning card centred at `base`, width `w` along `dir`, height `h`, top shifted by `lean`.
    #[allow(clippy::too_many_arguments)]
    fn card(&mut self, base: [f32; 3], dir: [f32; 3], w: f32, h: f32, lean: [f32; 3], uv: Uv, occ_b: u32, occ_t: u32) {
        let hw = w * 0.5;
        let b0 = [base[0] - dir[0] * hw, base[1], base[2] - dir[2] * hw];
        let b1 = [base[0] + dir[0] * hw, base[1], base[2] + dir[2] * hw];
        let t0 = [b0[0] + lean[0], base[1] + h + lean[1], b0[2] + lean[2]];
        let t1 = [b1[0] + lean[0], base[1] + h + lean[1], b1[2] + lean[2]];
        // Upward-biased normal so foliage catches sunlight like the turf rather than side-shadowing darkly.
        let n = unit([lean[0] * 0.6, 1.0, lean[2] * 0.6]);
        self.quad([b0, b1, t1, t0], n, uv, occ_b, occ_t);
    }

    /// An upward-facing tilted disc/square (flower head, shrub sprig, lichen patch) centred at `c`.
    fn disc(&mut self, c: [f32; 3], size: f32, normal: [f32; 3], spin: f32, uv: Uv, occ: u32) {
        let n = unit(normal);
        let up = if n[1].abs() < 0.9 { [0.0, 1.0, 0.0] } else { [1.0, 0.0, 0.0] };
        let e1 = unit(cross(up, n));
        let e2 = cross(n, e1);
        let (s, co) = spin.sin_cos();
        let r = size * 0.5;
        let rx = [e1[0] * co + e2[0] * s, e1[1] * co + e2[1] * s, e1[2] * co + e2[2] * s];
        let ry = [e2[0] * co - e1[0] * s, e2[1] * co - e1[1] * s, e2[2] * co - e1[2] * s];
        let at = |sx: f32, sy: f32| {
            [
                c[0] + (rx[0] * sx + ry[0] * sy) * r,
                c[1] + (rx[1] * sx + ry[1] * sy) * r,
                c[2] + (rx[2] * sx + ry[2] * sy) * r,
            ]
        };
        self.quad([at(-1.0, -1.0), at(1.0, -1.0), at(1.0, 1.0), at(-1.0, 1.0)], n, uv, occ, occ);
    }

    /// Low, airy meadow grass tuft: 3 slightly splayed blade cards at 60° intervals, jittered off the block grid.
    fn tuft(&mut self) {
        let cx = self.range(0.32, 0.68);
        let cz = self.range(0.32, 0.68);
        let yaw0 = self.range(0.0, TAU);
        let h_base = self.range(0.25, 0.38);
        let w_base = self.range(0.54, 0.72);
        for k in 0..3 {
            let a = yaw0 + k as f32 * FRAC_PI_3 + self.range(-0.18, 0.18);
            let (s, c) = a.sin_cos();
            let dir = [c, 0.0, s];
            let out = [-s, 0.0, c];
            let shift = self.range(-0.06, 0.06);
            let splay = self.range(0.04, 0.10) * if k == 1 { -1.0 } else { 1.0 };
            let h = h_base * self.range(0.86, 1.12);
            let w = w_base * self.range(0.90, 1.08);
            let base = [cx + out[0] * shift, 0.0, cz + out[2] * shift];
            self.card(base, dir, w, h, [out[0] * splay, 0.0, out[2] * splay], FULL, 4, 7);
        }
    }

    /// Wildflower cluster: low basal leaf rosette plus 2–4 slender stems each topped by an upward-facing tilted blossom.
    fn blossom(&mut self, spec: &BlossomSpec) {
        let cx = self.range(0.36, 0.64);
        let cz = self.range(0.36, 0.64);
        let yaw0 = self.range(0.0, TAU);
        // Basal leaf rosette hugging the ground.
        for k in 0..2 {
            let a = yaw0 + k as f32 * FRAC_PI_2 + self.range(-0.2, 0.2);
            let (s, c) = a.sin_cos();
            self.card([cx, 0.0, cz], [c, 0.0, s], spec.basal.0, spec.basal.1, [0.0, 0.0, 0.0], BASAL, 4, 6);
        }
        let extra = u32::from((self.next() * 1000.0) < spec.extra_permille as f32);
        let count = spec.min_count + extra;
        for i in 0..count {
            let az = yaw0 + i as f32 * (TAU / count as f32) + self.range(-0.38, 0.38);
            let (sa, ca) = az.sin_cos();
            let r_base = self.range(0.03, 0.09);
            let r_head = self.range(spec.spread * 0.55, spec.spread);
            let h = self.range(spec.h.0, spec.h.1);
            let base = [cx + ca * r_base, 0.0, cz + sa * r_base];
            let head = [cx + ca * r_head, h, cz + sa * r_head];
            // Two orthogonal narrow stem quads running straight from `base` to tucked right under `head`.
            let lean = [head[0] - base[0], -0.024, head[2] - base[2]];
            let sa0 = az + self.range(-0.4, 0.4);
            for k in 0..2 {
                let (ss, cc) = (sa0 + k as f32 * FRAC_PI_2).sin_cos();
                self.card(base, [cc, 0.0, ss], spec.stem_w, h, lean, STEM, 4, 7);
            }
            // Upward-facing blossom tilted gently outward so it catches the sun and reads from both above and the side.
            let tilt = self.range(spec.tilt.0, spec.tilt.1);
            let (st, ct) = tilt.sin_cos();
            let n = [ca * st, ct, sa * st];
            let head_size = self.range(spec.head.0, spec.head.1);
            let spin = self.range(0.0, TAU);
            let uv = if i & 1 == 0 { HEAD_A } else { HEAD_B };
            self.disc(head, head_size, n, spin, uv, 7);
        }
    }

    /// Outward-radiating fountain of 3 arching bellflower stems rooted together at `root_u` and fanning 120° apart.
    fn arching(&mut self, root_u: f32, h_min: f32, h_max: f32, w_min: f32, w_max: f32) {
        let cx = self.range(0.38, 0.62);
        let cz = self.range(0.38, 0.62);
        let yaw0 = self.range(0.0, TAU);
        for k in 0..3 {
            let a = yaw0 + k as f32 * (TAU / 3.0) + self.range(-0.22, 0.22);
            let (s, c) = a.sin_cos();
            let dir = [c, 0.0, s];
            let perp = [-s, 0.0, c];
            let h = self.range(h_min, h_max);
            let w = self.range(w_min, w_max);
            let b0 = [cx - dir[0] * (root_u * w), 0.0, cz - dir[2] * (root_u * w)];
            let b1 = [cx + dir[0] * ((1.0 - root_u) * w), 0.0, cz + dir[2] * ((1.0 - root_u) * w)];
            let out_lean = h * 0.14;
            let side_lean = self.range(-0.05, 0.05);
            let top = |p: [f32; 3]| {
                [p[0] + dir[0] * out_lean + perp[0] * side_lean, h, p[2] + dir[2] * out_lean + perp[2] * side_lean]
            };
            let n = unit([dir[0] * 0.35 + perp[0] * 0.2, 0.92, dir[2] * 0.35 + perp[2] * 0.2]);
            self.quad([b0, b1, top(b1), top(b0)], n, FULL, 4, 7);
        }
    }

    /// True 3D woodland fern rosette: 5 two-segment arching fronds radiating low and wide from a central crown.
    fn fern(&mut self) {
        let cx = self.range(0.38, 0.62);
        let cz = self.range(0.38, 0.62);
        let yaw0 = self.range(0.0, TAU);
        let count = 5;
        for k in 0..count {
            let a = yaw0 + k as f32 * (TAU / count as f32) + self.range(-0.20, 0.20);
            let (s, c) = a.sin_cos();
            let dir = [c, 0.0, s];
            let side = [-s, 0.0, c];
            let len = self.range(0.42, 0.56);
            let hw = self.range(0.11, 0.145);
            // Alternate slightly steeper inner fronds and lower outer fronds for a natural layered bowl.
            let inner = k & 1 == 0;
            let lift1 = len * if inner { self.range(0.46, 0.54) } else { self.range(0.34, 0.42) };
            let lift2 = len * if inner { self.range(0.14, 0.22) } else { self.range(0.02, 0.08) };
            let p0 = [cx, 0.01, cz];
            let p1 = [cx + dir[0] * len * 0.48, lift1, cz + dir[2] * len * 0.48];
            let p2 = [p1[0] + dir[0] * len * 0.48, lift1 + lift2, p1[2] + dir[2] * len * 0.48];
            let wide = |p: [f32; 3], sgn: f32| [p[0] + side[0] * hw * sgn, p[1], p[2] + side[2] * hw * sgn];
            let n1 = unit([-dir[0] * 0.35, 0.94, -dir[2] * 0.35]);
            let n2 = unit([dir[0] * 0.15, 0.98, dir[2] * 0.15]);
            self.quad(
                [wide(p0, -1.0), wide(p0, 1.0), wide(p1, 1.0), wide(p1, -1.0)],
                n1,
                Uv { u0: 0, u1: 60, v0: 30, v1: 60 },
                4,
                6,
            );
            self.quad(
                [wide(p1, -1.0), wide(p1, 1.0), wide(p2, 1.0), wide(p2, -1.0)],
                n2,
                Uv { u0: 0, u1: 60, v0: 0, v1: 30 },
                6,
                7,
            );
        }
    }

    /// Slender riverside cattail & sedge clump: 3 slightly offset cards of stepped heights with gentle breeze lean.
    fn reed(&mut self) {
        let cx = self.range(0.38, 0.62);
        let cz = self.range(0.38, 0.62);
        let yaw0 = self.range(0.0, TAU);
        for k in 0..3 {
            let a = yaw0 + k as f32 * (TAU / 3.0) + self.range(-0.25, 0.25);
            let (s, c) = a.sin_cos();
            let off = self.range(0.06, 0.15);
            let bx = cx + c * off;
            let bz = cz + s * off;
            let h = self.range(0.62, 0.90) * if k == 0 { 1.04 } else { 0.92 };
            let w = self.range(0.38, 0.50);
            let lean = [self.range(-0.07, 0.07), 0.0, self.range(-0.07, 0.07)];
            let (ds, dc) = (a + FRAC_PI_2).sin_cos();
            self.card([bx, 0.0, bz], [dc, 0.0, ds], w, h, lean, FULL, 4, 7);
        }
    }

    /// Splayed desert & basalt dry scrub: 3 branching twig cards fanning outward at 120°.
    fn dry_brush(&mut self) {
        let cx = self.range(0.34, 0.66);
        let cz = self.range(0.34, 0.66);
        let yaw0 = self.range(0.0, TAU);
        for k in 0..3 {
            let a = yaw0 + k as f32 * (TAU / 3.0) + self.range(-0.25, 0.25);
            let (s, c) = a.sin_cos();
            let out = [-s, 0.0, c];
            let h = self.range(0.30, 0.45);
            let w = self.range(0.42, 0.56);
            let tilt = self.range(0.06, 0.13);
            self.card([cx, 0.0, cz], [c, 0.0, s], w, h, [out[0] * tilt, 0.0, out[2] * tilt], FULL, 4, 7);
        }
    }

    /// Flat ground-hugging foliose rock lichen lying just above the rock surface (avoids z-fighting).
    fn lichen(&mut self) {
        let cx = self.range(0.34, 0.66);
        let cz = self.range(0.34, 0.66);
        let size = self.range(0.54, 0.80);
        let spin = self.range(0.0, TAU);
        self.disc([cx, 0.018, cz], size, [0.0, 1.0, 0.0], spin, FULL, 6);
        if self.next() < 0.5 {
            let a = self.range(0.0, TAU);
            let d = self.range(0.14, 0.24);
            let sub = size * self.range(0.55, 0.72);
            let sx = (cx + a.cos() * d).clamp(0.22, 0.78);
            let sz = (cz + a.sin() * d).clamp(0.22, 0.78);
            self.disc([sx, 0.025, sz], sub, [0.0, 1.0, 0.0], spin + 1.9, FULL, 6);
        }
    }

    /// Coal-indicator woody heath shrub: crossed dark branches at the base crowned by 4 domed foliage & berry sprigs.
    fn coal_shrub(&mut self) {
        let cx = self.range(0.36, 0.64);
        let cz = self.range(0.36, 0.64);
        let yaw0 = self.range(0.0, TAU);
        let stem_h = self.range(0.24, 0.32);
        let stem_w = self.range(0.28, 0.36);
        for (k, uv) in [(0, STEM), (1, BASAL)] {
            let (s, c) = (yaw0 + k as f32 * FRAC_PI_2).sin_cos();
            self.card([cx, 0.0, cz], [c, 0.0, s], stem_w, stem_h, [0.0, 0.0, 0.0], uv, 4, 6);
        }
        // Three angled side foliage sprigs + one top crown sprig forming a compact 3D dark-berry bush.
        for k in 0..3 {
            let a = yaw0 + k as f32 * (TAU / 3.0) + self.range(-0.25, 0.25);
            let (s, c) = a.sin_cos();
            let rad = self.range(0.11, 0.16);
            let pos = [cx + c * rad, stem_h * self.range(0.75, 0.98), cz + s * rad];
            let n = [c * 0.65, 0.76, s * 0.65];
            let size = self.range(0.30, 0.38);
            let spin = self.range(0.0, TAU);
            let uv = if k & 1 == 0 { HEAD_A } else { HEAD_B };
            self.disc(pos, size, n, spin, uv, 6);
        }
        let tilt = self.range(0.14, 0.30);
        let ta = yaw0 + PI * 0.7;
        let cap_n = [ta.cos() * tilt.sin(), tilt.cos(), ta.sin() * tilt.sin()];
        let cap_y = stem_h + self.range(0.06, 0.11);
        let cap_size = self.range(0.32, 0.40);
        let cap_spin = self.range(0.0, TAU);
        self.disc([cx, cap_y, cz], cap_size, cap_n, cap_spin, HEAD_A, 7);
    }
}

fn cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]]
}

fn unit(a: [f32; 3]) -> [f32; 3] {
    let l = (a[0] * a[0] + a[1] * a[1] + a[2] * a[2]).sqrt().max(1e-6);
    [a[0] / l, a[1] / l, a[2] / l]
}
