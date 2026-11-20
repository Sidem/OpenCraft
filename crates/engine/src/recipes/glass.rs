//! Stained glass machine rows (`block/glass.rs`, GLASS): sixteen hues made in an assembler from plain glass and mineral or
//! metal colorants (never from dyes or pigments, per `docs/DECISIONS.md` 2026-11-18). Unlocked by Architecture alongside
//! framed glass and the first five concretes. Appended after `dye::DYE` at `STAINED_GLASS_FIRST` (rows 177–192) of
//! `MACHINE_RECIPES`. To add a hue: append a block id in `block/glass.rs` and a row at the end of `STAINED_GLASS`.

use crate::block::*;
use crate::item::{COPPER_INGOT, COPPER_WIRE, IRON_INGOT, QUICKLIME};

use super::Category::Assembly;
use super::{b, MachineRecipe};

#[cfg(test)]
mod tests;

/// Four glass plus one mineral or metal colorant make four stained glass blocks.
macro_rules! stain1 {
    ($a:expr, $out:expr) => {
        MachineRecipe { category: Assembly, inputs: &[(b(GLASS), 4), ($a, 1)], outputs: &[(b($out), 4)], seconds: 3.0 }
    };
}

/// Four glass plus two mineral or metal colorants make four blended stained glass blocks.
macro_rules! stain2 {
    ($a:expr, $b:expr, $out:expr) => {
        MachineRecipe {
            category: Assembly,
            inputs: &[(b(GLASS), 4), ($a, 1), ($b, 1)],
            outputs: &[(b($out), 4)],
            seconds: 3.0,
        }
    };
}

pub const STAINED_GLASS: &[MachineRecipe] = &[
    stain1!(IRON_INGOT, STAINED_GLASS_FIRST),                     // Red
    stain1!(COPPER_INGOT, STAINED_GLASS_FIRST + 1),               // Green
    stain1!(b(COAL_ORE), STAINED_GLASS_FIRST + 2),                // Black
    stain1!(QUICKLIME, STAINED_GLASS_FIRST + 3),                  // White
    stain1!(b(SANDSTONE), STAINED_GLASS_FIRST + 4),               // Yellow
    stain1!(COPPER_WIRE, STAINED_GLASS_FIRST + 5),                // Blue
    stain2!(IRON_INGOT, b(SANDSTONE), STAINED_GLASS_FIRST + 6),   // Orange
    stain2!(IRON_INGOT, COPPER_WIRE, STAINED_GLASS_FIRST + 7),    // Purple
    stain2!(COPPER_INGOT, COPPER_WIRE, STAINED_GLASS_FIRST + 8),  // Cyan
    stain2!(COPPER_INGOT, b(SANDSTONE), STAINED_GLASS_FIRST + 9), // Lime
    stain2!(IRON_INGOT, QUICKLIME, STAINED_GLASS_FIRST + 10),     // Pink
    stain2!(COPPER_WIRE, QUICKLIME, STAINED_GLASS_FIRST + 11),    // Sky
    stain2!(b(COAL_ORE), QUICKLIME, STAINED_GLASS_FIRST + 12),    // Grey
    stain2!(IRON_INGOT, COPPER_INGOT, STAINED_GLASS_FIRST + 13),  // Brown
    stain2!(IRON_INGOT, b(COAL_ORE), STAINED_GLASS_FIRST + 14),   // Maroon
    stain2!(COPPER_WIRE, b(COAL_ORE), STAINED_GLASS_FIRST + 15),  // Navy
];
