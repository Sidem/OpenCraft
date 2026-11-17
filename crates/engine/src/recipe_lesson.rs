//! L2, the "no recipe" lesson (a need lesson, beside `need_lesson.rs`): a machine whose recipe the player must choose
//! (`Pick::Chosen`: assembler, chemical plant, upgrader...) has stood near them with none chosen for `STALL_TICKS`. The
//! card says to right-click and pick, and a prompt hangs over the machine (`api/lessons.rs`, `web/src/ui/hints.ts`). It
//! is not a stall (a recipe-less machine reports nothing to `factory::efficiency`), so it has its own watcher.
//!
//! Presentation: the watch lives in `NeedWatch`, scans once a second until the world was taught
//! (`lessons::NO_RECIPE_BIT`, never cleared) and is never saved. Choosing a recipe on the watched machine teaches it, and
//! so does H; a machine that is removed or left behind is dropped without teaching. One card at a time: it starts only
//! beyond a stall lesson and the quiet spell, and shows only while no stall or power lesson does.

use crate::action::Action;
use crate::block::{self, BlockId};
use crate::lessons::{mask, NO_RECIPE_BIT};
use crate::math::IVec3;
use crate::stall_lesson::{QUIET_TICKS, RANGE, STALL_TICKS};
use crate::Game;

/// The card. `{m}` is the machine's name.
const NO_RECIPE: &str =
    "No recipe: the {m} doesn't know what to make. [Right-click] it and pick a recipe from the list.";

/// The machine watched: its cell (anchor), its block and the tick it was first seen without a recipe, in range.
#[derive(Clone, Copy)]
pub(crate) struct Watch {
    pos: IVec3,
    block: BlockId,
    since: u64,
}

impl Game {
    /// The no-recipe lesson on show now: the machine's cell. Never over a stall lesson or the low-power card.
    pub(crate) fn recipe_lesson(&self) -> Option<IVec3> {
        let w = self.need.recipe?;
        let shown = self.sim.tick >= w.since + STALL_TICKS;
        (shown && self.stall_lesson().is_none() && self.low_power_lesson().is_none()).then_some(w.pos)
    }

    /// The card's text for the lesson on show (empty for none).
    pub(crate) fn recipe_text(&self) -> String {
        let Some(w) = self.need.recipe.filter(|_| self.recipe_lesson().is_some()) else { return String::new() };
        let name = self.sim.factory.name_at(w.pos).unwrap_or_else(|| block::def(w.block).name.into());
        NO_RECIPE.replace("{m}", &name)
    }

    /// H on the card: taught.
    pub(crate) fn recipe_dismiss(&mut self) {
        if self.recipe_lesson().is_some() {
            self.need.recipe = None;
            self.teach_recipe(true);
        }
    }

    /// Called once a second by `watch_needs`, only until the world was taught.
    pub(crate) fn scan_recipes(&mut self) {
        let tick = self.sim.tick;
        let me = self.body().pos;
        let near = |p: IVec3| (p.x as f64 - me.x).hypot(p.z as f64 - me.z);
        let watched = self.need.recipe;
        let mut still_bare = false;
        let mut nearest: Option<(f64, Watch)> = None;
        self.sim.factory.unchosen(|block, pos| {
            still_bare |= watched.is_some_and(|w| w.pos == pos);
            let d = near(pos);
            if d > RANGE || nearest.is_some_and(|(best, _)| best <= d) {
                return;
            }
            nearest = Some((d, Watch { pos, block, since: tick }));
        });
        if let Some(w) = watched {
            if still_bare && near(w.pos) <= RANGE + 8.0 {
                return; // still bare, still here
            }
            // A machine that stands with a recipe now was set up by the player: learned. Gone or out of reach: dropped.
            self.need.recipe = None;
            if !still_bare && self.sim.factory.name_at(w.pos).is_some() {
                self.teach_recipe(tick >= w.since + STALL_TICKS);
            }
        }
        if self.need.recipe.is_none() && tick >= self.stall.quiet_until && self.stall_lesson().is_none() {
            self.need.recipe = nearest.map(|(_, w)| w);
        }
    }

    /// Marks the lesson taught; the quiet spell follows only a card the player saw.
    fn teach_recipe(&mut self, shown: bool) {
        self.act(Action::LessonTaught { bit: NO_RECIPE_BIT });
        if shown {
            self.stall.quiet_until = self.sim.tick + QUIET_TICKS;
        }
    }

    /// Whether the scan still has work: the world was not taught yet.
    pub(crate) fn recipe_untaught(&self) -> bool {
        self.sim.taught & mask(NO_RECIPE_BIT) == 0
    }
}
