//! Saving and loading worlds for the host, which stores the bytes (format: `save.rs`), plus the
//! figures its world list shows.

use wasm_bindgen::prelude::*;

use crate::action::Action;
use crate::net::Role;
use crate::worldgen::{DEPOSITS_LATEST, WORLDGEN_VERSION};
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

    /// The primary terrain (`ground`) generator version this world was made with (1 = before Milestone 4).
    pub fn worldgen_version(&self) -> u32 {
        self.sim.world.generator().version()
    }

    /// The four generator layer versions `[ground, water, cover, deposits]`.
    pub fn worldgen_layers(&self) -> Vec<u32> {
        self.sim.world.generator().layers().all().to_vec()
    }

    /// The `deposits` layer version of this world.
    pub fn deposits_version(&self) -> u32 {
        self.sim.world.generator().layers().deposits
    }

    /// The newest generator version that changed the `deposits` layer.
    pub fn latest_deposits_version(&self) -> u32 {
        DEPOSITS_LATEST
    }

    /// The newest overall world generator version.
    pub fn latest_worldgen_version(&self) -> u32 {
        WORLDGEN_VERSION
    }

    /// Whether this world can safely upgrade its `deposits` layer (host/solo only).
    pub fn can_upgrade_deposits(&self) -> bool {
        !matches!(self.role, Role::Client(_)) && self.sim.can_upgrade_deposits()
    }

    /// Queues upgrading this world's `deposits` layer to the newest version (host/solo only).
    pub fn upgrade_deposits(&mut self) {
        if self.can_upgrade_deposits() {
            self.act(Action::UpgradeDeposits);
        }
    }

    /// Game time played in this world, in seconds.
    pub fn play_seconds(&self) -> f64 {
        self.sim.tick as f64 * TICK
    }
}
