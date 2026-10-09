//! Hand recipes of the tools (`tools.rs`): the count of each is its uses, so a craft makes one fresh tool.
//! To add one: a `pub const` here, and its name in `RECIPES`.

use crate::block::*;
use crate::item::*;
use crate::tools::{IRON_TIER, STEEL_TIER, STONE_TIER};

use super::{b, Group, Recipe};

pub const STONE_PICKAXE_RECIPE: Recipe = Recipe {
    output: STONE_PICKAXE,
    group: Group::Tools,
    count: STONE_TIER.uses,
    inputs: &[(b(STONE), 3), (STICK, 2)],
    blurb: "Hold it to break stone and ore twice as fast. Wears out after 150 blocks.",
};

pub const STONE_AXE_RECIPE: Recipe = Recipe {
    output: STONE_AXE,
    group: Group::Tools,
    count: STONE_TIER.uses,
    inputs: &[(b(STONE), 3), (STICK, 2)],
    blurb: "Hold it to chop wood twice as fast. Wears out after 150 blocks.",
};

pub const STONE_SHOVEL_RECIPE: Recipe = Recipe {
    output: STONE_SHOVEL,
    group: Group::Tools,
    count: STONE_TIER.uses,
    inputs: &[(b(STONE), 1), (STICK, 2)],
    blurb: "Hold it to dig dirt, grass and sand twice as fast. Wears out after 150 blocks.",
};

pub const IRON_PICKAXE_RECIPE: Recipe = Recipe {
    output: IRON_PICKAXE,
    group: Group::Tools,
    count: IRON_TIER.uses,
    inputs: &[(IRON_PLATE, 3), (IRON_ROD, 2)],
    blurb: "Breaks stone and ore four times as fast and keeps 4 ore per block instead of 3. Lasts 600 blocks.",
};

pub const IRON_AXE_RECIPE: Recipe = Recipe {
    output: IRON_AXE,
    group: Group::Tools,
    count: IRON_TIER.uses,
    inputs: &[(IRON_PLATE, 3), (IRON_ROD, 2)],
    blurb: "Chops wood four times as fast. Lasts 600 blocks.",
};

pub const IRON_SHOVEL_RECIPE: Recipe = Recipe {
    output: IRON_SHOVEL,
    group: Group::Tools,
    count: IRON_TIER.uses,
    inputs: &[(IRON_PLATE, 1), (IRON_ROD, 2)],
    blurb: "Digs dirt, grass and sand four times as fast. Lasts 600 blocks.",
};

/// Built on a Mk1 scanner (which it uses up), so scanning stays a ladder.
pub const SCANNER_MK2_RECIPE: Recipe = Recipe {
    output: SCANNER_MK2,
    group: Group::Tools,
    count: 1,
    inputs: &[(SCANNER, 1), (CIRCUIT, 3), (STEEL_PLATE, 2)],
    blurb: "A scanner with a violet screen: lists the ore deposits within 96 blocks, twice as far, shows each one's \
            ore units and mining time, filters by ore (R) and points to the nearest match (U: the next one). Made \
            from a Mk1 scanner. Never wears out.",
};

pub const STEEL_PICKAXE_RECIPE: Recipe = Recipe {
    output: STEEL_PICKAXE,
    group: Group::Tools,
    count: STEEL_TIER.uses,
    inputs: &[(STEEL_PLATE, 3), (IRON_ROD, 2)],
    blurb: "Breaks stone and ore six times as fast and keeps 5 ore per block instead of 3. Lasts 1,500 blocks.",
};

pub const STEEL_AXE_RECIPE: Recipe = Recipe {
    output: STEEL_AXE,
    group: Group::Tools,
    count: STEEL_TIER.uses,
    inputs: &[(STEEL_PLATE, 3), (IRON_ROD, 2)],
    blurb: "Chops wood six times as fast. Lasts 1,500 blocks.",
};

pub const STEEL_SHOVEL_RECIPE: Recipe = Recipe {
    output: STEEL_SHOVEL,
    group: Group::Tools,
    count: STEEL_TIER.uses,
    inputs: &[(STEEL_PLATE, 1), (IRON_ROD, 2)],
    blurb: "Digs dirt, grass and sand six times as fast. Lasts 1,500 blocks.",
};

pub const JETPACK_RECIPE: Recipe = Recipe {
    output: JETPACK,
    group: Group::Tools,
    count: 1,
    inputs: &[(STEEL_PLATE, 4), (MOTOR, 2), (CIRCUIT, 2)],
    blurb:
        "Wear it in the Jetpack slot (open the inventory and Shift-click it) and hold jump in the air to climb on a \
            plume of fire. Burns one coal from your pack for every 10 seconds of thrust. Never wears out.",
};

pub const HOVER_PACK_RECIPE: Recipe = Recipe {
    output: HOVER_PACK,
    group: Group::Tools,
    count: 1,
    inputs: &[(ALUMINIUM_PLATE, 6), (MOTOR, 2), (BATTERY, 4), (PROCESSOR, 2)],
    blurb: "Keep it in your pack and hold jump in the air to hover: it holds your height (jump rises, crouch sinks) \
            and carries you at 8 blocks a second, 13 sprinting. It runs on charge, 90 seconds of hover at most, \
            filled twice as fast as it drains while you stand within 6 blocks of a power pole. Never wears out.",
};

pub const PERSONAL_DRONE_RECIPE: Recipe = Recipe {
    output: PERSONAL_DRONE,
    group: Group::Tools,
    count: 1,
    inputs: &[(DRONE, 1), (CIRCUIT, 2)],
    blurb: "Keep it in your pack. With an item in your hand, press Y: the drone flies to the nearest storage box \
            within 32 blocks that holds it and brings back a stack. One errand at a time. Never wears out.",
};

pub const PLANNER_RECIPE: Recipe = Recipe {
    output: PLANNER,
    group: Group::Tools,
    count: 1,
    inputs: &[(IRON_PLATE, 3), (COPPER_WIRE, 4), (b(GLASS), 2), (CIRCUIT, 1)],
    blurb: "Marks ground for drones to level. Hold it, right-click two corners (up to 64 blocks away), choose to dig, \
            fill or flatten to a height and see what it would move. A drone port within reach does the work: it \
            digs into the storage boxes beside the pad and fills from them. Right-click a marked site to remove it. \
            Never wears out.",
};
