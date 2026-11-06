//! The lesson cards for the host (`stall_lesson.rs`, `need_lesson.rs`, `lessons.rs`): which one is on show, its words,
//! the floating prompt over the machine, and the way to wave it away. The host calls `lesson_now` each frame; the
//! watchers only scan once a second.

use wasm_bindgen::prelude::*;

use crate::lessons::LOW_POWER_BIT;
use crate::math::Vec3;
use crate::need_lesson::NEED_BASE;
use crate::stall_lesson::RANGE;
use crate::Game;

/// The prompt hangs this far above the middle of the machine's cell.
const PROMPT_LIFT: f64 = 0.35;

#[wasm_bindgen]
impl Game {
    /// The lesson on show now: 0 for none, a machine-stopped lesson as its stall cause's number plus one, a need lesson
    /// as `NEED_BASE` plus its bit (the host titles the two kinds differently).
    pub fn lesson_now(&self) -> u32 {
        match self.stall_lesson() {
            Some((_, cause)) => cause as u32 + 1,
            None => self.low_power_lesson().map_or(0, |_| NEED_BASE + LOW_POWER_BIT as u32),
        }
    }

    /// The card's heading: a stopped machine's, or a need lesson's.
    pub fn lesson_head(&self) -> String {
        if self.stall_lesson().is_some() { "Machine stopped" } else { "Good to know" }.to_string()
    }

    /// The card's text for the lesson on show (empty for none), in the hint markup.
    pub fn lesson_text(&self) -> String {
        match self.stall_lesson() {
            Some((pos, cause)) => self.stall_text(pos, cause),
            None => self.low_power_text(),
        }
    }

    /// The floating prompt over the stalled machine, as `opening_marker` gives one: camera-relative x, y, z and the
    /// prompt kind; empty for none (also when no right-click would fix the cause: `stall_prompt`; a need lesson is a
    /// card only).
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
        if self.stall_lesson().is_some() {
            self.stall_dismiss();
        } else {
            self.need_dismiss();
        }
    }

    /// The power screen (P) was opened: while the low-power lesson shows, its job is done.
    pub fn power_screen_opened(&mut self) {
        self.need_dismiss();
    }
}
