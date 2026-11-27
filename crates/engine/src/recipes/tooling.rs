//! Hand recipes of the hand-held tools and devices (`tools.rs`; the Tool Belt that mines is worn gear: `gear.rs`).
//! To add one: a `pub const` here, and its name in `RECIPES`.

use crate::block::*;
use crate::item::*;

use super::{b, Group, Recipe};

/// The first iron a smelter makes buys it: it breaks no block, so it never wears out and costs no stack of uses.
pub const BLADERANG_RECIPE: Recipe = Recipe {
    output: BLADERANG,
    group: Group::Tools,
    count: 1,
    inputs: &[(IRON_INGOT, 2), (STICK, 2)],
    blurb:
        "Right-click to throw it: it loops out 12 blocks and back, cutting the grass, flowers and other small plants \
            under it so they do not grow back, and brings back the loose items it passes. Trees are not touched. \
            Never wears out.",
};

/// Refining-era land-clearing torch (unlocked by Flamethrower): weed-torches ground plants and burns grown trees to charcoal.
pub const FLAMETHROWER_RECIPE: Recipe = Recipe {
    output: FLAMETHROWER,
    group: Group::Tools,
    count: 1,
    inputs: &[(STEEL_PLATE, 4), (b(PIPE), 2), (IRON_ROD, 2), (EMPTY_BARREL, 1)],
    blurb:
        "Hold right-click for a steady torch flame: clears grass and ground plants where you aim (without spreading \
            to neighbouring plants) and sets grown trees ablaze so they burn down to charcoal. Runs on naphtha or \
            diesel barrels from the refinery or cracker (returns empty barrels). Never touches placed blocks, machines \
            or houses. Never wears out.",
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
    blurb: "Keep it in your pack. Press Y to send it: it fetches from storage boxes and machine outputs within 64 \
            blocks, puts your backpack away into the boxes that hold the same things, and fetches the parts for a \
            craft, then crafts it. It tops up the stack in your hand as it runs low. Never wears out.",
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

pub const SITE_GRADER_RECIPE: Recipe = Recipe {
    output: SITE_GRADER_ITEM,
    group: Group::Production,
    count: 1,
    inputs: &[(STEEL_BEAM, 8), (STEEL_PLATE, 10), (MOTOR, 4), (GEAR, 8)],
    blurb: "Telescoping bucket-wheel site grader (3×3×2, 45 kW). Levels a 19×18 pad flush with the ground it sits on: \
            cuts hills and fells trees above grade, auto-fills hollows from its internal cut-and-fill hopper, routes \
            ores, logs and excess spoil out the rear port, and optionally paves the finished surface with solid \
            building blocks fed into its left port.",
};
