//! Tiered families (Mk1, Mk2, …; boxes, pumps and quarries stop at Mk3, generators at Mk2): which machines come in tiers and which item is each tier.
//!
//! A tier is machine state: `tier: u8`, an index (0 is Mk1) into the family's numbers, which live atop
//! the machine's own module as one array (`BELT_TIERS`, `MINER_TIERS`, `LAB_TIERS`, `POLE_TIERS`,
//! `BOX_SLOTS`, `PUMP_TIERS`, `QUARRY_TIERS`, `GENERATOR_TIERS`, a processor spec's `tiers`). Splitters and
//! filters have none: they pass an item a tick, faster than any belt.
//! Every tier of a family puts the same block into the world; only the machine's `tier` and the item
//! that places it differ.
//!
//! Invariants: a family has one item per tier, and as many as its numbers array has rows (tested); an
//! item is at most one family's tier. `FAST_BELT` and `MINER_MK2` are legacy blocks: worlds from before
//! tiers hold them, and their items are now the Mk2 items, which place the family's block.
//!
//! To add a tier: a row in the family's numbers array, its item appended to `items` (a new item id) and
//! its hand recipe (the previous tier plus kits; the lint checks it). To add a family: a `FAMILIES` row,
//! a saved `tier` field on the machine (read only from `SAVE_VERSION` 21 on, so old saves load at Mk1), its numbers
//! array, and its arms in `Factory::tiered_at` and `Factory::set_tier` (`upgrades.rs`).

use crate::block::{
    BlockId, ASSEMBLER, BELT, BLAST_FURNACE, CONSTRUCTOR, FAST_BELT, GENERATOR, LAB, MINER, MINER_MK2, POLE, PUMP,
    QUARRY, SMELTER, STORAGE,
};
use crate::item::{
    ItemId, ASSEMBLER_MK2, ASSEMBLER_MK3, ASSEMBLER_MK4, BELT_MK3, BELT_MK4, BLAST_FURNACE_MK2, BLAST_FURNACE_MK3,
    BLAST_FURNACE_MK4, BOX_MK2, BOX_MK3, CONSTRUCTOR_MK2, CONSTRUCTOR_MK3, CONSTRUCTOR_MK4, GENERATOR_MK2, LAB_MK2,
    LAB_MK3, LAB_MK4, MINER_MK3, MINER_MK4, POLE_MK2, POLE_MK3, POLE_MK4, PUMP_MK2, PUMP_MK3, QUARRY_MK2, QUARRY_MK3,
    SMELTER_MK2, SMELTER_MK3, SMELTER_MK4,
};

pub struct Family {
    /// The block every tier puts into the world.
    pub block: BlockId,
    /// The item of each tier, Mk1 first.
    pub items: &'static [ItemId],
    /// Kits one upgrade step takes (`upgrades.rs`): belt-like 1, machine 4, multi-block 8.
    pub kits: u32,
}

pub const FAMILIES: &[Family] = &[
    Family { block: BELT, items: &[ItemId::block(BELT), ItemId::block(FAST_BELT), BELT_MK3, BELT_MK4], kits: 1 },
    Family { block: MINER, items: &[ItemId::block(MINER), ItemId::block(MINER_MK2), MINER_MK3, MINER_MK4], kits: 4 },
    Family { block: SMELTER, items: &[ItemId::block(SMELTER), SMELTER_MK2, SMELTER_MK3, SMELTER_MK4], kits: 4 },
    Family {
        block: CONSTRUCTOR,
        items: &[ItemId::block(CONSTRUCTOR), CONSTRUCTOR_MK2, CONSTRUCTOR_MK3, CONSTRUCTOR_MK4],
        kits: 4,
    },
    Family {
        block: ASSEMBLER,
        items: &[ItemId::block(ASSEMBLER), ASSEMBLER_MK2, ASSEMBLER_MK3, ASSEMBLER_MK4],
        kits: 8,
    },
    Family {
        block: BLAST_FURNACE,
        items: &[ItemId::block(BLAST_FURNACE), BLAST_FURNACE_MK2, BLAST_FURNACE_MK3, BLAST_FURNACE_MK4],
        kits: 8,
    },
    Family { block: POLE, items: &[ItemId::block(POLE), POLE_MK2, POLE_MK3, POLE_MK4], kits: 1 },
    Family { block: STORAGE, items: &[ItemId::block(STORAGE), BOX_MK2, BOX_MK3], kits: 4 },
    Family { block: PUMP, items: &[ItemId::block(PUMP), PUMP_MK2, PUMP_MK3], kits: 4 },
    Family { block: QUARRY, items: &[ItemId::block(QUARRY), QUARRY_MK2, QUARRY_MK3], kits: 4 },
    Family { block: LAB, items: &[ItemId::block(LAB), LAB_MK2, LAB_MK3, LAB_MK4], kits: 4 },
    Family { block: GENERATOR, items: &[ItemId::block(GENERATOR), GENERATOR_MK2], kits: 4 },
];

/// The family block and tier that `item` places, if it is a tiered machine.
pub fn placed_by(item: ItemId) -> Option<(BlockId, u8)> {
    FAMILIES.iter().find_map(|f| f.items.iter().position(|&i| i == item).map(|t| (f.block, t as u8)))
}

/// The family whose block is `block`.
pub fn family(block: BlockId) -> Option<&'static Family> {
    FAMILIES.iter().find(|f| f.block == block)
}

/// The item that is tier `tier` of the family whose block is `block`.
pub fn item_of(block: BlockId, tier: u8) -> Option<ItemId> {
    family(block)?.items.get(tier as usize).copied()
}

#[cfg(test)]
mod tests;
