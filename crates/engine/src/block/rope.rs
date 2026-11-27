//! The rope's block (id 273, GEAR-ROPE): a wide block paired with `ROPE_ITEM` in `item::WIDE_BLOCK_ITEMS`. A strand
//! hangs down from where it was tied and is climbed like a ladder (`player::climbable`); it drops nothing by itself,
//! because a coil unrolls several cells and the strand is taken up whole (`action/rope.rs`).

use super::make::cube;
use super::{sound, tex, BlockDef, BlockId, Render};

/// One cell of a hanging rope.
pub const ROPE: BlockId = 273;

pub const ROPE_COUNT: usize = 1;

pub(super) const ROPE_DEFS: [BlockDef; ROPE_COUNT] = [BlockDef {
    render: Render::Plant,
    solid: false,
    drop: super::AIR,
    ..cube("Rope", 0.2, [tex::ROPE; 6], ROPE, sound::LEAVES)
}];
