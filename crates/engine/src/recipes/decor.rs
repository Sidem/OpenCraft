//! The decor set's machine rows (R7, `block/decor.rs`), made by machines only and unlocked together by the Architecture
//! tech. Coloured concretes are plain concrete tinted by a ground pigment (`pigment.rs`: red from iron ore, green from
//! copper ore, black from coal, white from quicklime, yellow from sulfur; one pigment colours four blocks); rock bricks are fired in a smelter from the
//! rock of each province, slag bricks use the blast furnace's slag; the rest come from an assembler. Rows 136 on
//! (`DECOR_FIRST`) of `MACHINE_RECIPES`, followed only by `refit::REFIT_LATER`.

use crate::block::*;
use crate::item::*;

use super::{
    b,
    Category::{Assembly, Smelting},
    MachineRecipe,
};

#[cfg(test)]
mod tests;

/// Four concrete and one pinch of pigment make four coloured blocks.
macro_rules! tint {
    ($pigment:expr, $out:expr) => {
        MachineRecipe {
            category: Assembly,
            inputs: &[(b(CONCRETE), 4), ($pigment, 1)],
            outputs: &[(b($out), 4)],
            seconds: 4.0,
        }
    };
}

pub(super) use tint;

/// One block of rock fired into one block of bricks.
macro_rules! fire {
    ($rock:expr, $out:expr) => {
        MachineRecipe { category: Smelting, inputs: &[(b($rock), 1)], outputs: &[(b($out), 1)], seconds: 2.0 }
    };
}

pub const DECOR: &[MachineRecipe] = &[
    tint!(PIGMENT_RED, RED_CONCRETE),
    tint!(PIGMENT_GREEN, GREEN_CONCRETE),
    tint!(PIGMENT_BLACK, BLACK_CONCRETE),
    tint!(PIGMENT_WHITE, WHITE_CONCRETE),
    tint!(PIGMENT_YELLOW, YELLOW_CONCRETE),
    fire!(BASALT, BASALT_TILES),
    fire!(GRANITE, GRANITE_BRICKS),
    fire!(SANDSTONE, SANDSTONE_BRICKS),
    fire!(SLAG, SLAG_BRICKS),
    MachineRecipe {
        category: Assembly,
        inputs: &[(b(GLASS), 4), (IRON_ROD, 2)],
        outputs: &[(b(FRAMED_GLASS), 4)],
        seconds: 3.0,
    },
    MachineRecipe { category: Assembly, inputs: &[(IRON_PLATE, 4)], outputs: &[(b(METAL_PANEL), 4)], seconds: 2.0 },
    MachineRecipe {
        category: Assembly,
        inputs: &[(IRON_ROD, 4), (IRON_PLATE, 1)],
        outputs: &[(b(CATWALK), 4)],
        seconds: 3.0,
    },
    MachineRecipe { category: Assembly, inputs: &[(IRON_ROD, 4)], outputs: &[(b(RAILING), 4)], seconds: 2.0 },
    MachineRecipe {
        category: Assembly,
        inputs: &[(b(GLASS), 1), (IRON_PLATE, 1), (COPPER_WIRE, 1)],
        outputs: &[(b(PANEL_LIGHT), 2)],
        seconds: 3.0,
    },
];
