//! Building blocks for item models: `Build` puts parts and shapes together into one const assembly, round bodies are
//! real ones (`cylinder` and `frustum` upright, `bar_x` and `cone_x` lying along x), and heaps come from a fixed set
//! of lumps (`pile`). All sizes are fractions of the item cube, y up.

use super::{turned, Axis, Form, Part};

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
    Part { center: c, size, yaw: 0.0, tex: [top, side, top], form: Form::Box }
}

/// A round upright body of diameter `d` and height `h` centred at `c` (a drum, a disc, a coin).
pub(super) const fn cylinder(c: [f32; 3], d: f32, h: f32, layer: u16) -> Part {
    frustum(c, d, h, 1.0, layer)
}

/// An upright cone's frustum `d` across its foot and `top` times that across its top (a nozzle, a nose cone).
pub(super) const fn frustum(c: [f32; 3], d: f32, h: f32, top: f32, layer: u16) -> Part {
    Part { center: c, size: [d, h, d], yaw: 0.0, tex: [layer; 3], form: Form::Round(Axis::Up, top) }
}

/// A round bar of diameter `d` and length `len` lying along x (motors, rods, a boule).
pub(super) const fn bar_x(c: [f32; 3], len: f32, d: f32, layer: u16) -> Part {
    cone_x(c, len, d, 1.0, layer)
}

/// A round body lying along x, `d` across its -x end and `end` times that across its +x end (a boule's shoulder).
pub(super) const fn cone_x(c: [f32; 3], len: f32, d: f32, end: f32, layer: u16) -> Part {
    Part { center: c, size: [len, d, d], yaw: 0.0, tex: [layer; 3], form: Form::Round(Axis::X, end) }
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
