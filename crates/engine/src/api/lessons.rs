//! The lesson cards for the host (`stall_lesson.rs`, `lessons.rs`): which one is on show, its words, the floating prompt
//! over the machine, and the way to wave it away. The host calls `lesson_now` each frame; it only scans once a second.

use wasm_bindgen::prelude::*;

use crate::math::Vec3;
use crate::stall_lesson::RANGE;
use crate::Game;

/// The prompt hangs this far above the middle of the machine's cell.
const PROMPT_LIFT: f64 = 0.35;

#[wasm_bindgen]
impl Game {
    /// The machine-stopped lesson on show now: 0 for none, else the stall cause's number plus one.
    pub fn lesson_now(&self) -> u32 {
        self.stall_lesson().map_or(0, |(_, cause)| cause as u32 + 1)
    }

    /// The card's text for the lesson on show (empty for none), in the hint markup.
    pub fn lesson_text(&self) -> String {
        self.stall_lesson().map_or(String::new(), |(pos, cause)| self.stall_text(pos, cause))
    }

    /// The floating prompt over the stalled machine, as `opening_marker` gives one: camera-relative x, y, z and the
    /// prompt kind; empty for none (also when no right-click would fix the cause: `stall_prompt`).
    pub fn lesson_marker(&self) -> Vec<f32> {
        let (Some((pos, _)), Some(kind)) = (self.stall_lesson(), self.stall_prompt()) else { return Vec::new() };
        let at = pos.as_vec3() + Vec3::new(0.5, 0.5 + PROMPT_LIFT, 0.5) - self.render_eye;
        if at.length() > RANGE + 8.0 {
            return Vec::new();
        }
        vec![at.x as f32, at.y as f32, at.z as f32, kind as f32]
    }

    /// The player waves the lesson away (H): it counts as taught for the world.
    pub fn lesson_dismiss(&mut self) {
        self.stall_dismiss();
    }
}
