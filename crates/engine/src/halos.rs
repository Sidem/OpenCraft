//! Halos round the blocks that give light (lamps, panel lights, torches): a soft glow the host draws as a
//! camera-facing sprite with additive blending (`render/halos.ts`), so a lamp reads as lit from afar, not only by
//! the ground it lights. Presentation only, from the render cache's list of light-giving blocks (`world/glows.rs`).
//!
//! A halo is [`HALO_FLOATS`] floats: camera-relative centre, radius in blocks, colour (0..1) and strength. To give a
//! new light-giving block its own look: a row in `look`; others get the default.

use crate::block::{self, BlockId};
use crate::chunk::CHUNK_SIZE;
use crate::math::Vec3;
use crate::Game;

/// Floats per halo: x, y, z (camera-relative), radius, r, g, b, strength.
pub const HALO_FLOATS: usize = 8;
/// Halos further than this from the camera are not drawn (the light they give still shows on the ground).
pub const HALO_RANGE: f64 = 160.0;
/// At most this many halos a frame, nearest chunks first in no particular order beyond that (a base lit by
/// thousands of lamps keeps its frame cost bounded).
const MAX_HALOS: usize = 2048;

/// How a light-giving block's halo looks: offset of its centre from the cell's centre (up), radius, colour, strength.
fn look(b: BlockId) -> (f64, f32, [f32; 3], f32) {
    match b {
        block::LAMP => (0.0, 1.7, [1.0, 0.72, 0.32], 1.0),
        block::TORCH => (0.15, 0.75, [1.0, 0.58, 0.22], 0.8),
        block::PANEL_LIGHT => (0.0, 1.2, [0.86, 0.92, 1.0], 0.7),
        _ => (0.0, 1.0, [1.0, 0.9, 0.7], 0.7),
    }
}

impl Game {
    /// Fills `halos` for this frame from the light-giving blocks within [`HALO_RANGE`] of `eye`.
    pub(crate) fn write_halos(&mut self, eye: Vec3) {
        self.halos.clear();
        let reach = HALO_RANGE + f64::from(CHUNK_SIZE);
        for (chunk, cells) in self.sim.world.glows() {
            let mid = Vec3::new(f64::from(chunk.x), f64::from(chunk.y), f64::from(chunk.z)) * f64::from(CHUNK_SIZE)
                + Vec3::new(1.0, 1.0, 1.0) * (f64::from(CHUNK_SIZE) * 0.5)
                - eye;
            if mid.length() > reach {
                continue;
            }
            for &(cell, b) in cells {
                if self.halos.len() >= MAX_HALOS * HALO_FLOATS {
                    return;
                }
                let (up, radius, [r, g, bl], strength) = look(b);
                let at = cell.as_vec3() + Vec3::new(0.5, 0.5 + up, 0.5) - eye;
                if at.length() <= HALO_RANGE {
                    self.halos.extend_from_slice(&[at.x as f32, at.y as f32, at.z as f32, radius, r, g, bl, strength]);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests;
