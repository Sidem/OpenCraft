//! The belt ejector (id 157, the Underpasses tech): a processor (`factory/process/ejector.rs`) that drops what a belt
//! brings onto the ground. Its look borrows the frame and belt layers until the art pack has its own
//! (`docs/ART_HANDOVER.md`).
//! To add a block after this one: a set file of its own like this, joined in `defs.rs` and counted in `BLOCK_COUNT`.

use super::make::{machine, pillar};
use super::{tex, BlockDef, BlockId};

pub const BELT_EJECTOR: BlockId = 157;

pub const EJECTOR_COUNT: usize = 1;

/// The rows of `DEFS` from block 157 on, in id order.
pub(super) const EJECTOR_DEFS: [BlockDef; EJECTOR_COUNT] =
    [machine("Belt Ejector", true, 0.8, pillar(tex::FRAME, tex::BELT_TOP, tex::FRAME), BELT_EJECTOR)];
