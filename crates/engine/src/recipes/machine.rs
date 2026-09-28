//! What machines make ([`MACHINE_RECIPES`]), grouped into categories that machines take whole
//! ([`MACHINE_CATEGORIES`]), and what burns as fuel ([`FUELS`]). A better machine of a category reuses
//! the same recipes (docs/TECH_TREE.md section 8).
//!
//! Invariants: machines save the index of their recipe, so rows are appended, never reordered. A
//! recipe's first output is its main product (readouts name it); the rest are byproducts, and all
//! must fit the machine's output buffer (`recipes/tests.rs` checks this and more).
//!
//! To add a recipe: append a row. A category: a `Category` variant (and its entry in the lint's list) and
//! the machines that take it in `MACHINE_CATEGORIES`.

use crate::block::*;
use crate::item::{ItemId, COPPER_INGOT, COPPER_WIRE, IRON_INGOT, IRON_PLATE, IRON_ROD, SCREW};

/// A kind of machine work.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Category {
    /// Ore and sand into ingots and glass, with fuel.
    Smelting,
    /// Ingots into plates, rods, screws and wire.
    Pressing,
}

/// The categories each machine block makes.
pub const MACHINE_CATEGORIES: &[(BlockId, &[Category])] =
    &[(SMELTER, &[Category::Smelting]), (CONSTRUCTOR, &[Category::Pressing])];

/// Something a machine makes: `inputs` are used up when a batch starts, `outputs` appear after
/// `seconds` of work.
pub struct MachineRecipe {
    pub category: Category,
    pub inputs: &'static [(ItemId, u32)],
    /// The main product first, then any byproducts.
    pub outputs: &'static [(ItemId, u32)],
    pub seconds: f64,
}

use Category::{Pressing, Smelting};

pub const MACHINE_RECIPES: &[MachineRecipe] = &[
    MachineRecipe { category: Smelting, inputs: &[(b(IRON_ORE), 1)], outputs: &[(IRON_INGOT, 1)], seconds: 1.5 },
    MachineRecipe { category: Smelting, inputs: &[(b(COPPER_ORE), 1)], outputs: &[(COPPER_INGOT, 1)], seconds: 1.5 },
    MachineRecipe { category: Pressing, inputs: &[(IRON_INGOT, 2)], outputs: &[(IRON_PLATE, 1)], seconds: 2.0 },
    MachineRecipe { category: Pressing, inputs: &[(IRON_INGOT, 1)], outputs: &[(IRON_ROD, 1)], seconds: 2.0 },
    MachineRecipe { category: Pressing, inputs: &[(IRON_ROD, 1)], outputs: &[(SCREW, 4)], seconds: 3.0 },
    MachineRecipe { category: Pressing, inputs: &[(COPPER_INGOT, 1)], outputs: &[(COPPER_WIRE, 2)], seconds: 2.0 },
    MachineRecipe { category: Smelting, inputs: &[(b(QUARTZ_ORE), 1)], outputs: &[(b(GLASS), 2)], seconds: 2.0 },
    MachineRecipe { category: Smelting, inputs: &[(b(SAND), 1)], outputs: &[(b(GLASS), 1)], seconds: 2.0 },
];

impl MachineRecipe {
    /// The main product and how many a batch makes.
    pub fn main(&self) -> (ItemId, u32) {
        self.outputs[0]
    }
}

/// Whether the machine `machine` makes recipes of `category`.
pub fn makes(machine: BlockId, category: Category) -> bool {
    MACHINE_CATEGORIES.iter().any(|&(m, cats)| m == machine && cats.contains(&category))
}

/// The recipe `i` if `machine` makes it.
pub fn machine_recipe(machine: BlockId, i: u16) -> Option<&'static MachineRecipe> {
    MACHINE_RECIPES.get(i as usize).filter(|r| makes(machine, r.category))
}

/// Index of `machine`'s recipe that uses `item`, if any.
pub fn machine_recipe_using(machine: BlockId, item: ItemId) -> Option<u16> {
    let using = |r: &MachineRecipe| makes(machine, r.category) && r.inputs.iter().any(|i| i.0 == item);
    MACHINE_RECIPES.iter().position(using).map(|i| i as u16)
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
