//! Tiered families (Mk1, Mk2, …): which machines come in tiers and which item is each tier.
//!
//! A tier is machine state: `tier: u8`, an index (0 is Mk1) into the family's numbers, which live atop
//! the machine's own module as one array (`BELT_TIERS`, `MINER_TIERS`). Every tier of a family puts the
//! same block into the world; only the machine's `tier` and the item that places it differ.
//!
//! Invariants: a family has one item per tier, and as many as its numbers array has rows (tested); an
//! item is at most one family's tier. `FAST_BELT` and `MINER_MK2` are legacy blocks: worlds from before
//! tiers hold them, and their items are now the Mk2 items, which place the family's block.
//!
//! To add a tier: a row in the family's numbers array, its item appended to `items` (a new item id) and
//! its hand recipe (the previous tier plus kits; the lint checks it). To add a family: a `FAMILIES` row,
//! a saved `tier` field on the machine, its numbers array, and its arm in `Factory::tiered_at`.

use crate::block::{BlockId, BELT, FAST_BELT, MINER, MINER_MK2};
use crate::item::ItemId;

pub struct Family {
    /// The block every tier puts into the world.
    pub block: BlockId,
    /// The item of each tier, Mk1 first.
    pub items: &'static [ItemId],
    /// Kits one upgrade step takes (`upgrades.rs`): belt-like 1, machine 4, multi-block 8.
    pub kits: u32,
}

pub const FAMILIES: &[Family] = &[
    Family { block: BELT, items: &[ItemId::block(BELT), ItemId::block(FAST_BELT)], kits: 1 },
    Family { block: MINER, items: &[ItemId::block(MINER), ItemId::block(MINER_MK2)], kits: 4 },
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
