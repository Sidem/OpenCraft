//! Hit boxes for blocks whose model is mostly empty space (poles, belts, splitters, pipes): the crosshair
//! hits the model, not the whole cell, so a pole never hides the block behind it.
//!
//! Owns: `Boxes`, a cell's model boxes as oriented boxes in cell-local coordinates (0..1 across the cell).
//! `Factory::hit_boxes` (`factory/hitbox.rs`) builds them from a machine's own model; `interaction/aim.rs`
//! asks them whether a ray touches the cell. Presentation only: the core never reads this.
//! To extend: nothing here; give another machine kind a hit box by adding it to `Factory::hit_boxes`.

use crate::factory::INSTANCE_FLOATS;
use crate::math::Vec3;

/// Slack around every box, so a thin post is not a hairline to hit.
const PAD: f64 = 0.04;
/// How far up the cell's origin `bounds` reach: a power pole stands two blocks tall (`factory::pole::POLE_RISE`).
const TALLEST: f64 = 2.2;

/// One oriented box: centre, half sizes and the rotation (as the box shader applies it) to world axes.
struct Obb {
    centre: [f64; 3],
    half: [f64; 3],
    rot: [[f64; 3]; 3],
}

/// The model boxes of one cell.
pub struct Boxes(Vec<Obb>);

impl Boxes {
    /// Reads box instances (`INSTANCE_FLOATS` each, centres relative to the cell centre). None for no boxes.
    pub fn from_instances(floats: &[f32]) -> Option<Boxes> {
        let boxes: Vec<Obb> = floats.chunks_exact(INSTANCE_FLOATS).map(Obb::new).collect();
        (!boxes.is_empty()).then_some(Boxes(boxes))
    }

    /// One axis-aligned box about the cell centre, `size` across.
    pub fn centred(size: f64) -> Boxes {
        let rot = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];
        Boxes(vec![Obb { centre: [0.5; 3], half: [size * 0.5 + PAD; 3], rot }])
    }

    /// Distance along `dir` (a unit vector) at which a ray from `origin` (relative to the cell's lowest corner)
    /// first touches a box; 0 when it starts inside one.
    pub fn enter(&self, origin: Vec3, dir: Vec3) -> Option<f64> {
        let (o, d) = ([origin.x, origin.y, origin.z], [dir.x, dir.y, dir.z]);
        self.0.iter().filter_map(|b| b.enter(o, d)).min_by(f64::total_cmp)
    }

    /// The smallest axis-aligned box around all of them: its lowest corner and its size, in cell units.
    pub fn bounds(&self) -> [f32; 6] {
        let (mut lo, mut hi) = ([f64::MAX; 3], [f64::MIN; 3]);
        for b in &self.0 {
            for axis in 0..3 {
                // The reach of a rotated box along a world axis: the rotation's row weighs each half size.
                let reach: f64 = (0..3).map(|k| b.rot[axis][k].abs() * b.half[k]).sum();
                lo[axis] = lo[axis].min(b.centre[axis] - reach);
                hi[axis] = hi[axis].max(b.centre[axis] + reach);
            }
        }
        let lo = lo.map(|v| v.max(0.0));
        let hi = hi.map(|v| v.min(TALLEST));
        [lo[0], lo[1], lo[2], hi[0] - lo[0], hi[1] - lo[1], hi[2] - lo[2]].map(|v| v as f32)
    }
}

impl Obb {
    /// From one instance: centre xyz, yaw, size xyz, ..., pitch, roll (`factory/render.rs`). A taper is ignored.
    fn new(rec: &[f32]) -> Obb {
        let v = |i: usize| f64::from(rec[i]);
        let (cy, sy) = (v(3).cos(), v(3).sin());
        let (cp, sp) = (v(12).cos(), v(12).sin());
        let (cr, sr) = (v(13).cos(), v(13).sin());
        // The shader's rot = yaw * roll * tilt, applied to a local point (rows here).
        let yaw = [[cy, 0.0, -sy], [0.0, 1.0, 0.0], [sy, 0.0, cy]];
        let roll = [[cr, -sr, 0.0], [sr, cr, 0.0], [0.0, 0.0, 1.0]];
        let tilt = [[1.0, 0.0, 0.0], [0.0, cp, -sp], [0.0, sp, cp]];
        Obb {
            centre: [v(0) + 0.5, v(1) + 0.5, v(2) + 0.5],
            half: [v(4) * 0.5 + PAD, v(5) * 0.5 + PAD, v(6) * 0.5 + PAD],
            rot: mul(&yaw, &mul(&roll, &tilt)),
        }
    }

    /// Slab test in the box's own frame (the ray rotated by the transpose of `rot`).
    fn enter(&self, origin: [f64; 3], dir: [f64; 3]) -> Option<f64> {
        let rel = [0, 1, 2].map(|i| origin[i] - self.centre[i]);
        let local = |v: [f64; 3], axis: usize| (0..3).map(|k| self.rot[k][axis] * v[k]).sum::<f64>();
        let (mut near, mut far) = (0.0_f64, f64::INFINITY);
        for axis in 0..3 {
            let (o, d) = (local(rel, axis), local(dir, axis));
            if d.abs() < 1e-12 {
                if o.abs() > self.half[axis] {
                    return None;
                }
                continue;
            }
            let (a, b) = ((-self.half[axis] - o) / d, (self.half[axis] - o) / d);
            near = near.max(a.min(b));
            far = far.min(a.max(b));
        }
        (near <= far).then_some(near)
    }
}

fn mul(a: &[[f64; 3]; 3], b: &[[f64; 3]; 3]) -> [[f64; 3]; 3] {
    let mut m = [[0.0; 3]; 3];
    for (i, row) in m.iter_mut().enumerate() {
        for (j, cell) in row.iter_mut().enumerate() {
            *cell = (0..3).map(|k| a[i][k] * b[k][j]).sum();
        }
    }
    m
}

#[cfg(test)]
mod tests;
