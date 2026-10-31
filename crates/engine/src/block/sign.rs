//! The note sign (id 158): a wooden board two cells wide whose text and colours the players write (`signs/`). Both
//! cells hold this block; the board is drawn as instanced boxes (`signs/view.rs`), so its own faces only show as its
//! item. Its look borrows the planks layer until the art pack has its own (`docs/ART_HANDOVER.md`).
//! To add a block after this one: a set file of its own like this, joined in `defs.rs` and counted in `BLOCK_COUNT`.

use super::make::{all, machine};
use super::{sound, tex, BlockDef, BlockId};

pub const SIGN: BlockId = 158;

pub const SIGN_COUNT: usize = 1;

/// The rows of `DEFS` from block 158 on, in id order.
pub(super) const SIGN_DEFS: [BlockDef; SIGN_COUNT] =
    [BlockDef { sound: sound::WOOD, ..machine("Sign", true, 0.6, all(tex::PLANKS), SIGN) }];
