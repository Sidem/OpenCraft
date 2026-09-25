//! Data the host needs to draw and play a frame: streaming work, chunk mesh events, the camera,
//! box instances, sound events and the texture atlas. Bulk data is exposed as `*_ptr` + length
//! so the host reads it zero-copy from wasm memory.

use wasm_bindgen::prelude::*;

use crate::block;
use crate::factory::INSTANCE_FLOATS;
use crate::textures;
use crate::world::Event;
use crate::Game;

#[wasm_bindgen]
impl Game {
    pub fn set_view_radius(&mut self, r: u32) {
        self.sim.world.set_view_radius(r as i32);
    }

    /// Prepares this frame's work queues. Call once per frame before [`Game::work_step`].
    pub fn begin_work(&mut self) {
        self.sim.world.begin_work();
    }

    /// Generates or meshes one chunk (nearest first). Returns false when there is nothing to do.
    /// The host calls this in a loop until its per-frame time budget is spent.
    pub fn work_step(&mut self) -> bool {
        self.sim.world.work_step()
    }

    pub fn ready(&self) -> bool {
        self.sim.world.area_ready(1)
    }

    /// Pops the next renderer event: 0 = none, 1 = chunk mesh (see `mesh_*`), 2 = chunk unloaded.
    /// The position of either event is available through `event_x/y/z`.
    pub fn next_event(&mut self) -> u32 {
        self.cur_mesh = None;
        match self.sim.world.events.pop_front() {
            None => 0,
            Some(Event::Mesh(m)) => {
                self.cur_event_pos = m.pos;
                self.cur_mesh = Some(m);
                1
            }
            Some(Event::Unload(p)) => {
                self.cur_event_pos = p;
                2
            }
        }
    }

    pub fn event_x(&self) -> i32 {
        self.cur_event_pos.x
    }

    pub fn event_y(&self) -> i32 {
        self.cur_event_pos.y
    }

    pub fn event_z(&self) -> i32 {
        self.cur_event_pos.z
    }

    /// Byte offset of the current mesh's packed vertices in wasm memory.
    pub fn mesh_ptr(&self) -> usize {
        self.cur_mesh.as_ref().map_or(0, |m| m.verts.as_ptr() as usize)
    }

    /// Number of u32 vertices in the current mesh.
    pub fn mesh_len(&self) -> usize {
        self.cur_mesh.as_ref().map_or(0, |m| m.verts.len())
    }

    pub fn mesh_opaque_quads(&self) -> u32 {
        self.cur_mesh.as_ref().map_or(0, |m| m.opaque_quads)
    }

    pub fn mesh_cutout_quads(&self) -> u32 {
        self.cur_mesh.as_ref().map_or(0, |m| m.cutout_quads)
    }

    /// Camera position for this frame, interpolated between the last two ticks.
    pub fn eye_x(&self) -> f64 {
        self.render_eye.x
    }

    pub fn eye_y(&self) -> f64 {
        self.render_eye.y
    }

    pub fn eye_z(&self) -> f64 {
        self.render_eye.z
    }

    pub fn yaw(&self) -> f64 {
        self.player.yaw
    }

    pub fn pitch(&self) -> f64 {
        self.player.pitch
    }

    /// Byte offset of this frame's box instances (dropped items, belt items, machine parts):
    /// `instance_count()` records of `factory::INSTANCE_FLOATS` f32 each.
    pub fn instance_ptr(&self) -> usize {
        self.instances.as_ptr() as usize
    }

    pub fn instance_count(&self) -> usize {
        self.instances.len() / INSTANCE_FLOATS
    }

    /// Byte offset of this frame's sound events: `sound_count()` records of
    /// (kind, material, camera-relative x, y, z, volume) as f32.
    pub fn sound_ptr(&self) -> usize {
        self.sounds.as_ptr() as usize
    }

    pub fn sound_count(&self) -> usize {
        self.sounds.count()
    }

    /// Call after the host has played this frame's sounds.
    pub fn clear_sounds(&mut self) {
        self.sounds.clear();
    }

    /// Texture layer for a block face (0..6 = +X, -X, +Y, -Y, +Z, -Z).
    pub fn block_face_texture(&self, id: u8, face: u32) -> u32 {
        block::def(id).faces[(face as usize).min(5)] as u32
    }

    pub fn texture_ptr(&self) -> usize {
        self.textures.as_ptr() as usize
    }

    pub fn texture_byte_len(&self) -> usize {
        self.textures.len()
    }

    pub fn texture_size(&self) -> u32 {
        textures::TEX_SIZE as u32
    }

    pub fn texture_layers(&self) -> u32 {
        block::tex::COUNT as u32
    }
}
