//! Player input from the host: movement, look, mining and using, hotbar selection, fly toggle.

use wasm_bindgen::prelude::*;

use crate::player::PlayerInput;
use crate::Game;

#[wasm_bindgen]
impl Game {
    pub fn set_move(&mut self, forward: f64, strafe: f64, jump: bool, sprint: bool, crouch: bool) {
        self.player.input = PlayerInput { forward, strafe, jump, sprint, crouch };
    }

    /// Mouse look, in radians.
    pub fn look(&mut self, d_yaw: f64, d_pitch: f64) {
        self.player.yaw = (self.player.yaw + d_yaw).rem_euclid(std::f64::consts::TAU);
        self.player.pitch = (self.player.pitch - d_pitch).clamp(-1.55, 1.55);
    }

    pub fn set_look(&mut self, yaw: f64, pitch: f64) {
        self.player.yaw = yaw.rem_euclid(std::f64::consts::TAU);
        self.player.pitch = pitch.clamp(-1.55, 1.55);
    }

    pub fn set_mining(&mut self, on: bool) {
        self.mining = on;
    }

    /// Hold to place blocks: places immediately, then repeats while held.
    pub fn set_using(&mut self, on: bool) {
        if on && !self.using {
            self.use_cooldown = 0.0;
        }
        self.using = on;
    }

    pub fn select_slot(&mut self, slot: u32) {
        self.inventory.select(slot as usize);
    }

    pub fn scroll_slot(&mut self, delta: i32) {
        self.inventory.scroll(delta);
    }

    pub fn toggle_fly(&mut self) {
        self.player.flying = !self.player.flying;
        self.player.vel.y = 0.0;
    }

    /// Throws one item from the selected slot.
    pub fn drop_selected(&mut self) {
        if let Some((item, n)) = self.inventory.take_selected(1) {
            self.throw(item, n);
        }
    }
}
