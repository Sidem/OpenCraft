//! The Pigments tech (CHEM-DYE, second step): the chemical plant makes blue from copper and acid and mixes ten more hues of
//! two pigments each; an assembler then tints concrete with every new pigment (`tint!`, as `decor.rs` does for the five
//! ground ones, `pigment.rs`). Rows 155 on (`DYE_FIRST`) of `MACHINE_RECIPES`, in `HUES` order: the plant's eleven rows,
//! then the assembler's eleven. Saved machines hold row indices, so a hue appends: one `HUES` row (the pigment item, the
//! concrete block, its colour), one plant row, one tint row at the end of each half, with the ids appended too
//! (`block/tints.rs`). Dyes colour objects only: LEDs make their own colour and stained glass is not made from them.

use crate::block::*;
use crate::item::*;

use super::decor::tint;
use super::Category::{Assembly, Chemistry};
use super::{b, MachineRecipe};

#[cfg(test)]
mod tests;

/// The mixed hues' colours (RGB), in id order: hue `i` is the pigment `PIGMENT_BLUE + i`, the concrete `BLUE_CONCRETE + i`, the
/// plant's row `DYE_FIRST + i` and the assembler's row `DYE_FIRST + HUES.len() + i` (`tests.rs` checks all four agree). The
/// concrete's face is painted from it (`textures/decor.rs`).
pub const HUES: [[f64; 3]; TINT_COUNT] = [
    [66.0, 104.0, 176.0],  // blue
    [208.0, 124.0, 52.0],  // orange
    [128.0, 78.0, 152.0],  // purple
    [64.0, 152.0, 160.0],  // cyan
    [150.0, 184.0, 64.0],  // lime
    [220.0, 150.0, 160.0], // pink
    [140.0, 176.0, 214.0], // sky
    [128.0, 130.0, 136.0], // grey
    [116.0, 82.0, 56.0],   // brown
    [100.0, 30.0, 46.0],   // maroon
    [40.0, 52.0, 104.0],   // navy
];

/// Two pigments stirred into twice as much of a new one.
macro_rules! mix {
    ($a:expr, $b:expr, $out:expr) => {
        MachineRecipe { category: Chemistry, inputs: &[($a, 1), ($b, 1)], outputs: &[($out, 2)], seconds: 2.0 }
    };
}

pub const DYE: &[MachineRecipe] = &[
    // Copper sulfate: a copper ingot dissolved in a barrel of acid (and a unit of water), the empty shell comes back.
    MachineRecipe {
        category: Chemistry,
        inputs: &[(COPPER_INGOT, 1), (ACID_BARREL, 1)],
        outputs: &[(PIGMENT_BLUE, 2), (EMPTY_BARREL, 1)],
        seconds: 3.0,
    },
    mix!(PIGMENT_RED, PIGMENT_YELLOW, PIGMENT_ORANGE),
    mix!(PIGMENT_RED, PIGMENT_BLUE, PIGMENT_PURPLE),
    mix!(PIGMENT_BLUE, PIGMENT_GREEN, PIGMENT_CYAN),
    mix!(PIGMENT_GREEN, PIGMENT_YELLOW, PIGMENT_LIME),
    mix!(PIGMENT_RED, PIGMENT_WHITE, PIGMENT_PINK),
    mix!(PIGMENT_BLUE, PIGMENT_WHITE, PIGMENT_SKY),
    mix!(PIGMENT_BLACK, PIGMENT_WHITE, PIGMENT_GREY),
    mix!(PIGMENT_RED, PIGMENT_GREEN, PIGMENT_BROWN),
    mix!(PIGMENT_RED, PIGMENT_BLACK, PIGMENT_MAROON),
    mix!(PIGMENT_BLUE, PIGMENT_BLACK, PIGMENT_NAVY),
    tint!(PIGMENT_BLUE, BLUE_CONCRETE),
    tint!(PIGMENT_ORANGE, BLUE_CONCRETE + 1),
    tint!(PIGMENT_PURPLE, BLUE_CONCRETE + 2),
    tint!(PIGMENT_CYAN, BLUE_CONCRETE + 3),
    tint!(PIGMENT_LIME, BLUE_CONCRETE + 4),
    tint!(PIGMENT_PINK, BLUE_CONCRETE + 5),
    tint!(PIGMENT_SKY, BLUE_CONCRETE + 6),
    tint!(PIGMENT_GREY, BLUE_CONCRETE + 7),
    tint!(PIGMENT_BROWN, BLUE_CONCRETE + 8),
    tint!(PIGMENT_MAROON, BLUE_CONCRETE + 9),
    tint!(PIGMENT_NAVY, BLUE_CONCRETE + 10),
];
