//! Axis-aligned collision and aim boxes for every block shape (`shape/mod.rs`).
//!
//! Stack-allocated (`CellBoxes`, up to 8 AABBs, zero heap allocations) so player physics, dropped items
//! and raycast hit-testing share the exact same sub-voxel geometry. Full cubes fast-path to a single unit box.

use crate::hitbox::Boxes;
use crate::math::{IVec3, Vec3};
use crate::physics::Aabb;

use super::{kind, to_local_f64, Corner, Kind};

/// Up to 8 axis-aligned boxes for one voxel cell, stack-allocated with zero heap traffic.
#[derive(Clone, Copy, Debug)]
pub struct CellBoxes {
    pub boxes: [Aabb; 8],
    pub len: u8,
}

impl CellBoxes {
    pub const EMPTY: Self = Self { boxes: [Aabb { min: Vec3::ZERO, max: Vec3::ZERO }; 8], len: 0 };

    #[inline]
    pub fn slice(&self) -> &[Aabb] {
        &self.boxes[..self.len as usize]
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    #[inline]
    fn push(&mut self, min: Vec3, max: Vec3) {
        debug_assert!((self.len as usize) < self.boxes.len());
        self.boxes[self.len as usize] = Aabb { min, max };
        self.len += 1;
    }

    /// Single full unit cube at `cell`.
    #[inline]
    pub fn full(cell: IVec3) -> Self {
        let min = cell.as_vec3();
        let mut out = Self::EMPTY;
        out.push(min, min + Vec3::new(1.0, 1.0, 1.0));
        out
    }
}

/// Pushes a box specified in canonical `(right, y, back)` cell coordinates `[0..1]³`, rotated by `dir`
/// and offset by `origin`.
#[inline]
fn push_local(out: &mut CellBoxes, origin: Vec3, dir: u8, lo: [f64; 3], hi: [f64; 3]) {
    let (xa, za) = to_local_f64(dir, lo[0], lo[2]);
    let (xb, zb) = to_local_f64(dir, hi[0], hi[2]);
    let min = Vec3::new(origin.x + xa.min(xb), origin.y + lo[1], origin.z + za.min(zb));
    let max = Vec3::new(origin.x + xa.max(xb), origin.y + hi[1], origin.z + za.max(zb));
    out.push(min, max);
}

/// World-space collision boxes for a solid cell at `cell` with `shape` and auto-connected `corner`.
pub fn boxes_for(cell: IVec3, shape: u8, corner: Corner) -> CellBoxes {
    let origin = cell.as_vec3();
    let mut out = CellBoxes::EMPTY;
    match kind(shape) {
        Kind::Full => {
            out.push(origin, origin + Vec3::new(1.0, 1.0, 1.0));
        }
        Kind::Slab { quarters, top: false } => {
            let h = f64::from(quarters) * 0.25;
            out.push(origin, origin + Vec3::new(1.0, h, 1.0));
        }
        Kind::Slab { quarters, top: true } => {
            let y0 = 1.0 - f64::from(quarters) * 0.25;
            out.push(origin + Vec3::new(0.0, y0, 0.0), origin + Vec3::new(1.0, 1.0, 1.0));
        }
        Kind::VSlab { quarters, dir } => {
            let t = f64::from(quarters) * 0.25;
            push_local(&mut out, origin, dir, [0.0, 0.0, 1.0 - t], [1.0, 1.0, 1.0]);
        }
        Kind::Stair { dir, inverted } => {
            let (base_y0, base_y1, step_y0, step_y1) =
                if inverted { (0.5, 1.0, 0.0, 0.5) } else { (0.0, 0.5, 0.5, 1.0) };
            out.push(origin + Vec3::new(0.0, base_y0, 0.0), origin + Vec3::new(1.0, base_y1, 1.0));
            match corner {
                Corner::Straight => push_local(&mut out, origin, dir, [0.0, step_y0, 0.5], [1.0, step_y1, 1.0]),
                Corner::OuterLeft => push_local(&mut out, origin, dir, [0.0, step_y0, 0.5], [0.5, step_y1, 1.0]),
                Corner::OuterRight => push_local(&mut out, origin, dir, [0.5, step_y0, 0.5], [1.0, step_y1, 1.0]),
                Corner::InnerLeft => {
                    push_local(&mut out, origin, dir, [0.0, step_y0, 0.5], [1.0, step_y1, 1.0]);
                    push_local(&mut out, origin, dir, [0.0, step_y0, 0.0], [0.5, step_y1, 0.5]);
                }
                Corner::InnerRight => {
                    push_local(&mut out, origin, dir, [0.0, step_y0, 0.5], [1.0, step_y1, 1.0]);
                    push_local(&mut out, origin, dir, [0.5, step_y0, 0.0], [1.0, step_y1, 0.5]);
                }
            }
        }
        Kind::Slope { dir, inverted } => {
            // 4 quarter-height steps (0.25 risers): smoothly walkable with STEP_HEIGHT & StepGlide,
            // zero diagonal wedging bugs, and exact support for dropped items and crouch edge-guard.
            for k in 0..4u8 {
                let f0 = f64::from(k) * 0.25;
                let f1 = f0 + 0.25;
                let (y0, y1) = if inverted { (1.0 - f1, 1.0 - f0) } else { (f0, f1) };
                match corner {
                    Corner::Straight => push_local(&mut out, origin, dir, [0.0, y0, f0], [1.0, y1, 1.0]),
                    Corner::OuterLeft => push_local(&mut out, origin, dir, [0.0, y0, f0], [1.0 - f0, y1, 1.0]),
                    Corner::OuterRight => push_local(&mut out, origin, dir, [f0, y0, f0], [1.0, y1, 1.0]),
                    Corner::InnerLeft => {
                        if k == 0 {
                            push_local(&mut out, origin, dir, [0.0, y0, 0.0], [1.0, y1, 1.0]);
                        } else {
                            push_local(&mut out, origin, dir, [0.0, y0, f0], [1.0, y1, 1.0]);
                            push_local(&mut out, origin, dir, [0.0, y0, 0.0], [1.0 - f0, y1, f0]);
                        }
                    }
                    Corner::InnerRight => {
                        if k == 0 {
                            push_local(&mut out, origin, dir, [0.0, y0, 0.0], [1.0, y1, 1.0]);
                        } else {
                            push_local(&mut out, origin, dir, [0.0, y0, f0], [1.0, y1, 1.0]);
                            push_local(&mut out, origin, dir, [f0, y0, 0.0], [1.0, y1, f0]);
                        }
                    }
                }
            }
        }
    }
    out
}

/// Cell-local aim/selection hit boxes for a non-full shaped block (`hitbox::Boxes`), so rays pass through
/// empty space in the cell and the selection wireframe hugs the slab or stair.
pub fn hit_boxes(shape: u8, corner: Corner) -> Option<Boxes> {
    if shape == super::FULL {
        return None;
    }
    let cb = boxes_for(IVec3::ZERO, shape, corner);
    Boxes::from_aabbs(cb.slice())
}
