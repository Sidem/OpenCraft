//! Town perks (step 13.4): what a town gives back while it is supplied. One module a perk; this file holds what they
//! share. A perk works only while its town is on (`Town::perk_on`: a camp at least, awake, not stalled), so a stalled
//! town pauses it until its upkeep is back.
//!
//! - **Basalt forge** (`forge.rs`): ore in, plates and ingots out, half again what a smelter gives.
//! - **Highland workshop** (`workshop.rs`): worn tools in, whole tools out; gear costs half in hand crafting anywhere.
//! - **River town** (`relay.rs`): drone ports near it reach and fly twice as far.
//! - The harbour's contract work comes in step 13.4b.
//!
//! Invariants: what a perk takes waits in the depot's perk buffer and what it gives in its out buffer (`Want::perk`,
//! `Factory::depot_perk`), never in the store, so a perk can't starve the upkeep. Perks work in batches on a fixed
//! schedule (`period`, a function of the tick and the stage), so they need no saved state.
//! To add a perk: a module here, its arm in `wants` and `step`.

mod forge;
pub mod harbour;
mod relay;
mod workshop;

#[cfg(test)]
mod tests;

pub use workshop::{discounted, gear_cost};

use crate::factory::{Buffer, Want};
use crate::sim::SimEvent;

use super::orders::UPKEEP_PER_MINUTE;
use super::run::MINUTE;
use super::{Settlements, Town, FORGE, HARBOUR, RIVER_TOWN, WORKSHOP};

/// Items (or tools) a perk handles in one batch.
pub const BATCH: u32 = 4;

/// Items a perk handles a minute in a town that has built `built` stages (at least a camp): the rate of its upkeep,
/// so 20 at a camp and 600 at a city.
pub fn rate(built: u8) -> u32 {
    UPKEEP_PER_MINUTE[usize::from(built.max(1)) - 1]
}

/// Ticks between two batches of `BATCH` items at that rate.
pub fn period(built: u8) -> u64 {
    MINUTE * u64::from(BATCH) / u64::from(rate(built))
}

impl Town {
    /// Whether its perk works now: a camp at least, awake and supplied.
    pub fn perk_on(&self) -> bool {
        self.built >= 1 && !self.dormant && !self.stalled
    }

    /// What its perk takes in, and up to how many its depot holds of each (nothing before the camp).
    pub fn perk_wants(&self) -> Vec<Want> {
        if self.built == 0 {
            return Vec::new();
        }
        let lines = match self.kind {
            FORGE => forge::wants(self.built),
            WORKSHOP => workshop::wants(),
            HARBOUR => harbour::wants(self),
            _ => Vec::new(),
        };
        lines.into_iter().map(|(item, cap)| Want { item, cap, perk: true }).collect()
    }

    /// What its perk does, for the depot's readout (`None` for a kind without one yet).
    pub fn perk_text(&self) -> Option<String> {
        let rate = rate(self.built);
        let text = match self.kind {
            FORGE => {
                format!("{rate} iron or copper ore a minute in, half again a smelter's plates and ingots out the front")
            }
            WORKSHOP => format!(
                "hauler packs, jetpacks and gloves cost half to craft · worn tools in, {} whole ones a minute out the \
                 front",
                rate / BATCH
            ),
            RIVER_TOWN => format!("drone ports within {} blocks reach and fly twice as far", relay::RELAY_RANGE),
            HARBOUR if self.product == crate::item::ItemId::NONE => format!(
                "pick a product your machines have made below: it comes out the front for {} times its raw \
                 materials, {rate} of them a minute in",
                harbour::PRICE
            ),
            HARBOUR => format!(
                "{} out the front for {} times its raw materials, {rate} of them a minute in",
                crate::item::name(self.product),
                harbour::PRICE
            ),
            _ => return None,
        };
        let state = match (self.built, self.stalled) {
            (0, _) => " (from the camp on)",
            (_, true) => " (paused while stalled)",
            _ => "",
        };
        Some(format!("Perk: {text}{state}"))
    }

    /// One tick of its perk: a batch when one is due, from what it has taken in to what it gives.
    pub fn step_perk(&mut self, input: &mut Buffer, out: &mut Buffer, tick: u64, events: &mut Vec<SimEvent>) {
        let period = period(self.built);
        if !self.perk_on() || !tick.is_multiple_of(period) {
            return;
        }
        match self.kind {
            FORGE => forge::batch(input, out, tick / period, events),
            WORKSHOP => workshop::repair(input, out),
            HARBOUR => harbour::batch(self, input, out, events),
            _ => {}
        }
    }
}

impl Settlements {
    /// Whether a supplied highland workshop stands anywhere: its gear costs half in hand crafting.
    pub fn gear_works(&self) -> bool {
        self.towns.iter().any(|t| t.kind == WORKSHOP && t.perk_on())
    }
}
