//! Shoulder-camera targeting. The crosshair's ray chooses a surface; hands still enforce reach and
//! line of sight, so the offset cannot mine through walls or increase the interaction range.

use crate::block::BlockId;
use crate::hitbox::Boxes;
use crate::math::{IVec3, Vec3};
use crate::raycast::{raycast, RayHit};

/// `slim` gives the hit boxes of a cell holding a slender model (a pole, a belt): the ray must touch
/// them, so it passes through the rest of the cell to whatever lies behind.
pub(super) fn target(
    eye: Vec3,
    camera: Vec3,
    dir: Vec3,
    reach: f64,
    probe: impl Fn(IVec3) -> Option<BlockId>,
    slim: impl Fn(IVec3) -> Option<Boxes>,
) -> Option<RayHit> {
    let cast = |origin: Vec3, dir: Vec3, max: f64| {
        raycast(origin, dir, max, |p| {
            let id = probe(p)?;
            let touched = |b: Boxes| b.enter(origin - p.as_vec3(), dir).is_some_and(|t| t <= max + 0.002);
            slim(p).is_none_or(touched).then_some(id)
        })
    };
    let hit = cast(camera, dir, reach + (camera - eye).length())?;
    if (camera - eye).length() < 0.001 {
        return Some(hit);
    }
    // The entry point on the selected block, nudged inside for a reliable hands-to-surface trace.
    let surface = surface(camera, dir, hit, slim(hit.block).as_ref());
    let to = surface - eye;
    let distance = to.length();
    if distance > reach + 0.002 || distance < 0.001 {
        return None;
    }
    let visible = cast(eye, to * (1.0 / distance), distance + 0.002)?;
    (visible.block == hit.block).then_some(hit)
}

/// Where the ray first touches the target: its cell, or its hit boxes when it has them.
pub(super) fn surface(camera: Vec3, dir: Vec3, hit: RayHit, slim: Option<&Boxes>) -> Vec3 {
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
    if let Some(t) = slim.and_then(|b| b.enter(camera - min, dir)) {
        enter = enter.max(t);
    }
    camera + dir * (enter + 0.001)
}

#[cfg(test)]
mod tests;
