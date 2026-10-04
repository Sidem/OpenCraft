//! Hand tools: pickaxe, axe and shovel in tiers (stone, iron, steel). A tool held in the
//! selected hotbar slot breaks its class of blocks faster, and a pickaxe of a good tier keeps a little
//! more ore by hand (still far below a miner's recovery).
//!
//! Wear is the stack count: a tool's stack size is its uses, a fresh one is a full stack, and breaking a
//! block of its class takes one away (the core does it in `Sim::break_block`), so a tool at zero is gone.
//! Everything that moves items (boxes, drops, pickups, shift-clicks) therefore carries wear as it is, and
//! two worn tools of one kind simply pool their uses. The break speed is the hands' business
//! (`interaction.rs`, presentation), the uses and the ore kept are the core's.
//!
//! The prospecting devices (scanner, core drill: `prospect.rs`) are tools too, of [`DEVICE_TIER`]: one
//! per slot, and no block wants them, so they never wear.
//!
//! To add a tier: a `Tier` constant, three item ids and rows in `item.rs`, three rows in `TOOLS` and hand
//! recipes in `recipes.rs`.

use crate::block::{self, sound, BlockId};
use crate::deposits::HAND_YIELD;
use crate::item::{
    ItemId, CORE_DRILL, IRON_AXE, IRON_PICKAXE, IRON_SHOVEL, JETPACK, PERSONAL_DRONE, SCANNER, SCANNER_MK2, STEEL_AXE,
    STEEL_PICKAXE, STEEL_SHOVEL, STONE_AXE, STONE_PICKAXE, STONE_SHOVEL,
};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ToolKind {
    Pickaxe,
    Axe,
    Shovel,
    Scanner,
    CoreDrill,
    /// Worn in the pack, no wear: see `helpers/`.
    Jetpack,
    PersonalDrone,
}

pub struct Tier {
    /// Blocks it breaks before it wears out (its stack size).
    pub uses: u32,
    /// Break speed on its class of blocks, as a multiple of bare hands.
    pub speed: f32,
    /// Ore a pickaxe of this tier keeps per block mined by hand.
    pub ore_yield: u32,
}

pub const STONE_TIER: Tier = Tier { uses: 150, speed: 2.0, ore_yield: HAND_YIELD };
pub const IRON_TIER: Tier = Tier { uses: 600, speed: 4.0, ore_yield: HAND_YIELD + 1 };
/// Keeps 5 ore a block, still far below a machine (docs/TECH_ERAS.md).
pub const STEEL_TIER: Tier = Tier { uses: 1500, speed: 6.0, ore_yield: HAND_YIELD + 2 };
/// Prospecting devices: a stack of one that no block wears down.
pub const DEVICE_TIER: Tier = Tier { uses: 1, speed: 1.0, ore_yield: HAND_YIELD };

pub struct ToolDef {
    pub item: ItemId,
    pub kind: ToolKind,
    pub tier: &'static Tier,
}

pub const TOOLS: [ToolDef; 14] = [
    ToolDef { item: STONE_PICKAXE, kind: ToolKind::Pickaxe, tier: &STONE_TIER },
    ToolDef { item: STONE_AXE, kind: ToolKind::Axe, tier: &STONE_TIER },
    ToolDef { item: STONE_SHOVEL, kind: ToolKind::Shovel, tier: &STONE_TIER },
    ToolDef { item: IRON_PICKAXE, kind: ToolKind::Pickaxe, tier: &IRON_TIER },
    ToolDef { item: IRON_AXE, kind: ToolKind::Axe, tier: &IRON_TIER },
    ToolDef { item: IRON_SHOVEL, kind: ToolKind::Shovel, tier: &IRON_TIER },
    ToolDef { item: STEEL_PICKAXE, kind: ToolKind::Pickaxe, tier: &STEEL_TIER },
    ToolDef { item: STEEL_AXE, kind: ToolKind::Axe, tier: &STEEL_TIER },
    ToolDef { item: STEEL_SHOVEL, kind: ToolKind::Shovel, tier: &STEEL_TIER },
    ToolDef { item: SCANNER, kind: ToolKind::Scanner, tier: &DEVICE_TIER },
    ToolDef { item: CORE_DRILL, kind: ToolKind::CoreDrill, tier: &DEVICE_TIER },
    ToolDef { item: SCANNER_MK2, kind: ToolKind::Scanner, tier: &DEVICE_TIER },
    ToolDef { item: JETPACK, kind: ToolKind::Jetpack, tier: &DEVICE_TIER },
    ToolDef { item: PERSONAL_DRONE, kind: ToolKind::PersonalDrone, tier: &DEVICE_TIER },
];

/// Which prospecting device `item` is, if any.
pub fn device(item: ItemId) -> Option<ToolKind> {
    tool(item).map(|t| t.kind).filter(|k| matches!(k, ToolKind::Scanner | ToolKind::CoreDrill))
}

pub fn tool(item: ItemId) -> Option<&'static ToolDef> {
    TOOLS.iter().find(|t| t.item == item)
}

/// The tool `item` if it is the right one for breaking `block`.
pub fn tool_for(item: ItemId, block: BlockId) -> Option<&'static ToolDef> {
    tool(item).filter(|t| Some(t.kind) == kind_for(block))
}

/// How much faster `item` breaks `block` than bare hands (1 for anything else).
pub fn break_speed(item: ItemId, block: BlockId) -> f32 {
    tool_for(item, block).map_or(1.0, |t| t.tier.speed)
}

/// Ore kept by hand from one ore block while holding `item`.
pub fn ore_yield(item: ItemId) -> u32 {
    tool(item).filter(|t| t.kind == ToolKind::Pickaxe).map_or(HAND_YIELD, |t| t.tier.ore_yield)
}

/// Which tool a block wants, by what it is made of (its sound material): stone and ore take a pickaxe,
/// wood an axe, soil a shovel; leaves and metal none.
fn kind_for(b: BlockId) -> Option<ToolKind> {
    match block::def(b).sound {
        sound::STONE => Some(ToolKind::Pickaxe),
        sound::WOOD => Some(ToolKind::Axe),
        sound::DIRT | sound::GRASS | sound::SAND => Some(ToolKind::Shovel),
        _ => None,
    }
}

#[cfg(test)]
mod tests;
