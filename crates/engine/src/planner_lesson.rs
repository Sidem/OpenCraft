//! L4, the planner lesson (a need lesson, beside `blueprint_lesson.rs`): the player has crafted a planner
//! (`site_hands.rs`) and has not marked a terraforming site yet. A card says how, by stage: hold it, right-click one
//! corner, right-click the opposite one. Card only, no floating prompt. Done: the first site stands.
//!
//! Presentation: the watch lives in `NeedWatch`, scans once a second until the world was taught
//! (`lessons::PLANNER_BIT`, never cleared) and is never saved. The need is the planner in the pack, so there is no
//! range; the card shows after `LOOK_TICKS`, only past the quiet spell and never over another card. A world that
//! already has a site is taught at the first scan without a card (its player knows), so an old save stays quiet.
//! A planner that leaves the pack drops the watch without teaching; H teaches it.

use crate::action::Action;
use crate::item::PLANNER;
use crate::lessons::PLANNER_BIT;
use crate::stall_lesson::QUIET_TICKS;
use crate::{Game, TICK_RATE};

/// How long the planner stays unused in the pack before its card shows.
const LOOK_TICKS: u64 = 2 * TICK_RATE as u64;

/// The card for each stage of the planner's hands.
const NOT_HELD: &str = "Planner: hold it in your hand, then [right-click] one corner of the ground you want to move.";
const FIRST: &str = "Planner: [right-click] a block at one corner of the ground you want to move.";
const FAR: &str = "Planner: now [right-click] the opposite corner, then pick dig, fill or flatten in the panel.";

/// The tick the planner was first seen in the pack with no site standing.
#[derive(Clone, Copy)]
pub(crate) struct Watch {
    since: u64,
}

impl Game {
    /// Whether the planner lesson is on show now: never over another card.
    pub(crate) fn planner_lesson(&self) -> bool {
        let free = self.stall_lesson().is_none()
            && self.low_power_lesson().is_none()
            && self.recipe_lesson().is_none()
            && self.upgrade_lesson().is_none()
            && self.blueprint_lesson().is_none();
        free && self.need.planner.is_some_and(|w| self.sim.tick >= w.since + LOOK_TICKS)
    }

    /// The card's text for the lesson on show (empty for none).
    pub(crate) fn planner_text(&self) -> String {
        if !self.planner_lesson() {
            return String::new();
        }
        match (self.planner_held(), self.planner_corner_set()) {
            (false, _) => NOT_HELD,
            (true, false) => FIRST,
            (true, true) => FAR,
        }
        .to_string()
    }

    /// H on the card: taught.
    pub(crate) fn planner_dismiss(&mut self) {
        if self.planner_lesson() {
            self.need.planner = None;
            self.teach_planner(true);
        }
    }

    /// Called once a second by `watch_needs`, only until the world was taught.
    pub(crate) fn scan_planner(&mut self) {
        let tick = self.sim.tick;
        if !self.sim.factory.sites.list.is_empty() {
            // The job is done. A card the player saw leaves its quiet spell; a world that never needed one just learns.
            let shown = self.need.planner.take().is_some_and(|w| tick >= w.since + LOOK_TICKS);
            self.teach_planner(shown);
        } else if self.inventory().count(PLANNER) == 0 {
            self.need.planner = None; // no planner: dropped without teaching
        } else if self.need.planner.is_none() && tick >= self.stall.quiet_until && self.stall_lesson().is_none() {
            self.need.planner = Some(Watch { since: tick });
        }
    }

    /// Marks the lesson taught; the quiet spell follows only a card the player saw.
    fn teach_planner(&mut self, shown: bool) {
        self.act(Action::LessonTaught { bit: PLANNER_BIT });
        if shown {
            self.stall.quiet_until = self.sim.tick + QUIET_TICKS;
        }
    }
}

#[cfg(test)]
mod tests;
