//! The decor building set (R7, ids 129–142): coloured concretes, rock bricks, framed glass, metal panels, a catwalk, a
//! railing and a panel light. Machine-made only (`recipes/decor.rs`, unlocked by the Architecture tech); every one is a
//! plain block that drops itself. Cube-shaped on purpose: slabs and stairs need non-cube meshing, a step of their own.
//! To add a decor block: an id here, a row in `DECOR_DEFS`, a texture layer in `layers` (and `textures/decor.rs`), a
//! machine recipe row in `recipes/decor.rs` and `DECOR_COUNT` / `LAYERS_END` bumped.

use super::make::{all, cube, frame};
use super::{sound, BlockDef, BlockId, Render};

pub const RED_CONCRETE: BlockId = 129;
pub const GREEN_CONCRETE: BlockId = 130;
pub const BLACK_CONCRETE: BlockId = 131;
pub const WHITE_CONCRETE: BlockId = 132;
pub const YELLOW_CONCRETE: BlockId = 133;
pub const BASALT_TILES: BlockId = 134;
pub const GRANITE_BRICKS: BlockId = 135;
pub const SANDSTONE_BRICKS: BlockId = 136;
pub const SLAG_BRICKS: BlockId = 137;
/// See-through and solid, like glass.
pub const FRAMED_GLASS: BlockId = 138;
pub const METAL_PANEL: BlockId = 139;
/// A grating you walk on and see through.
pub const CATWALK: BlockId = 140;
/// Not solid: a body walks through it, like a ladder without the climbing.
pub const RAILING: BlockId = 141;
/// A soft light (a torch's reach), as a block.
pub const PANEL_LIGHT: BlockId = 142;

pub const DECOR_COUNT: usize = 14;

/// Texture layers, after `tex::VAULT_DOOR` (`tex.rs` re-exports them).
pub mod layers {
    pub const RED_CONCRETE: u16 = 333;
    pub const GREEN_CONCRETE: u16 = 334;
    pub const BLACK_CONCRETE: u16 = 335;
    pub const WHITE_CONCRETE: u16 = 336;
    pub const YELLOW_CONCRETE: u16 = 337;
    pub const BASALT_TILES: u16 = 338;
    pub const GRANITE_BRICKS: u16 = 339;
    pub const SANDSTONE_BRICKS: u16 = 340;
    pub const SLAG_BRICKS: u16 = 341;
    pub const FRAMED_GLASS: u16 = 342;
    pub const METAL_PANEL: u16 = 343;
    pub const CATWALK: u16 = 344;
    pub const RAILING: u16 = 345;
    pub const PANEL_LIGHT: u16 = 346;
    /// One past the last decor layer: `tex::COUNT`.
    pub const LAYERS_END: u16 = 347;
}

use layers as t;

/// The rows of `DEFS` from block 129 on, in id order.
pub(super) const DECOR_DEFS: [BlockDef; DECOR_COUNT] = [
    cube("Red Concrete", 1.4, all(t::RED_CONCRETE), RED_CONCRETE, sound::STONE),
    cube("Green Concrete", 1.4, all(t::GREEN_CONCRETE), GREEN_CONCRETE, sound::STONE),
    cube("Black Concrete", 1.4, all(t::BLACK_CONCRETE), BLACK_CONCRETE, sound::STONE),
    cube("White Concrete", 1.4, all(t::WHITE_CONCRETE), WHITE_CONCRETE, sound::STONE),
    cube("Yellow Concrete", 1.4, all(t::YELLOW_CONCRETE), YELLOW_CONCRETE, sound::STONE),
    cube("Basalt Tiles", 1.4, all(t::BASALT_TILES), BASALT_TILES, sound::STONE),
    cube("Granite Bricks", 1.4, all(t::GRANITE_BRICKS), GRANITE_BRICKS, sound::STONE),
    cube("Sandstone Bricks", 1.0, all(t::SANDSTONE_BRICKS), SANDSTONE_BRICKS, sound::STONE),
    cube("Slag Bricks", 1.2, all(t::SLAG_BRICKS), SLAG_BRICKS, sound::STONE),
    BlockDef { render: Render::Cutout, ..cube("Framed Glass", 0.4, all(t::FRAMED_GLASS), FRAMED_GLASS, sound::STONE) },
    cube("Metal Panel", 1.2, all(t::METAL_PANEL), METAL_PANEL, sound::METAL),
    BlockDef { render: Render::Cutout, ..cube("Catwalk", 0.8, all(t::CATWALK), CATWALK, sound::METAL) },
    frame("Railing", 0.6, all(t::RAILING), RAILING, sound::METAL),
    BlockDef {
        light: crate::light::TORCH_LIGHT,
        ..cube("Panel Light", 0.4, all(t::PANEL_LIGHT), PANEL_LIGHT, sound::METAL)
    },
];
