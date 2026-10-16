//! The satellites the sky draws (`satellites.rs`) and the launches the HUD toasts (`orbit_fx.rs`): queries and
//! presentation, never a change to the world.

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

    /// Pops the next satellite launch to toast ("in orbit"): its item, or 0 when there is none (`orbit_fx.rs`).
    pub fn next_launch(&mut self) -> u16 {
        self.orbit_fx.news.pop_front().map_or(0, |item| item.0)
    }
}
