//! Seismic prospecting block (F-SEIS, id 261): the Seismic Thumper (`THUMPER`, wide block paired with `THUMPER_ITEM`
//! in `item::WIDE_BLOCK_ITEMS`, tiered Mk1–Mk3 in `factory/tiers.rs`).
//!
//! To add another seismic block: append its id, its `THUMPER_DEFS` row, its `WIDE_BLOCK_ITEMS` pair and bump
//! `THUMPER_COUNT`.

use super::make::{machine, pillar};
use super::{tex, BlockDef, BlockId};

/// Powered 2×2×2 seismic thumper that sounds a 64-block vertical cross-section down to bedrock (`factory/process/thumper.rs`).
pub const THUMPER: BlockId = 261;

pub const THUMPER_COUNT: usize = 1;

/// Machine block is `Render::None` (never meshed as a terrain cube), so its cube-face fallback layers stay under 512;
/// its placed model and wide-block item icons use dedicated `tex::THUMPER_*` layers.
pub(super) const THUMPER_DEFS: [BlockDef; THUMPER_COUNT] =
    [machine("Seismic Thumper", true, 1.6, pillar(tex::CHEM_SIDE, tex::CHEM_TOP, tex::FRAME), THUMPER)];
