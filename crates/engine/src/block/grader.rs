//! The Site Grader's block (id 272): a wide block paired with `SITE_GRADER_ITEM` in
//! `item::WIDE_BLOCK_ITEMS`. What it does: `factory/process/grader/`.

use super::make::{machine, pillar};
use super::{tex, BlockDef, BlockId};

/// A telescoping bucket-wheel site grader that levels a horizontal pad (cut, fill and pave).
pub const SITE_GRADER: BlockId = 272;

pub const GRADER_COUNT: usize = 1;

pub(super) const GRADER_DEFS: [BlockDef; GRADER_COUNT] =
    [machine("Site Grader", true, 1.8, pillar(tex::GRADER_LOUVRE, tex::GRADER_YELLOW, tex::GRADER_TREAD), SITE_GRADER)];
