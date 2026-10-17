//! Where rain and snow stop, for the host's precipitation pass (`web/src/render/precip.ts`). Presentation only: read from
//! the loaded blocks, never saved, never part of the core state; polled a few times a second while it falls.
//!
//! `roof_grid` samples a square of columns around the camera at a coarse stride and returns, for each, the height a drop
//! lands at: the top of the first block that stops it (anything meshed but a plant: stone, leaves, glass, water) found
//! looking down from `UP` blocks over the eye, or the bottom of the range when nothing does. A column under a roof or in a
//! cave therefore reads high (the roof, or the top of the range inside rock), and the host draws no drops below that.
//! Layout of the floats: the grid's lowest corner `x`, `z` (block coordinates), then `SIDE * SIDE` heights, row by row
//! along x. To tune the area or the detail: the constants below (the host's `ROOF_*` constants in precip.ts follow them).

use crate::block::{self, AIR};
use crate::math::{IVec3, Vec3};
use crate::world::World;
use crate::worldgen::WORLD_HEIGHT;

/// Cells each way from the camera's, blocks per cell, and the cells along one side.
pub const HALF: i32 = 8;
pub const STRIDE: i32 = 2;
pub const SIDE: usize = (2 * HALF + 1) as usize;
/// How far above and below the eye the columns are read.
const UP: i32 = 40;
const DOWN: i32 = 20;

pub fn roof_grid(world: &World, eye: Vec3) -> Vec<f32> {
    let cell = eye.floor();
    let top = (cell.y + UP).min(WORLD_HEIGHT - 1);
    let bottom = cell.y - DOWN;
    let ox = cell.x.div_euclid(STRIDE) * STRIDE - HALF * STRIDE;
    let oz = cell.z.div_euclid(STRIDE) * STRIDE - HALF * STRIDE;
    let mut out = Vec::with_capacity(2 + SIDE * SIDE);
    out.push(ox as f32);
    out.push(oz as f32);
    for j in 0..SIDE as i32 {
        for i in 0..SIDE as i32 {
            let (x, z) = (ox + i * STRIDE, oz + j * STRIDE);
            let stop = (bottom..=top).rev().find(|&y| {
                let b = world.get_block(IVec3::new(x, y, z)).unwrap_or(AIR);
                block::MESHED[b as usize] && !block::PLANT[b as usize]
            });
            out.push(stop.map_or(bottom, |y| y + 1) as f32);
        }
    }
    out
}
