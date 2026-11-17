//! The merger (id 219): a belt router, the splitter's mirror (`factory/router.rs`). It takes from up to three belts
//! (back, left, right) in turn and puts everything out in front. Its look is drawn as boxes (`factory/router_model.rs`);
//! the block's own faces show only as breaking particles, so it borrows the splitter's frame and its own top layer.
//! To add a block after this one: a set file of its own like this, joined in `defs.rs` and counted in `BLOCK_COUNT`.

use super::make::{machine, pillar};
use super::{tex, BlockDef, BlockId};

pub const MERGER: BlockId = 219;

pub const MERGER_COUNT: usize = 1;

/// The rows of `DEFS` from block 219 on, in id order.
pub(super) const MERGER_DEFS: [BlockDef; MERGER_COUNT] =
    [machine("Merger", false, 0.4, pillar(tex::FRAME, tex::MERGER_TOP, tex::FRAME), MERGER)];
