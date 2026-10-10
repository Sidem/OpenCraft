//! Player modifiers: the one place that sums what changes a player's own rates. Hand-crafting speed is the first
//! stat; mining speed, reach and carry capacity can read it later. Sources today: finished techs that list
//! `Unlock::Perk(stat, tier)` (`research/hands.rs`), each adding `PERCENT_PER_TIER`, linear like the bonus techs.
//! Worn gear joins next (DEV_PLAN, less hand-crafting step 3).
//!
//! Invariants: derived from core state only (research now, gear later), never saved, so every machine in co-op
//! gets the same value and the state hash needs nothing new.
//!
//! To add a stat: a `Stat` variant, techs listing `Unlock::Perk(Stat::New, tier)` (tiers 1, 2, ...), then read
//! `player_bonus` where the work is done.

use crate::research::{Research, TechState, Unlock, TECHS};

/// What each finished tier of a stat adds, in percent.
pub const PERCENT_PER_TIER: u32 = 25;

/// A rate of the player's own that techs (and later gear) raise.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Stat {
    /// How fast hand crafts run (`crafting.rs`).
    Crafting,
}

/// A player's multiplier on `stat` in thousandths: 1000 with nothing researched, 250 more per finished tier.
pub fn player_bonus(research: &Research, stat: Stat) -> u32 {
    let lists = |i: usize| TECHS[i].unlocks.iter().any(|u| matches!(*u, Unlock::Perk(s, _) if s == stat));
    let tiers = (0..TECHS.len()).filter(|&i| lists(i) && research.state(i as u8) == TechState::Done).count();
    1000 + PERCENT_PER_TIER * 10 * tiers as u32
}

#[cfg(test)]
mod tests;
