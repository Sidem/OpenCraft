//! The forester (id 159, the Forestry tech): a processor (`factory/process/forester.rs`) that fells the trees on a patch
//! in front of it and replants them. Its look borrows the log and frame layers until the art pack has its own
//! (`docs/ART_HANDOVER.md`).
//! To add a block after this one: a set file of its own like this, joined in `defs.rs` and counted in `BLOCK_COUNT`.

use super::make::{machine, pillar};
use super::{sound, tex, BlockDef, BlockId};

pub const FORESTER: BlockId = 159;

pub const FORESTER_COUNT: usize = 1;

/// The rows of `DEFS` from block 159 on, in id order.
pub(super) const FORESTER_DEFS: [BlockDef; FORESTER_COUNT] = [BlockDef {
    sound: sound::WOOD,
    ..machine("Forester", true, 1.0, pillar(tex::LOG_SIDE, tex::FRAME, tex::FRAME), FORESTER)
}];
