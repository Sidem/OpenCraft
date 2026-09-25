//! Hand-crafting recipes for the first factory buildings, as a data table. Inputs are raw resources
//! for now; ingots and parts arrive with smelting. To add a recipe: add a row to [`RECIPES`]; the
//! build menu lists every row in order.

use crate::block::*;
use crate::inventory::Inventory;

pub struct Recipe {
    pub output: BlockId,
    pub count: u32,
    pub inputs: &'static [(BlockId, u32)],
    /// One line for the build menu.
    pub blurb: &'static str,
}

impl Recipe {
    /// How many times `inv` can pay for this recipe (inputs are distinct items).
    pub fn affordable(&self, inv: &Inventory) -> u32 {
        self.inputs.iter().map(|&(item, n)| inv.count(item) / n).min().unwrap_or(0)
    }
}

pub const RECIPES: &[Recipe] = &[
    Recipe {
        output: MINER,
        count: 1,
        inputs: &[(IRON_ORE, 10), (COPPER_ORE, 6), (STONE, 12)],
        blurb: "Place it against an ore block. It drills the whole deposit, recovers 60% of what it draws, \
                and pushes ore into a belt or box beside it.",
    },
    Recipe {
        output: BELT,
        count: 4,
        inputs: &[(IRON_ORE, 1), (STONE, 2)],
        blurb: "Carries items the way you are facing when you place it. Belts feed into belts, boxes, and \
                other belts from the side.",
    },
    Recipe {
        output: STORAGE,
        count: 1,
        inputs: &[(LOG, 6), (IRON_ORE, 2)],
        blurb: "Holds 24 stacks. Belts deliver into it; a belt leading away from it is fed from it. \
                Right-click to empty it into your inventory.",
    },
];
