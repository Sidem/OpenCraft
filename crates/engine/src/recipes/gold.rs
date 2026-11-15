//! Hand recipes of the Mk5 items (`factory/tiers.rs`, the Mk5 Machines tech): each is the Mk4 plus gold kits, and the
//! lint in `tests.rs` checks it. Listed in `RECIPES`. To add one: a `pub const` here, and its name in `RECIPES`.

use crate::block::PUMPJACK;
use crate::item::*;

use super::{b, Group, Recipe};

pub const MINER_MK5_RECIPE: Recipe = Recipe {
    output: MINER_MK5,
    group: Group::Production,
    count: 1,
    inputs: &[(MINER_MK4, 1), (GOLD_KIT, 4)],
    blurb: "Drills eight times as fast as a Mk1 and recovers 97% of what it draws. Needs 150 kW. Or upgrade a placed \
            miner with 4 gold kits.",
};

pub const SMELTER_MK5_RECIPE: Recipe = Recipe {
    output: SMELTER_MK5,
    group: Group::Production,
    count: 1,
    inputs: &[(SMELTER_MK4, 1), (GOLD_KIT, 4)],
    blurb: "Smelts eight times as fast as a Mk1, electric: 130 kW and no fuel. Or upgrade a placed smelter with 4 \
            gold kits.",
};

pub const CONSTRUCTOR_MK5_RECIPE: Recipe = Recipe {
    output: CONSTRUCTOR_MK5,
    group: Group::Production,
    count: 1,
    inputs: &[(CONSTRUCTOR_MK4, 1), (GOLD_KIT, 4)],
    blurb: "Works eight times as fast as a Mk1 and draws 120 kW. Or upgrade a placed constructor with 4 gold kits.",
};

pub const ASSEMBLER_MK5_RECIPE: Recipe = Recipe {
    output: ASSEMBLER_MK5,
    group: Group::Production,
    count: 1,
    inputs: &[(ASSEMBLER_MK4, 1), (GOLD_KIT, 8)],
    blurb: "Assembles eight times as fast as a Mk1 and draws 160 kW. Or upgrade a placed assembler with 8 gold kits.",
};

pub const BLAST_FURNACE_MK5_RECIPE: Recipe = Recipe {
    output: BLAST_FURNACE_MK5,
    group: Group::Production,
    count: 1,
    inputs: &[(BLAST_FURNACE_MK4, 1), (GOLD_KIT, 8)],
    blurb: "Makes steel eight times as fast as a Mk1. Or upgrade a placed blast furnace with 8 gold kits.",
};

pub const DRONE_PORT_MK5_RECIPE: Recipe = Recipe {
    output: DRONE_PORT_MK5,
    group: Group::Logistics,
    count: 1,
    inputs: &[(DRONE_PORT_MK4, 1), (GOLD_KIT, 8)],
    blurb:
        "Keeps 24 drones and sends them up to 128 blocks out; draws 200 kW while they fly. Or upgrade a placed port \
            with 8 gold kits.",
};

pub const BOX_MK5_RECIPE: Recipe = Recipe {
    output: BOX_MK5,
    group: Group::Logistics,
    count: 1,
    inputs: &[(BOX_MK4, 1), (GOLD_KIT, 4)],
    blurb: "Holds 72 stacks. Or upgrade a placed box with 4 gold kits; what is in it stays.",
};

/// The pumpjack arrived with violet science, so its first step up is a gold one (`Family::first_kit`).
pub const PUMPJACK_MK2_RECIPE: Recipe = Recipe {
    output: PUMPJACK_MK2,
    group: Group::Production,
    count: 1,
    inputs: &[(b(PUMPJACK), 1), (GOLD_KIT, 4)],
    blurb:
        "Draws twice as fast and gets 97% of the oil to the barrel; needs 160 kW. Or upgrade a placed pumpjack with \
            4 gold kits.",
};

pub const RESEARCH_CENTER_MK5_RECIPE: Recipe = Recipe {
    output: RESEARCH_CENTER_MK5,
    group: Group::Science,
    count: 1,
    inputs: &[(RESEARCH_CENTER_MK4, 1), (GOLD_KIT, 8)],
    blurb: "Ten times a lab's speed, and every 2nd unit takes no packs; draws 100 kW. Or upgrade a placed center with \
            8 gold kits.",
};
