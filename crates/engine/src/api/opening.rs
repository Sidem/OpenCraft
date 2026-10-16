//! A new normal world with its opening scene, and the tip, floating prompt and outline that walk the player
//! through it (`opening.rs`).

use wasm_bindgen::prelude::*;

use crate::ghost_mode::push_box;
use crate::math::Vec3;
use crate::opening::{Step, PITCH, STEPS};
use crate::worldgen::WorldGen;
use crate::Game;

/// The gap's outline colour (the cyan of ghosts).
const GAP_CYAN: i32 = 0x4cc9f0;
/// The floating prompt hangs this far above the middle of its cell, and shows within this many blocks.
const PROMPT_LIFT: f64 = 0.35;
const PROMPT_RANGE: f64 = 24.0;

#[wasm_bindgen]
impl Game {
    /// A new normal world: the opening scene stands in front of the player, who looks at it.
    pub fn new_normal(seed: u32, view_radius: u32) -> Game {
        let mut game = Game::with_generator(WorldGen::new(seed), view_radius);
        let yaw = game.sim.build_opening();
        if let Some(Some(body)) = game.bodies.get_mut(game.local.0 as usize) {
            (body.yaw, body.pitch) = (yaw, PITCH);
        }
        game
    }

    /// What the opening scene asks of the local player now: 0 nothing (done, or no scene), else a step
    /// for `opening_text`. Cheap enough for every frame.
    pub fn opening_step(&self) -> u32 {
        let step = self.opening_now();
        STEPS.iter().position(|&s| s == step).map_or(0, |i| i as u32)
    }

    /// The tip for an `opening_step`.
    pub fn opening_text(&self, step: u32) -> String {
        STEPS.get(step as usize).map_or("", |s| s.text()).to_string()
    }

    /// Whether the opening scene's line runs again (its gap belt is back and faces the line's way), so the
    /// host can cheer the moment it starts.
    pub fn opening_runs(&self) -> bool {
        self.sim.opening.is_some_and(|o| self.sim.factory.placed_as(o.gap).is_some_and(|(dir, _)| dir == o.run()))
    }

    /// The floating prompt over the scene this frame: the camera-relative point it hangs at and what it shows
    /// (`opening::PROMPT_*`), or nothing.
    pub fn opening_marker(&self) -> Vec<f32> {
        let Some((cell, kind)) = self.sim.opening.and_then(|o| o.marker(self.opening_now())) else {
            return Vec::new();
        };
        let at = cell.as_vec3() + Vec3::new(0.5, 0.5 + PROMPT_LIFT, 0.5) - self.render_eye;
        if at.length() > PROMPT_RANGE {
            return Vec::new();
        }
        vec![at.x as f32, at.y as f32, at.z as f32, kind as f32]
    }
}

impl Game {
    /// The gap's outline while the belt is still to be put back (`placement_box`).
    pub(crate) fn opening_boxes(&self) -> Vec<i32> {
        let mut out = Vec::new();
        if let Some(o) =
            self.sim.opening.filter(|_| matches!(self.opening_now(), Step::Pick | Step::Collect | Step::Place))
        {
            push_box(&mut out, &[o.gap], GAP_CYAN);
        }
        out
    }

    fn opening_now(&self) -> Step {
        self.sim.opening.map_or(Step::Done, |o| o.step(self.inventory(), &self.sim.factory))
    }
}
