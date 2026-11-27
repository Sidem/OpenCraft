//! 3D botanical geometry for ground cover and ore-indicator plants (`mesher/cover.rs`), emitted into the chunk's
//! shaped cutout list (`mesher/shaped.rs`, `KIND_PLANT`). Each archetype gets a compact, sculpted multi-segment 3D
//! shape matched to its texture layout (`tools/art/plants.py`, `textures/hires.rs`) so blossoms form 3D domes/cups on
//! curved stems, bellflowers and reeds arch in two segments, ferns sweep in three-segment rosettes, coal shrubs build
//! peaked foliage domes, and `shaped.ts` sways them smoothly in the wind and parts them around the player's feet.
//!
//! UV contract (6-bit u and v in 60ths, v = 0 top of tile, v = 60 ground):
//! - Full tile (`tuft`, `bluebell`, `fern`, `reed`, `dry_brush`, `lichen`, `copper_bell`): `0..=60` × `0..=60`.
//! - Split 2×2 tile (`daisy`, `poppy`, `rust_bloom`, `pale_star`, `coal_shrub`): top-left and top-right (`1..29` /
//!   `31..59` × `1..29`) are two blossom/foliage variants; bottom-left (`1..29` × `32..60`) is the stem; bottom-right
//!   (`31..59` × `32..60`) is the basal leaf rosette or second woody branch.

use super::super::shaped::{pack, Shaped, Vert, KIND_PLANT};
use crate::block::tex;
use std::f32::consts::{FRAC_PI_2, PI, TAU};

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
const STEM_LO: Uv = Uv { u0: 1, u1: 29, v0: 46, v1: 60 };
const STEM_HI: Uv = Uv { u0: 1, u1: 29, v0: 32, v1: 46 };
const STEM: Uv = Uv { u0: 1, u1: 29, v0: 32, v1: 60 };
const BASAL: Uv = Uv { u0: 31, u1: 59, v0: 32, v1: 60 };

/// Proportions of a multi-blossom wildflower cluster (`daisy`, `poppy`, `rust_bloom`, `pale_star`).
/// Positive `cup` lifts outer petals into a chalice (poppy); negative domes the centre above relaxed petals (daisy).
struct BlossomSpec {
    min_count: u32,
    extra_permille: u32,
    h: (f32, f32),
    head: (f32, f32),
    tilt: (f32, f32),
    spread: f32,
    stem_w: f32,
    basal: (f32, f32),
    cup: f32,
}

const DAISY_SPEC: BlossomSpec = BlossomSpec {
    min_count: 3,
    extra_permille: 450,
    h: (0.22, 0.36),
    head: (0.18, 0.24),
    tilt: (0.16, 0.44),
    spread: 0.24,
    stem_w: 0.18,
    basal: (0.34, 0.15),
    cup: -0.020,
};
const POPPY_SPEC: BlossomSpec = BlossomSpec {
    min_count: 2,
    extra_permille: 700,
    h: (0.26, 0.40),
    head: (0.20, 0.27),
    tilt: (0.22, 0.52),
    spread: 0.23,
    stem_w: 0.19,
    basal: (0.34, 0.16),
    cup: 0.028,
};
const RUST_SPEC: BlossomSpec = BlossomSpec {
    min_count: 3,
    extra_permille: 300,
    h: (0.25, 0.39),
    head: (0.21, 0.27),
    tilt: (0.18, 0.44),
    spread: 0.24,
    stem_w: 0.19,
    basal: (0.38, 0.17),
    cup: 0.024,
};
const PALE_SPEC: BlossomSpec = BlossomSpec {
    min_count: 3,
    extra_permille: 300,
    h: (0.27, 0.42),
    head: (0.20, 0.26),
    tilt: (0.16, 0.40),
    spread: 0.24,
    stem_w: 0.18,
    basal: (0.34, 0.15),
    cup: -0.020,
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
        tex::BLUEBELL => b.arching(0.22, 0.26, 0.39, 0.34, 0.44),
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

    fn vert(&self, off: [f32; 3], normal: [f32; 3], u: u32, v: u32, occlusion: u32) -> [u32; 3] {
        let bits = u.min(60) | (v.min(60) << 6);
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
        let n = unit([lean[0] * 0.6, 1.0, lean[2] * 0.6]);
        self.quad([b0, b1, t1, t0], n, uv, occ_b, occ_t);
    }

    /// Curved 2-segment card from `base` through `mid_off` to `top_off`, splitting `uv` vertically at its midpoint.
    #[allow(clippy::too_many_arguments)]
    fn bent_card(&mut self, base: [f32; 3], dir: [f32; 3], w: f32, mid: [f32; 3], top: [f32; 3], uv: Uv) {
        let hw = w * 0.5;
        let p =
            |o: [f32; 3], s: f32| [base[0] + o[0] + dir[0] * hw * s, base[1] + o[1], base[2] + o[2] + dir[2] * hw * s];
        let vm = (uv.v0 + uv.v1) / 2;
        let n1 = unit([mid[0] * 0.45, 1.0, mid[2] * 0.45]);
        let n2 = unit([(top[0] - mid[0]) * 0.65, 1.0, (top[2] - mid[2]) * 0.65]);
        self.quad([p([0.0; 3], -1.0), p([0.0; 3], 1.0), p(mid, 1.0), p(mid, -1.0)], n1, Uv { v0: vm, ..uv }, 4, 6);
        self.quad([p(mid, -1.0), p(mid, 1.0), p(top, 1.0), p(top, -1.0)], n2, Uv { v1: vm, ..uv }, 6, 7);
    }

    /// Tangent basis `(rx, ry, n)` for a disc tilted along `normal` and rotated by `spin`.
    fn frame(normal: [f32; 3], spin: f32) -> ([f32; 3], [f32; 3], [f32; 3]) {
        let n = unit(normal);
        let up = if n[1].abs() < 0.9 { [0.0, 1.0, 0.0] } else { [1.0, 0.0, 0.0] };
        let e1 = unit(cross(up, n));
        let e2 = cross(n, e1);
        let (s, co) = spin.sin_cos();
        let rx = [e1[0] * co + e2[0] * s, e1[1] * co + e2[1] * s, e1[2] * co + e2[2] * s];
        let ry = [e2[0] * co - e1[0] * s, e2[1] * co - e1[1] * s, e2[2] * co - e1[2] * s];
        (rx, ry, n)
    }

    fn disc(&mut self, c: [f32; 3], size: f32, normal: [f32; 3], spin: f32, uv: Uv, occ: u32) {
        let (rx, ry, n) = Self::frame(normal, spin);
        let r = size * 0.5;
        let at = |sx: f32, sy: f32| add3(c, scale3(rx, sx * r), scale3(ry, sy * r));
        self.quad([at(-1.0, -1.0), at(1.0, -1.0), at(1.0, 1.0), at(-1.0, 1.0)], n, uv, occ, occ);
    }

    /// True 3D 4-quad cupped chalice (`cup > 0`) or domed corolla (`cup < 0`) split 2×2 across `uv`.
    #[allow(clippy::too_many_arguments)]
    fn cup_disc(&mut self, c: [f32; 3], size: f32, normal: [f32; 3], spin: f32, uv: Uv, cup: f32, occ: u32) {
        let (rx, ry, n) = Self::frame(normal, spin);
        let r = size * 0.5;
        let um = (uv.u0 + uv.u1) / 2;
        let vm = (uv.v0 + uv.v1) / 2;
        let pt = |sx: f32, sy: f32| {
            // Radial lift: centre shifts by `-0.55 * cup`, mid-edges by `+0.35 * cup`, outer corners by `+cup`.
            let lift = match (sx == 0.0, sy == 0.0) {
                (true, true) => -0.55 * cup,
                (false, false) => cup,
                _ => 0.35 * cup,
            };
            add3(add3(c, scale3(rx, sx * r), scale3(ry, sy * r)), scale3(n, lift), [0.0; 3])
        };
        let sub = [
            (-1.0, -1.0, 0.0, 0.0, Uv { u0: uv.u0, u1: um, v0: vm, v1: uv.v1 }),
            (0.0, -1.0, 1.0, 0.0, Uv { u0: um, u1: uv.u1, v0: vm, v1: uv.v1 }),
            (0.0, 0.0, 1.0, 1.0, Uv { u0: um, u1: uv.u1, v0: uv.v0, v1: vm }),
            (-1.0, 0.0, 0.0, 1.0, Uv { u0: uv.u0, u1: um, v0: uv.v0, v1: vm }),
        ];
        for (x0, y0, x1, y1, suv) in sub {
            let mx = (x0 + x1) * 0.5;
            let my = (y0 + y1) * 0.5;
            let tilt = -cup * 9.0;
            let qn = unit(add3(n, scale3(rx, mx * tilt), scale3(ry, my * tilt)));
            self.quad([pt(x0, y0), pt(x1, y0), pt(x1, y1), pt(x0, y1)], qn, suv, occ, occ);
        }
    }

    /// 3D peaked 2-quad foliage dome for woody shrub crowns (adds volume from low side views).
    #[allow(clippy::too_many_arguments)]
    fn ridge_disc(&mut self, c: [f32; 3], size: f32, normal: [f32; 3], spin: f32, uv: Uv, peak: f32, occ: u32) {
        let (rx, ry, n) = Self::frame(normal, spin);
        let r = size * 0.5;
        let um = (uv.u0 + uv.u1) / 2;
        let pt =
            |sx: f32, sy: f32, dz: f32| add3(add3(c, scale3(rx, sx * r), scale3(ry, sy * r)), scale3(n, dz), [0.0; 3]);
        let n_l = unit(add3(n, scale3(rx, -0.32), [0.0; 3]));
        let n_r = unit(add3(n, scale3(rx, 0.32), [0.0; 3]));
        self.quad(
            [pt(-1.0, -1.0, -peak * 0.3), pt(0.0, -1.0, peak), pt(0.0, 1.0, peak), pt(-1.0, 1.0, -peak * 0.3)],
            n_l,
            Uv { u1: um, ..uv },
            occ,
            occ,
        );
        self.quad(
            [pt(0.0, -1.0, peak), pt(1.0, -1.0, -peak * 0.3), pt(1.0, 1.0, -peak * 0.3), pt(0.0, 1.0, peak)],
            n_r,
            Uv { u0: um, ..uv },
            occ,
            occ,
        );
    }

    /// Curved meadow grass tuft (4 two-segment outward-arching sprays; hidden when 3D Lush Grass is active).
    fn tuft(&mut self) {
        let (cx, cz, yaw0) = (self.range(0.32, 0.68), self.range(0.32, 0.68), self.range(0.0, TAU));
        let (h_base, w_base) = (self.range(0.26, 0.38), self.range(0.46, 0.62));
        for k in 0..4 {
            let a = yaw0 + k as f32 * FRAC_PI_2 + self.range(-0.22, 0.22);
            let (s, c) = a.sin_cos();
            let (dir, out) = ([c, 0.0, s], [-s, 0.0, c]);
            let h = h_base * self.range(0.84, 1.14);
            let w = w_base * self.range(0.88, 1.08);
            let sgn = if k & 1 == 0 { 1.0 } else { -1.0 };
            let arch = self.range(0.08, 0.16) * sgn;
            let shift = self.range(-0.05, 0.05);
            let base = [cx + out[0] * shift, 0.0, cz + out[2] * shift];
            let mid = [out[0] * arch * 0.36, h * 0.56, out[2] * arch * 0.36];
            let top = [out[0] * arch, h, out[2] * arch];
            self.bent_card(base, dir, w, mid, top, FULL);
        }
    }

    /// Wildflower cluster: 4 outward-leaning basal leaves + 2–4 curved two-segment stems + 3D cupped/domed blossoms.
    fn blossom(&mut self, spec: &BlossomSpec) {
        let (cx, cz, yaw0) = (self.range(0.36, 0.64), self.range(0.36, 0.64), self.range(0.0, TAU));
        for k in 0..4 {
            let a = yaw0 + k as f32 * FRAC_PI_2 + self.range(-0.24, 0.24);
            let (s, c) = a.sin_cos();
            let lean = self.range(0.05, 0.10);
            self.card([cx, 0.0, cz], [c, 0.0, s], spec.basal.0, spec.basal.1, [-s * lean, 0.0, c * lean], BASAL, 4, 6);
        }
        let extra = u32::from((self.next() * 1000.0) < spec.extra_permille as f32);
        let count = spec.min_count + extra;
        for i in 0..count {
            let az = yaw0 + i as f32 * (TAU / count as f32) + self.range(-0.36, 0.36);
            let (sa, ca) = az.sin_cos();
            let r_base = self.range(0.03, 0.08);
            let r_head = self.range(spec.spread * 0.55, spec.spread);
            let h = self.range(spec.h.0, spec.h.1);
            let base = [cx + ca * r_base, 0.0, cz + sa * r_base];
            let dx = ca * (r_head - r_base);
            let dz = sa * (r_head - r_base);
            let mid = [dx * 0.42, h * 0.54, dz * 0.42];
            let top = [dx, (h - 0.018).max(0.08), dz];
            let sa0 = az + self.range(-0.35, 0.35);
            for k in 0..2 {
                let (ss, cc) = (sa0 + k as f32 * FRAC_PI_2).sin_cos();
                let dir = [cc, 0.0, ss];
                let hw = spec.stem_w * 0.5;
                let p = |o: [f32; 3], s: f32| {
                    [base[0] + o[0] + dir[0] * hw * s, base[1] + o[1], base[2] + o[2] + dir[2] * hw * s]
                };
                let n1 = unit([mid[0] * 0.5, 1.0, mid[2] * 0.5]);
                let n2 = unit([(top[0] - mid[0]) * 0.6, 1.0, (top[2] - mid[2]) * 0.6]);
                self.quad([p([0.0; 3], -1.0), p([0.0; 3], 1.0), p(mid, 1.0), p(mid, -1.0)], n1, STEM_LO, 4, 6);
                self.quad([p(mid, -1.0), p(mid, 1.0), p(top, 1.0), p(top, -1.0)], n2, STEM_HI, 6, 7);
            }
            let head = [cx + ca * r_head, h, cz + sa * r_head];
            let tilt = self.range(spec.tilt.0, spec.tilt.1);
            let (st, ct) = tilt.sin_cos();
            let n = [ca * st, ct, sa * st];
            let uv = if i & 1 == 0 { HEAD_A } else { HEAD_B };
            let (sz, spin) = (self.range(spec.head.0, spec.head.1), self.range(0.0, TAU));
            self.cup_disc(head, sz, n, spin, uv, spec.cup, 7);
        }
    }

    /// Outward-radiating fountain of 4 curved two-segment arching bellflower stems rooted at `root_u`.
    fn arching(&mut self, root_u: f32, h_min: f32, h_max: f32, w_min: f32, w_max: f32) {
        let (cx, cz, yaw0) = (self.range(0.38, 0.62), self.range(0.38, 0.62), self.range(0.0, TAU));
        for k in 0..4 {
            let a = yaw0 + k as f32 * FRAC_PI_2 + self.range(-0.24, 0.24);
            let (s, c) = a.sin_cos();
            let (dir, perp) = ([c, 0.0, s], [-s, 0.0, c]);
            let (h, w) = (self.range(h_min, h_max), self.range(w_min, w_max));
            let b0 = [cx - dir[0] * (root_u * w), 0.0, cz - dir[2] * (root_u * w)];
            let b1 = [cx + dir[0] * ((1.0 - root_u) * w), 0.0, cz + dir[2] * ((1.0 - root_u) * w)];
            let side = self.range(-0.04, 0.04);
            let shift = |p: [f32; 3], out: f32, y: f32, sl: f32| {
                [p[0] + dir[0] * out + perp[0] * sl, y, p[2] + dir[2] * out + perp[2] * sl]
            };
            let (m0, m1) = (shift(b0, h * 0.05, h * 0.56, side * 0.4), shift(b1, h * 0.05, h * 0.56, side * 0.4));
            let (t0, t1) = (shift(b0, h * 0.19, h, side), shift(b1, h * 0.19, h * 0.94, side));
            let n1 = unit([dir[0] * 0.25 + perp[0] * 0.15, 0.96, dir[2] * 0.25 + perp[2] * 0.15]);
            let n2 = unit([dir[0] * 0.42 + perp[0] * 0.20, 0.88, dir[2] * 0.42 + perp[2] * 0.20]);
            self.quad([b0, b1, m1, m0], n1, Uv { v0: 30, ..FULL }, 4, 6);
            self.quad([m0, m1, t1, t0], n2, Uv { v1: 30, ..FULL }, 6, 7);
        }
    }

    /// True 3D woodland fern rosette: 6 three-segment sweeping arching fronds in an inner/outer layered bowl.
    fn fern(&mut self) {
        let (cx, cz, yaw0) = (self.range(0.42, 0.58), self.range(0.42, 0.58), self.range(0.0, TAU));
        for k in 0..6 {
            let a = yaw0 + k as f32 * (TAU / 6.0) + self.range(-0.18, 0.18);
            let (s, c) = a.sin_cos();
            let (dir, side) = ([c, 0.0, s], [-s, 0.0, c]);
            let len = self.range(0.38, 0.47);
            let hw = self.range(0.105, 0.135);
            let inner = k & 1 == 0;
            let y1 = len * if inner { self.range(0.36, 0.44) } else { self.range(0.24, 0.32) };
            let y2 = y1 + len * if inner { self.range(0.20, 0.26) } else { self.range(0.11, 0.16) };
            let y3 = y2 + len * if inner { self.range(-0.02, 0.04) } else { self.range(-0.07, -0.02) };
            let roll = self.range(-0.022, 0.022);
            let p0 = [cx, 0.01, cz];
            let p1 = [cx + dir[0] * len * 0.34, y1, cz + dir[2] * len * 0.34];
            let p2 = [cx + dir[0] * len * 0.70, y2, cz + dir[2] * len * 0.70];
            let p3 = [cx + dir[0] * len * 1.00, y3.max(0.05), cz + dir[2] * len * 1.00];
            let wide = |p: [f32; 3], w_s: f32, sgn: f32| {
                [p[0] + side[0] * hw * w_s * sgn, p[1] + roll * sgn, p[2] + side[2] * hw * w_s * sgn]
            };
            let segs = [
                (
                    p0,
                    p1,
                    0.85,
                    1.05,
                    Uv { u0: 0, u1: 60, v0: 40, v1: 60 },
                    [-dir[0] * 0.38, 0.92, -dir[2] * 0.38],
                    4,
                    5,
                ),
                (p1, p2, 1.05, 0.95, Uv { u0: 0, u1: 60, v0: 20, v1: 40 }, [dir[0] * 0.05, 0.99, dir[2] * 0.05], 5, 6),
                (p2, p3, 0.95, 0.78, Uv { u0: 0, u1: 60, v0: 0, v1: 20 }, [dir[0] * 0.32, 0.94, dir[2] * 0.32], 6, 7),
            ];
            for (pa, pb, wa, wb, uv, n, oa, ob) in segs {
                self.quad(
                    [wide(pa, wa, -1.0), wide(pa, wa, 1.0), wide(pb, wb, 1.0), wide(pb, wb, -1.0)],
                    unit(n),
                    uv,
                    oa,
                    ob,
                );
            }
        }
    }

    /// Slender riverside cattail & sedge clump: 4 curved two-segment stalks of stepped heights leaning softly.
    fn reed(&mut self) {
        let (cx, cz, yaw0) = (self.range(0.38, 0.62), self.range(0.38, 0.62), self.range(0.0, TAU));
        for k in 0..4 {
            let a = yaw0 + k as f32 * FRAC_PI_2 + self.range(-0.26, 0.26);
            let (s, c) = a.sin_cos();
            let off = self.range(0.05, 0.14);
            let h = self.range(0.60, 0.90) * if k == 0 { 1.04 } else { 0.90 };
            let w = self.range(0.34, 0.46);
            let bow = self.range(0.06, 0.15);
            let base = [cx + c * off, 0.0, cz + s * off];
            let mid = [c * bow * 0.32, h * 0.52, s * bow * 0.32];
            let top = [c * bow, h, s * bow];
            self.bent_card(base, [-s, 0.0, c], w, mid, top, FULL);
        }
    }

    /// Splayed desert & basalt dry scrub: 4 two-segment gnarled branching twigs fanning outward.
    fn dry_brush(&mut self) {
        let (cx, cz, yaw0) = (self.range(0.34, 0.66), self.range(0.34, 0.66), self.range(0.0, TAU));
        for k in 0..4 {
            let a = yaw0 + k as f32 * FRAC_PI_2 + self.range(-0.28, 0.28);
            let (s, c) = a.sin_cos();
            let out = [-s, 0.0, c];
            let h = self.range(0.28, 0.44);
            let w = self.range(0.38, 0.52);
            let tilt = self.range(0.09, 0.17) * if k & 1 == 0 { 1.0 } else { -1.0 };
            let mid = [out[0] * tilt * 0.35, h * 0.52, out[2] * tilt * 0.35];
            let top = [out[0] * tilt, h, out[2] * tilt];
            self.bent_card([cx, 0.0, cz], [c, 0.0, s], w, mid, top, FULL);
        }
    }

    /// Flat ground-hugging foliose rock lichen lying just above the rock surface (avoids z-fighting).
    fn lichen(&mut self) {
        let (cx, cz, size, spin) =
            (self.range(0.34, 0.66), self.range(0.34, 0.66), self.range(0.54, 0.80), self.range(0.0, TAU));
        self.disc([cx, 0.018, cz], size, [0.0, 1.0, 0.0], spin, FULL, 6);
        if self.next() < 0.5 {
            let (a, d, sub) = (self.range(0.0, TAU), self.range(0.14, 0.24), size * self.range(0.55, 0.72));
            let sx = (cx + a.cos() * d).clamp(0.22, 0.78);
            let sz = (cz + a.sin() * d).clamp(0.22, 0.78);
            self.disc([sx, 0.025, sz], sub, [0.0, 1.0, 0.0], spin + 1.9, FULL, 6);
        }
    }

    /// Coal-indicator woody heath shrub: splayed woody base crowned by 4 peaked 3D foliage & berry domes.
    fn coal_shrub(&mut self) {
        let (cx, cz, yaw0) = (self.range(0.36, 0.64), self.range(0.36, 0.64), self.range(0.0, TAU));
        let (stem_h, stem_w) = (self.range(0.24, 0.32), self.range(0.28, 0.36));
        for (k, uv) in [(0, STEM), (1, BASAL)] {
            let (s, c) = (yaw0 + k as f32 * FRAC_PI_2).sin_cos();
            self.card([cx, 0.0, cz], [c, 0.0, s], stem_w, stem_h, [0.0; 3], uv, 4, 6);
        }
        for k in 0..3 {
            let a = yaw0 + k as f32 * (TAU / 3.0) + self.range(-0.25, 0.25);
            let (s, c) = a.sin_cos();
            let rad = self.range(0.11, 0.16);
            let pos = [cx + c * rad, stem_h * self.range(0.75, 0.96), cz + s * rad];
            let uv = if k & 1 == 0 { HEAD_A } else { HEAD_B };
            let (sz, spin) = (self.range(0.30, 0.38), self.range(0.0, TAU));
            self.ridge_disc(pos, sz, [c * 0.62, 0.78, s * 0.62], spin, uv, 0.042, 6);
        }
        let tilt = self.range(0.14, 0.28);
        let ta = yaw0 + PI * 0.7;
        let cap_n = [ta.cos() * tilt.sin(), tilt.cos(), ta.sin() * tilt.sin()];
        let (cap_y, cap_sz, cap_spin) = (stem_h + self.range(0.05, 0.10), self.range(0.32, 0.40), self.range(0.0, TAU));
        self.ridge_disc([cx, cap_y, cz], cap_sz, cap_n, cap_spin, HEAD_A, 0.046, 7);
    }
}

#[inline]
fn add3(a: [f32; 3], b: [f32; 3], c: [f32; 3]) -> [f32; 3] {
    [a[0] + b[0] + c[0], a[1] + b[1] + c[1], a[2] + b[2] + c[2]]
}

#[inline]
fn scale3(a: [f32; 3], s: f32) -> [f32; 3] {
    [a[0] * s, a[1] * s, a[2] * s]
}

fn cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]]
}

fn unit(a: [f32; 3]) -> [f32; 3] {
    let l = (a[0] * a[0] + a[1] * a[1] + a[2] * a[2]).sqrt().max(1e-6);
    [a[0] / l, a[1] / l, a[2] / l]
}
