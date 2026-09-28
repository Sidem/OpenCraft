//! Processing machines as data: one [`ProcessSpec`] row per block that turns inputs into outputs
//! (docs/TECH_TREE.md section 8): the recipe categories it takes, how it is driven, how it picks a
//! recipe, its buffers, its numbers per tier and its model parts.
//!
//! Invariants: a spec has a tier row for every tier of its family (`tiers.rs`; tested); an electric
//! spec has no fuel buffer; its block is a `Kind::Process` row in `MACHINES`.
//!
//! To add a processor: its block (`block/`), a `MACHINES` row of `Kind::Process`, a row here with its
//! parts, and its recipes' category (`recipes/machine.rs`).

use crate::block::{tex, BlockId, CONSTRUCTOR, SMELTER};
use crate::item::ItemId;
use crate::recipes::{Category, MachineRecipe, MACHINE_RECIPES};

use super::model::{part, Look, Part};

/// How a processor is driven.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Energy {
    /// Burns fuel (`recipes::FUELS`) from its fuel buffer while a batch runs.
    Burner,
    /// Draws `ProcessTier::power` from a power grid (`power.rs`) while it works.
    Electric,
}

/// How a processor picks its recipe.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Pick {
    /// The player chooses in its panel (`SetRecipe`); it takes only that recipe's inputs.
    Chosen,
    /// What it holds decides: it takes anything an unlocked recipe of its categories uses.
    ByInput,
}

/// One tier's numbers.
pub struct ProcessTier {
    /// Work per tick at full power, in thousandths of a Mk1 tick.
    pub speed: u32,
    /// Fuel per unit of work, in thousandths (burners: 750 burns a quarter less a batch).
    pub fuel: u32,
    /// kW drawn while working (electric).
    pub power: u32,
}

pub struct ProcessSpec {
    pub block: BlockId,
    pub categories: &'static [Category],
    pub energy: Energy,
    pub pick: Pick,
    /// Slots of the input, fuel and output buffers.
    pub buffers: [usize; 3],
    /// Mk1 first.
    pub tiers: &'static [ProcessTier],
    /// Status words: what it does ("Smelting"), what it makes ("ingots"), its line while it holds
    /// nothing to work on (`Pick::ByInput`).
    pub verb: &'static str,
    pub products: &'static str,
    pub waiting: &'static str,
    /// Its mark on the maps, 0xRRGGBB.
    pub map_colour: i32,
    pub parts: &'static [Part],
}

pub const SPECS: &[ProcessSpec] = &[
    ProcessSpec {
        block: SMELTER,
        categories: &[Category::Smelting],
        energy: Energy::Burner,
        pick: Pick::ByInput,
        buffers: [1, 1, 1],
        tiers: &[ProcessTier { speed: 1000, fuel: 1000, power: 0 }, ProcessTier { speed: 2000, fuel: 750, power: 0 }],
        verb: "Smelting",
        products: "ingots",
        waiting: "Waiting for ore, sand or stone",
        map_colour: 0xe5533d,
        parts: &SMELTER_PARTS,
    },
    ProcessSpec {
        block: CONSTRUCTOR,
        categories: &[Category::Pressing],
        energy: Energy::Electric,
        pick: Pick::Chosen,
        buffers: [1, 0, 1],
        tiers: &[ProcessTier { speed: 1000, fuel: 0, power: 15 }, ProcessTier { speed: 2000, fuel: 0, power: 30 }],
        verb: "Making",
        products: "parts",
        waiting: "",
        map_colour: 0x4a90e2,
        parts: &CONSTRUCTOR_PARTS,
    },
];

/// The spec of the processor `block` is, if it is one.
pub fn spec(block: BlockId) -> Option<&'static ProcessSpec> {
    SPECS.iter().find(|s| s.block == block)
}

/// Whether the machine `block` makes recipes of `category`.
pub fn makes(block: BlockId, category: Category) -> bool {
    spec(block).is_some_and(|s| s.categories.contains(&category))
}

impl ProcessSpec {
    /// Recipe `i` if this machine makes it.
    pub fn recipe(&self, i: u16) -> Option<&'static MachineRecipe> {
        MACHINE_RECIPES.get(i as usize).filter(|r| self.categories.contains(&r.category))
    }

    /// The first recipe it makes that uses `item` and `unlocked` allows (by recipe index).
    pub fn recipe_using(&self, item: ItemId, unlocked: &[bool]) -> Option<u16> {
        let fits = |(i, r): &(usize, &MachineRecipe)| {
            self.categories.contains(&r.category)
                && unlocked.get(*i) == Some(&true)
                && r.inputs.iter().any(|x| x.0 == item)
        };
        MACHINE_RECIPES.iter().enumerate().find(fits).map(|(i, _)| i as u16)
    }
}

const BRICK: [u16; 3] = [tex::SMELTER_TOP, tex::SMELTER_SIDE, tex::FRAME];
const IVORY: [u16; 3] = [tex::CONSTRUCTOR_TOP, tex::CONSTRUCTOR_SIDE, tex::FRAME];
const FRAME: Look = Look::Tex([tex::FRAME; 3]);
const SOOT: Look = Look::Tex([tex::GENERATOR_SIDE; 3]);

/// A brick furnace: a dark upper ore mouth over a glowing crucible, a tall chimney, a status lamp.
const SMELTER_PARTS: [Part; 9] = [
    part([0.0, -0.42, 0.0], [0.94, 0.16, 0.94], Look::Band(tex::FRAME)),
    part([0.0, -0.06, -0.1], [0.82, 0.67, 0.72], Look::Tex(BRICK)),
    part([0.0, 0.31, -0.1], [0.9, 0.1, 0.8], FRAME),
    part([0.0, 0.14, 0.32], [0.46, 0.23, 0.13], SOOT),
    part([0.0, -0.24, 0.34], [0.55, 0.2, 0.15], Look::Fire(tex::GENERATOR_SIDE)),
    part([0.0, -0.36, 0.46], [0.65, 0.07, 0.08], FRAME),
    part([-0.23, 0.48, -0.23], [0.27, 0.48, 0.27], Look::Tex(BRICK)),
    part([-0.23, 0.75, -0.23], [0.33, 0.07, 0.33], SOOT),
    part([0.33, 0.34, 0.32], [0.12, 0.08, 0.12], Look::Lamp),
];

/// An open press: tray, rear gantry, a ram and head that pump while working, a status lamp.
const CONSTRUCTOR_PARTS: [Part; 9] = [
    part([0.0, -0.4, 0.0], [0.94, 0.18, 0.94], Look::Band(tex::CONSTRUCTOR_TOP)),
    part([0.0, -0.25, 0.14], [0.67, 0.08, 0.57], FRAME),
    part([-0.35, 0.0, -0.32], [0.13, 0.67, 0.16], Look::Tex(IVORY)),
    part([0.35, 0.0, -0.32], [0.13, 0.67, 0.16], Look::Tex(IVORY)),
    part([0.0, 0.35, -0.32], [0.85, 0.13, 0.22], Look::Tex(IVORY)),
    part([0.0, 0.23, -0.1], [0.18, 0.34, 0.18], Look::Press(0.5, [tex::DRILL; 3])),
    part([0.0, 0.04, -0.1], [0.48, 0.08, 0.48], Look::Press(1.0, [tex::IRON_PLATE; 3])),
    part([0.0, -0.36, 0.49], [0.64, 0.09, 0.06], Look::Tex([tex::COPPER_INGOT; 3])),
    part([0.39, 0.37, -0.31], [0.12, 0.08, 0.12], Look::Lamp),
];
