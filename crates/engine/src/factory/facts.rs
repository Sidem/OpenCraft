//! The build menu's card for a building: `facts(item)` gives its size, what drives it (fuel or electricity and how
//! many kW), what it produces (generators) and a few numbers, each in a field of its own so the player sees
//! them at a glance. Presentation only: every number is read from the machine's own tables (`MINER_TIERS`,
//! `SPECS`, ...), never copied.
//!
//! `Facts::rows` is the text the host draws: one row per line, `kind<TAB>label<TAB>value`, where the kind is `size`,
//! `use` (electricity drawn), `fuel`, `make` (power produced), `none` (needs no energy) or `stat`.
//! To give another building a card: an arm in `facts` (processors need none: their spec has it all).

use super::belt::BELT_TIERS;
use super::describe::fmt_int;
use super::generator::GENERATOR_TIERS;
use super::lab::LAB_TIERS;
use super::miner::MINER_TIERS;
use super::pole::{CABLE_STATS, POLE_TIERS};
use super::process::{facts as process_facts, spec};
use super::pumping::{OUTLET_RATE, PUMP_TIERS};
use super::quarry::{DEPTHS, QUARRY_TIERS, WIDTHS};
use super::storage::BOX_SLOTS;
use super::tiers::placed_by;
use super::underpass::UNDERPASS_SPAN;
use crate::block::{
    BELT, CABLE, FAST_BELT, FILTER, GENERATOR, LAB, LIFT, MINER, MINER_MK2, OUTLET, PIPE, POLE, PUMP, QUARRY, SENSOR,
    SPLITTER, STORAGE, UNDERPASS_IN,
};
use crate::item::{self, ItemId};
use crate::recipes::FUELS;

/// What the card shows for one building (one tier of it).
pub struct Facts {
    /// Width, depth and height in blocks; all 0 when the player sets it (a quarry).
    pub size: [i32; 3],
    /// Electricity drawn while working, in kW.
    pub uses: Option<u32>,
    /// What it burns, as names.
    pub fuel: Option<String>,
    /// Power produced at most, in kW, and when ("in full sun").
    pub makes: Option<(u32, &'static str)>,
    /// Any other number worth a glance: label and value.
    pub stats: Vec<(&'static str, String)>,
}

impl Facts {
    pub fn new(size: [i32; 3]) -> Facts {
        Facts { size, uses: None, fuel: None, makes: None, stats: Vec::new() }
    }

    pub fn stat(&mut self, label: &'static str, value: impl Into<String>) {
        self.stats.push((label, value.into()));
    }

    /// The card's text (see the module header).
    pub fn rows(&self) -> String {
        let mut out = Vec::new();
        let [w, d, h] = self.size;
        let size = if w == 0 { "Set when placed".to_string() } else { format!("{w}×{d}×{h}") };
        out.push(format!("size\tSize\t{size}"));
        if let Some(kw) = self.uses {
            out.push(format!("use\tUses\t{} kW", fmt_int(kw.into())));
        }
        if let Some(fuel) = &self.fuel {
            out.push(format!("fuel\tBurns\t{fuel}"));
        }
        if let Some((kw, note)) = self.makes {
            out.push(format!("make\tMakes\t{}{note}", kw_text(kw)));
        }
        if self.uses.is_none() && self.fuel.is_none() && self.makes.is_none() {
            out.push("none\tEnergy\tNone needed".to_string());
        }
        out.extend(self.stats.iter().map(|(label, value)| format!("stat\t{label}\t{value}")));
        out.join("\n")
    }
}

fn kw_text(kw: u32) -> String {
    format!("up to {} kW", fmt_int(kw.into()))
}

/// The names of everything a burner takes.
pub fn fuel_names() -> String {
    FUELS.iter().map(|f| item::name(f.0)).collect::<Vec<_>>().join(", ")
}

/// The card of the building `item` places, or None for anything that is not a machine.
pub fn facts(item: ItemId) -> Option<Facts> {
    let (block, tier) = placed_by(item).or_else(|| u8::try_from(item.0).ok().map(|b| (b, 0)))?;
    let tier = usize::from(tier);
    if let Some(s) = spec(block) {
        return Some(process_facts(s, tier));
    }
    let mut f = Facts::new([1, 1, 1]);
    match block {
        MINER | MINER_MK2 => {
            let t = &MINER_TIERS[tier.min(MINER_TIERS.len() - 1)];
            f.uses = Some(t.power);
            f.stat("Mines", format!("{} ore/s", t.rate as u32));
            f.stat("Keeps", format!("{}% of the ore", (t.recovery * 100.0).round() as u32));
        }
        BELT | FAST_BELT => {
            f.stat("Speed", format!("{} blocks/s", BELT_TIERS[tier.min(BELT_TIERS.len() - 1)].speed as u32));
        }
        UNDERPASS_IN => {
            f.stat("Reach", format!("{} blocks", UNDERPASS_SPAN[tier.min(UNDERPASS_SPAN.len() - 1)]));
        }
        SPLITTER => f.stat("Splits", "Evenly between the belts leading away"),
        FILTER => f.stat("Passes", "One chosen item straight on, the rest aside"),
        LIFT => f.stat("Carries", "Items up or down one block at a time"),
        STORAGE => f.stat("Holds", format!("{} stacks", BOX_SLOTS[tier.min(BOX_SLOTS.len() - 1)])),
        PUMP => {
            let t = &PUMP_TIERS[tier.min(PUMP_TIERS.len() - 1)];
            f.uses = Some(t.power);
            f.stat("Lifts", format!("{} blocks of water/s", t.rate));
        }
        PIPE => f.stat("Carries", "Water or steam between machines"),
        OUTLET => f.stat("Pours", format!("{OUTLET_RATE} blocks of water/s")),
        GENERATOR => {
            let t = &GENERATOR_TIERS[tier.min(GENERATOR_TIERS.len() - 1)];
            f.fuel = Some(fuel_names());
            f.makes = Some((t.power, ""));
            f.stat("Fuel yield", format!("{}%", t.yield_percent));
        }
        POLE => {
            let t = &POLE_TIERS[tier.min(POLE_TIERS.len() - 1)];
            f.stat("Links poles", format!("within {} blocks", t.link));
            f.stat("Powers machines", format!("within {} blocks", t.reach));
            f.stat("Connections", t.slots.to_string());
        }
        CABLE => f.stat("Powers machines", format!("within {} blocks", CABLE_STATS.reach)),
        LAB => {
            let t = &LAB_TIERS[tier.min(LAB_TIERS.len() - 1)];
            f.uses = Some(t.power);
            f.stat("Research speed", format!("×{}", t.speed));
            if t.free_every > 0 {
                f.stat("Free unit", format!("every {}", t.free_every));
            }
        }
        QUARRY => {
            let t = &QUARRY_TIERS[tier.min(QUARRY_TIERS.len() - 1)];
            f.size = [0; 3];
            f.uses = Some(t.power);
            f.stat("Speed", format!("{} blocks/s", 60 / t.dig_ticks));
            let w = WIDTHS.iter().max().copied().unwrap_or(0);
            f.stat("Digs", format!("a {w}×{w} pit at most, {}", DEPTHS[DEPTHS.len() - 1].label()));
        }
        SENSOR => f.stat("Reads", "The box, silo or belt behind it"),
        _ => return None,
    }
    Some(f)
}

#[cfg(test)]
mod tests;
