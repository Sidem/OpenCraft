//! Ore blocks: which ones there are and the short name deposits go by (Deposit::name). To add an ore: its block id
//! and definition in mod.rs (an ore(..) row), then a line in each function here.

use super::*;

#[inline]
pub fn is_ore(id: BlockId) -> bool {
    matches!(id, COAL_ORE | IRON_ORE | COPPER_ORE | LIMESTONE | QUARTZ_ORE | BAUXITE_ORE | OIL_SAND | URANIUM_ORE)
}

/// Short resource name used in deposit names ("Iron vein").
pub fn ore_label(id: BlockId) -> &'static str {
    match id {
        COAL_ORE => "Coal",
        IRON_ORE => "Iron",
        COPPER_ORE => "Copper",
        LIMESTONE => "Limestone",
        QUARTZ_ORE => "Quartz",
        BAUXITE_ORE => "Bauxite",
        OIL_SAND => "Oil sand",
        URANIUM_ORE => "Uranium",
        _ => "Ore",
    }
}
