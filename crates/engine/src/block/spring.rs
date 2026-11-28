//! Natural river spring (id 275): unbreakable aquifer rock embedded at high-elevation river headwaters and highland
//! springs (`worldgen/rivers.rs`, `worldgen/springs.rs`). Emits finite water horizontally out of its 4 side faces at
//! its own elevation (`sim/water.rs`), and can be capped or redirected by placing solid blocks against those faces.

use super::make::{all, cube};
use super::{sound, tex, BlockDef, BlockId};

/// Natural aquifer rock block (unbreakable, not placeable, emits water sideways).
pub const SPRING: BlockId = 275;

pub const SPRING_COUNT: usize = 1;

pub(super) const SPRING_DEFS: [BlockDef; SPRING_COUNT] =
    [BlockDef { placeable: false, ..cube("River Spring", -1.0, all(tex::SPRING), SPRING, sound::STONE) }];
