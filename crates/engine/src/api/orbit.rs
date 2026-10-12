//! The satellites the sky draws (`satellites.rs`): a query, never a change to the world.

use wasm_bindgen::prelude::*;

use crate::Game;

#[wasm_bindgen]
impl Game {
    /// The satellites above the horizon for the camera: x, y, z of a unit direction and the kind's index, four floats each
    /// (empty when none are up, so a frame without satellites allocates nothing).
    pub fn sky_satellites(&self) -> Vec<f32> {
        let eye = self.render_eye;
        self.sim.factory.orbit.sky_directions(eye.x, eye.z, self.sim.tick)
    }
}
