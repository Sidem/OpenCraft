//! The "machine stopped" lesson: the first time a machine near the local player has stood stalled for a good while
//! for a cause the world has not been taught yet, the host shows a card naming the cause and the fix, and a prompt over
//! the machine (`api/lessons.rs`, `web/src/ui/hints.ts`). Machines already say all this when pointed at; the lesson
//! only makes the first sighting of each cause hard to miss. Fixing it teaches the cause for good (`Action::LessonTaught`,
//! `lessons.rs`), and so does H on the card.
//!
//! Presentation: the watcher lives on `Game`, reads the factory's efficiency samples once a second and is never saved.
//! A machine full because its line is backed up is normal in a big base, so a cause is taught once and never again; the
//! flapping of a busy machine never lasts `STALL_TICKS`. Machines at the opening scene are left alone, and a quiet
//! spell follows each lesson. To change the wording: `LESSONS`, one row per [`Cause`].

use crate::action::Action;
use crate::block::{self, BlockId};
use crate::factory::efficiency::{Cause, Sample};
use crate::lessons::{mask, stall_bit, STALL_MASK};
use crate::math::IVec3;
use crate::{Game, TICK_RATE};

/// How long a machine stands stalled, near the player, before its lesson shows.
pub const STALL_TICKS: u64 = 30 * TICK_RATE as u64;
/// No new lesson starts this long after one ends.
pub const QUIET_TICKS: u64 = 60 * TICK_RATE as u64;
/// The machine must be this near (blocks), and the prompt hangs over it from this far.
pub const RANGE: f64 = 24.0;
/// The floating prompt's kind for "look at this machine" (`web/src/ui/key-prompt.ts`).
pub const PROMPT_CHECK: u8 = 5;
/// Machines this near the opening scene's gap belong to the scene (its furnace waits for ore by design).
const SCENE_RADIUS: i32 = 10;

/// What the card says, by [`Cause`] number. `{m}` is the machine's name. The text before the first colon is the card's title.
const LESSONS: [&str; crate::factory::efficiency::CAUSES] = [
    "No power: the {m} is not wired. Select a *power pole* and click the machine to wire it; a generator on the same \
     pole needs fuel.",
    "Waiting: the {m} has nothing to work on. Feed it by *belt*, or [Right-click] it and put the ingredients in.",
    "Full: the {m} has nowhere to put what it made. Lead a *belt* away from it or set a *storage box* beside it, or take the \
     products out by hand. A full machine just waits and loses nothing.",
    "Out of fuel: [Right-click] the {m} and give it *coal ore* or *logs*, or belt the fuel in.",
    "No water: pipe the {m}'s blue inlet to a *pump* that stands in water.",
    "No ore: the {m} has nothing left to dig here. Move it onto a fresh deposit; [M] marks the ore you have seen.",
    "Can't work: the {m} is overheated or flooded. Give it time to cool, or pump the water out.",
];

#[derive(Clone, Copy)]
struct Watch {
    pos: IVec3,
    block: BlockId,
    cause: Cause,
    /// The tick the machine was first seen stalled, in range.
    since: u64,
}

#[derive(Default)]
pub struct StallWatch {
    watch: Option<Watch>,
    quiet_until: u64,
}

impl Game {
    /// Called after every core tick (`step_core`): once a second, scans the factory, and only until the world has
    /// been taught every cause.
    pub(crate) fn watch_stalls(&mut self) {
        if self.sim.tick.is_multiple_of(TICK_RATE as u64) {
            self.scan_stalls();
        }
    }

    /// The stall lesson on show now, if any: the stalled machine's cell and the cause.
    pub(crate) fn stall_lesson(&self) -> Option<(IVec3, Cause)> {
        let shown = |w: &Watch| self.sim.tick >= w.since + STALL_TICKS;
        self.stall.watch.filter(shown).map(|w| (w.pos, w.cause))
    }

    /// The floating prompt for the lesson on show: "right-click to check" only where a right-click is the way to fix it
    /// (feeding, emptying or fuelling a machine that opens), never for a cause fixed with a pole, a pipe or a move, nor
    /// for a machine with nothing to open.
    pub(crate) fn stall_prompt(&self) -> Option<u8> {
        let (pos, cause) = self.stall_lesson()?;
        let block = self.stall.watch.filter(|w| w.pos == pos)?.block;
        let by_hand = matches!(cause, Cause::Input | Cause::Output | Cause::Fuel);
        let opens = crate::factory::machine(block).is_some_and(|m| m.panel || m.slots > 0);
        (by_hand && opens).then_some(PROMPT_CHECK)
    }

    /// The card's text for a lesson from `stall_lesson`.
    pub(crate) fn stall_text(&self, pos: IVec3, cause: Cause) -> String {
        let block = self.stall.watch.filter(|w| w.pos == pos).map_or(block::AIR, |w| w.block);
        let name = self.sim.factory.name_at(pos).unwrap_or_else(|| block::def(block).name.into());
        LESSONS[cause as usize].replace("{m}", &name)
    }

    /// The player waves the lesson away (H): taught, and quiet for a while.
    pub(crate) fn stall_dismiss(&mut self) {
        if let Some(w) = self.stall.watch.take() {
            self.teach(w.cause);
        }
    }

    fn teach(&mut self, cause: Cause) {
        self.act(Action::LessonTaught { bit: stall_bit(cause) });
        self.stall.quiet_until = self.sim.tick + QUIET_TICKS;
    }

    fn scan_stalls(&mut self) {
        let taught = self.sim.taught;
        if taught & STALL_MASK == STALL_MASK {
            self.stall.watch = None;
            return;
        }
        let tick = self.sim.tick;
        let me = self.body().pos;
        let near = |p: IVec3, from: crate::math::Vec3| (p.x as f64 - from.x).hypot(p.z as f64 - from.z);
        let scene = self.sim.opening.map(|o| o.gap);
        let watched = self.stall.watch;
        let mut seen: Option<Option<Sample>> = None;
        let mut nearest: Option<(f64, Watch)> = None;
        self.sim.factory.samples(|block, pos, sample| {
            if watched.is_some_and(|w| w.pos == pos) {
                seen = Some(sample);
            }
            let Some(Sample { work: 0, short: Some(cause) }) = sample else { return };
            let in_scene = scene.is_some_and(|g| (pos.x - g.x).abs().max((pos.z - g.z).abs()) <= SCENE_RADIUS);
            let d = near(pos, me);
            if taught & mask(stall_bit(cause)) != 0
                || in_scene
                || d > RANGE
                || nearest.is_some_and(|(best, _)| best <= d)
            {
                return;
            }
            nearest = Some((d, Watch { pos, block, cause, since: tick }));
        });
        if let Some(w) = watched {
            let stalled = |s: Option<Sample>| s.is_some_and(|s| s.work == 0 && s.short == Some(w.cause));
            if seen.is_some_and(stalled) && near(w.pos, me) <= RANGE + 8.0 {
                return; // still the same trouble, still here
            }
            // Fixed: a machine that now works, after the lesson showed, is the lesson learned. Gone, out of reach or
            // stalled for another cause: dropped without teaching.
            self.stall.watch = None;
            if tick >= w.since + STALL_TICKS && seen.is_some_and(|s| s.is_some_and(|s| s.work > 0)) {
                self.teach(w.cause);
            }
        }
        if self.stall.watch.is_none() && tick >= self.stall.quiet_until {
            self.stall.watch = nearest.map(|(_, w)| w);
        }
    }
}

#[cfg(test)]
mod tests;
