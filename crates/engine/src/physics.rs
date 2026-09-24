//! Axis-separated swept AABB collision against the voxel grid.

use crate::math::Vec3;

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
    pub fn has_support(&self, solid: &mut impl FnMut(i32, i32, i32) -> bool) -> bool {
        let y = (self.min.y - 0.05).floor() as i32;
        let (x0, x1) = self.cells(0);
        let (z0, z1) = self.cells(2);
        (x0..=x1).any(|x| (z0..=z1).any(|z| solid(x, y, z)))
    }
}

/// Moves `bb` by up to `delta` along `axis`, stopping flush against the first solid voxel.
/// Voxels the box already overlaps are ignored, so a box can always move out of a block.
/// Returns the distance actually travelled.
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
mod tests {
    use super::*;

    fn floor_at_zero(_x: i32, y: i32, _z: i32) -> bool {
        y < 0
    }

    #[test]
    fn falls_onto_floor() {
        let mut bb = Aabb::from_feet(Vec3::new(0.5, 1.0, 0.5), 0.3, 1.8);
        let moved = move_axis(&mut bb, 1, -5.0, &mut floor_at_zero);
        assert!((moved + 1.0).abs() < 1e-9, "moved {moved}");
        assert!(bb.min.y.abs() < 1e-9);
        // Resting on the floor: further downward motion is blocked entirely.
        assert_eq!(move_axis(&mut bb, 1, -0.1, &mut floor_at_zero), 0.0);
    }

    #[test]
    fn blocked_by_wall() {
        let mut wall = |x: i32, _y: i32, _z: i32| x >= 3;
        let mut bb = Aabb::from_feet(Vec3::new(1.5, 0.0, 0.5), 0.3, 1.8);
        let moved = move_axis(&mut bb, 0, 5.0, &mut wall);
        assert!((bb.max.x - 3.0).abs() < 1e-9);
        assert!((moved - 1.2).abs() < 1e-9);
        let mut wall_neg = |x: i32, _y: i32, _z: i32| x < 0;
        let mut bb = Aabb::from_feet(Vec3::new(1.5, 0.0, 0.5), 0.3, 1.8);
        move_axis(&mut bb, 0, -5.0, &mut wall_neg);
        assert!(bb.min.x.abs() < 1e-9);
    }

    #[test]
    fn can_leave_overlapping_block() {
        let mut solid = |x: i32, y: i32, z: i32| (x, y, z) == (0, 0, 0);
        let mut bb = Aabb::centered(Vec3::new(0.5, 0.5, 0.5), 0.2);
        assert!((move_axis(&mut bb, 1, 1.0, &mut solid) - 1.0).abs() < 1e-9);
    }
}
