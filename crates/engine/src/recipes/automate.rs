//! Machine rows that automate what was once only crafted by hand (user request 2026-10-15): planks and sticks from the
//! constructor, and from the assembler poles, long-distance poles, solar panels, accumulators, lamps, boxes, underpasses
//! and rails. Each makes what its hand recipe makes for the same inputs (a test checks), at most three kinds of them (the
//! assembler's slots) and no more of one kind than a stack. The Mk2 and higher of these are the upgrader's (`refit.rs`).
//! Rows append after the orbit rows (indices in `rows.rs`); research unlocks each with its hand recipe's tech.

use crate::block::*;
use crate::item::*;

use super::{
    b,
    Category::{Assembly, Pressing},
    MachineRecipe,
};

pub const AUTOMATE: &[MachineRecipe] = &[
    // 78, 79: the constructor saws a log into planks and planks into sticks, so poles and boxes can be built from belts.
    MachineRecipe { category: Pressing, inputs: &[(b(LOG), 1)], outputs: &[(b(PLANKS), 4)], seconds: 1.5 },
    MachineRecipe { category: Pressing, inputs: &[(b(PLANKS), 2)], outputs: &[(STICK, 4)], seconds: 1.5 },
    // 80: pole, 81: long-distance pole, 82: solar panel, 83: accumulator, 84: lamp, 85: box, 86: underpass, 87: rail.
    MachineRecipe {
        category: Assembly,
        inputs: &[(STICK, 2), (COPPER_WIRE, 2)],
        outputs: &[(b(POLE), 2)],
        seconds: 3.0,
    },
    MachineRecipe {
        category: Assembly,
        inputs: &[(STEEL_BEAM, 12), (ALUMINIUM_PLATE, 16), (COPPER_WIRE, 60)],
        outputs: &[(b(LONG_POLE), 1)],
        seconds: 20.0,
    },
    MachineRecipe {
        category: Assembly,
        inputs: &[(SILICON, 6), (b(GLASS), 2), (COPPER_WIRE, 6)],
        outputs: &[(b(SOLAR_PANEL), 1)],
        seconds: 8.0,
    },
    MachineRecipe {
        category: Assembly,
        inputs: &[(STEEL_PLATE, 6), (CIRCUIT, 2), (COPPER_WIRE, 12)],
        outputs: &[(b(ACCUMULATOR), 1)],
        seconds: 10.0,
    },
    MachineRecipe {
        category: Assembly,
        inputs: &[(b(GLASS), 1), (IRON_PLATE, 1), (COPPER_WIRE, 2)],
        outputs: &[(b(LAMP), 2)],
        seconds: 3.0,
    },
    MachineRecipe {
        category: Assembly,
        inputs: &[(b(PLANKS), 8), (IRON_PLATE, 2)],
        outputs: &[(b(STORAGE), 1)],
        seconds: 4.0,
    },
    MachineRecipe { category: Assembly, inputs: &[(b(BELT), 4)], outputs: &[(b(UNDERPASS_IN), 2)], seconds: 4.0 },
    MachineRecipe {
        category: Assembly,
        inputs: &[(STEEL_BEAM, 1), (b(CONCRETE), 1)],
        outputs: &[(b(RAIL), 6)],
        seconds: 3.0,
    },
];
