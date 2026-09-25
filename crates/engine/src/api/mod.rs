//! The JavaScript-facing API of [`crate::Game`], split by area. Each file is its own
//! `#[wasm_bindgen] impl Game` block; methods only read state or forward to a module, so the
//! game logic never lives here.
//!
//! To add a wasm method: put it in the file for its area, keep it a one- or few-liner, and run
//! `npm run build:wasm` so `web/src/wasm/engine.d.ts` picks it up.

mod content;
mod crafting;
mod debug;
mod hud;
mod input;
mod inventory;
mod render;
mod save;
