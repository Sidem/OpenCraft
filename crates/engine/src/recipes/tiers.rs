//! Hand recipes of the tier items (`factory/tiers.rs`): each is the previous tier plus kits, and the lint
//! in `tests.rs` checks it. They live here to keep `mod.rs` short and are listed in `RECIPES`.
//! To add one: a `pub const` here, and its name in `RECIPES`.

use crate::block::*;
use crate::item::*;

use super::{b, Group, Recipe};
pub const POLE_MK2_RECIPE: Recipe = Recipe {
    output: POLE_MK2,
    group: Group::Power,
    count: 1,
    inputs: &[(b(POLE), 1), (GREEN_KIT, 1)],
    blurb: "A taller pole: links to poles within 16 blocks and powers machines within 7. Or upgrade a placed \
            pole with a green kit.",
};

pub const POLE_MK3_RECIPE: Recipe = Recipe {
    output: POLE_MK3,
    group: Group::Power,
    count: 1,
    inputs: &[(POLE_MK2, 1), (BLUE_KIT, 1)],
    blurb: "A pylon: links to poles within 32 blocks and powers machines within 9. Or upgrade a placed pole \
            with a blue kit.",
};

pub const BOX_MK2_RECIPE: Recipe = Recipe {
    output: BOX_MK2,
    group: Group::Logistics,
    count: 1,
    inputs: &[(b(STORAGE), 1), (GREEN_KIT, 4)],
    blurb: "Holds 36 stacks. Or upgrade a placed box with 4 green kits; what is in it stays.",
};

pub const BOX_MK3_RECIPE: Recipe = Recipe {
    output: BOX_MK3,
    group: Group::Logistics,
    count: 1,
    inputs: &[(BOX_MK2, 1), (BLUE_KIT, 4)],
    blurb: "Holds 48 stacks. Or upgrade a placed box with 4 blue kits; what is in it stays.",
};

pub const PUMP_MK2_RECIPE: Recipe = Recipe {
    output: PUMP_MK2,
    group: Group::Logistics,
    count: 1,
    inputs: &[(b(PUMP), 1), (GREEN_KIT, 4)],
    blurb: "Lifts 4 blocks of water a second and holds 4; needs 10 kW. Or upgrade a placed pump with 4 green kits.",
};

pub const PUMP_MK3_RECIPE: Recipe = Recipe {
    output: PUMP_MK3,
    group: Group::Logistics,
    count: 1,
    inputs: &[(PUMP_MK2, 1), (BLUE_KIT, 4)],
    blurb: "Lifts 6 blocks of water a second and holds 6; needs 20 kW. Or upgrade a placed pump with 4 blue kits.",
};

pub const QUARRY_MK2_RECIPE: Recipe = Recipe {
    output: QUARRY_MK2,
    group: Group::Production,
    count: 1,
    inputs: &[(b(QUARRY), 1), (GREEN_KIT, 4)],
    blurb: "Digs 4 blocks a second and needs 20 kW. Or upgrade a placed quarry with 4 green kits.",
};

pub const QUARRY_MK3_RECIPE: Recipe = Recipe {
    output: QUARRY_MK3,
    group: Group::Production,
    count: 1,
    inputs: &[(QUARRY_MK2, 1), (BLUE_KIT, 4)],
    blurb: "Digs 6 blocks a second and needs 30 kW. Or upgrade a placed quarry with 4 blue kits.",
};

pub const LAB_MK2_RECIPE: Recipe = Recipe {
    output: LAB_MK2,
    group: Group::Science,
    count: 1,
    inputs: &[(b(LAB), 1), (GREEN_KIT, 4)],
    blurb: "Researches twice as fast and needs 20 kW. Or upgrade a placed lab with 4 green kits.",
};

pub const LAB_MK3_RECIPE: Recipe = Recipe {
    output: LAB_MK3,
    group: Group::Science,
    count: 1,
    inputs: &[(LAB_MK2, 1), (BLUE_KIT, 4)],
    blurb: "Researches three times as fast, needs 30 kW, and every fifth unit takes no packs: 4 packs do the \
            work of 5. Or upgrade a placed lab with 4 blue kits.",
};

pub const GENERATOR_MK2_RECIPE: Recipe = Recipe {
    output: GENERATOR_MK2,
    group: Group::Power,
    count: 1,
    inputs: &[(b(GENERATOR), 1), (GREEN_KIT, 4)],
    blurb: "Gives up to 100 kW and gets a quarter more energy from every fuel item. Or upgrade a placed \
            generator with 4 green kits.",
};

pub const MINER_MK2_RECIPE: Recipe = Recipe {
    output: b(MINER_MK2),
    group: Group::Production,
    count: 1,
    inputs: &[(b(MINER), 1), (GREEN_KIT, 4)],
    blurb: "Drills twice as fast as a Mk1 and recovers 75% of what it draws, so the same deposit gives \
            more ore. Needs 20 kW. Or upgrade a placed miner with 4 green kits.",
};

pub const SMELTER_MK2_RECIPE: Recipe = Recipe {
    output: SMELTER_MK2,
    group: Group::Production,
    count: 1,
    inputs: &[(b(SMELTER), 1), (GREEN_KIT, 4)],
    blurb: "Smelts twice as fast as a Mk1 and burns a quarter less fuel an ingot. Or upgrade a placed \
            smelter with 4 green kits.",
};

pub const CONSTRUCTOR_MK2_RECIPE: Recipe = Recipe {
    output: CONSTRUCTOR_MK2,
    group: Group::Production,
    count: 1,
    inputs: &[(b(CONSTRUCTOR), 1), (GREEN_KIT, 4)],
    blurb: "Works twice as fast as a Mk1 and draws 30 kW. Or upgrade a placed constructor with 4 green kits.",
};

pub const MINER_MK3_RECIPE: Recipe = Recipe {
    output: MINER_MK3,
    group: Group::Production,
    count: 1,
    inputs: &[(b(MINER_MK2), 1), (BLUE_KIT, 4)],
    blurb: "Drills four times as fast as a Mk1 and recovers 85% of what it draws. Needs 45 kW. Or upgrade a \
            placed miner with 4 blue kits.",
};

pub const SMELTER_MK3_RECIPE: Recipe = Recipe {
    output: SMELTER_MK3,
    group: Group::Production,
    count: 1,
    inputs: &[(SMELTER_MK2, 1), (BLUE_KIT, 4)],
    blurb: "Smelts three times as fast as a Mk1, electric: 40 kW and no fuel. Or upgrade a placed smelter \
            with 4 blue kits.",
};

pub const CONSTRUCTOR_MK3_RECIPE: Recipe = Recipe {
    output: CONSTRUCTOR_MK3,
    group: Group::Production,
    count: 1,
    inputs: &[(CONSTRUCTOR_MK2, 1), (BLUE_KIT, 4)],
    blurb: "Works three times as fast as a Mk1 and draws 45 kW. Or upgrade a placed constructor with 4 blue kits.",
};

pub const ASSEMBLER_MK2_RECIPE: Recipe = Recipe {
    output: ASSEMBLER_MK2,
    group: Group::Production,
    count: 1,
    inputs: &[(b(ASSEMBLER), 1), (GREEN_KIT, 8)],
    blurb: "Assembles twice as fast as a Mk1 and draws 40 kW. Or upgrade a placed assembler with 8 green kits.",
};

pub const ASSEMBLER_MK3_RECIPE: Recipe = Recipe {
    output: ASSEMBLER_MK3,
    group: Group::Production,
    count: 1,
    inputs: &[(ASSEMBLER_MK2, 1), (BLUE_KIT, 8)],
    blurb: "Assembles three times as fast as a Mk1 and draws 60 kW. Or upgrade a placed assembler with 8 blue kits.",
};

pub const BLAST_FURNACE_MK2_RECIPE: Recipe = Recipe {
    output: BLAST_FURNACE_MK2,
    group: Group::Production,
    count: 1,
    inputs: &[(b(BLAST_FURNACE), 1), (GREEN_KIT, 8)],
    blurb: "Makes steel twice as fast as a Mk1. Or upgrade a placed blast furnace with 8 green kits.",
};

pub const BLAST_FURNACE_MK3_RECIPE: Recipe = Recipe {
    output: BLAST_FURNACE_MK3,
    group: Group::Production,
    count: 1,
    inputs: &[(BLAST_FURNACE_MK2, 1), (BLUE_KIT, 8)],
    blurb: "Makes steel three times as fast as a Mk1. Or upgrade a placed blast furnace with 8 blue kits.",
};

pub const BELT_MK2_RECIPE: Recipe = Recipe {
    output: b(FAST_BELT),
    group: Group::Logistics,
    count: 1,
    inputs: &[(b(BELT), 1), (GREEN_KIT, 1)],
    blurb: "Carries items twice as fast as a Mk1 belt and mixes freely with them. Or drag along placed \
            belts with green kits to upgrade them.",
};

pub const BELT_MK3_RECIPE: Recipe = Recipe {
    output: BELT_MK3,
    group: Group::Logistics,
    count: 1,
    inputs: &[(b(FAST_BELT), 1), (BLUE_KIT, 1)],
    blurb: "Carries items four times as fast as a Mk1 belt (10 a second) and mixes freely with the others. Or \
            drag along placed belts with blue kits to upgrade them.",
};

pub const BELT_MK4_RECIPE: Recipe = Recipe {
    output: BELT_MK4,
    group: Group::Logistics,
    count: 1,
    inputs: &[(BELT_MK3, 1), (VIOLET_KIT, 1)],
    blurb: "Carries items eight times as fast as a Mk1 belt (23 a second) and mixes freely with the others. Or drag \
            along placed belts with violet kits to upgrade them.",
};

pub const POLE_MK4_RECIPE: Recipe = Recipe {
    output: POLE_MK4,
    group: Group::Power,
    count: 1,
    inputs: &[(POLE_MK3, 1), (VIOLET_KIT, 1)],
    blurb: "A substation: links to poles within 32 blocks like a pylon and powers machines within 16. Or upgrade \
            a placed pole with a violet kit.",
};

pub const LAB_MK4_RECIPE: Recipe = Recipe {
    output: LAB_MK4,
    group: Group::Science,
    count: 1,
    inputs: &[(LAB_MK3, 1), (VIOLET_KIT, 4)],
    blurb: "Researches four times as fast, needs 40 kW, and every third unit takes no packs: 2 packs do the work \
            of 3. Or upgrade a placed lab with 4 violet kits.",
};

pub const MINER_MK4_RECIPE: Recipe = Recipe {
    output: MINER_MK4,
    group: Group::Production,
    count: 1,
    inputs: &[(MINER_MK3, 1), (VIOLET_KIT, 4)],
    blurb: "Drills six times as fast as a Mk1 and recovers 92% of what it draws. Needs 90 kW. Or upgrade a \
            placed miner with 4 violet kits.",
};

pub const SMELTER_MK4_RECIPE: Recipe = Recipe {
    output: SMELTER_MK4,
    group: Group::Production,
    count: 1,
    inputs: &[(SMELTER_MK3, 1), (VIOLET_KIT, 4)],
    blurb: "Smelts five times as fast as a Mk1, electric: 80 kW and no fuel. Or upgrade a placed smelter with 4 \
            violet kits.",
};

pub const CONSTRUCTOR_MK4_RECIPE: Recipe = Recipe {
    output: CONSTRUCTOR_MK4,
    group: Group::Production,
    count: 1,
    inputs: &[(CONSTRUCTOR_MK3, 1), (VIOLET_KIT, 4)],
    blurb: "Works five times as fast as a Mk1 and draws 75 kW. Or upgrade a placed constructor with 4 violet kits.",
};

pub const ASSEMBLER_MK4_RECIPE: Recipe = Recipe {
    output: ASSEMBLER_MK4,
    group: Group::Production,
    count: 1,
    inputs: &[(ASSEMBLER_MK3, 1), (VIOLET_KIT, 8)],
    blurb: "Assembles five times as fast as a Mk1 and draws 100 kW. Or upgrade a placed assembler with 8 violet kits.",
};

pub const BLAST_FURNACE_MK4_RECIPE: Recipe = Recipe {
    output: BLAST_FURNACE_MK4,
    group: Group::Production,
    count: 1,
    inputs: &[(BLAST_FURNACE_MK3, 1), (VIOLET_KIT, 8)],
    blurb: "Makes steel five times as fast as a Mk1. Or upgrade a placed blast furnace with 8 violet kits.",
};
