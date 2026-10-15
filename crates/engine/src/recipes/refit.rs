//! The upgrader's recipes: for every tier of every family in `factory/tiers.rs`, one machine row that takes the Mk1 item and
//! the kits of every step up to that tier and makes the tier's item, so a Mk4 pole is one machine, not three in a row.
//! Its inputs are worked out from `FAMILIES` and `upgrades::KITS` at compile time (`ladder`), so the cost is exactly what
//! upgrading in place step by step costs (`recipes/tests.rs` checks every tier has a row). Also the upgrader's hand recipe.
//!
//! To add a tier: nothing here but its number in `REFIT`'s row of the family (the test names a missing one). To add a
//! family: a line in `REFIT`.

use crate::block::*;
use crate::factory::tiers::{Family, FAMILIES};
use crate::factory::upgrades::KITS;
use crate::item::*;

use super::{b, Category, Group, MachineRecipe, Recipe};

pub const UPGRADER_RECIPE: Recipe = Recipe {
    output: b(UPGRADER),
    group: Group::Production,
    count: 1,
    inputs: &[(IRON_PLATE, 12), (GEAR, 10), (COPPER_WIRE, 16), (MOTOR, 2)],
    blurb: "Raises a machine's Mk without you: belts bring in the Mk1 item and the kits of each step (Mk3 takes green and \
            blue kits, Mk4 adds violet) and the kits are used up. It is 2×2×2 (R turns it before you place it). Right-click \
            to choose the tier it makes: poles, boxes, belts, miners and the rest. 30 kW.",
};

/// Seconds an upgrade step takes at full power.
const STEP_SECONDS: f64 = 4.0;

const fn family(block: BlockId) -> &'static Family {
    let mut i = 0;
    while i < FAMILIES.len() {
        if FAMILIES[i].block == block {
            return &FAMILIES[i];
        }
        i += 1;
    }
    panic!("no such family")
}

/// The Mk1 item and each step's kits up to `tier`: `N` = `tier` + 1 rows.
const fn ladder<const N: usize>(f: &Family, tier: usize) -> [(ItemId, u32); N] {
    let mut inputs = [(f.items[0], 1); N];
    let mut step = 1;
    while step <= tier {
        inputs[step] = (KITS[step], f.kits);
        step += 1;
    }
    inputs
}

/// One row per `BLOCK: tiers…`: the tiers above Mk1 (1 is Mk2).
macro_rules! rows {
    ($($block:ident: $($tier:literal)+;)+) => {
        &[$($(MachineRecipe {
            category: Category::Refit,
            inputs: &ladder::<{ $tier + 1 }>(family($block), $tier),
            outputs: &[(family($block).items[$tier], 1)],
            seconds: STEP_SECONDS * $tier as f64,
        },)+)+]
    };
}

pub const REFIT: &[MachineRecipe] = rows! {
    BELT: 1 2 3;
    UNDERPASS_IN: 1 2 3;
    POLE: 1 2 3;
    STORAGE: 1 2;
    MINER: 1 2 3 4;
    SMELTER: 1 2 3 4;
    CONSTRUCTOR: 1 2 3 4;
    ASSEMBLER: 1 2 3 4;
    BLAST_FURNACE: 1 2 3 4;
    PUMP: 1 2;
    QUARRY: 1 2;
    LAB: 1 2 3;
    GENERATOR: 1;
    DRONE_PORT: 1 2 3 4;
    RESEARCH_CENTER: 1 2 3 4;
};
