//! One frame of the host (`Game::update`): turns frame time into whole ticks (`run_tick` in `lib.rs`), then does the
//! per-frame presentation work: the interpolated camera, targeting, box instances, previews and the map.
//!
//! Invariant: game state advances only in ticks, so the same inputs give the same results at any frame rate; nothing
//! here changes the core. To draw something new each frame: its `write_*` call at the end of `update`.

use wasm_bindgen::prelude::*;

use crate::math::Vec3;
use crate::{block, factory, Game, TICK};

/// Most ticks one frame may run. A 0.1 s frame (the host's cap) plus a leftover partial tick needs
/// 7; anything longer (e.g. a hidden tab) drops the excess instead of spiralling.
pub(crate) const MAX_TICKS_PER_FRAME: u32 = 8;
/// Float slack when comparing accumulated frame time with `TICK`, so 60 frames of 1/60 s run 60 ticks.
const TICK_SLACK: f64 = 1e-9;

#[wasm_bindgen]
impl Game {
    /// Per frame: runs the whole ticks that `dt` seconds of frame time add up to, then prepares this
    /// frame's camera and box instances.
    pub fn update(&mut self, dt: f64) {
        self.stream_around_players();
        self.accumulator += dt.max(0.0);
        let mut ran = 0;
        while self.accumulator >= TICK - TICK_SLACK && ran < MAX_TICKS_PER_FRAME {
            self.run_tick();
            self.accumulator -= TICK;
            ran += 1;
        }
        if self.accumulator >= TICK {
            self.accumulator = 0.0;
        }
        self.catch_up();

        let alpha = (self.accumulator / TICK).clamp(0.0, 1.0);
        let eye = self.prev_eye + (self.body().eye() - self.prev_eye) * alpha;
        let feet = eye - Vec3::new(0.0, self.body().eye_height(), 0.0);
        let eye = self.crouch_glide.apply(eye, self.body().crouched(), dt);
        let (look, solid) = (self.body().look_dir(), &self.sim.world);
        let solid = |p| solid.get_block(p).is_some_and(|b| block::SOLID[b as usize]);
        let before = self.render_eye;
        if self.strategy.active() {
            self.advance_strategy(feet, dt);
            self.render_eye = self.strategy.eye();
        } else {
            self.render_eye = self.third_person.camera(eye, feet, look, dt, solid);
        }
        self.settle_view(before, self.render_eye, dt);
        if self.strategy.active() {
            self.strategy_target();
        } else if self.third_person.active() {
            self.update_target_at(self.render_eye);
        } else {
            self.update_target();
        }
        self.update_scope(dt, self.render_eye);
        let (eye, time) = (self.render_eye, (self.sim.tick as f64 + alpha) * TICK);
        self.instances.clear();
        self.write_item_instances(dt, eye);
        self.write_avatars(dt, eye);
        self.write_move_marker(eye);
        self.write_drone_instances(eye, alpha);
        self.sim.factory.write_instances(&mut self.instances, eye, time, self.sim.world.view_distance());
        let world = &mut self.sim.world;
        factory::light_boxes(&mut self.instances, eye, |cell| world.light_at(cell));
        self.write_line_preview(eye, time);
        self.write_power_preview(eye);
        self.write_rail_preview(eye);
        self.hide_unexplored_instances(eye);
        self.write_scope_beacons(eye);
        self.write_strategy_fog(eye);
        self.minimap.atlas.refresh_some(&self.sim.world);
    }
}
