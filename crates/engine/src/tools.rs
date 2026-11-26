//! Hand-held tools and devices, and which tool class a block wants ([`kind_for`]: what the worn Tool Belt,
//! `tool_belt/`, mines it with).
//!
//! The pickaxes, axes and shovels (stone, iron, steel) are legacy since the Tool Belt (save 87): no recipe makes them
//! and they break nothing faster, but old saves still hold them and the belt's heads borrow their models. Their stack
//! count was their wear, so everything that moves items (boxes, belts, the recycler) still moves one as a whole
//! ([`lot`]) and the recycler pays per use.
//!
//! The prospecting devices (scanner, core drill: `prospect.rs`) are tools too, of [`DEVICE_TIER`]: one
//! per slot, and no block wants them, so they never wear.
//!
//! To add a device: an item id and row in `item.rs`, a `ToolKind` and a row in `TOOLS` (no wear: `DEVICE_TIER`).

use crate::block::{self, sound, BlockId};
use crate::item::{
    ItemId, CORE_DRILL, FLAMETHROWER, HOVER_PACK, IRON_AXE, IRON_PICKAXE, IRON_SHOVEL, JETPACK, PAINT_SPRAYER,
    PERSONAL_DRONE, PLANNER, SCANNER, SCANNER_MK2, SICKLE, STEEL_AXE, STEEL_PICKAXE, STEEL_SHOVEL, STONE_AXE,
    STONE_PICKAXE, STONE_SHOVEL, TELESCOPE,
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
    HoverPack,
    PersonalDrone,
    /// Marks terraforming sites (`site_hands.rs`).
    Planner,
    /// Looks far and pins ground (`telescope.rs`).
    Telescope,
    /// Cuts the ground plants of a patch (`sickle.rs`); breaks no block.
    Sickle,
    /// Throws a jet of flame that clears plants and burns trees to charcoal (`flame_hands.rs`).
    Flamethrower,
    /// Sprays surface paint onto concrete, plaster, masonry, metal and roof tiles (`colour_hands.rs`).
    PaintSprayer,
}

pub struct Tier {
    /// Blocks it breaks before it wears out (its stack size).
    pub uses: u32,
}

pub const STONE_TIER: Tier = Tier { uses: 150 };
pub const IRON_TIER: Tier = Tier { uses: 600 };
pub const STEEL_TIER: Tier = Tier { uses: 1500 };
/// Prospecting devices: a stack of one that no block wears down.
pub const DEVICE_TIER: Tier = Tier { uses: 1 };

pub struct ToolDef {
    pub item: ItemId,
    pub kind: ToolKind,
    pub tier: &'static Tier,
}

pub const TOOLS: [ToolDef; 20] = [
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
    ToolDef { item: PLANNER, kind: ToolKind::Planner, tier: &DEVICE_TIER },
    ToolDef { item: HOVER_PACK, kind: ToolKind::HoverPack, tier: &DEVICE_TIER },
    ToolDef { item: TELESCOPE, kind: ToolKind::Telescope, tier: &DEVICE_TIER },
    ToolDef { item: SICKLE, kind: ToolKind::Sickle, tier: &DEVICE_TIER },
    ToolDef { item: FLAMETHROWER, kind: ToolKind::Flamethrower, tier: &DEVICE_TIER },
    ToolDef { item: PAINT_SPRAYER, kind: ToolKind::PaintSprayer, tier: &DEVICE_TIER },
];

/// Which prospecting device `item` is, if any.
pub fn device(item: ItemId) -> Option<ToolKind> {
    tool(item).map(|t| t.kind).filter(|k| matches!(k, ToolKind::Scanner | ToolKind::CoreDrill))
}

pub fn tool(item: ItemId) -> Option<&'static ToolDef> {
    TOOLS.iter().find(|t| t.item == item)
}

/// How many of a `count` stack of `item` move as one piece: a tool's whole count (it is wear, so a worn shovel
/// is one shovel, not 87), one for anything else. Belts, routers and the recycler move things by this.
pub fn lot(item: ItemId, count: u32) -> u32 {
    if tool(item).is_some() {
        count
    } else {
        count.min(1)
    }
}

/// Which tool a block wants, by what it is made of (its sound material): stone and ore take a pickaxe,
/// wood an axe, soil a shovel; leaves and metal none.
pub(crate) fn kind_for(b: BlockId) -> Option<ToolKind> {
    match block::def(b).sound {
        sound::STONE => Some(ToolKind::Pickaxe),
        sound::WOOD => Some(ToolKind::Axe),
        sound::DIRT | sound::GRASS | sound::SAND => Some(ToolKind::Shovel),
        _ => None,
    }
}

#[cfg(test)]
mod tests;
