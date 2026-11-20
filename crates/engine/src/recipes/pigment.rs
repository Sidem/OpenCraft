//! The five ground pigments (CHEM-DYE, first step): one mineral ground fine by hand, a pigment for a mineral. The
//! coloured concretes (`decor.rs`) and the roof tiles (`house.rs`) take a pigment where they once took the mineral, so
//! every colour in the game is dye-made. Ungated like the roof tiles they colour; what each needs is its own mineral
//! (quicklime comes with Masonry, sulfur from the refinery). No hand recipe takes raw ore (`recipes/tests.rs`), so red
//! and green grind from an iron and a copper ingot (rust and verdigris, the metals that really colour them). To add a
//! hue: an item, a layer, a row here.

use crate::block::*;
use crate::item::*;

use super::{b, Group, Recipe};

#[cfg(test)]
mod tests;

macro_rules! grind {
    ($output:expr, $mineral:expr, $blurb:expr) => {
        Recipe { output: $output, group: Group::Materials, count: 1, inputs: &[($mineral, 1)], blurb: $blurb }
    };
}

pub const RED_PIGMENT_RECIPE: Recipe = grind!(
    PIGMENT_RED,
    IRON_INGOT,
    "Iron rusted and ground to a red powder, the colour of clay roofs and red concrete."
);
pub const GREEN_PIGMENT_RECIPE: Recipe = grind!(
    PIGMENT_GREEN,
    COPPER_INGOT,
    "Copper weathered and ground to a green powder, as verdigris colours old roofs."
);
pub const BLACK_PIGMENT_RECIPE: Recipe =
    grind!(PIGMENT_BLACK, b(COAL_ORE), "Coal ground to a black powder: slate roofs and black concrete.");
pub const WHITE_PIGMENT_RECIPE: Recipe = grind!(
    PIGMENT_WHITE,
    QUICKLIME,
    "Quicklime ground to a white powder, for white concrete and for lightening a hue."
);
pub const YELLOW_PIGMENT_RECIPE: Recipe =
    grind!(PIGMENT_YELLOW, SULFUR, "Sulfur ground to a yellow powder: yellow concrete, and a hue to mix with others.");
