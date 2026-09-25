//! Item registry: `ItemId` and one `ItemDef` per item (name, stack size, look, the block it places).
//!
//! Invariants: ids below 256 are the blocks with the same number, and their rows are derived from
//! `block::BLOCK_DEFS`, so block ids and old saves stay valid items. Non-block items start at 256
//! (`EXTRA`, in id order). `ItemId::NONE` (air) is "no item". Every item is drawn as a box with three
//! texture layers (top, side, bottom) and proportions `size`: blocks are full cubes; the HUD icon and
//! the loose and belt models all use the same two fields.
//!
//! To add an item: an id constant (append, never renumber; ids are saved) and its `EXTRA` row, plus a
//! texture layer in `block::tex` with its pattern in `textures::pixel` if it needs a new look.

use crate::block::{self, tex, BlockId, AIR, BLOCK_COUNT, FACE_BOTTOM, FACE_SIDE, FACE_TOP};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Default)]
pub struct ItemId(pub u16);

impl ItemId {
    pub const NONE: ItemId = ItemId(AIR as u16);

    /// The item that is the block `b`.
    pub const fn block(b: BlockId) -> ItemId {
        ItemId(b as u16)
    }

    /// The block this item puts into the world, if it can be placed.
    pub fn places(self) -> Option<BlockId> {
        def(self).map(|d| d.places).filter(|&b| b != AIR)
    }

    /// A real item (not `NONE`, and in the table).
    pub fn is_valid(self) -> bool {
        self != ItemId::NONE && def(self).is_some()
    }
}

impl From<BlockId> for ItemId {
    fn from(b: BlockId) -> ItemId {
        ItemId::block(b)
    }
}

pub const IRON_INGOT: ItemId = ItemId(256);
pub const COPPER_INGOT: ItemId = ItemId(257);
pub const IRON_PLATE: ItemId = ItemId(258);
pub const IRON_ROD: ItemId = ItemId(259);
pub const SCREW: ItemId = ItemId(260);
pub const COPPER_WIRE: ItemId = ItemId(261);
pub const RED_PACK: ItemId = ItemId(262);
pub const GREEN_PACK: ItemId = ItemId(263);

/// Stack size of every item so far.
pub const MAX_STACK: u32 = 64;

pub struct ItemDef {
    pub name: &'static str,
    /// Most items one slot holds.
    pub stack: u32,
    /// Texture layers of its model and icon: top, side, bottom.
    pub tex: [u16; 3],
    /// Model proportions (x, y, z); 1.0 on every axis is a full cube.
    pub size: [f32; 3],
    /// The block it places, or `AIR`.
    pub places: BlockId,
}

/// Looks up an item; `None` for ids in neither table.
#[inline]
pub fn def(id: ItemId) -> Option<&'static ItemDef> {
    let i = id.0 as usize;
    if i < 256 {
        BLOCK_ITEMS.get(i)
    } else {
        EXTRA.get(i - 256)
    }
}

pub fn name(id: ItemId) -> &'static str {
    def(id).map_or("", |d| d.name)
}

/// Most of `id` one slot holds (unknown items: [`MAX_STACK`]).
#[inline]
pub fn stack_size(id: ItemId) -> u32 {
    def(id).map_or(MAX_STACK, |d| d.stack)
}

/// An ingot: a small bar of metal.
const fn ingot(name: &'static str, layer: u16) -> ItemDef {
    ItemDef { name, stack: MAX_STACK, tex: [layer; 3], size: [0.9, 0.45, 0.55], places: AIR }
}

/// A part: `layer` on every face, proportions `size`.
const fn part(name: &'static str, layer: u16, size: [f32; 3]) -> ItemDef {
    ItemDef { name, stack: MAX_STACK, tex: [layer; 3], size, places: AIR }
}

const EXTRA: [ItemDef; 8] = [
    ingot("Iron Ingot", tex::IRON_INGOT),
    ingot("Copper Ingot", tex::COPPER_INGOT),
    part("Iron Plate", tex::IRON_PLATE, [0.85, 0.14, 0.85]),
    part("Iron Rod", tex::IRON_INGOT, [1.0, 0.2, 0.2]),
    part("Screws", tex::DRILL, [0.35, 0.35, 0.35]),
    part("Copper Wire", tex::COPPER_WIRE, [0.6, 0.4, 0.6]),
    part("Red Science Pack", tex::RED_PACK, [0.4, 0.6, 0.4]),
    part("Green Science Pack", tex::GREEN_PACK, [0.4, 0.6, 0.4]),
];

/// One row per block: its name and faces, placeable blocks place themselves.
static BLOCK_ITEMS: [ItemDef; BLOCK_COUNT] = {
    const EMPTY: ItemDef = ItemDef { name: "", stack: MAX_STACK, tex: [0; 3], size: [1.0; 3], places: AIR };
    let mut t = [EMPTY; BLOCK_COUNT];
    let mut i = 0;
    while i < BLOCK_COUNT {
        let b = &block::DEFS[i];
        let f = b.faces;
        t[i] = ItemDef {
            name: b.name,
            stack: MAX_STACK,
            tex: [f[FACE_TOP], f[FACE_SIDE], f[FACE_BOTTOM]],
            size: [1.0; 3],
            places: if b.placeable { i as BlockId } else { AIR },
        };
        i += 1;
    }
    t
};

#[cfg(test)]
mod tests;
