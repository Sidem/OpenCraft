//! Where a long line of sight meets the ground: the telescope's aim (presentation only, a query: it reads blocks and the
//! generator and never creates or changes state).
//!
//! The ground of a column is its highest solid block that is not part of a tree: read from the loaded chunks where they
//! are loaded (so edits and buildings count), else from the generator's heightfield (`WorldGen::height_at`, so ground
//! that is not loaded yet still has a height). `cast` marches a ray over that surface in steps that grow with the
//! distance and with the height above the ground, then bisects the last step. A ray that starts under the surface, or
//! never meets it within `RANGE`, finds nothing. A thin ridge narrower than a step can be skipped on a far, grazing
//! ray; the hit is then the ground behind it, still a real point on the ray.
//! To tune the cost against the accuracy: `STEP_PER_DISTANCE`, `MAX_STEP`, `SLOPE_SAFETY`.

use crate::block::{is_tree, SOLID};
use crate::chunk::{CHUNK_MASK, CHUNK_SHIFT, CHUNK_SIZE};
use crate::math::{IVec3, Vec3};
use crate::world::World;
use crate::worldgen::{WORLD_HEIGHT, WORLD_HEIGHT_CHUNKS};

/// How far a line of sight is followed, in blocks.
pub const RANGE: f64 = 6000.0;
/// The smallest step near the eye, and its growth with the distance, in blocks.
const NEAR_STEP: f64 = 1.0;
const STEP_PER_DISTANCE: f64 = 0.005;
/// The largest step, and the share of the height above the ground a step may cover (steeper ground than this is skipped).
const MAX_STEP: f64 = 48.0;
const SLOPE_SAFETY: f64 = 0.5;
/// Halvings of the last step.
const REFINE: u32 = 12;

/// The ground a line of sight meets: its column, the y of that column's top block, the distance along the line, and
/// whether the column was loaded (else its height is the generator's).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Aim {
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub dist: f64,
    pub loaded: bool,
}

/// The y of the top ground block of column (x, z), and whether it came from loaded chunks.
pub fn column_top(world: &World, x: i32, z: i32) -> (i32, bool) {
    match loaded_top(world, x, z) {
        Some(y) => (y, true),
        None => (world.generator().height_at(x, z), false),
    }
}

/// Follows the line from `eye` along the unit vector `dir` to the first ground it meets within `range`.
pub fn cast(world: &World, eye: Vec3, dir: Vec3, range: f64) -> Option<Aim> {
    // The signed height of the line above the surface (the top face of the top block) at distance `t`.
    let probe = |t: f64| {
        let p = eye + dir * t;
        let (top, loaded) = column_top(world, p.x.floor() as i32, p.z.floor() as i32);
        (p, top, loaded, p.y - f64::from(top + 1))
    };
    let (_, _, _, mut gap) = probe(0.0);
    if gap < 0.0 {
        return None;
    }
    let mut t = 0.0;
    while t < range {
        let step = (gap * SLOPE_SAFETY).clamp((t * STEP_PER_DISTANCE).max(NEAR_STEP), MAX_STEP);
        let next = (t + step).min(range);
        let (p, _, _, below) = probe(next);
        if below <= 0.0 {
            let (mut lo, mut hi) = (t, next);
            for _ in 0..REFINE {
                let mid = (lo + hi) / 2.0;
                if probe(mid).3 > 0.0 {
                    lo = mid;
                } else {
                    hi = mid;
                }
            }
            let (p, top, loaded, _) = probe(hi);
            return Some(Aim { x: p.x.floor() as i32, y: top, z: p.z.floor() as i32, dist: hi, loaded });
        }
        if dir.y >= 0.0 && p.y > f64::from(WORLD_HEIGHT) {
            return None; // climbing out above every mountain
        }
        (t, gap) = (next, below);
    }
    None
}

/// The top ground block of column (x, z) in the loaded chunks, or `None` when the column is not loaded.
fn loaded_top(world: &World, x: i32, z: i32) -> Option<i32> {
    let (cx, cz) = (x >> CHUNK_SHIFT, z >> CHUNK_SHIFT);
    let (lx, lz) = ((x & CHUNK_MASK) as usize, (z & CHUNK_MASK) as usize);
    for cy in (0..WORLD_HEIGHT_CHUNKS).rev() {
        let chunk = world.loaded_chunk(IVec3::new(cx, cy, cz))?;
        for ly in (0..CHUNK_SIZE as usize).rev() {
            let b = chunk.get(lx, ly, lz);
            if SOLID[b as usize] && !is_tree(b) {
                return Some(cy * CHUNK_SIZE + ly as i32);
            }
        }
    }
    Some(-1)
}
