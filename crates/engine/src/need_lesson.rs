//! Teaching at the moment of need, beside the machine-stopped lesson (`stall_lesson.rs`). Presentation: lives on
//! `Game`, never saved, reads the core only.
//!
//! - Stuck lines: a processing machine waiting for an item, or stopped with an output it cannot put down, says in its
//!   readout and in its stall card where that item comes from or what it is good for (`item_guide::source_line`,
//!   `use_line`): sand from a quarry on a beach or a crusher, slag into sand, quicklime from limestone a scanner finds.
//! - The low-power lesson: a machine near the player that has worked short of power (slower, not stopped) for
//!   `STALL_TICKS` brings one card naming the power screen (P). Getting it full power again, opening that screen, or H
//!   teaches it for good (`lessons::LOW_POWER_BIT`).
//!
//! Invariants: the stall lesson's rules hold (near, one card at a time, never over a stall lesson, the shared quiet
//! spell after each); a world that was taught it never scans. To add a need lesson: a bit in `lessons.rs`, a watcher
//! here, its number in `api/lessons.rs` (from `NEED_BASE`).

use crate::action::Action;
use crate::block::{self, BlockId};
use crate::factory::efficiency::{Cause, Sample};
use crate::item_guide;
use crate::lessons::{mask, LOW_POWER_BIT};
use crate::math::IVec3;
use crate::stall_lesson::{QUIET_TICKS, RANGE, STALL_TICKS};
use crate::{Game, TICK_RATE};

/// `lesson_now` numbers a need lesson from here on (its bit added), past the stall causes.
pub const NEED_BASE: u32 = 100;

/// The low-power card. `{m}` is the machine's name, `{p}` the share of power it gets.
const LOW_POWER: &str = "Short of power: the {m} gets {p}% of the power it asks for, so it works slower. Press [P] to \
                         see what each grid makes and uses; another generator or more fuel mends it.";

#[derive(Clone, Copy)]
struct Watch {
    pos: IVec3,
    block: BlockId,
    /// The share of power it got when last seen, in percent.
    percent: u32,
    since: u64,
    /// Scans in a row without a reading.
    blank: u32,
}

/// A watched machine with no reading this many scans (seconds) in a row has gone idle: its card goes. Long enough to
/// outlast the reading's restart on a new recipe or tech (`factory::efficiency`'s 10 s window).
const BLANK_SCANS: u32 = 10;

#[derive(Default)]
pub struct NeedWatch {
    watch: Option<Watch>,
}

impl Game {
    /// Where the items a machine at `pos` waits for come from, and what its stuck output is good for: one line each.
    pub(crate) fn stuck_lines(&self, pos: IVec3) -> Vec<String> {
        let (lacking, blocked) = self.sim.factory.stuck_on(pos);
        let (gen, research) = (self.sim.world.generator(), &self.sim.factory.research);
        let named = |i, line: Option<String>| line.map(|l| format!("{}: {l}", crate::item::name(i)));
        let mut out: Vec<String> =
            lacking.into_iter().filter_map(|i| named(i, item_guide::source_line(gen, research, i))).collect();
        out.extend(blocked.and_then(|i| named(i, item_guide::use_line(research, i))));
        out
    }

    /// Called after every core tick, beside `watch_stalls`: scans once a second until the world was taught.
    pub(crate) fn watch_needs(&mut self) {
        if self.sim.tick.is_multiple_of(TICK_RATE as u64) && self.sim.taught & mask(LOW_POWER_BIT) == 0 {
            self.scan_power();
        }
    }

    /// The low-power lesson on show now: the machine's cell.
    pub(crate) fn low_power_lesson(&self) -> Option<IVec3> {
        let w = self.need.watch?;
        (self.sim.tick >= w.since + STALL_TICKS && self.stall_lesson().is_none()).then_some(w.pos)
    }

    /// The card's text for the lesson on show (empty for none).
    pub(crate) fn low_power_text(&self) -> String {
        let Some(w) = self.need.watch.filter(|_| self.low_power_lesson().is_some()) else { return String::new() };
        let name = self.sim.factory.name_at(w.pos).unwrap_or_else(|| block::def(w.block).name.into());
        LOW_POWER.replace("{m}", &name).replace("{p}", &w.percent.to_string())
    }

    /// H on the card, or the power screen opened while it shows: taught.
    pub(crate) fn need_dismiss(&mut self) {
        if self.low_power_lesson().is_some() {
            self.need.watch = None;
            self.teach_need(LOW_POWER_BIT);
        }
    }

    fn teach_need(&mut self, bit: u8) {
        self.act(Action::LessonTaught { bit });
        self.stall.quiet_until = self.sim.tick + QUIET_TICKS;
    }

    fn scan_power(&mut self) {
        let tick = self.sim.tick;
        let me = self.body().pos;
        let near = |p: IVec3| (p.x as f64 - me.x).hypot(p.z as f64 - me.z);
        let watched = self.need.watch;
        let mut seen: Option<Option<Sample>> = None;
        let mut nearest: Option<(f64, Watch)> = None;
        self.sim.factory.samples(|block, pos, sample| {
            if watched.is_some_and(|w| w.pos == pos) {
                seen = Some(sample);
            }
            let Some(Sample { work, short: Some(Cause::Power) }) = sample else { return };
            let d = near(pos);
            if work == 0 || d > RANGE || nearest.is_some_and(|(best, _)| best <= d) {
                return;
            }
            let percent = (work / 10).max(1);
            nearest = Some((d, Watch { pos, block, percent, since: tick, blank: 0 }));
        });
        if let Some(w) = watched {
            let short = |s: Option<Sample>| s.is_none_or(|s| s.work > 0 && s.short == Some(Cause::Power));
            let blank = if seen.is_some_and(|s| s.is_none()) { w.blank + 1 } else { 0 };
            if let Some(s) = seen.filter(|&s| short(s)).filter(|_| near(w.pos) <= RANGE + 8.0 && blank <= BLANK_SCANS) {
                // Still short, still here. A short spell without a reading (a new recipe or tech restarts it) changes
                // nothing; a long one is a machine gone idle, dropped below.
                let percent = s.map_or(w.percent, |s| (s.work / 10).max(1));
                self.need.watch = Some(Watch { percent, blank, ..w });
                return;
            }
            // Mended after the card showed: the lesson learned. Gone or out of reach: dropped without teaching.
            self.need.watch = None;
            if tick >= w.since + STALL_TICKS && seen.is_some_and(|s| s.is_some_and(|s| s.short.is_none())) {
                self.teach_need(LOW_POWER_BIT);
            }
        }
        if self.need.watch.is_none() && tick >= self.stall.quiet_until && self.stall_lesson().is_none() {
            self.need.watch = nearest.map(|(_, w)| w);
        }
    }
}

#[cfg(test)]
mod tests;
