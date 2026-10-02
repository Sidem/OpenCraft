//! Shoulder-camera targeting. The crosshair's ray chooses a surface; hands still enforce reach and
//! line of sight, so the offset cannot mine through walls or increase the interaction range.

use super::REACH;
use crate::block::BlockId;
use crate::math::{IVec3, Vec3};
use crate::raycast::{raycast, RayHit};

pub(super) fn target(eye: Vec3, camera: Vec3, dir: Vec3, probe: impl Fn(IVec3) -> Option<BlockId>) -> Option<RayHit> {
    let hit = raycast(camera, dir, REACH + (camera - eye).length(), &probe)?;
    if (camera - eye).length() < 0.001 {
        return Some(hit);
    }
    // The entry point on the selected block, nudged inside for a reliable hands-to-surface trace.
    let surface = surface(camera, dir, hit);
    let to = surface - eye;
    let distance = to.length();
    if distance > REACH + 0.002 || distance < 0.001 {
        return None;
    }
    let visible = raycast(eye, to * (1.0 / distance), distance + 0.002, probe)?;
    (visible.block == hit.block).then_some(hit)
}

pub(super) fn surface(camera: Vec3, dir: Vec3, hit: RayHit) -> Vec3 {
    let min = hit.block.as_vec3();
    let mut enter: f64 = 0.0;
    for axis in 0..3 {
        let d = dir.get(axis);
        if d.abs() > 1e-9 {
            let a = (min.get(axis) - camera.get(axis)) / d;
            let b = (min.get(axis) + 1.0 - camera.get(axis)) / d;
            enter = enter.max(a.min(b));
        }
    }
    camera + dir * (enter + 0.001)
}

#[cfg(test)]
mod tests;
