//! The JavaScript-facing API of [`crate::Game`], split by area. Each file is its own
//! `#[wasm_bindgen] impl Game` block; methods only read state or forward to a module, so the
//! game logic never lives here.
//!
//! To add a wasm method: put it in the file for its area, keep it a one- or few-liner, and run
//! `npm run build:wasm` so `web/src/wasm/engine.d.ts` picks it up.

mod analytics;
mod blueprint;
mod content;
mod crafting;
mod creative;
mod debug;
mod guide;
mod hand_waste;
mod hud;
mod input;
mod inventory;
mod lessons;
mod machine;
mod minimap;
mod net;
mod opening;
mod orbit;
mod prospect;
mod render;
mod research;
mod ruins;
mod save;
mod sites;
mod surroundings;
mod telescope;
mod towns;
mod view;

use wasm_bindgen::prelude::*;

/// Runs once when the wasm module loads: panics show in the browser console.
#[wasm_bindgen(start)]
pub fn start() {
    console_error_panic_hook::set_once();
}
