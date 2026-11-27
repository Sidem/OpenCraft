//! Presentation-only factory beat readings for the host's factory instrument (`factory/beat.rs`,
//! `web/src/audio/factory-beat.ts`). Never mutates core state.

use wasm_bindgen::prelude::*;

use crate::factory;
use crate::Game;

#[wasm_bindgen]
impl Game {
    /// Per-family summary of working machines near the camera (`factory/beat.rs` layout).
    pub fn factory_beat(&self) -> Vec<f32> {
        self.sim.factory.beat_near(self.render_eye).to_vec()
    }

    /// Current deterministic simulation tick (60 Hz), used to lock the factory instrument to a shared beat.
    pub fn factory_beat_tick(&self) -> f64 {
        self.sim.tick as f64
    }

    /// Hard hearing radius (blocks) of the factory instrument (`factory::BEAT_RADIUS`).
    pub fn factory_beat_radius(&self) -> f32 {
        factory::BEAT_RADIUS as f32
    }
}
