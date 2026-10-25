//! The coal miner (id 143): the drill that comes before electricity. It is a `Miner` machine (`factory/miner.rs`) that
//! burns fuel instead of drawing power (`factory/miner_burner.rs`), so it has no tier family and no upgrades.
//! Its look borrows the smelter's layers until the art pack has its own (`docs/ART_HANDOVER.md`).
//! To add a block after this one: a set file of its own like this, joined in `defs.rs` and counted in `BLOCK_COUNT`.

use super::make::{machine, pillar};
use super::{tex, BlockDef, BlockId};

pub const COAL_MINER: BlockId = 143;

pub const COAL_MINER_COUNT: usize = 1;

/// The rows of `DEFS` from block 143 on, in id order.
pub(super) const COAL_MINER_DEFS: [BlockDef; COAL_MINER_COUNT] =
    [machine("Coal Miner", true, 0.8, pillar(tex::SMELTER_SIDE, tex::MINER_TOP, tex::FRAME), COAL_MINER)];
