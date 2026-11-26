//! The space elevator's block (M14A, id 270): the Elevator Anchor (`ELEVATOR_ANCHOR`, a wide block paired with
//! `ELEVATOR_ANCHOR_ITEM` in `item::WIDE_BLOCK_ITEMS`). What it does: `elevator/`; its machine row:
//! `factory/process/anchor.rs`.
//!
//! To add another elevator block: append its id, its `ELEVATOR_DEFS` row, its `WIDE_BLOCK_ITEMS` pair and bump
//! `ELEVATOR_COUNT`.

use super::make::{machine, pillar};
use super::{tex, BlockDef, BlockId};

/// The anchor the tether rises from, beside a grown town's depot.
pub const ELEVATOR_ANCHOR: BlockId = 270;

pub const ELEVATOR_COUNT: usize = 1;

pub(super) const ELEVATOR_DEFS: [BlockDef; ELEVATOR_COUNT] =
    [machine("Elevator Anchor", true, 3.0, pillar(tex::CONCRETE, tex::STEEL, tex::FRAME), ELEVATOR_ANCHOR)];
