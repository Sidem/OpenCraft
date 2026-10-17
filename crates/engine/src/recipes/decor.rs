//! The decor set's machine rows (R7, `block/decor.rs`), made by machines only and unlocked together by the Architecture
//! tech. Coloured concretes are plain concrete tinted by a mineral already mined (iron ore red, copper ore green, coal
//! black, quicklime white, sulfur yellow; one mineral colours four blocks); rock bricks are fired in a smelter from the
//! rock of each province, slag bricks use the blast furnace's slag; the rest come from an assembler. Rows 136 on
//! (`DECOR_FIRST`), the last table of `MACHINE_RECIPES`.

use crate::block::*;
use crate::item::*;

use super::{
    b,
    Category::{Assembly, Smelting},
    MachineRecipe,
};

#[cfg(test)]
mod tests;

/// Four concrete and one pinch of a mineral make four coloured blocks.
macro_rules! tint {
    ($mineral:expr, $out:expr) => {
        MachineRecipe {
            category: Assembly,
            inputs: &[(b(CONCRETE), 4), ($mineral, 1)],
            outputs: &[(b($out), 4)],
            seconds: 4.0,
        }
    };
}

/// One block of rock fired into one block of bricks.
macro_rules! fire {
    ($rock:expr, $out:expr) => {
        MachineRecipe { category: Smelting, inputs: &[(b($rock), 1)], outputs: &[(b($out), 1)], seconds: 2.0 }
    };
}

pub const DECOR: &[MachineRecipe] = &[
    tint!(b(IRON_ORE), RED_CONCRETE),
    tint!(b(COPPER_ORE), GREEN_CONCRETE),
    tint!(b(COAL_ORE), BLACK_CONCRETE),
    tint!(QUICKLIME, WHITE_CONCRETE),
    tint!(SULFUR, YELLOW_CONCRETE),
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
