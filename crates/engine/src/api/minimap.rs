//! The minimap (minimap.rs): redraw on request, the image (zero-copy) and other players' marks.

use wasm_bindgen::prelude::*;

use crate::minimap::{Minimap, MAP_SIZE};
use crate::Game;

#[wasm_bindgen]
impl Game {
    /// Redraws the map around the local player if they moved to another column or chunks changed
    /// nearby. Returns whether the image changed. The host calls it a few times a second at most.
    pub fn minimap_redraw(&mut self) -> bool {
        let p = self.body().pos.floor();
        self.minimap.redraw(&self.sim.world, (p.x, p.z))
    }

    /// Byte offset of the map's RGBA pixels (`minimap_size()`² of them, north up) in wasm memory.
    pub fn minimap_ptr(&self) -> usize {
        self.minimap.pixels.as_ptr() as usize
    }

    pub fn minimap_size(&self) -> usize {
        MAP_SIZE
    }

    /// Other players on the map: (x offset, z offset in blocks, yaw) per player.
    pub fn minimap_players(&self) -> Vec<f32> {
        Minimap::player_marks(&self.bodies, self.local, self.body().pos)
    }
}
