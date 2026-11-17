//! L6, the map and pins lesson (a need lesson, beside `planner_lesson.rs`): the player has walked `FAR` blocks out from
//! spawn (the world's column 0, 0, where everyone starts) and has never opened the world map. A card says the map keeps
//! what they explored and that a click on it pins a spot (their base), so the way back is never lost. Card only.
//!
//! Presentation: the watch lives in `NeedWatch`, scans once a second until the world was taught (`lessons::MAP_BIT`,
//! never cleared) and is never saved. The map and its pins live in the browser (`web/src/ui/worldmap.ts`, `pins.ts`),
//! so the core cannot see a pin: opening the map is the job, reported by the host through `Game::map_opened`. That
//! teaches it even before the player ever strays (they know the map), and H teaches it too. Walking back inside
//! `FAR - BACK` drops the watch without teaching. The card shows after `LOOK_TICKS`, past the quiet spell, never over
//! another card. Older saves count it taught (`lessons::KNOWN_BEFORE_76`): their players found the map long ago.

use crate::action::Action;
use crate::lessons::{mask, MAP_BIT};
use crate::stall_lesson::QUIET_TICKS;
use crate::{Game, TICK_RATE};

/// How far from spawn (blocks, on the ground plane) the need begins.
const FAR: f64 = 150.0;
/// Walking this much closer than `FAR` ends the need, so standing on the line cannot flicker the card.
const BACK: f64 = 20.0;
/// How long the player stays out there before the card shows: a sprint across the line brings nothing.
const LOOK_TICKS: u64 = 3 * TICK_RATE as u64;

/// The card.
const MAP: &str = "Far from your start: press [M] for the map of where you've been, then click it to pin your base.";

/// The tick the player was first seen beyond `FAR`.
#[derive(Clone, Copy)]
pub(crate) struct Watch {
    since: u64,
}

impl Game {
    /// Whether the map lesson is on show now: never over another card.
    pub(crate) fn map_lesson(&self) -> bool {
        let free = self.stall_lesson().is_none()
            && self.low_power_lesson().is_none()
            && self.recipe_lesson().is_none()
            && self.upgrade_lesson().is_none()
            && self.blueprint_lesson().is_none()
            && !self.planner_lesson();
        free && self.need.map.is_some_and(|w| self.sim.tick >= w.since + LOOK_TICKS)
    }

    /// The card's text for the lesson on show (empty for none).
    pub(crate) fn map_text(&self) -> String {
        if self.map_lesson() {
            MAP.to_string()
        } else {
            String::new()
        }
    }

    /// H on the card: taught.
    pub(crate) fn map_dismiss(&mut self) {
        if self.map_lesson() {
            self.need.map = None;
            self.teach_map(true);
        }
    }

    /// The host opened the world map: its job is done, card or not.
    pub(crate) fn map_known(&mut self) {
        if self.sim.taught & mask(MAP_BIT) == 0 {
            let shown = self.map_lesson();
            self.need.map = None;
            self.teach_map(shown);
        }
    }

    /// Called once a second by `watch_needs`, only until the world was taught.
    pub(crate) fn scan_map(&mut self) {
        let tick = self.sim.tick;
        let p = self.body().pos;
        let out = p.x.hypot(p.z);
        match self.need.map {
            Some(_) if out < FAR - BACK => self.need.map = None, // back home: dropped without teaching
            None if out >= FAR && tick >= self.stall.quiet_until && self.stall_lesson().is_none() => {
                self.need.map = Some(Watch { since: tick });
            }
            _ => {}
        }
    }

    /// Marks the lesson taught; the quiet spell follows only a card the player saw.
    fn teach_map(&mut self, shown: bool) {
        self.act(Action::LessonTaught { bit: MAP_BIT });
        if shown {
            self.stall.quiet_until = self.sim.tick + QUIET_TICKS;
        }
    }
}

#[cfg(test)]
mod tests;
