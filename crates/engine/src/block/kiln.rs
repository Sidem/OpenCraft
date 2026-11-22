//! Glass Kiln block (L-COLOUR, id 262): a brick-and-iron burner furnace that melts glass, sand or quartz with a
//! pigment into stained glass (`factory/process/kiln.rs`, `recipes/glass.rs`). Wide block paired with
//! `GLASS_KILN_ITEM` in `item::WIDE_BLOCK_ITEMS`.

use super::make::{machine, pillar};
use super::{sound, tex, BlockDef, BlockId};

/// Burner 1×1×1 glassblowing kiln that melts silica (`GLASS`, `SAND` or `QUARTZ_ORE`) + `Pigment` into stained glass.
pub const GLASS_KILN: BlockId = 262;

pub const KILN_COUNT: usize = 1;

pub(super) const KILN_DEFS: [BlockDef; KILN_COUNT] = [BlockDef {
    sound: sound::STONE,
    ..machine("Glass Kiln", true, 1.2, pillar(tex::SMELTER_SIDE, tex::SMELTER_TOP, tex::SMELTER_TOP), GLASS_KILN)
}];
