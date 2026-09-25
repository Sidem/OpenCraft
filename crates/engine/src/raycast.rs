//! Voxel traversal (Amanatides & Woo) for block targeting. `raycast` walks the grid cell by cell
//! and returns the first cell the `probe` accepts, with the face normal it entered through.

use crate::block::BlockId;
use crate::math::{IVec3, Vec3};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RayHit {
    pub block: IVec3,
    /// Normal of the face the ray entered through; zero if the ray started inside the block.
    pub normal: IVec3,
    pub id: BlockId,
}

/// Walks the grid from `origin` along unit vector `dir` and returns the first cell for which
/// `probe` yields a block id.
pub fn raycast(
    origin: Vec3,
    dir: Vec3,
    max_dist: f64,
    mut probe: impl FnMut(IVec3) -> Option<BlockId>,
) -> Option<RayHit> {
    let mut cell = origin.floor();
    if let Some(id) = probe(cell) {
        return Some(RayHit { block: cell, normal: IVec3::ZERO, id });
    }
    let d = [dir.x, dir.y, dir.z];
    let o = [origin.x, origin.y, origin.z];
    let c = [cell.x, cell.y, cell.z];
    let mut step = [0i32; 3];
    let mut t_max = [f64::INFINITY; 3];
    let mut t_delta = [f64::INFINITY; 3];
    for a in 0..3 {
        if d[a] > 0.0 {
            step[a] = 1;
            t_delta[a] = 1.0 / d[a];
            t_max[a] = (c[a] as f64 + 1.0 - o[a]) / d[a];
        } else if d[a] < 0.0 {
            step[a] = -1;
            t_delta[a] = -1.0 / d[a];
            t_max[a] = (o[a] - c[a] as f64) / -d[a];
        }
    }
    loop {
        let a = if t_max[0] < t_max[1] {
            if t_max[0] < t_max[2] {
                0
            } else {
                2
            }
        } else if t_max[1] < t_max[2] {
            1
        } else {
            2
        };
        if t_max[a] > max_dist {
            return None;
        }
        let mut normal = IVec3::ZERO;
        match a {
            0 => {
                cell.x += step[0];
                normal.x = -step[0];
            }
            1 => {
                cell.y += step[1];
                normal.y = -step[1];
            }
            _ => {
                cell.z += step[2];
                normal.z = -step[2];
            }
        }
        t_max[a] += t_delta[a];
        if let Some(id) = probe(cell) {
            return Some(RayHit { block: cell, normal, id });
        }
    }
}

#[cfg(test)]
mod tests;
