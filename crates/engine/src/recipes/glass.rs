//! Stained glass & Glass Kiln recipes (L-COLOUR, `block/glass.rs`, `block/kiln.rs`): sixteen hues melted in a
//! Glass Kiln (`Category::Glassmaking`) from silica (`GLASS`, `SAND`, or `QUARTZ_ORE`) plus one of the 16 pigments.
//!
//! - `STAINED_GLASS` (rows 177–192, `GLASS_FIRST`): `4 Glass + 1 Pigment → 4 Stained Glass` (re-melt, 2.5s).
//! - `STAINED_GLASS_RAW` (rows 200–231, `GLASS_RAW_FIRST`): 16 Sand batch rows (`4 Sand + 1 Pigment → 4 Stained Glass`,
//!   4.0s) followed by 16 Quartz crystal-melt rows (`2 Quartz Ore + 1 Pigment → 4 Stained Glass`, 3.0s).
//! - Primary 5 hues (Red, Green, Black, White, Yellow) unlock with Architecture (Tech 85) alongside `GLASS_KILN_ITEM`
//!   and `PAINT_SPRAYER`; mixed 11 hues (Blue..Navy) unlock with Pigments (Tech 89).

use crate::block::*;
use crate::item::*;

use super::Category::Glassmaking;
use super::{b, Group, MachineRecipe, Recipe};

#[cfg(test)]
mod tests;

pub const GLASS_KILN_RECIPE: Recipe = Recipe {
    output: GLASS_KILN_ITEM,
    group: Group::Production,
    count: 1,
    inputs: &[(b(STONE_BRICKS), 6), (IRON_PLATE, 4), (b(GLASS), 2)],
    blurb: "A brick furnace that melts glass, sand or quartz with any pigment into stained glass.",
};

pub const PAINT_SPRAYER_RECIPE: Recipe = Recipe {
    output: PAINT_SPRAYER,
    group: Group::Tools,
    count: 1,
    inputs: &[(IRON_PLATE, 2), (IRON_ROD, 2), (GEAR, 1)],
    blurb: "Spray-paints concrete, plaster, masonry, metal and roof surfaces using pigments (24 surfaces per pigment).",
};

macro_rules! melt {
    ($silica:expr, $qty:expr, $pigment:expr, $out:expr, $secs:expr) => {
        MachineRecipe {
            category: Glassmaking,
            inputs: &[(b($silica), $qty), ($pigment, 1)],
            outputs: &[(b($out), 4)],
            seconds: $secs,
        }
    };
}

/// Re-melting plain glass with any of the 16 pigments in a Glass Kiln (rows 177–192).
pub const STAINED_GLASS: &[MachineRecipe] = &[
    melt!(GLASS, 4, PIGMENT_RED, STAINED_GLASS_FIRST, 2.5),
    melt!(GLASS, 4, PIGMENT_GREEN, STAINED_GLASS_FIRST + 1, 2.5),
    melt!(GLASS, 4, PIGMENT_BLACK, STAINED_GLASS_FIRST + 2, 2.5),
    melt!(GLASS, 4, PIGMENT_WHITE, STAINED_GLASS_FIRST + 3, 2.5),
    melt!(GLASS, 4, PIGMENT_YELLOW, STAINED_GLASS_FIRST + 4, 2.5),
    melt!(GLASS, 4, PIGMENT_BLUE, STAINED_GLASS_FIRST + 5, 2.5),
    melt!(GLASS, 4, PIGMENT_ORANGE, STAINED_GLASS_FIRST + 6, 2.5),
    melt!(GLASS, 4, PIGMENT_PURPLE, STAINED_GLASS_FIRST + 7, 2.5),
    melt!(GLASS, 4, PIGMENT_CYAN, STAINED_GLASS_FIRST + 8, 2.5),
    melt!(GLASS, 4, PIGMENT_LIME, STAINED_GLASS_FIRST + 9, 2.5),
    melt!(GLASS, 4, PIGMENT_PINK, STAINED_GLASS_FIRST + 10, 2.5),
    melt!(GLASS, 4, PIGMENT_SKY, STAINED_GLASS_FIRST + 11, 2.5),
    melt!(GLASS, 4, PIGMENT_GREY, STAINED_GLASS_FIRST + 12, 2.5),
    melt!(GLASS, 4, PIGMENT_BROWN, STAINED_GLASS_FIRST + 13, 2.5),
    melt!(GLASS, 4, PIGMENT_MAROON, STAINED_GLASS_FIRST + 14, 2.5),
    melt!(GLASS, 4, PIGMENT_NAVY, STAINED_GLASS_FIRST + 15, 2.5),
];

/// Direct batch-melting from raw Sand (first 16 rows) or high-purity Quartz Ore (next 16 rows) with any pigment.
pub const STAINED_GLASS_RAW: &[MachineRecipe] = &[
    // Sand + Pigment -> 4 Stained Glass (4.0s)
    melt!(SAND, 4, PIGMENT_RED, STAINED_GLASS_FIRST, 4.0),
    melt!(SAND, 4, PIGMENT_GREEN, STAINED_GLASS_FIRST + 1, 4.0),
    melt!(SAND, 4, PIGMENT_BLACK, STAINED_GLASS_FIRST + 2, 4.0),
    melt!(SAND, 4, PIGMENT_WHITE, STAINED_GLASS_FIRST + 3, 4.0),
    melt!(SAND, 4, PIGMENT_YELLOW, STAINED_GLASS_FIRST + 4, 4.0),
    melt!(SAND, 4, PIGMENT_BLUE, STAINED_GLASS_FIRST + 5, 4.0),
    melt!(SAND, 4, PIGMENT_ORANGE, STAINED_GLASS_FIRST + 6, 4.0),
    melt!(SAND, 4, PIGMENT_PURPLE, STAINED_GLASS_FIRST + 7, 4.0),
    melt!(SAND, 4, PIGMENT_CYAN, STAINED_GLASS_FIRST + 8, 4.0),
    melt!(SAND, 4, PIGMENT_LIME, STAINED_GLASS_FIRST + 9, 4.0),
    melt!(SAND, 4, PIGMENT_PINK, STAINED_GLASS_FIRST + 10, 4.0),
    melt!(SAND, 4, PIGMENT_SKY, STAINED_GLASS_FIRST + 11, 4.0),
    melt!(SAND, 4, PIGMENT_GREY, STAINED_GLASS_FIRST + 12, 4.0),
    melt!(SAND, 4, PIGMENT_BROWN, STAINED_GLASS_FIRST + 13, 4.0),
    melt!(SAND, 4, PIGMENT_MAROON, STAINED_GLASS_FIRST + 14, 4.0),
    melt!(SAND, 4, PIGMENT_NAVY, STAINED_GLASS_FIRST + 15, 4.0),
    // Quartz Ore + Pigment -> 4 Stained Glass (3.0s, half the silica volume)
    melt!(QUARTZ_ORE, 2, PIGMENT_RED, STAINED_GLASS_FIRST, 3.0),
    melt!(QUARTZ_ORE, 2, PIGMENT_GREEN, STAINED_GLASS_FIRST + 1, 3.0),
    melt!(QUARTZ_ORE, 2, PIGMENT_BLACK, STAINED_GLASS_FIRST + 2, 3.0),
    melt!(QUARTZ_ORE, 2, PIGMENT_WHITE, STAINED_GLASS_FIRST + 3, 3.0),
    melt!(QUARTZ_ORE, 2, PIGMENT_YELLOW, STAINED_GLASS_FIRST + 4, 3.0),
    melt!(QUARTZ_ORE, 2, PIGMENT_BLUE, STAINED_GLASS_FIRST + 5, 3.0),
    melt!(QUARTZ_ORE, 2, PIGMENT_ORANGE, STAINED_GLASS_FIRST + 6, 3.0),
    melt!(QUARTZ_ORE, 2, PIGMENT_PURPLE, STAINED_GLASS_FIRST + 7, 3.0),
    melt!(QUARTZ_ORE, 2, PIGMENT_CYAN, STAINED_GLASS_FIRST + 8, 3.0),
    melt!(QUARTZ_ORE, 2, PIGMENT_LIME, STAINED_GLASS_FIRST + 9, 3.0),
    melt!(QUARTZ_ORE, 2, PIGMENT_PINK, STAINED_GLASS_FIRST + 10, 3.0),
    melt!(QUARTZ_ORE, 2, PIGMENT_SKY, STAINED_GLASS_FIRST + 11, 3.0),
    melt!(QUARTZ_ORE, 2, PIGMENT_GREY, STAINED_GLASS_FIRST + 12, 3.0),
    melt!(QUARTZ_ORE, 2, PIGMENT_BROWN, STAINED_GLASS_FIRST + 13, 3.0),
    melt!(QUARTZ_ORE, 2, PIGMENT_MAROON, STAINED_GLASS_FIRST + 14, 3.0),
    melt!(QUARTZ_ORE, 2, PIGMENT_NAVY, STAINED_GLASS_FIRST + 15, 3.0),
];
