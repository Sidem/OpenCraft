//! Codex achievements (`codex/mod.rs`): pure data rows evaluated over the world's `obtained` item
//! bitset. No rewards beyond the card and a HUD toast (`docs/DECISIONS.md` 2026-11-16).
//! To add an achievement: append a row to `ACHIEVEMENTS` (up to 32 rows fit in the `u32` toast mask).

use crate::block::IRON_ORE;
use crate::item::{
    ItemId, ACID_BARREL, ALUMINIUM_INGOT, CIRCUIT, COPPER_WIRE, CRUDE_BARREL, FUEL_CELL, HYDROGEN_BARREL, IRON_INGOT,
    METHANE_BARREL, OXYGEN_BARREL, PIGMENT_BLUE, PLASTIC, ROCKET_FUEL_BARREL, SILICON, STEEL_INGOT,
};

use super::data::{ALL_ORES, ALL_PIGMENTS, ALL_SATELLITES, ELEMENTS};
use super::Codex;

/// How an achievement's progress `(done, target)` is derived from `Codex`.
pub enum Rule {
    /// Any one of these items obtained.
    Any(&'static [ItemId]),
    /// Every one of these items obtained.
    All(&'static [ItemId]),
    /// At least `n` distinct items from the list obtained.
    Count(&'static [ItemId], u32),
    /// At least `n` distinct periodic table elements discovered.
    Elements(u32),
    /// Every element reachable in this version discovered.
    AllReachableElements,
    /// At least `n` elements with every one of their forms obtained.
    FullElements(u32),
}

pub struct Achievement {
    pub title: &'static str,
    pub desc: &'static str,
    pub icon: ItemId,
    pub rule: Rule,
}

pub const ACHIEVEMENTS: &[Achievement] = &[
    Achievement {
        title: "First Strike",
        desc: "Mine or extract your first raw ore.",
        icon: ItemId::block(IRON_ORE),
        rule: Rule::Any(ALL_ORES),
    },
    Achievement {
        title: "Iron Age",
        desc: "Smelt your first Iron Ingot.",
        icon: IRON_INGOT,
        rule: Rule::All(&[IRON_INGOT]),
    },
    Achievement {
        title: "Wired Up",
        desc: "Draw Copper Wire for power and machines.",
        icon: COPPER_WIRE,
        rule: Rule::All(&[COPPER_WIRE]),
    },
    Achievement {
        title: "Steelworks",
        desc: "Forge a Steel Ingot in a blast furnace.",
        icon: STEEL_INGOT,
        rule: Rule::All(&[STEEL_INGOT]),
    },
    Achievement {
        title: "Silicon Valley",
        desc: "Reduce quartz to Silicon in an arc furnace.",
        icon: SILICON,
        rule: Rule::All(&[SILICON]),
    },
    Achievement {
        title: "Digital Age",
        desc: "Assemble an electronic Circuit.",
        icon: CIRCUIT,
        rule: Rule::All(&[CIRCUIT]),
    },
    Achievement {
        title: "Light Metal",
        desc: "Refine an Aluminium Ingot from bauxite.",
        icon: ALUMINIUM_INGOT,
        rule: Rule::All(&[ALUMINIUM_INGOT]),
    },
    Achievement {
        title: "Black Gold",
        desc: "Pump your first barrel of Crude Oil.",
        icon: CRUDE_BARREL,
        rule: Rule::All(&[CRUDE_BARREL]),
    },
    Achievement {
        title: "Polymer Age",
        desc: "Synthesise Plastic in a chemical plant.",
        icon: PLASTIC,
        rule: Rule::All(&[PLASTIC]),
    },
    Achievement {
        title: "Sulfuric",
        desc: "Produce a barrel of Sulfuric Acid.",
        icon: ACID_BARREL,
        rule: Rule::All(&[ACID_BARREL]),
    },
    Achievement {
        title: "Split Water",
        desc: "Produce both Hydrogen and Oxygen by electrolysis.",
        icon: HYDROGEN_BARREL,
        rule: Rule::All(&[HYDROGEN_BARREL, OXYGEN_BARREL]),
    },
    Achievement {
        title: "Natural Gas",
        desc: "Sweeten and fractionate Methane from natural gas.",
        icon: METHANE_BARREL,
        rule: Rule::All(&[METHANE_BARREL]),
    },
    Achievement {
        title: "Atomic Era",
        desc: "Enrich a Uranium Fuel Cell in a centrifuge.",
        icon: FUEL_CELL,
        rule: Rule::All(&[FUEL_CELL]),
    },
    Achievement {
        title: "True Colours",
        desc: "Grind or synthesise 5 different pigments.",
        icon: PIGMENT_BLUE,
        rule: Rule::Count(ALL_PIGMENTS, 5),
    },
    Achievement {
        title: "Orbital Reach",
        desc: "Synthesise Rocket Fuel or build a satellite for launch.",
        icon: ROCKET_FUEL_BARREL,
        rule: Rule::Any(&[
            ROCKET_FUEL_BARREL,
            ALL_SATELLITES[0],
            ALL_SATELLITES[1],
            ALL_SATELLITES[2],
            ALL_SATELLITES[3],
        ]),
    },
    Achievement {
        title: "Periodic Pioneer",
        desc: "Discover 5 elements on the periodic table.",
        icon: SILICON,
        rule: Rule::Elements(5),
    },
    Achievement {
        title: "Master Chemist",
        desc: "Discover every element reachable in this world.",
        icon: FUEL_CELL,
        rule: Rule::AllReachableElements,
    },
    Achievement {
        title: "Full Spectrum",
        desc: "Obtain every chemical form of 3 different elements.",
        icon: ACID_BARREL,
        rule: Rule::FullElements(3),
    },
];

impl Codex {
    /// Progress `(current, target)` of achievement `index`.
    pub fn achievement_progress(&self, index: usize) -> (u32, u32) {
        let Some(a) = ACHIEVEMENTS.get(index) else { return (0, 1) };
        match a.rule {
            Rule::Any(items) => (u32::from(items.iter().any(|&i| self.has(i))), 1),
            Rule::All(items) => {
                let done = items.iter().filter(|&&i| self.has(i)).count() as u32;
                (done, items.len() as u32)
            }
            Rule::Count(items, need) => {
                let done = (items.iter().filter(|&&i| self.has(i)).count() as u32).min(need);
                (done, need)
            }
            Rule::Elements(need) => ((self.discovered_elements() as u32).min(need), need),
            Rule::AllReachableElements => {
                let total = ELEMENTS.len() as u32;
                ((self.discovered_elements() as u32).min(total), total)
            }
            Rule::FullElements(need) => {
                let full = ELEMENTS
                    .iter()
                    .filter(|e| {
                        let (done, total) = self.element_forms_done(e.z);
                        total > 0 && done == total
                    })
                    .count() as u32;
                (full.min(need), need)
            }
        }
    }

    pub fn achievement_done(&self, index: usize) -> bool {
        let (cur, target) = self.achievement_progress(index);
        target > 0 && cur >= target
    }

    /// Bitmask of unlocked achievements (bit `i` = `ACHIEVEMENTS[i]`), used by the HUD to toast new unlocks.
    pub fn unlocked_mask(&self) -> u32 {
        let mut mask = 0u32;
        for i in 0..ACHIEVEMENTS.len().min(32) {
            if self.achievement_done(i) {
                mask |= 1 << i;
            }
        }
        mask
    }

    pub fn unlocked_count(&self) -> usize {
        self.unlocked_mask().count_ones() as usize
    }
}
