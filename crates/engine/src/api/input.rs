//! Player input from the host: movement, look, mining and using, hotbar selection, fly toggle,
//! turning a belt (R) and dropping a belt line being dragged.
//! Movement, look and the mining/using buttons drive the body and hands directly; hotbar changes and
//! drops are actions, applied at the next tick.

use wasm_bindgen::prelude::*;

use crate::action::Action;
use crate::block;
use crate::factory::{self, Kind};
use crate::math::Vec3;
use crate::player::PlayerInput;
use crate::sound;
use crate::Game;

#[wasm_bindgen]
impl Game {
    pub fn set_move(&mut self, forward: f64, strafe: f64, jump: bool, sprint: bool, crouch: bool) {
        self.body_mut().input = PlayerInput { forward, strafe, jump, sprint, crouch };
    }

    /// Mouse look, in radians.
    pub fn look(&mut self, d_yaw: f64, d_pitch: f64) {
        let body = self.body_mut();
        body.yaw = (body.yaw + d_yaw).rem_euclid(std::f64::consts::TAU);
        body.pitch = (body.pitch - d_pitch).clamp(-1.55, 1.55);
    }

    /// B: ghost mode on or off (`ghost_mode.rs`).
    pub fn toggle_ghost_mode(&mut self) {
        self.ghost_mode = !self.ghost_mode;
        self.using = false;
        self.mine_block = None;
        self.mine_progress = 0.0;
    }

    pub fn ghost_mode(&self) -> bool {
        self.ghost_mode
    }

    /// Turns the targeted belt, splitter or filter a quarter turn clockwise, at the next tick, or the
    /// box of a quarry or the footprint of a multi-block machine in hand at once; with an advanced scanner in
    /// hand it steps the scanner's ore filter instead. False when nothing turns.
    pub fn rotate_target(&mut self) -> bool {
        if self.cycle_scan_filter() {
            return true;
        }
        if self.ghost_mode || self.holds_quarry() || self.held_footprint().is_some() {
            if !(self.ghost_mode && self.turn_held_blueprint()) {
                self.turn_placement();
            }
            self.play(sound::PLACE, block::sound::METAL, self.body().eye(), 0.4);
            return true;
        }
        let Some(hit) = self.target else { return false };
        if !factory::machine(hit.id).is_some_and(|m| matches!(m.kind, Kind::Belt | Kind::Router | Kind::Sensor)) {
            return false;
        }
        self.act(Action::Rotate { pos: hit.block });
        self.play(sound::PLACE, block::sound::METAL, hit.block.as_vec3() + Vec3::new(0.5, 0.5, 0.5), 0.6);
        true
    }

    /// Drops a belt line being dragged out (the host calls this when it frees the pointer).
    pub fn cancel_line(&mut self) {
        self.cancel_belt_line();
    }

    pub fn set_look(&mut self, yaw: f64, pitch: f64) {
        let body = self.body_mut();
        body.yaw = yaw.rem_euclid(std::f64::consts::TAU);
        body.pitch = pitch.clamp(-1.55, 1.55);
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
        self.act(Action::SelectSlot { slot: slot.min(u8::MAX as u32) as u8 });
    }

    pub fn scroll_slot(&mut self, delta: i32) {
        self.act(Action::ScrollSlot { delta: delta.signum() as i8 });
    }

    pub fn toggle_fly(&mut self) {
        let body = self.body_mut();
        body.flying = !body.flying;
        body.vel.y = 0.0;
    }

    /// Throws one item from the selected slot.
    pub fn drop_selected(&mut self) {
        self.act(Action::DropSelected { count: 1 });
    }
}
