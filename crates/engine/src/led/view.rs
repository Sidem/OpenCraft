//! Drawing the LED lamps' colour (presentation only; nothing here changes the core). A lamp a player coloured gets a
//! flat self-lit box over its cell in that colour (a tinted beacon, `factory::mark_beacon_tinted`) while within
//! [`NEAR`] blocks; farther off, and for lamps left at the default, the halo (`halos.rs`) carries the colour.
//!
//! To change the look: `write_leds` and the constants below.

use crate::block::{is_led, tex};
use crate::chunk::CHUNK_SIZE;
use crate::factory::{mark_beacon_tinted, push_box};
use crate::math::Vec3;
use crate::Game;

/// Coloured lamps within this many blocks of the eye are drawn as a tinted box.
const NEAR: f64 = 64.0;
/// At most this many tint boxes a frame.
const MAX_BOXES: usize = 256;
/// A hair larger than the cell, so the box covers the lamp's own faces and never fights them.
const SIZE: f32 = 1.01;

impl Game {
    /// Draws the tinted lamps near the camera at `eye`.
    pub(crate) fn write_leds(&mut self, eye: Vec3) {
        if self.sim.leds.is_empty() {
            return;
        }
        let reach = NEAR + f64::from(CHUNK_SIZE);
        let mut drawn = 0;
        for (chunk, cells) in self.sim.world.glows() {
            let mid = Vec3::new(f64::from(chunk.x), f64::from(chunk.y), f64::from(chunk.z)) * f64::from(CHUNK_SIZE)
                + Vec3::new(1.0, 1.0, 1.0) * (f64::from(CHUNK_SIZE) * 0.5)
                - eye;
            if mid.length() > reach {
                continue;
            }
            for &(cell, b) in cells {
                let Some(rgb) = self.sim.leds.picked(cell).filter(|_| is_led(b)) else { continue };
                let at = cell.as_vec3() + Vec3::new(0.5, 0.5, 0.5) - eye;
                if at.length() > NEAR || drawn >= MAX_BOXES {
                    continue;
                }
                push_box(&mut self.instances, at, 0.0, [SIZE; 3], 0.0, [tex::LED_PANEL; 3], false);
                mark_beacon_tinted(&mut self.instances, rgb);
                drawn += 1;
            }
        }
    }
}
