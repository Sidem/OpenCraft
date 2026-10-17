//! The salvage row (R5, the ruins): a smelter melts scrap metal back into an iron ingot. Open from the start, since
//! the ruins' first scrap comes long before steel. Row 135, the last of `MACHINE_RECIPES`.

use super::machine::Category::Smelting;
use super::MachineRecipe;
use crate::item::{IRON_INGOT, SCRAP_METAL};

pub const SCRAP: &[MachineRecipe] =
    &[MachineRecipe { category: Smelting, inputs: &[(SCRAP_METAL, 3)], outputs: &[(IRON_INGOT, 1)], seconds: 2.0 }];
