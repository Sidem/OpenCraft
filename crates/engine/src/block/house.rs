//! The house set (ids 144–156): wooden and iron doors that open and close, three roof tiles, thatch, plaster, a timber
//! frame, a wood floor, a window and chimney bricks. Hand-crafted from early materials (`recipes/house.rs`). Cube-shaped
//! on purpose, like the decor set: slabs, stairs, sloped roofs and thin doors need non-cube meshing, a step of their own.
//! A door is a pair of blocks: the closed one (solid, the item) and the open one (see-through, walk-through, never held,
//! dropping the closed one). `Action::ToggleDoor` swaps them (`doors.rs`); `toggled` is the pairing.
//! To add a house block: an id here, a row in `HOUSE_DEFS`, a layer in `layers` (and `textures/house.rs`), a recipe in
//! `recipes/house.rs`, and `HOUSE_COUNT` / `LAYERS_END` bumped.

use super::make::{all, cube};
use super::{sound, BlockDef, BlockId, Render};

pub const WOOD_DOOR: BlockId = 144;
pub const WOOD_DOOR_OPEN: BlockId = 145;
pub const IRON_DOOR: BlockId = 146;
pub const IRON_DOOR_OPEN: BlockId = 147;
pub const RED_ROOF: BlockId = 148;
pub const SLATE_ROOF: BlockId = 149;
pub const GREEN_ROOF: BlockId = 150;
pub const THATCH: BlockId = 151;
pub const TIMBER_FRAME: BlockId = 152;
pub const WOOD_FLOOR: BlockId = 153;
/// See-through and solid, like glass.
pub const WINDOW: BlockId = 154;
pub const CHIMNEY_BRICKS: BlockId = 155;
pub const PLASTER: BlockId = 156;

pub const HOUSE_COUNT: usize = 13;

/// Texture layers, after the plain materials (`tex::MAT_AETHER`; `tex.rs` re-exports them).
pub mod layers {
    pub const WOOD_DOOR: u16 = 354;
    pub const WOOD_DOOR_OPEN: u16 = 355;
    pub const IRON_DOOR: u16 = 356;
    pub const IRON_DOOR_OPEN: u16 = 357;
    pub const RED_ROOF: u16 = 358;
    pub const SLATE_ROOF: u16 = 359;
    pub const GREEN_ROOF: u16 = 360;
    pub const THATCH: u16 = 361;
    pub const TIMBER_FRAME: u16 = 362;
    pub const WOOD_FLOOR: u16 = 363;
    pub const WINDOW: u16 = 364;
    pub const CHIMNEY_BRICKS: u16 = 365;
    pub const PLASTER: u16 = 366;
    /// One past the last house layer: `tex::COUNT`.
    pub const HOUSE_LAYERS_END: u16 = 367;
}

use layers as t;

/// The other state of a door: the closed block's open twin and back. `None` for every other block.
pub const fn toggled(id: BlockId) -> Option<BlockId> {
    match id {
        WOOD_DOOR => Some(WOOD_DOOR_OPEN),
        WOOD_DOOR_OPEN => Some(WOOD_DOOR),
        IRON_DOOR => Some(IRON_DOOR_OPEN),
        IRON_DOOR_OPEN => Some(IRON_DOOR),
        _ => None,
    }
}

/// Whether `id` is a door that is open (see-through, not solid).
pub const fn is_open_door(id: BlockId) -> bool {
    matches!(id, WOOD_DOOR_OPEN | IRON_DOOR_OPEN)
}

/// An open door: a see-through block that is not held and breaks into its closed twin.
const fn open_door(name: &'static str, break_time: f32, layer: u16, closed: BlockId, sound: u8) -> BlockDef {
    BlockDef {
        render: Render::Cutout,
        solid: false,
        placeable: false,
        ..cube(name, break_time, all(layer), closed, sound)
    }
}

/// The rows of `DEFS` from block 144 on, in id order.
pub(super) const HOUSE_DEFS: [BlockDef; HOUSE_COUNT] = [
    cube("Wooden Door", 0.7, all(t::WOOD_DOOR), WOOD_DOOR, sound::WOOD),
    open_door("Open Wooden Door", 0.7, t::WOOD_DOOR_OPEN, WOOD_DOOR, sound::WOOD),
    cube("Iron Door", 1.6, all(t::IRON_DOOR), IRON_DOOR, sound::METAL),
    open_door("Open Iron Door", 1.6, t::IRON_DOOR_OPEN, IRON_DOOR, sound::METAL),
    cube("Red Roof Tiles", 0.8, all(t::RED_ROOF), RED_ROOF, sound::STONE),
    cube("Slate Roof Tiles", 0.9, all(t::SLATE_ROOF), SLATE_ROOF, sound::STONE),
    cube("Green Roof Tiles", 0.8, all(t::GREEN_ROOF), GREEN_ROOF, sound::STONE),
    cube("Thatch", 0.3, all(t::THATCH), THATCH, sound::GRASS),
    cube("Timber Frame", 0.8, all(t::TIMBER_FRAME), TIMBER_FRAME, sound::WOOD),
    cube("Wood Floor", 0.7, all(t::WOOD_FLOOR), WOOD_FLOOR, sound::WOOD),
    BlockDef { render: Render::Cutout, ..cube("Window", 0.4, all(t::WINDOW), WINDOW, sound::WOOD) },
    cube("Chimney Bricks", 1.1, all(t::CHIMNEY_BRICKS), CHIMNEY_BRICKS, sound::STONE),
    cube("Plaster", 0.5, all(t::PLASTER), PLASTER, sound::STONE),
];
