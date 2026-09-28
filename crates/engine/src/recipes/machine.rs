//! What machines make ([`MACHINE_RECIPES`]), grouped into categories that processors take whole
//! (their spec rows: `factory/process/specs.rs`), and what burns as fuel ([`FUELS`]). A better machine
//! of a category reuses the same recipes (docs/TECH_TREE.md section 8).
//!
//! Invariants: machines save the index of their recipe, so rows are appended, never reordered. A
//! recipe's first output is its main product (readouts name it); the rest are byproducts, which a
//! machine with a byproduct port keeps apart, and all must fit its buffers (`recipes/tests.rs` checks
//! this and more).
//!
//! To add a recipe: append a row. A category: a `Category` variant (and its entry in the lint's list) and
//! the processor specs that take it.

use crate::block::*;
use crate::item::{
    ItemId, COPPER_INGOT, COPPER_WIRE, GEAR, GREEN_KIT, GREEN_PACK, IRON_INGOT, IRON_PLATE, IRON_ROD, MOTOR, QUICKLIME,
    RED_PACK, SCREW, STEEL_BEAM, STEEL_INGOT, STEEL_PLATE,
};

/// A kind of machine work.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Category {
    /// Ore, sand and stone into ingots, glass, bricks and quicklime, with fuel.
    Smelting,
    /// Ingots into plates, rods, screws and wire.
    Pressing,
    /// Several parts into one: motors, concrete, packs and kits (the assembler).
    Assembly,
    /// Ore, coal and flux into steel and slag (the blast furnace).
    Blasting,
}

/// Something a machine makes: `inputs` are used up when a batch starts, `outputs` appear after
/// `seconds` of work.
pub struct MachineRecipe {
    pub category: Category,
    pub inputs: &'static [(ItemId, u32)],
    /// The main product first, then any byproducts.
    pub outputs: &'static [(ItemId, u32)],
    pub seconds: f64,
}

use Category::{Assembly, Blasting, Pressing, Smelting};

pub const MACHINE_RECIPES: &[MachineRecipe] = &[
    MachineRecipe { category: Smelting, inputs: &[(b(IRON_ORE), 1)], outputs: &[(IRON_INGOT, 1)], seconds: 1.5 },
    MachineRecipe { category: Smelting, inputs: &[(b(COPPER_ORE), 1)], outputs: &[(COPPER_INGOT, 1)], seconds: 1.5 },
    MachineRecipe { category: Pressing, inputs: &[(IRON_INGOT, 2)], outputs: &[(IRON_PLATE, 1)], seconds: 2.0 },
    MachineRecipe { category: Pressing, inputs: &[(IRON_INGOT, 1)], outputs: &[(IRON_ROD, 1)], seconds: 2.0 },
    MachineRecipe { category: Pressing, inputs: &[(IRON_ROD, 1)], outputs: &[(SCREW, 4)], seconds: 3.0 },
    MachineRecipe { category: Pressing, inputs: &[(COPPER_INGOT, 1)], outputs: &[(COPPER_WIRE, 2)], seconds: 2.0 },
    MachineRecipe { category: Smelting, inputs: &[(b(QUARTZ_ORE), 1)], outputs: &[(b(GLASS), 2)], seconds: 2.0 },
    MachineRecipe { category: Smelting, inputs: &[(b(SAND), 1)], outputs: &[(b(GLASS), 1)], seconds: 2.0 },
    MachineRecipe { category: Pressing, inputs: &[(IRON_PLATE, 1)], outputs: &[(GEAR, 1)], seconds: 2.0 },
    MachineRecipe { category: Smelting, inputs: &[(b(STONE), 2)], outputs: &[(b(STONE_BRICKS), 1)], seconds: 3.0 },
    MachineRecipe { category: Smelting, inputs: &[(b(LIMESTONE), 1)], outputs: &[(QUICKLIME, 1)], seconds: 2.0 },
    MachineRecipe {
        category: Assembly,
        inputs: &[(IRON_ROD, 1), (GEAR, 2), (COPPER_WIRE, 4)],
        outputs: &[(MOTOR, 1)],
        seconds: 5.0,
    },
    MachineRecipe {
        category: Assembly,
        inputs: &[(QUICKLIME, 1), (b(SAND), 2), (b(STONE), 2)],
        outputs: &[(b(CONCRETE), 4)],
        seconds: 4.0,
    },
    MachineRecipe {
        category: Assembly,
        inputs: &[(IRON_PLATE, 1), (COPPER_WIRE, 2)],
        outputs: &[(RED_PACK, 1)],
        seconds: 5.0,
    },
    MachineRecipe {
        category: Assembly,
        inputs: &[(b(BELT), 2), (SCREW, 4)],
        outputs: &[(GREEN_PACK, 1)],
        seconds: 6.0,
    },
    MachineRecipe {
        category: Assembly,
        inputs: &[(GEAR, 2), (SCREW, 4), (COPPER_WIRE, 2)],
        outputs: &[(GREEN_KIT, 4)],
        seconds: 5.0,
    },
    MachineRecipe {
        category: Blasting,
        inputs: &[(b(IRON_ORE), 2), (b(COAL_ORE), 1), (QUICKLIME, 1)],
        outputs: &[(STEEL_INGOT, 1), (b(SLAG), 1)],
        seconds: 4.0,
    },
    MachineRecipe { category: Pressing, inputs: &[(STEEL_INGOT, 1)], outputs: &[(STEEL_PLATE, 1)], seconds: 3.0 },
    MachineRecipe { category: Pressing, inputs: &[(STEEL_INGOT, 2)], outputs: &[(STEEL_BEAM, 1)], seconds: 4.0 },
];

/// Rows research locks: the gear (Mechanics), bricks and quicklime (Masonry), the assembler's.
pub const GEAR_RECIPE: u16 = 8;
pub const BRICK_RECIPE: u16 = 9;
pub const QUICKLIME_RECIPE: u16 = 10;
/// The assembler's rows (Assembly): motor, concrete, red and green packs, green kits.
pub const ASSEMBLY_RECIPES: [u16; 5] = [11, 12, 13, 14, 15];
/// The blast furnace's row and the constructor's steel plate and beam (Steelmaking).
pub const STEEL_RECIPES: [u16; 3] = [16, 17, 18];

impl MachineRecipe {
    /// The main product and how many a batch makes.
    pub fn main(&self) -> (ItemId, u32) {
        self.outputs[0]
    }
}

/// Fuel: the seconds of smelting one item keeps a fire going, and the energy it gives a generator in
/// kJ (a coal runs a Mk1 miner long enough to mine about 32 coal).
pub const FUELS: &[(ItemId, f64, u32)] = &[(b(COAL_ORE), 8.0, 270), (b(LOG), 4.0, 135), (b(PLANKS), 1.0, 34)];

/// Seconds of work one `item` fuels, if it burns.
pub fn burn_time(item: ItemId) -> Option<f64> {
    FUELS.iter().find(|f| f.0 == item).map(|f| f.1)
}

/// The energy one `item` gives a generator, in kJ, if it burns.
pub fn fuel_energy(item: ItemId) -> Option<u32> {
    FUELS.iter().find(|f| f.0 == item).map(|f| f.2)
}

const fn b(id: BlockId) -> ItemId {
    ItemId::block(id)
}
