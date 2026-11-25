//! Clay and kilns (MAT-CLAY, `block/clay.rs`): what furnaces fire from clay and kaolin. Rows from `CLAY_FIRST` (232),
//! the table after `glass::STAINED_GLASS_RAW`; locked by Clay & Kilns (bricks, terracotta) and Porcelain & Glazes.
//!
//! The real process, in play amounts: brick clay fires in any furnace; roof tiles want a kiln's steadier heat;
//! porcelain is kaolin fired hot with china stone, the feldspar-rich granite real potteries used (Cornish stone), so
//! granite stands in for feldspar until pegmatites (MAT-SALAR) bring it; a glaze takes its colour from a metal oxide,
//! copper for turquoise and iron for amber, which is what the red and green pigments are (rust and verdigris).
//! Every input differs between rows, so a kiln picks its recipe from what it is fed (`Pick::ByInput`).

use crate::block::*;
use crate::item::{PIGMENT_GREEN, PIGMENT_RED};

use super::Category::{Firing, Smelting};
use super::{b, MachineRecipe};

pub const CLAY_RECIPES: &[MachineRecipe] = &[
    MachineRecipe { category: Smelting, inputs: &[(b(CLAY), 1)], outputs: &[(b(CLAY_BRICKS), 1)], seconds: 3.0 },
    MachineRecipe { category: Firing, inputs: &[(b(CLAY), 2)], outputs: &[(b(TERRACOTTA), 2)], seconds: 4.0 },
    MachineRecipe {
        category: Firing,
        inputs: &[(b(KAOLIN), 2), (b(GRANITE), 1)],
        outputs: &[(b(PORCELAIN), 2)],
        seconds: 6.0,
    },
    MachineRecipe {
        category: Firing,
        inputs: &[(b(PORCELAIN), 4), (PIGMENT_GREEN, 1)],
        outputs: &[(b(TURQUOISE_TILES), 4)],
        seconds: 3.0,
    },
    MachineRecipe {
        category: Firing,
        inputs: &[(b(PORCELAIN), 4), (PIGMENT_RED, 1)],
        outputs: &[(b(AMBER_TILES), 4)],
        seconds: 3.0,
    },
];
