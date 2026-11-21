//! Tiered families (Mk1, Mk2, …; pumps, quarries and foresters stop at Mk3, generators and pumpjacks at Mk2): which machines come in tiers and which item is each tier.
//!
//! A tier is machine state: `tier: u8`, an index (0 is Mk1) into the family's numbers, which live atop
//! the machine's own module as one array (`BELT_TIERS`, `MINER_TIERS`, `LAB_TIERS`, `POLE_TIERS`,
//! `BOX_SLOTS`, `PUMP_TIERS`, `QUARRY_TIERS`, `GENERATOR_TIERS`, a processor spec's `tiers`). Splitters and
//! filters have none: they pass an item a tick, faster than any belt.
//! Every tier of a family puts the same block into the world; only the machine's `tier` and the item
//! that places it differ.
//!
//! Invariants: a family has one item per tier, and as many as its numbers array has rows (tested); an
//! item is at most one family's tier. A tier's kit and paint are its `Family::colour`, the tier itself unless the
//! family starts at a later kit (`first_kit`: the pumpjack's Mk2 is gold). `FAST_BELT` and `MINER_MK2` are legacy blocks: worlds from before
//! tiers hold them, and their items are now the Mk2 items, which place the family's block.
//!
//! To add a tier: a row in the family's numbers array, its item appended to `items` (a new item id) and
//! its hand recipe (the previous tier plus kits; the lint checks it). To add a family: a `FAMILIES` row,
//! a saved `tier` field on the machine (read only from `SAVE_VERSION` 21 on, so old saves load at Mk1), its numbers
//! array, and its arms in `Factory::tiered_at` and `Factory::set_tier` (`upgrades.rs`).

use crate::block::{
    BlockId, ASSEMBLER, BELT, BLAST_FURNACE, CONSTRUCTOR, DRONE_PORT, FAST_BELT, FORESTER, GENERATOR, LAB, MINER,
    MINER_MK2, POLE, PUMP, PUMPJACK, QUARRY, RESEARCH_CENTER, SMELTER, STORAGE, THUMPER, UNDERPASS_IN,
};
use crate::item::{
    ItemId, ASSEMBLER_MK2, ASSEMBLER_MK3, ASSEMBLER_MK4, BELT_MK3, BELT_MK4, BLAST_FURNACE_MK2, BLAST_FURNACE_MK3,
    BLAST_FURNACE_MK4, BOX_MK2, BOX_MK3, CONSTRUCTOR_MK2, CONSTRUCTOR_MK3, CONSTRUCTOR_MK4, GENERATOR_MK2, LAB_MK2,
    LAB_MK3, LAB_MK4, MINER_MK3, MINER_MK4, POLE_MK2, POLE_MK3, POLE_MK4, PUMP_MK2, PUMP_MK3, QUARRY_MK2, QUARRY_MK3,
    SMELTER_MK2, SMELTER_MK3, SMELTER_MK4,
};
use crate::item::{
    ASSEMBLER_MK5, BLAST_FURNACE_MK5, CONSTRUCTOR_MK5, DRONE_PORT_MK5, MINER_MK5, RESEARCH_CENTER_MK5, SMELTER_MK5,
};
use crate::item::{BOX_MK4, BOX_MK5, PUMPJACK_MK2, THUMPER_ITEM, THUMPER_MK2, THUMPER_MK3};
use crate::item::{DRONE_PORT_MK2, DRONE_PORT_MK3, DRONE_PORT_MK4, FORESTER_MK2, FORESTER_MK3};
use crate::item::{RESEARCH_CENTER_MK2, RESEARCH_CENTER_MK3, RESEARCH_CENTER_MK4};
use crate::item::{UNDERPASS_MK2, UNDERPASS_MK3, UNDERPASS_MK4};

pub struct Family {
    /// The block every tier puts into the world.
    pub block: BlockId,
    /// The item of each tier, Mk1 first.
    pub items: &'static [ItemId],
    /// Kits one upgrade step takes (`upgrades.rs`): belt-like 1, machine 4, multi-block 8.
    pub kits: u32,
    /// The colour (an index into `upgrades::KITS`; 1 is green) of the kit that makes the family's Mk2. Most families
    /// start at green and climb one colour a tier; a machine that arrived late starts higher (the pumpjack, with
    /// violet science, takes gold kits) and its tiers follow that colour on.
    pub first_kit: u8,
}

impl Family {
    /// A family whose Mk2 takes green kits, the usual climb.
    const fn new(block: BlockId, items: &'static [ItemId], kits: u32) -> Family {
        Family { block, items, kits, first_kit: 1 }
    }

    /// The kit colour of tier `tier` (0, Mk1, has none and counts as red): the index into `KITS` and `TIER_COLOURS`.
    pub const fn colour(&self, tier: u8) -> u8 {
        if tier == 0 {
            0
        } else {
            self.first_kit + tier - 1
        }
    }
}

pub const FAMILIES: &[Family] = &[
    Family::new(BELT, &[ItemId::block(BELT), ItemId::block(FAST_BELT), BELT_MK3, BELT_MK4], 1),
    Family::new(MINER, &[ItemId::block(MINER), ItemId::block(MINER_MK2), MINER_MK3, MINER_MK4, MINER_MK5], 4),
    Family::new(SMELTER, &[ItemId::block(SMELTER), SMELTER_MK2, SMELTER_MK3, SMELTER_MK4, SMELTER_MK5], 4),
    Family::new(
        CONSTRUCTOR,
        &[ItemId::block(CONSTRUCTOR), CONSTRUCTOR_MK2, CONSTRUCTOR_MK3, CONSTRUCTOR_MK4, CONSTRUCTOR_MK5],
        4,
    ),
    Family::new(ASSEMBLER, &[ItemId::block(ASSEMBLER), ASSEMBLER_MK2, ASSEMBLER_MK3, ASSEMBLER_MK4, ASSEMBLER_MK5], 8),
    Family::new(
        BLAST_FURNACE,
        &[ItemId::block(BLAST_FURNACE), BLAST_FURNACE_MK2, BLAST_FURNACE_MK3, BLAST_FURNACE_MK4, BLAST_FURNACE_MK5],
        8,
    ),
    Family::new(UNDERPASS_IN, &[ItemId::block(UNDERPASS_IN), UNDERPASS_MK2, UNDERPASS_MK3, UNDERPASS_MK4], 2),
    Family::new(POLE, &[ItemId::block(POLE), POLE_MK2, POLE_MK3, POLE_MK4], 1),
    Family::new(STORAGE, &[ItemId::block(STORAGE), BOX_MK2, BOX_MK3, BOX_MK4, BOX_MK5], 4),
    Family::new(PUMP, &[ItemId::block(PUMP), PUMP_MK2, PUMP_MK3], 4),
    Family::new(QUARRY, &[ItemId::block(QUARRY), QUARRY_MK2, QUARRY_MK3], 4),
    Family::new(FORESTER, &[ItemId::block(FORESTER), FORESTER_MK2, FORESTER_MK3], 4),
    Family::new(LAB, &[ItemId::block(LAB), LAB_MK2, LAB_MK3, LAB_MK4], 4),
    Family::new(GENERATOR, &[ItemId::block(GENERATOR), GENERATOR_MK2], 4),
    Family::new(
        DRONE_PORT,
        &[ItemId::block(DRONE_PORT), DRONE_PORT_MK2, DRONE_PORT_MK3, DRONE_PORT_MK4, DRONE_PORT_MK5],
        8,
    ),
    Family::new(
        RESEARCH_CENTER,
        &[
            ItemId::block(RESEARCH_CENTER),
            RESEARCH_CENTER_MK2,
            RESEARCH_CENTER_MK3,
            RESEARCH_CENTER_MK4,
            RESEARCH_CENTER_MK5,
        ],
        8,
    ),
    // The pumpjack came with violet science, so its Mk2 is the gold kit's (`first_kit` 4).
    Family { first_kit: 4, ..Family::new(PUMPJACK, &[ItemId::block(PUMPJACK), PUMPJACK_MK2], 4) },
    // Seismic thumper arrives after Advanced Scanning (violet science): Mk2 takes violet kits, Mk3 gold kits.
    Family { first_kit: 3, ..Family::new(THUMPER, &[THUMPER_ITEM, THUMPER_MK2, THUMPER_MK3], 4) },
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

/// What a placed machine of family `block` at `tier` is called: its tier's item name, with the Mk added where the
/// name has none ("Smelter Mk1", "Miner Mk3"), so a map or panel never reads as a lower tier.
pub fn name(block: BlockId, tier: u8) -> String {
    let base = item_of(block, tier).map_or(crate::block::def(block).name, crate::item::name);
    if base.contains(" Mk") {
        base.to_string()
    } else {
        format!("{base} Mk{}", u32::from(tier) + 1)
    }
}

#[cfg(test)]
mod tests;
