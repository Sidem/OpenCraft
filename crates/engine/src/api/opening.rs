//! A new normal world with its opening scene, and the tip, floating prompt and outline that walk the player
//! through it and to the starter patches (`opening.rs`, `opening/miners.rs`); past it, the copper patch for the Go
//! electric tip and the recipe the build menu points at (`first_steps.rs`).

use wasm_bindgen::prelude::*;

use crate::ghost_mode::push_box;
use crate::math::{IVec3, Vec3};
use crate::opening::{box_spot, Step, PITCH, PROMPT_COAL, PROMPT_COPPER, PROMPT_IRON, PROMPT_RIGHT_CLICK, STEPS};
use crate::worldgen::WorldGen;
use crate::Game;

const UP: IVec3 = IVec3::new(0, 1, 0);

/// The gap's outline colour (the cyan of ghosts).
const GAP_CYAN: i32 = 0x4cc9f0;
/// The floating prompt hangs this far above the middle of its cell, and shows within this many blocks.
const PROMPT_LIFT: f64 = 0.35;
const PROMPT_RANGE: f64 = 24.0;
/// A starter patch's prompt shows from farther: the patches lie a walk away (`worldgen/strata.rs`, `STARTERS_V4`).
const PATCH_RANGE: f64 = 160.0;

fn is_patch(kind: u8) -> bool {
    kind == PROMPT_COAL || kind == PROMPT_IRON || kind == PROMPT_COPPER
}

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

    /// The tip for an `opening_step`; a patch to walk to also says which way it lies from the player now.
    pub fn opening_text(&self, step: u32) -> String {
        let Some(&s) = STEPS.get(step as usize) else { return String::new() };
        let mut text = s.text().to_string();
        let patch = self.sim.opening.and_then(|o| o.marker(s, &self.sim.factory)).filter(|m| is_patch(m.1));
        if let Some((cell, _)) = patch {
            text += &self.bearing(cell);
        }
        text
    }

    /// Whether the opening scene's line runs again (its gap belt is back and faces the line's way), so the
    /// host can cheer the moment it starts.
    pub fn opening_runs(&self) -> bool {
        self.sim.opening.is_some_and(|o| self.sim.factory.placed_as(o.gap).is_some_and(|(dir, _)| dir == o.run()))
    }

    /// The floating prompt over the scene this frame: the camera-relative point it hangs at and what it shows
    /// (`opening::PROMPT_*`), or nothing.
    pub fn opening_marker(&self) -> Vec<f32> {
        self.opening_prompt().map_or_else(Vec::new, |(cell, kind)| self.float_prompt(cell, kind))
    }

    /// The floating prompt for the main tip, as `opening_marker`: over the copper patch while Go electric sends the
    /// player there.
    pub fn hint_marker(&self) -> Vec<f32> {
        self.copper_goal().map_or_else(Vec::new, |cell| self.float_prompt(cell, PROMPT_COPPER))
    }

    /// The item the build menu makes glow (its tab and tile) because the tips ask for it now, or -1.
    pub fn guide_craft(&self) -> i32 {
        self.guide_craft_item().map_or(-1, |id| id.0 as i32)
    }
}

impl Game {
    /// The gap's outline while the belt is still to be put back, the patch's top block while a coal miner is to
    /// be set on it, and the cell beside the coal miner its box goes in (`placement_box`).
    pub(crate) fn opening_boxes(&self) -> Vec<i32> {
        let mut out = Vec::new();
        let Some(o) = self.sim.opening else { return out };
        match (self.opening_now(), self.opening_prompt()) {
            (Step::Pick | Step::Collect | Step::Place, _) => push_box(&mut out, &[o.gap], GAP_CYAN),
            (Step::PlaceCoal | Step::PlaceIron, Some((cell, _))) => push_box(&mut out, &[cell], GAP_CYAN),
            (Step::BoxCoal, Some((ground, PROMPT_RIGHT_CLICK))) => push_box(&mut out, &[ground + UP], GAP_CYAN),
            _ => {}
        }
        out
    }

    pub(crate) fn opening_now(&self) -> Step {
        self.sim.opening.map_or(Step::Done, |o| o.step(self.inventory(), &self.sim.factory))
    }

    /// The opening's prompt: `Opening::marker`, but the box for the coal miner is set on the ground beside it.
    pub(crate) fn opening_prompt(&self) -> Option<(IVec3, u8)> {
        let step = self.opening_now();
        let (cell, kind) = self.sim.opening?.marker(step, &self.sim.factory)?;
        if step == Step::BoxCoal && kind == PROMPT_RIGHT_CLICK {
            return box_spot(&self.sim.world, cell).map(|c| (c - UP, kind));
        }
        Some((cell, kind))
    }

    /// A floating prompt over `cell`: the camera-relative point and the kind, or nothing when it is too far to show.
    fn float_prompt(&self, cell: IVec3, kind: u8) -> Vec<f32> {
        let at = cell.as_vec3() + Vec3::new(0.5, 0.5 + PROMPT_LIFT, 0.5) - self.render_eye;
        if at.length() > if is_patch(kind) { PATCH_RANGE } else { PROMPT_RANGE } {
            return Vec::new();
        }
        vec![at.x as f32, at.y as f32, at.z as f32, kind as f32]
    }
}
