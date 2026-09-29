//! Saving and loading worlds for the host, which stores the bytes (format: `save.rs`), plus the
//! figures its world list shows.

use wasm_bindgen::prelude::*;

use crate::{Game, TICK};

#[wasm_bindgen]
impl Game {
    /// The whole world as bytes.
    pub fn save(&self) -> Vec<u8> {
        self.save_bytes()
    }

    /// A game from `save()` bytes, without the co-op guests the host had when it saved. Throws a
    /// message a player can read if the bytes can't be loaded.
    pub fn load(bytes: &[u8], view_radius: u32) -> Result<Game, String> {
        let mut game = Game::from_save(bytes, view_radius)?;
        game.release_guests();
        Ok(game)
    }

    pub fn seed(&self) -> u32 {
        self.sim.world.generator().seed()
    }

    /// The generator version this world was made with (1 = the terrain before Milestone 4).
    pub fn worldgen_version(&self) -> u32 {
        self.sim.world.generator().version()
    }

    /// Game time played in this world, in seconds.
    pub fn play_seconds(&self) -> f64 {
        self.sim.tick as f64 * TICK
    }
}
