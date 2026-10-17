//! The ruins' blocks (generator version 9, `worldgen/ruins.rs`): rusted plating, machine husks and girders, left by
//! the last factory. They are generated, never held (`placeable: false`) and drop nothing as blocks: breaking one
//! leaves scrap metal and now and then a circuit (`sim/scrap.rs`). To add a ruin block: an id here, a `DEFS` row and
//! a line in `is_rusted`.

use super::BlockId;

pub const RUSTED_PLATING: BlockId = 124;
pub const RUSTED_MACHINE: BlockId = 125;
pub const RUSTED_GIRDER: BlockId = 126;

/// Any block of the ruins: what breaks into scrap.
#[inline]
pub fn is_rusted(b: BlockId) -> bool {
    matches!(b, RUSTED_PLATING | RUSTED_MACHINE | RUSTED_GIRDER)
}
