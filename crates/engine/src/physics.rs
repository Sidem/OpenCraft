//! Axis-separated swept AABB collision against the voxel grid, shared by the player and dropped
//! items. `move_axis` moves a box along one axis and stops it at the first solid block; callers pass
//! a `solid(x, y, z)` closure, so this module never touches the world directly.

use crate::math::Vec3;
use crate::shape::CellBoxes;

#[derive(Clone, Copy, Debug)]
pub struct Aabb {
    pub min: Vec3,
    pub max: Vec3,
}

const EPS: f64 = 1e-7;

impl Aabb {
    /// Box standing on `feet` (bottom-centre).
    pub fn from_feet(feet: Vec3, half_width: f64, height: f64) -> Self {
        Self {
            min: Vec3::new(feet.x - half_width, feet.y, feet.z - half_width),
            max: Vec3::new(feet.x + half_width, feet.y + height, feet.z + half_width),
        }
    }

    pub fn centered(c: Vec3, half: f64) -> Self {
        Self { min: Vec3::new(c.x - half, c.y - half, c.z - half), max: Vec3::new(c.x + half, c.y + half, c.z + half) }
    }

    pub fn intersects(&self, o: &Aabb) -> bool {
        self.min.x < o.max.x - EPS
            && self.max.x > o.min.x + EPS
            && self.min.y < o.max.y - EPS
            && self.max.y > o.min.y + EPS
            && self.min.z < o.max.z - EPS
            && self.max.z > o.min.z + EPS
    }

    pub fn center(&self) -> Vec3 {
        Vec3::new((self.min.x + self.max.x) * 0.5, (self.min.y + self.max.y) * 0.5, (self.min.z + self.max.z) * 0.5)
    }

    fn cells(&self, axis: usize) -> (i32, i32) {
        ((self.min.get(axis) + EPS).floor() as i32, (self.max.get(axis) - EPS).ceil() as i32 - 1)
    }

    /// True if any solid voxel lies in the thin layer just below the box.
    #[cfg(test)]
    pub fn has_support(&self, solid: &mut impl FnMut(i32, i32, i32) -> bool) -> bool {
        let y = (self.min.y - 0.05).floor() as i32;
        let (x0, x1) = self.cells(0);
        let (z0, z1) = self.cells(2);
        (x0..=x1).any(|x| (z0..=z1).any(|z| solid(x, y, z)))
    }

    /// True if any sub-voxel collision box lies immediately below `self.min.y` (for crouch edge-guard
    /// and smooth step-down on slabs, stairs and slopes).
    pub fn has_support_boxes(&self, boxes: &mut impl FnMut(i32, i32, i32) -> CellBoxes) -> bool {
        let y_lo = self.min.y - 0.05;
        let y_hi = self.min.y + EPS;
        let cy0 = y_lo.floor() as i32;
        let cy1 = y_hi.floor() as i32;
        let (x0, x1) = self.cells(0);
        let (z0, z1) = self.cells(2);
        for y in cy0..=cy1 {
            for z in z0..=z1 {
                for x in x0..=x1 {
                    for b in boxes(x, y, z).slice() {
                        if self.min.x < b.max.x - EPS
                            && self.max.x > b.min.x + EPS
                            && self.min.z < b.max.z - EPS
                            && self.max.z > b.min.z + EPS
                            && b.max.y >= y_lo
                            && b.min.y < y_hi
                            && b.max.y <= self.min.y + 0.05
                        {
                            return true;
                        }
                    }
                }
            }
        }
        false
    }
}

/// Moves `bb` by up to `delta` along `axis`, stopping flush against the first sub-voxel collision box
/// from `boxes(x, y, z)`. Boxes `bb` already overlaps along `axis` are ignored so a body can move out.
pub fn move_axis_boxes(
    bb: &mut Aabb,
    axis: usize,
    delta: f64,
    boxes: &mut impl FnMut(i32, i32, i32) -> CellBoxes,
) -> f64 {
    if delta == 0.0 {
        return 0.0;
    }
    let (a1, a2) = match axis {
        0 => (1, 2),
        1 => (0, 2),
        _ => (0, 1),
    };
    let (lo1, hi1) = bb.cells(a1);
    let (lo2, hi2) = bb.cells(a2);
    let mut moved = delta;

    if delta > 0.0 {
        let face = bb.max.get(axis);
        let from = (face - EPS).floor() as i32;
        let to = (face + delta - EPS).floor() as i32;
        for layer in from..=to {
            if (layer as f64) > face + moved + EPS {
                break;
            }
            for i in lo1..=hi1 {
                for j in lo2..=hi2 {
                    let mut c = [0i32; 3];
                    c[axis] = layer;
                    c[a1] = i;
                    c[a2] = j;
                    for b in boxes(c[0], c[1], c[2]).slice() {
                        if bb.min.get(a1) < b.max.get(a1) - EPS
                            && bb.max.get(a1) > b.min.get(a1) + EPS
                            && bb.min.get(a2) < b.max.get(a2) - EPS
                            && bb.max.get(a2) > b.min.get(a2) + EPS
                        {
                            let bm = b.min.get(axis);
                            if bm >= face - EPS && bm < face + moved {
                                moved = (bm - face).max(0.0);
                            }
                        }
                    }
                }
            }
        }
    } else {
        let face = bb.min.get(axis);
        let from = (face + EPS).floor() as i32;
        let to = (face + delta + EPS).floor() as i32;
        for layer in (to..=from).rev() {
            if ((layer + 1) as f64) < face + moved - EPS {
                break;
            }
            for i in lo1..=hi1 {
                for j in lo2..=hi2 {
                    let mut c = [0i32; 3];
                    c[axis] = layer;
                    c[a1] = i;
                    c[a2] = j;
                    for b in boxes(c[0], c[1], c[2]).slice() {
                        if bb.min.get(a1) < b.max.get(a1) - EPS
                            && bb.max.get(a1) > b.min.get(a1) + EPS
                            && bb.min.get(a2) < b.max.get(a2) - EPS
                            && bb.max.get(a2) > b.min.get(a2) + EPS
                        {
                            let bm = b.max.get(axis);
                            if bm <= face + EPS && bm > face + moved {
                                moved = (bm - face).min(0.0);
                            }
                        }
                    }
                }
            }
        }
    }
    bb.min.set(axis, bb.min.get(axis) + moved);
    bb.max.set(axis, bb.max.get(axis) + moved);
    moved
}

/// Result of a horizontal walk step with optional auto step-up (`STEP_HEIGHT`) and crouch edge-guard.
pub struct WalkStep {
    pub blocked_x: bool,
    pub blocked_z: bool,
    pub step_dy: f64,
}

/// Moves `bb` horizontally by `(want_x, want_z)` against sub-voxel boxes. When `step_height > 0` and
/// horizontal motion is blocked, tests lifting `bb` up to `step_height`, advancing, and dropping onto the
/// raised surface (slabs, stairs, pitched slopes). Respects `edge_guard` (crouching never walks off an edge).
pub fn move_walk_boxes(
    bb: &mut Aabb,
    want_x: f64,
    want_z: f64,
    step_height: f64,
    edge_guard: bool,
    boxes: &mut impl FnMut(i32, i32, i32) -> CellBoxes,
) -> WalkStep {
    let mut flat = *bb;
    let mut blocked = [false; 2];
    for (idx, (axis, want)) in [(0, want_x), (2, want_z)].into_iter().enumerate() {
        let before = flat;
        let moved = move_axis_boxes(&mut flat, axis, want, boxes);
        if edge_guard && !flat.has_support_boxes(boxes) {
            flat = before;
            blocked[idx] = true;
        } else if (moved - want).abs() > 1e-9 {
            blocked[idx] = true;
        }
    }
    if step_height > 0.0 && (blocked[0] || blocked[1]) && (want_x != 0.0 || want_z != 0.0) {
        let mut cand = *bb;
        let lifted = move_axis_boxes(&mut cand, 1, step_height, boxes);
        if lifted > 1e-4 {
            let cx = move_axis_boxes(&mut cand, 0, want_x, boxes);
            let cz = move_axis_boxes(&mut cand, 2, want_z, boxes);
            let dropped = move_axis_boxes(&mut cand, 1, -lifted, boxes);
            let rise = cand.min.y - bb.min.y;
            let (fx, fz) = (flat.min.x - bb.min.x, flat.min.z - bb.min.z);
            let landed = (-dropped) < lifted - 1e-5 || cand.has_support_boxes(boxes);
            if rise > 1e-4
                && rise <= step_height + 1e-5
                && landed
                && cx * cx + cz * cz > fx * fx + fz * fz + 1e-9
                && (!edge_guard || cand.has_support_boxes(boxes))
            {
                *bb = cand;
                return WalkStep {
                    blocked_x: (cx - want_x).abs() > 1e-9,
                    blocked_z: (cz - want_z).abs() > 1e-9,
                    step_dy: rise,
                };
            }
        }
    }
    *bb = flat;
    WalkStep { blocked_x: blocked[0], blocked_z: blocked[1], step_dy: 0.0 }
}

/// Moves `bb` by up to `delta` along `axis`, stopping flush against the first solid voxel.
/// Voxels the box already overlaps are ignored, so a box can always move out of a block.
/// Returns the distance actually travelled.
#[cfg(test)]
pub fn move_axis(bb: &mut Aabb, axis: usize, delta: f64, solid: &mut impl FnMut(i32, i32, i32) -> bool) -> f64 {
    if delta == 0.0 {
        return 0.0;
    }
    let (a1, a2) = match axis {
        0 => (1, 2),
        1 => (0, 2),
        _ => (0, 1),
    };
    let (lo1, hi1) = bb.cells(a1);
    let (lo2, hi2) = bb.cells(a2);
    let mut hit = |layer: i32| {
        for i in lo1..=hi1 {
            for j in lo2..=hi2 {
                let mut c = [0i32; 3];
                c[axis] = layer;
                c[a1] = i;
                c[a2] = j;
                if solid(c[0], c[1], c[2]) {
                    return true;
                }
            }
        }
        false
    };

    let mut moved = delta;
    if delta > 0.0 {
        let face = bb.max.get(axis);
        let from = (face - EPS).ceil() as i32;
        let to = (face + delta - EPS).floor() as i32;
        for layer in from..=to {
            if hit(layer) {
                moved = (layer as f64 - face).max(0.0);
                break;
            }
        }
    } else {
        let face = bb.min.get(axis);
        let from = (face + EPS).floor() as i32 - 1;
        let to = (face + delta + EPS).ceil() as i32 - 1;
        for layer in (to..=from).rev() {
            if hit(layer) {
                moved = ((layer + 1) as f64 - face).min(0.0);
                break;
            }
        }
    }
    bb.min.set(axis, bb.min.get(axis) + moved);
    bb.max.set(axis, bb.max.get(axis) + moved);
    moved
}

#[cfg(test)]
mod tests;
