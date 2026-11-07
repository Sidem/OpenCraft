//! The shaped list: tree surfaces that are not cubes (log cylinders, `logs.rs`; rounded canopies, `foliage.rs`).
//! Three `u32` per vertex, four vertices per quad (the host's shared quad index buffer), opaque quads then cutout ones:
//!
//! ```text
//! word 0: bits  0..12  x   (blocks + SHAPED_OFFSET, in 1/SHAPED_SCALE: chunk-local, a little outside the chunk allowed)
//!         bits 12..24  y
//!         bits 24..27  kind: 0 a canopy (textured by its normal's main axis, per pixel), 1..=3 a log's side around
//!                      x, y, z (u below, v along that axis), 4..=6 a log's end, flat across x, y, z, 7 a canopy's
//!                      sparser outer layer (textured as 0, shifted and thinned)
//!         bits 27..30  occlusion (0 darkest .. 7 open)
//! word 1: bits  0..12  z
//!         bits 12..24  u   (texture tiles in 1/256: around a log)
//!         bits 24..32  light (sky | block << 4, as the cube mesher's light byte)
//! word 2: bits  0..16  normal, octahedral (8 bits each)
//!         bits 16..25  texture layer
//! ```
//!
//! Invariant: a vertex's position is its cell (an integer) plus an offset computed only from the blocks around it,
//! quantised apart (`fixed`), so two chunks that both write a vertex on their shared border write the same bits.

use crate::block::{is_leaves, is_log, BlockId, OPAQUE};

/// Blocks drawn by the shaped list, not as cubes: logs and leaves.
pub const SHAPED: [bool; 256] = {
    let mut t = [false; 256];
    let mut i = 0;
    while i < 256 {
        t[i] = is_log(i as BlockId) || is_leaves(i as BlockId);
        i += 1;
    }
    t
};

/// Blocks that hide the cube faces against them: the opaque cubes. A log is opaque to light, but round.
pub const HIDES: [bool; 256] = {
    let mut t = OPAQUE;
    let mut i = 0;
    while i < 256 {
        t[i] = t[i] && !is_log(i as BlockId);
        i += 1;
    }
    t
};

pub const SHAPED_SCALE: f32 = 100.0;
pub const SHAPED_OFFSET: i32 = 2;
pub const KIND_CANOPY: u32 = 0;
pub const KIND_SIDE: u32 = 1;
pub const KIND_END: u32 = 4;
pub const KIND_FRINGE: u32 = 7;

/// One vertex before packing: `cell` is chunk-local, `off` the position in it (may run a little outside it).
#[derive(Clone, Copy)]
pub struct Vert {
    pub cell: [i32; 3],
    pub off: [f32; 3],
    pub normal: [f32; 3],
    pub u: f32,
    pub kind: u32,
    pub occlusion: u32,
    pub light: u8,
    pub layer: u16,
}

/// The shaped quads of one chunk, opaque and cutout apart until the mesher joins them.
#[derive(Default)]
pub struct Shaped {
    pub opaque: Vec<u32>,
    pub cutout: Vec<u32>,
}

impl Shaped {
    pub fn clear(&mut self) {
        self.opaque.clear();
        self.cutout.clear();
    }

    /// Appends a quad, wound counter-clockwise seen from where it faces, of vertices packed already (with layer 0),
    /// giving them `layer`. Not inlined: callers loop
    /// over it, and unrolled copies would each carry the `Vec` growth path.
    #[inline(never)]
    pub fn packed(&mut self, cutout: bool, q: [[u32; 3]; 4], layer: u16) {
        let out = if cutout { &mut self.cutout } else { &mut self.opaque };
        for w in q {
            out.extend_from_slice(&[w[0], w[1], w[2] | u32::from(layer) << 16]);
        }
    }
}

/// Fixed-point coordinate of an offset `off` in cell `cell` (`SHAPED_SCALE` steps a block, `SHAPED_OFFSET` blocks shifted).
#[inline]
pub fn fixed(cell: i32, off: f32) -> u32 {
    ((cell + SHAPED_OFFSET) * SHAPED_SCALE as i32 + (off * SHAPED_SCALE).round() as i32).clamp(0, 4095) as u32
}

/// Never inlined: it is called from many places, and each copy costs wasm size.
#[inline(never)]
pub fn pack(v: &Vert) -> [u32; 3] {
    let [x, y, z] = std::array::from_fn(|i| fixed(v.cell[i], v.off[i]));
    let u = ((v.u * 256.0).round() as i32).clamp(0, 4095) as u32;
    [
        x | y << 12 | v.kind << 24 | v.occlusion.min(7) << 27,
        z | u << 12 | u32::from(v.light) << 24,
        octahedral(v.normal) | u32::from(v.layer) << 16,
    ]
}

/// A unit normal in 16 bits: x and z folded onto the octahedron, 8 bits each (`shaped.ts` unfolds it).
pub fn octahedral(n: [f32; 3]) -> u32 {
    let s = n[0].abs() + n[1].abs() + n[2].abs();
    let (mut a, mut b) = (n[0] / s, n[2] / s);
    if n[1] < 0.0 {
        let (pa, pb) = (a, b);
        a = (1.0 - pb.abs()) * pa.signum();
        b = (1.0 - pa.abs()) * pb.signum();
    }
    let q = |t: f32| ((t * 0.5 + 0.5) * 255.0).round().clamp(0.0, 255.0) as u32;
    q(a) | q(b) << 8
}

/// The average of light bytes (sky and block nibble apart, rounded), or `None` for none.
pub fn mean_light(lights: impl Iterator<Item = u8>) -> Option<u8> {
    let (mut sky, mut block, mut n) = (0u32, 0u32, 0u32);
    for l in lights {
        sky += u32::from(l & 15);
        block += u32::from(l >> 4);
        n += 1;
    }
    (n > 0).then(|| (((sky + n / 2) / n) | ((block + n / 2) / n) << 4) as u8)
}
