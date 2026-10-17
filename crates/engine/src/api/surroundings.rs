//! The local player's surroundings for the host's ambient sound (`surroundings/mod.rs` has the layout).

use wasm_bindgen::prelude::*;

use crate::surroundings;
use crate::Game;

#[wasm_bindgen]
impl Game {
    /// Nine floats describing where the camera is (biome, sky, openness, height, depth, sea, fresh water, trees,
    /// machine din); the host polls this a few times a second.
    pub fn surroundings(&self) -> Vec<f32> {
        surroundings::survey(&self.sim.world, &self.sim.factory, self.render_eye).to_vec()
    }

    /// Where rain and snow stop around the camera: the grid's corner (x, z), then the landing height of each column
    /// (`surroundings/roof.rs`); the host polls it while it is precipitating.
    pub fn rain_roof(&self) -> Vec<f32> {
        surroundings::roof::roof_grid(&self.sim.world, self.render_eye)
    }
}
