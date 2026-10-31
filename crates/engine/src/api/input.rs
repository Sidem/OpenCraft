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
        if !self.ghost_mode {
            self.forget_blueprint_box();
        }
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
        let belt_like =
            factory::machine(hit.id).is_some_and(|m| matches!(m.kind, Kind::Belt | Kind::Router | Kind::Sensor));
        if !belt_like && !factory::turns(hit.id) {
            return false;
        }
        self.act(Action::Rotate { pos: hit.block });
        self.play(sound::PLACE, block::sound::METAL, hit.block.as_vec3() + Vec3::new(0.5, 0.5, 0.5), 0.6);
        true
    }

    /// Shift+R: bends the targeted flat belt to take items in from its left, its right or its back, in turn, or else steps
    /// the bend the next belt laid by hand gets (`belt_bend.rs`). False when neither applies.
    pub fn bend_target(&mut self) -> bool {
        self.bend_target_belt()
    }

    /// The bend the next belt laid by hand gets: 0 straight, 1 taking items in from its left, 2 from its right.
    pub fn held_bend(&self) -> u8 {
        self.next_bend()
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

    /// The mining button, told each frame. Pressing it also selects the belt item under the crosshair (`item_select.rs`).
    pub fn set_mining(&mut self, on: bool) {
        if on && !self.mining && !self.scope_active() {
            self.click_item();
        }
        self.mining = on;
    }

    /// Hold to place blocks: places immediately, then repeats while held.
    pub fn set_using(&mut self, on: bool) {
        if on && !self.using {
            self.use_cooldown = 0.0;
            self.refuse_cooldown = 0.0;
        }
        self.using = on;
    }

    pub fn select_slot(&mut self, slot: u32) {
        self.act(Action::SelectSlot { slot: slot.min(u8::MAX as u32) as u8 });
    }

    /// The wheel, one notch: zooms the telescope while one is up (`telescope.rs`), else changes the slot.
    pub fn scroll_slot(&mut self, delta: i32) {
        if !self.scope_wheel(delta) {
            self.act(Action::ScrollSlot { delta: delta.signum() as i8 });
        }
    }

    /// Free flight on or off; only in a creative world (`Sim::may_fly`).
    pub fn toggle_fly(&mut self) {
        if !self.sim.may_fly() {
            return;
        }
        let body = self.body_mut();
        body.flying = !body.flying;
        body.vel.y = 0.0;
    }

    /// Throws one item from the selected slot.
    pub fn drop_selected(&mut self) {
        self.act(Action::DropSelected { count: 1 });
    }
}
