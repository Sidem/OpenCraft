//! Natural and placed ice (id 274, FREEZE): a solid, translucent block paired with `ICE_ITEM` in
//! `item::WIDE_BLOCK_ITEMS`. Forms as a single top layer over still water below freezing (`ice_sweep.rs`,
//! `weather/ice.rs`), can be mined and placed anywhere, and melts back to water with a half-life above 0 °C
//! (`sim/timers.rs` `TimerKind::IceMelt`).

use super::make::{all, cube};
use super::{sound, tex, BlockDef, BlockId, Render};

/// One block of ice (mineable, translucent, melts above 0 °C).
pub const ICE: BlockId = 274;

pub const ICE_COUNT: usize = 1;

pub(super) const ICE_DEFS: [BlockDef; ICE_COUNT] =
    [BlockDef { render: Render::Translucent, ..cube("Ice", 0.5, all(tex::ICE), ICE, sound::STONE) }];
