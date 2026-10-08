//! Endless bonus techs (Milestone 11): repeatable techs that cost compute only and make one kind of work faster.
//!
//! - A bonus tech has no packs; each unit is a **level**. It can be researched only in an AI lab
//!   (`needs_ai_lab`), whose compute use is the cost: a unit takes `seconds` plus a quarter more per level already
//!   done (`Research::unit_seconds`).
//! - Each level adds 3% (`PERCENT_PER_LEVEL`) to its rate. `Research::rate_permille` is the one place the
//!   bonus is read; machines multiply it into their own rate (TECH_TREE rule 5).
//! - The level is the tech's progress, so saves need nothing new. `LEVELS` caps it: at 25% a level the cost makes
//!   it unreachable, and the cap keeps `progress <= units` and tests that finish every tech finite.
//!
//! To add one: a `Bonus` variant, a row in `BONUS` (needs AI Research, no packs, `Unlock::Bonus`) in the same order,
//! then read `rate_permille` where the work is done.

use super::{Research, Tech, Unlock};

/// Levels a bonus tech can reach (never reached in play: see above).
pub const LEVELS: u32 = 100;
/// What each level adds to the rate.
pub const PERCENT_PER_LEVEL: u32 = 3;
/// Extra cost per level done, in percent.
const COST_PERCENT: f64 = 25.0;
/// The tech index of the first bonus tech (`BONUS[0]`), right after AI Research (62).
const FIRST: u8 = 63;

/// A kind of work a bonus tech speeds up; the order is `BONUS`'s.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Bonus {
    /// How fast miners draw from deposits.
    Mining,
    /// How fast processing machines work through a batch.
    Machines,
    /// How fast drones fly.
    Drones,
}

const fn row(name: &'static str, blurb: &'static str, unlocks: &'static [Unlock]) -> Tech {
    Tech { name, blurb, needs: &[FIRST - 1], packs: &[], units: LEVELS, seconds: 60.0, unlocks }
}

pub const BONUS: [Tech; 3] = [
    row(
        "Mining Productivity",
        "Endless: miners draw 3% faster per level. Costs compute only, a quarter more each level; an AI lab researches it.",
        &[Unlock::Bonus(Bonus::Mining)],
    ),
    row(
        "Machine Speed",
        "Endless: smelters, assemblers and every other processing machine work 3% faster per level. Costs compute only.",
        &[Unlock::Bonus(Bonus::Machines)],
    ),
    row(
        "Drone Speed",
        "Endless: construction and cargo drones fly 3% faster per level. Costs compute only.",
        &[Unlock::Bonus(Bonus::Drones)],
    ),
];

/// Whether `tech` is a bonus tech (repeatable, shown as a level).
pub fn is_bonus(tech: u8) -> bool {
    (FIRST..FIRST + BONUS.len() as u8).contains(&tech)
}

impl Research {
    /// The level of a bonus tech (units done).
    pub fn level(&self, bonus: Bonus) -> u32 {
        self.progress(FIRST + bonus as u8)
    }

    /// Multiplier on a kind of work in thousandths: 1000 with no levels, 30 more per level.
    pub fn rate_permille(&self, bonus: Bonus) -> u32 {
        1000 + PERCENT_PER_LEVEL * 10 * self.level(bonus).min(LEVELS)
    }

    /// Lab seconds the next unit of `tech` takes at full speed: a bonus tech's grows with its level.
    pub fn unit_seconds(&self, tech: u8) -> f64 {
        let mut seconds = super::TECHS[tech as usize].seconds;
        if is_bonus(tech) {
            for _ in 0..self.progress(tech) {
                seconds += seconds * COST_PERCENT / 100.0;
            }
        }
        seconds
    }
}

#[cfg(test)]
mod tests;
