//! Drawing the street lamps (presentation only; nothing here changes the core). Every head block that is a street lamp's,
//! from the render cache's list of light-giving blocks (`world/glows.rs`), gets the lamp's model (`model.rs`) over its
//! column while within [`FAR`] blocks of the eye, at most [`MAX_LAMPS`] a frame.

use crate::block::STREET;
use crate::chunk::CHUNK_SIZE;
use crate::math::Vec3;
use crate::Game;

use super::{arm_dir, street_model};

/// Lamps farther than this from the eye are not drawn (their halo and the light they give still are).
const FAR: f64 = 128.0;
const MAX_LAMPS: usize = 256;

impl Game {
    /// Draws the lamps near the camera at `eye`.
    pub(crate) fn write_streets(&mut self, eye: Vec3) {
        let reach = FAR + f64::from(CHUNK_SIZE);
        let mut drawn = 0;
        for (chunk, cells) in self.sim.world.glows() {
            let mid = Vec3::new(f64::from(chunk.x), f64::from(chunk.y), f64::from(chunk.z)) * f64::from(CHUNK_SIZE)
                + Vec3::new(1.0, 1.0, 1.0) * (f64::from(CHUNK_SIZE) * 0.5)
                - eye;
            if mid.length() > reach {
                continue;
            }
            for &(head, _) in cells.iter().filter(|c| c.1 == STREET) {
                // The column's middle: the head cell's middle, one and a half cells down.
                let c = head.as_vec3() + Vec3::new(0.5, 0.5 - 1.5, 0.5) - eye;
                if c.length() > FAR || drawn >= MAX_LAMPS {
                    continue;
                }
                street_model(&mut self.instances, c, arm_dir(&self.sim.world, head));
                drawn += 1;
            }
        }
    }
}
