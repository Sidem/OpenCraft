//! Building blocks for item models: `Build` puts parts and shapes together into one const assembly, round upright
//! bodies come from turned squares (`cylinder`), round bars lying along x from three stepped boxes (`bar_x`), and
//! heaps from a fixed set of lumps (`pile`). All sizes are fractions of the item cube, y up.

use std::f32::consts::FRAC_PI_2;

use super::{part, turned, Part};

/// An unused slot while an assembly is filled in.
pub(super) const EMPTY: Part = turned([0.0; 3], [0.0; 3], 0.0, 0);

/// A const assembly of exactly `N` parts, filled in drawing order (back to front for the icon):
/// `Build::new().with(part).all(shape).done()`.
pub(super) struct Build<const N: usize> {
    parts: [Part; N],
    len: usize,
}

impl<const N: usize> Build<N> {
    pub(super) const fn new() -> Self {
        Build { parts: [EMPTY; N], len: 0 }
    }

    pub(super) const fn with(mut self, p: Part) -> Self {
        self.parts[self.len] = p;
        self.len += 1;
        self
    }

    pub(super) const fn all<const M: usize>(mut self, shape: [Part; M]) -> Self {
        let mut i = 0;
        while i < M {
            self.parts[self.len] = shape[i];
            self.len += 1;
            i += 1;
        }
        self
    }

    /// The parts; fails to compile unless all `N` were filled.
    pub(super) const fn done(self) -> [Part; N] {
        assert!(self.len == N, "an item model's part count does not match its parts");
        self.parts
    }
}

/// A box with `top` on its top and bottom and `side` round it (a board's print, a screen's bezel).
pub(super) const fn faced(c: [f32; 3], size: [f32; 3], top: u16, side: u16) -> Part {
    Part { center: c, size, yaw: 0.0, tex: [top, side, top] }
}

/// A round upright body of diameter `d` and height `h` centred at `c`: `N` squares turned evenly through a quarter
/// turn (2 make an octagon, 3 a 12-sided body), each a hair higher than the last so their tops never fight.
pub(super) const fn cylinder<const N: usize>(c: [f32; 3], d: f32, h: f32, layer: u16) -> [Part; N] {
    let mut out = [EMPTY; N];
    let mut i = 0;
    while i < N {
        let lift = 0.0015 * i as f32;
        out[i] = turned([c[0], c[1] + lift, c[2]], [d, h, d], FRAC_PI_2 * i as f32 / N as f32, layer);
        i += 1;
    }
    out
}

/// A round bar of diameter `d` lying along x (motors, rods, boules): a wide box, a tall one and a square whose
/// corners touch the circle step its outline round. Each is a hair shorter than the last so their ends never fight.
pub(super) const fn bar_x(c: [f32; 3], len: f32, d: f32, layer: u16) -> [Part; 3] {
    [
        part(c, [len, 0.38 * d, 0.92 * d], layer),
        part(c, [len - 0.004, 0.92 * d, 0.38 * d], layer),
        part(c, [len - 0.008, 0.7 * d, 0.7 * d], layer),
    ]
}

/// Lumps of a heap 0.4 high (centre x, y, z, size x, y, z, turn), back to front and bottom up for the icon.
const LUMPS: [[f32; 7]; 8] = [
    [-0.15, -0.12, -0.14, 0.22, 0.16, 0.2, 0.3],
    [0.12, -0.13, -0.16, 0.2, 0.14, 0.18, 1.1],
    [-0.2, -0.13, 0.08, 0.18, 0.14, 0.2, 0.7],
    [0.0, -0.08, 0.0, 0.26, 0.24, 0.24, 0.5],
    [0.18, -0.13, 0.06, 0.18, 0.14, 0.16, 0.2],
    [-0.04, 0.1, -0.03, 0.18, 0.2, 0.17, 1.3],
    [0.06, -0.14, 0.18, 0.2, 0.12, 0.16, 0.9],
    [-0.12, -0.15, 0.2, 0.14, 0.1, 0.12, 0.4],
];

/// A heap of rough lumps of one material, 0.4 high (crushed and washed ore, sulfur, quicklime).
pub(super) const fn pile(layer: u16) -> [Part; 8] {
    let mut out = [EMPTY; 8];
    let mut i = 0;
    while i < 8 {
        let l = LUMPS[i];
        out[i] = turned([l[0], l[1], l[2]], [l[3], l[4], l[5]], l[6], layer);
        i += 1;
    }
    out
}
