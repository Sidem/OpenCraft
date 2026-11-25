//! How long a hand craft takes: the one place to tune it. A recipe's time is a base plus some per material
//! (`BASE_TICKS`, `TICKS_PER_ITEM`, capped at `MAX_TICKS`), unless its output is listed in `OVERRIDES`,
//! which gives that recipe an exact time in tenths of a second. The queue (`crafting.rs`) and the build
//! menu's "Takes N s" line both read `Recipe::hand_ticks`, so nothing else needs to change. A tick is 1/60 s.
//!
//! To make one recipe faster or slower: a row in `OVERRIDES`, e.g. `(b(STORAGE), 40)` for 4 seconds a box.
//! To scale everything: the three constants. Crafting time is core state (a queued craft's progress is
//! saved), so changing it never breaks a save, but it changes the golden hash if a scripted craft moves.

use crate::block::{COAL_MINER, SMELTER, STORAGE};
use crate::item::ItemId;
use crate::TICK_RATE;

use super::Recipe;

/// Every craft takes this long, and this much more for each material it uses, in ticks.
pub const BASE_TICKS: u32 = 60;
pub const TICKS_PER_ITEM: u32 = 20;
/// The longest a formula-timed craft takes (20 s).
pub const MAX_TICKS: u32 = 1200;

/// Exact times by output item, in tenths of a second (`(item, 45)` is 4.5 s), keeping early bootstrap buildings
/// snappy while constructors and assemblers still save bulk hand work.
const OVERRIDES: &[(ItemId, u32)] =
    &[(ItemId::block(STORAGE), 35), (ItemId::block(COAL_MINER), 45), (ItemId::block(SMELTER), 50)];

impl Recipe {
    /// How long one craft takes by hand, in ticks.
    pub fn hand_ticks(&self) -> u32 {
        self.ticks_with(OVERRIDES)
    }

    /// `hand_ticks` with a given override table (so a test can plant one).
    pub(super) fn ticks_with(&self, overrides: &[(ItemId, u32)]) -> u32 {
        if let Some(&(_, tenths)) = overrides.iter().find(|o| o.0 == self.output) {
            return tenths * TICK_RATE / 10;
        }
        let items: u32 = self.inputs.iter().map(|i| i.1).sum();
        (BASE_TICKS + TICKS_PER_ITEM * items).min(MAX_TICKS)
    }
}
