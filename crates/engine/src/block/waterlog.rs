//! Which blocks can hold water alongside themselves (`world/waterlog.rs`, `sim/water.rs`), and the
//! conversion between a cell's sparse waterlog byte (`0` dry, `1..=7` `FLOW_1..=FLOW_7`, `8` `WATER`)
//! and its effective `BlockId`.
//!
//! To allow another block to be waterlogged: append its `BlockId` to [`WATERLOGGABLE`], nothing more.

use super::{
    BlockId, AIR, BELT, FAST_BELT, FLOW_1, FLOW_7, LIFT, RAMP_DOWN, RAMP_UP, UNDERPASS_IN, UNDERPASS_OUT, WATER,
};

/// Blocks that can hold water alongside themselves: belts and their tiers/variants.
/// To waterlog another block, append its id here.
const WATERLOGGABLE: &[BlockId] = &[BELT, FAST_BELT, RAMP_UP, RAMP_DOWN, LIFT, UNDERPASS_IN, UNDERPASS_OUT];

/// Whether `id` can hold water in the same cell (`world/waterlog.rs`).
#[inline]
pub const fn waterloggable(id: BlockId) -> bool {
    let mut i = 0;
    while i < WATERLOGGABLE.len() {
        if WATERLOGGABLE[i] == id {
            return true;
        }
        i += 1;
    }
    false
}

/// Waterlog level representing a full water source (`WATER`); `1..=7` represent `FLOW_1..=FLOW_7`, `0` is dry.
pub const WET_SOURCE: u8 = 8;

/// Converts a liquid block to its waterlog level (`8` for `WATER`, `1..=7` for `FLOW_1..=FLOW_7`, `0` otherwise).
#[inline]
pub const fn wet_of(id: BlockId) -> u8 {
    if id == WATER {
        WET_SOURCE
    } else if id >= FLOW_1 && id <= FLOW_7 {
        (id - FLOW_1 + 1) as u8
    } else {
        0
    }
}

/// Converts a waterlog level (`1..=8`) back to its liquid block (`AIR` for `0`).
#[inline]
pub const fn block_of_wet(level: u8) -> BlockId {
    if level == WET_SOURCE {
        WATER
    } else if level >= 1 && level <= 7 {
        FLOW_1 + (level as BlockId - 1)
    } else {
        AIR
    }
}
