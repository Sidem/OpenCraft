//! The telescope's API: whether it is up and the eased zoom the host draws, the pin the player just made, and the pins to
//! draw as columns (`telescope.rs`). The pins themselves live in the browser (`web/src/ui/pins.ts`).

use wasm_bindgen::prelude::*;

use crate::telescope::MAX_PINS;
use crate::Game;

#[wasm_bindgen]
impl Game {
    /// Whether the player is looking through a telescope.
    pub fn scope_on(&self) -> bool {
        self.scope_active()
    }

    /// The zoom to draw now, a multiple of the normal view (1 when no telescope is up); eased, never jumps.
    pub fn scope_zoom(&self) -> f64 {
        self.scope_view_zoom()
    }

    /// Lowers the telescope (the host calls this when it frees the pointer).
    pub fn close_scope(&mut self) {
        self.lower_scope();
    }

    /// The ground the player just pinned, once: `[x, y, z]` (y is the top block); empty when nothing is waiting.
    pub fn take_pin_request(&mut self) -> Vec<i32> {
        self.take_scope_pin().map(Vec::from).unwrap_or_default()
    }

    /// Replaces the pins drawn as columns: `[x, y, z, 0xRRGGBB]` each, `y` being `i32::MIN` for a pin without a height.
    pub fn set_scope_pins(&mut self, pins: &[i32]) {
        self.set_scope_marks(pins);
    }

    /// The most pins a world keeps (and draws).
    pub fn scope_pin_limit(&self) -> u32 {
        MAX_PINS as u32
    }

    /// The marker meaning "no height" in `set_scope_pins`.
    pub fn scope_no_height(&self) -> i32 {
        crate::telescope::NO_HEIGHT
    }
}
