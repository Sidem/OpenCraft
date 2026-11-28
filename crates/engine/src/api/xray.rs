//! The X-ray goggles' view for the host (`xray.rs`): the local switch, and this frame's records to draw.

use wasm_bindgen::prelude::*;

use crate::box_pack::PACKED_WORDS;
use crate::Game;

#[wasm_bindgen]
impl Game {
    /// Whether the local player wears X-ray goggles.
    pub fn xray_worn(&self) -> bool {
        self.inventory().has_xray_goggles()
    }

    /// Whether the view shows this frame: goggles worn, switched on, and not the strategy view.
    pub fn xray_active(&self) -> bool {
        self.xray.enabled && self.xray_worn() && !self.strategy.active()
    }

    /// The player's choice (kept by the host across sessions); it only matters while goggles are worn.
    pub fn xray_enabled(&self) -> bool {
        self.xray.enabled
    }

    pub fn set_xray_enabled(&mut self, on: bool) {
        self.xray.enabled = on;
    }

    /// The toggle key: flips the choice and returns "", or returns a line saying the goggles are not worn (and
    /// changes nothing).
    pub fn toggle_xray(&mut self) -> String {
        if !self.xray_worn() {
            return "Wear X-ray goggles in the Head slot first".to_string();
        }
        self.xray.enabled = !self.xray.enabled;
        String::new()
    }

    /// Byte offset of this frame's X-ray boxes (`xray_count()` of them, packed as `instance_ptr`'s, round bodies left
    /// out), the factory's boxes near the camera; the host draws them behind the terrain only.
    pub fn xray_ptr(&self) -> usize {
        self.packed.xray.as_ptr() as usize
    }

    pub fn xray_count(&self) -> usize {
        self.packed.xray.len() / PACKED_WORDS
    }

    /// The round bodies among them.
    pub fn xray_round_ptr(&self) -> usize {
        self.packed.xray_round.as_ptr() as usize
    }

    pub fn xray_round_count(&self) -> usize {
        self.packed.xray_round.len() / PACKED_WORDS
    }

    /// Blocks around the camera within which buried machines show, and the share of it where they start to fade.
    pub fn xray_radius(&self) -> f32 {
        crate::xray::RADIUS
    }

    pub fn xray_fade_from(&self) -> f32 {
        crate::xray::FADE_FROM
    }
}
