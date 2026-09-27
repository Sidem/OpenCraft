//! The minimap (minimap.rs): redraw on request, the image (zero-copy), other players' marks, and the
//! deposit and machine marks with the prospected deposits the host keeps (minimap/marks.rs).

use wasm_bindgen::prelude::*;

use crate::minimap::{Minimap, MAP_SIZE, MARK_FIELDS};
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

    /// Prospected deposits and machines inside the image, `minimap_mark_fields()` numbers each: x and z
    /// offset in blocks from the image's centre column, colour (0xRRGGBB), shape (0 deposit, 1 machine).
    pub fn minimap_marks(&self) -> Vec<i32> {
        self.minimap.marks(&self.sim.factory)
    }

    pub fn minimap_mark_fields(&self) -> u32 {
        MARK_FIELDS as u32
    }

    /// The prospected deposits still holding ore, for the host to keep with the world's record.
    pub fn known_deposits(&self) -> Vec<i32> {
        self.minimap.known.export(&self.sim.factory)
    }

    /// Restores `known_deposits` from an earlier session of this world.
    pub fn set_known_deposits(&mut self, data: &[i32]) {
        self.minimap.known.import(data, self.sim.world.generator());
    }
}
