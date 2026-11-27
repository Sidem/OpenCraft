//! Co-op flares for the host (`web/src/flare.ts`, `net/`): where one raised now would stand, and lighting one, the local
//! player's or one a peer raised. Presentation only (`flares.rs`).

use wasm_bindgen::prelude::*;

use crate::math::Vec3;
use crate::Game;

#[wasm_bindgen]
impl Game {
    /// The foot of a flare raised now (x, y, z), or empty when the view meets no ground in reach.
    pub fn flare_spot(&self) -> Vec<f64> {
        self.flare_spot_at().map_or_else(Vec::new, |p| vec![p.x, p.y, p.z])
    }

    /// Lights a flare at (`x`, `y`, `z`) in player slot `player`'s colour, for its whole burn.
    pub fn light_flare(&mut self, x: f64, y: f64, z: f64, player: u8) {
        self.light_flare_at(Vec3::new(x, y, z), player);
    }

    /// How many flares burn now.
    pub fn flare_count(&self) -> u32 {
        self.flares.list.len() as u32
    }
}
