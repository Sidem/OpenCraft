//! L3, the blueprint walk-through (a need lesson, beside `upgrade_lesson.rs`): the player accepted the blueprint prompt
//! (`blueprint/repeat.rs`: B pressed while "Building the same again" showed) and is now in ghost mode. A card and floating
//! key prompts take them through the job: Z on one corner of the group, Z on the opposite corner, Enter. Done: a blueprint
//! saved (the library is no longer empty).
//!
//! Presentation: the watch lives in `NeedWatch`, scans once a second until the world was taught
//! (`lessons::BLUEPRINT_BIT`, never cleared) and is never saved. The stage follows the library's hands, so it can never
//! disagree with the keys: a marked box asks for Enter, one corner asks for the far one, nothing marked asks for both
//! corners of the box the prompt suggested. The card and prompts show only in ghost mode and never over another card.
//! H teaches it too; walking away from the group drops it without teaching. Old saves are left untaught on purpose: the
//! repeat prompt only comes for a world whose player has no blueprint yet.

use crate::action::Action;
use crate::blueprint::GROUP_RANGE;
use crate::lessons::BLUEPRINT_BIT;
use crate::math::{IVec3, Vec3};
use crate::stall_lesson::{QUIET_TICKS, RANGE};
use crate::Game;

/// Key prompt kinds this lesson adds (`web/src/ui/key-prompt.ts` draws them): the Z key, and the Enter key.
pub const PROMPT_CORNER: u8 = 9;
pub const PROMPT_COPY: u8 = 10;

/// The prompts hang this far above the middle of their cell.
const PROMPT_LIFT: f64 = 0.35;
/// Walking this far from the group drops the lesson.
const AWAY: f64 = (GROUP_RANGE + 8) as f64;

/// The card for each stage.
const FIRST: &str =
    "Blueprint: aim at one corner of your machines and press [Z], then at the opposite corner and press [Z].";
const FAR: &str = "Blueprint: now aim at the opposite corner and press [Z].";
const COPY: &str = "Blueprint: press [Enter] to copy the machines in the gold box.";

/// The box the repeat prompt suggested: its lowest and highest cell.
#[derive(Clone, Copy)]
pub(crate) struct Watch {
    lo: IVec3,
    hi: IVec3,
}

/// What the library's hands are doing, hence what to ask next.
enum Stage {
    First,
    Far(IVec3),
    Copy,
}

impl Game {
    /// The walk-through on show now: ghost mode, nothing else's card up.
    pub(crate) fn blueprint_lesson(&self) -> Option<Watch> {
        let free = self.stall_lesson().is_none()
            && self.low_power_lesson().is_none()
            && self.recipe_lesson().is_none()
            && self.upgrade_lesson().is_none();
        self.need.blueprint.filter(|_| free && self.ghost_mode && self.library.list.is_empty())
    }

    fn blueprint_stage(&self) -> Stage {
        let lib = &self.library;
        match (lib.selection, lib.corner) {
            (Some(_), _) => Stage::Copy,
            (None, Some(c)) => Stage::Far(c),
            (None, None) => Stage::First,
        }
    }

    /// The card's text for the lesson on show (empty for none).
    pub(crate) fn blueprint_text(&self) -> String {
        if self.blueprint_lesson().is_none() {
            return String::new();
        }
        match self.blueprint_stage() {
            Stage::First => FIRST,
            Stage::Far(_) => FAR,
            Stage::Copy => COPY,
        }
        .to_string()
    }

    /// The floating key prompts for the stage, as groups of four numbers (camera-relative x, y, z and the prompt kind);
    /// empty when the lesson is not on show or a prompt would be out of reach.
    pub(crate) fn blueprint_markers(&self) -> Vec<f32> {
        let Some(w) = self.blueprint_lesson() else { return Vec::new() };
        let low = |x: i32, z: i32| IVec3::new(x, w.lo.y, z);
        let (a, b) = (low(w.lo.x, w.lo.z), low(w.hi.x, w.hi.z));
        let spots: Vec<(IVec3, u8)> = match self.blueprint_stage() {
            Stage::First => vec![(a, PROMPT_CORNER), (b, PROMPT_CORNER)],
            Stage::Far(c) => {
                let far = if dist2(a, c) >= dist2(b, c) { a } else { b };
                vec![(far, PROMPT_CORNER)]
            }
            Stage::Copy => {
                let top = IVec3::new((w.lo.x + w.hi.x) / 2, w.hi.y, (w.lo.z + w.hi.z) / 2);
                vec![(top, PROMPT_COPY)]
            }
        };
        let mut out = Vec::new();
        for (cell, kind) in spots {
            let at = cell.as_vec3() + Vec3::new(0.5, 0.5 + PROMPT_LIFT, 0.5) - self.render_eye;
            if at.length() <= RANGE + 8.0 {
                out.extend([at.x as f32, at.y as f32, at.z as f32, kind as f32]);
            }
        }
        out
    }

    /// H on the card: taught.
    pub(crate) fn blueprint_dismiss(&mut self) {
        if self.blueprint_lesson().is_some() {
            self.need.blueprint = None;
            self.teach_blueprint();
        }
    }

    /// Called once a second by `watch_needs`, only until the world was taught.
    pub(crate) fn scan_blueprints(&mut self) {
        if let Some(w) = self.need.blueprint {
            let me = self.body().pos;
            let mid = |a: i32, b: i32| (a + b) as f64 / 2.0;
            let away = (mid(w.lo.x, w.hi.x) - me.x).hypot(mid(w.lo.z, w.hi.z) - me.z) > AWAY;
            if !self.library.list.is_empty() {
                self.need.blueprint = None;
                self.teach_blueprint(); // saved: the lesson learned
            } else if away {
                self.need.blueprint = None; // left it: dropped without teaching
            }
            return;
        }
        if self.ghost_mode && self.library.list.is_empty() {
            if let Some((lo, hi)) = self.accept_repeat_prompt() {
                self.need.blueprint = Some(Watch { lo, hi });
            }
        }
    }

    fn teach_blueprint(&mut self) {
        self.act(Action::LessonTaught { bit: BLUEPRINT_BIT });
        self.stall.quiet_until = self.sim.tick + QUIET_TICKS;
    }
}

fn dist2(a: IVec3, b: IVec3) -> i64 {
    let (dx, dz) = ((a.x - b.x) as i64, (a.z - b.z) as i64);
    dx * dx + dz * dz
}

#[cfg(test)]
mod tests;
