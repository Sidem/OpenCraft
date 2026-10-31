//! Drawing a belt (presentation only): its frame (a scrolling top between two rails, the Mk studs, or its shape's or
//! corner's model: `belt_shape.rs`, `belt_curve.rs`) and where each riding item's box sits. `Belt::model`
//! (`belt.rs`) draws the frame and the items; the crosshair uses `frame` and `item_box`.

use std::f32::consts::FRAC_PI_2;

use crate::block::tex;
use crate::item;
use crate::math::Vec3;

use super::belt::{Belt, BeltItem, BELT_HEIGHT, BELT_TIERS, ITEM_SIZE};
use super::belt_shape::Shape;
use super::render::push_box;
use super::tier_mark;

impl Belt {
    /// Where a riding item's box centre is and how big it is, `base` being the cell's bottom centre.
    pub(super) fn item_pose(&self, it: &BeltItem, base: Vec3) -> Option<(Vec3, [f32; 3])> {
        let def = item::def(it.item)?;
        let (x, y, z) = self.item_at(it.p);
        let size = def.size.map(|s| s * ITEM_SIZE);
        Some((base + Vec3::new(x as f64, (y + BELT_HEIGHT + size[1] * 0.5) as f64, z as f64), size))
    }

    /// A plain box for item `it`, for the crosshair (`belt_items.rs`), unless it is out of sight; the cell's centre
    /// is `rel`. Returns whether a box was pushed.
    pub(super) fn item_box(&self, it: &BeltItem, out: &mut Vec<f32>, rel: Vec3) -> bool {
        let pose = self.item_pose(it, rel - Vec3::new(0.0, 0.5, 0.0)).filter(|_| self.shows(it.p));
        if let Some((pos, size)) = pose {
            push_box(out, pos, self.dir as f32 * FRAC_PI_2, size, 0.0, [0; 3], false);
        }
        pose.is_some()
    }

    /// The belt without its items (also what the crosshair hits, `hitbox.rs`).
    pub(super) fn frame(&self, out: &mut Vec<f32>, rel: Vec3, time: f64) {
        let base = rel - Vec3::new(0.0, 0.5, 0.0);
        let scroll = (time * self.speed() as f64).fract() as f32;
        let yaw = self.dir as f32 * FRAC_PI_2;
        let (s, c) = yaw.sin_cos();
        let at = |x: f32, y: f32, z: f32| base + Vec3::new((c * x - s * z) as f64, y as f64, (s * x + c * z) as f64);
        if let (Shape::Flat, Some(side)) = (self.shape, self.curve_from) {
            return self.curve_model(out, base, scroll, side);
        }
        if matches!(self.shape, Shape::Flat | Shape::Entry | Shape::Exit) {
            let top = [BELT_TIERS[self.tier as usize].top, tex::FRAME, tex::FRAME];
            push_box(out, at(0.0, BELT_HEIGHT * 0.5, 0.0), yaw, [0.84, BELT_HEIGHT, 1.0], scroll, top, true);
            for side in [-0.46, 0.46] {
                let rail = [tex::FRAME, tex::stripe(self.tier), tex::FRAME];
                push_box(out, at(side, 0.13, 0.0), yaw, [0.08, 0.26, 1.0], 0.0, rail, true);
            }
            for z in [-0.45, 0.45] {
                push_box(out, at(0.0, 0.2, z), yaw, [0.83, 0.07, 0.08], 0.0, [tex::DRILL; 3], false);
            }
            // A run shows its Mk every `BELT_EVERY` cells: studs along the tops of both rails.
            let marked = (self.pos.x + self.pos.z).rem_euclid(tier_mark::BELT_EVERY) == 0;
            if self.shape == Shape::Flat && marked && tier_mark::near(rel) {
                let (paint, d) = ([tex::paint(self.tier); 3], tier_mark::DEPTH);
                for (side, o) in
                    [-0.46, 0.46].into_iter().flat_map(|s| tier_mark::offsets(self.tier).map(move |o| (s, o)))
                {
                    push_box(out, at(side, 0.26 + d / 2.0, o), yaw, [0.1, d, tier_mark::STUD], 0.0, paint, false);
                }
            }
        }
        self.shape_model(out, &at, yaw, scroll);
    }
}
