//! Clay and kilns (MAT-CLAY, ids 263–269): river clay and white kaolin dug from their beds (`worldgen/clay.rs`), and
//! what furnaces and the Glass Kiln fire from them (`recipes/clay.rs`): clay bricks, terracotta roof tiles, porcelain
//! tiles and two glazed tiles coloured the way real glazes are, copper turquoise and iron amber.
//! Every id is 256 or more, so each pairs with an `EXTRA` item in `item::WIDE_BLOCK_ITEMS`.
//! To add a ceramic: an id here, a `CLAY_DEFS` row, a layer in `tex.rs` (and `textures/clay.rs`), its item pair,
//! a recipe in `recipes/clay.rs`, and `CLAY_COUNT` bumped. A new glaze colour comes with the family that brings its
//! colourant (cobalt blue with MAT-MAFIC, zircon white with MAT-SANDS).

use super::make::{all, cube, ore};
use super::{sound, tex, BlockDef, BlockId};

/// Grey-brown plastic clay from river banks and lake beds: an ore (deposits, miners), fired into bricks and tiles.
pub const CLAY: BlockId = 263;
/// White china clay from weathered highland granite: an ore, fired with granite into porcelain.
pub const KAOLIN: BlockId = 264;
pub const CLAY_BRICKS: BlockId = 265;
pub const TERRACOTTA: BlockId = 266;
pub const PORCELAIN: BlockId = 267;
/// Porcelain under a copper glaze.
pub const TURQUOISE_TILES: BlockId = 268;
/// Porcelain under an iron glaze.
pub const AMBER_TILES: BlockId = 269;

pub const CLAY_COUNT: usize = 7;

/// Soft ground: dug quicker than rock ore, and sounds like soil.
const fn soft_ore(name: &'static str, layer: u16, id: BlockId) -> BlockDef {
    BlockDef { break_time: 0.9, sound: sound::DIRT, ..ore(name, all(layer), id) }
}

pub(super) const CLAY_DEFS: [BlockDef; CLAY_COUNT] = [
    soft_ore("Clay", tex::CLAY, CLAY),
    soft_ore("Kaolin", tex::KAOLIN, KAOLIN),
    cube("Clay Bricks", 1.3, all(tex::CLAY_BRICKS), CLAY_BRICKS, sound::STONE),
    cube("Terracotta Tiles", 1.1, all(tex::TERRACOTTA), TERRACOTTA, sound::STONE),
    cube("Porcelain Tiles", 1.0, all(tex::PORCELAIN), PORCELAIN, sound::STONE),
    cube("Turquoise Tiles", 1.0, all(tex::TURQUOISE_TILES), TURQUOISE_TILES, sound::STONE),
    cube("Amber Tiles", 1.0, all(tex::AMBER_TILES), AMBER_TILES, sound::STONE),
];
