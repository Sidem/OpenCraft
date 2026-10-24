//! What the world remembers the players were taught, one bit each in `Sim::taught` (core state, saved since version 58,
//! set by `Action::LessonTaught` or, for side tips, by `Sim::step_tips`). A bit is never cleared, so a lesson that was
//! taught, waved away or finished by doing never comes back, however the player later tears the factory down.
//!
//! Rules every lesson follows (the design is in `docs/DEV_PLAN.md`, step L1): it shows only for a need the player
//! has right now and near them, one card at a time, with a quiet spell after each; a lesson whose job is already done
//! when it would start never shows; older saves count as taught (`STALL_MASK` on load: those players know). Bits:
//! 0..`CAUSES` the stall lessons (`stall_lesson.rs`, by `efficiency::Cause`), `SIDE_BIT` on the side tips
//! (`hints::SIDE_HINTS`). To add a lesson: take the next free bit here, a watcher like `stall_lesson.rs`, a card in
//! `web/src/ui/hints.ts`.

use crate::factory::efficiency::{Cause, CAUSES};
use crate::hints::SIDE_HINTS;

/// Every stall lesson's bit: bit `cause as u8` is the lesson for that [`Cause`].
pub const STALL_MASK: u32 = (1 << CAUSES) - 1;
/// The side tip `k` is bit `SIDE_BIT + k`.
pub const SIDE_BIT: u32 = 16;

const _: () = assert!(CAUSES as u32 <= SIDE_BIT && SIDE_BIT as usize + SIDE_HINTS.len() <= 32);

/// The bit for lesson number `bit` (nothing for a number out of range, so a bad action does nothing).
pub fn mask(bit: u8) -> u32 {
    1u32.checked_shl(bit as u32).unwrap_or(0)
}

/// The lesson number of the stall lesson for `cause`.
pub fn stall_bit(cause: Cause) -> u8 {
    cause as u8
}

#[cfg(test)]
mod tests;
