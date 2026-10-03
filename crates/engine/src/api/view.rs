//! Local view and strategy input. Navigation and camera rules live in strategy; no core state changes here.
use crate::math::Vec3;
use crate::Game;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
impl Game {
    pub fn view_mode(&self) -> u32 {
        self.strategy.mode
    }
    pub fn set_view_mode(&mut self, mode: u32) {
        self.switch_view(mode);
    }
    pub fn set_shoulder_distance(&mut self, distance: f64) {
        self.strategy.shoulder_distance = distance.clamp(0.0, crate::camera::MAX_DISTANCE);
        if self.strategy.mode == 1 {
            self.third_person.distance = self.strategy.shoulder_distance;
        }
    }
    pub fn cycle_view(&mut self) {
        self.switch_view((self.strategy.mode.min(2) + 1) % 3);
    }
    pub fn toggle_free_camera(&mut self) {
        self.switch_view(if self.strategy.mode == 3 { 2 } else { 3 });
    }
    pub fn recenter_camera(&mut self) {
        if self.strategy.active() {
            self.switch_view(2);
        }
    }
    pub fn camera_yaw(&self) -> f64 {
        self.strategy.render_yaw
    }
    pub fn camera_pitch(&self) -> f64 {
        self.strategy.render_pitch
    }
    pub fn strategy_cursor(&mut self, x: f64, y: f64, fov: f64, aspect: f64) {
        self.strategy.cursor = [x.clamp(-1.0, 1.0), y.clamp(-1.0, 1.0)];
        self.strategy.projection = [fov.clamp(0.5, 2.0), aspect.clamp(0.1, 10.0)];
    }
    pub fn pan_camera(&mut self, forward: f64, right: f64, fast: bool) {
        if !self.strategy.active() {
            return;
        }
        if forward != 0.0 || right != 0.0 {
            self.strategy.mode = 3;
        }
        let yaw = self.strategy.yaw;
        let v = Vec3::new(yaw.sin(), 0.0, -yaw.cos()) * forward + Vec3::new(yaw.cos(), 0.0, yaw.sin()) * right;
        self.strategy.pan = v * (if fast { 2.0 } else { 1.0 } / v.length().max(1.0));
    }
    /// Middle-drag: turn the overhead camera around the point it looks at (radians, as for `look`).
    pub fn orbit_camera(&mut self, dx: f64, dy: f64) {
        if self.strategy.active() {
            self.strategy.orbit(-dx, -dy);
        }
    }
    pub fn zoom_strategy(&mut self, delta: f64) {
        // Each wheel step changes the distance by a tenth, so close views stay fine to adjust.
        self.strategy.height = (self.strategy.height * (1.0 + delta * 0.1)).clamp(3.0, 80.0);
    }
    pub fn order_move(&mut self) {
        self.order_strategy_move();
    }
    pub fn cancel_move_order(&mut self) {
        self.strategy.navigation.cancel();
    }
    pub fn move_order_status(&self) -> u32 {
        self.strategy.navigation.status
    }
    pub fn strategy_hover(&self) -> Vec<i32> {
        self.strategy.hover.map_or_else(Vec::new, |h| vec![h.block.x, h.block.y, h.block.z])
    }
    pub fn camera_area_known(&self) -> bool {
        let p = self.strategy.focus.floor();
        self.minimap.atlas.tile(p.x >> 5, p.z >> 5).is_some()
    }
}
