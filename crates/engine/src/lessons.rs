//! What the world remembers the players were taught, one bit each in `Sim::taught` (core state, saved since version 58,
//! set by `Action::LessonTaught` or, for side tips, by `Sim::step_tips`). A bit is never cleared, so a lesson that was
//! taught, waved away or finished by doing never comes back, however the player later tears the factory down.
//!
//! Rules every lesson follows (the design is in `docs/DEV_PLAN.md`, step L1): it shows only for a need the player
//! has right now and near them, one card at a time, with a quiet spell after each; a lesson whose job is already done
//! when it would start never shows; older saves count as taught (`STALL_MASK` on load: those players know). Bits:
//! 0..`CAUSES` the stall lessons (`stall_lesson.rs`, by `efficiency::Cause`), then the need lessons
//! (`need_lesson.rs`: `LOW_POWER_BIT`; `recipe_lesson.rs`: `NO_RECIPE_BIT`; `upgrade_lesson.rs`: `UPGRADE_BIT`; `blueprint_lesson.rs`: `BLUEPRINT_BIT`; `planner_lesson.rs`: `PLANNER_BIT`; `map_lesson.rs`: `MAP_BIT`, the next free one is 13), `SIDE_BIT` on the
//! side tips (`hints::SIDE_HINTS`). A lesson added later is taught in older saves (`KNOWN_BEFORE_68`, `KNOWN_BEFORE_74`, `KNOWN_BEFORE_75`, `KNOWN_BEFORE_76`:
//! bump `SAVE_VERSION` and read it in `Sim::read_state`), as the stall lessons were before 58. To add a lesson: take the
//! next free bit here, a watcher like `stall_lesson.rs`, a case in `api/lessons.rs`.

use crate::factory::efficiency::{Cause, CAUSES};
use crate::hints::SIDE_HINTS;

/// Every stall lesson's bit: bit `cause as u8` is the lesson for that [`Cause`].
pub const STALL_MASK: u32 = (1 << CAUSES) - 1;
/// The low-power lesson (`need_lesson.rs`): a machine near the player works slower for want of power.
pub const LOW_POWER_BIT: u8 = 7;
/// The no-recipe lesson (`recipe_lesson.rs`): a machine near the player has no recipe chosen.
pub const NO_RECIPE_BIT: u8 = 8;
/// The upgrade lesson (`upgrade_lesson.rs`): a machine at full pace with inputs piling up, and kits to upgrade it.
pub const UPGRADE_BIT: u8 = 9;
/// The blueprint walk-through (`blueprint_lesson.rs`): after the blueprint prompt is accepted, up to a blueprint saved.
pub const BLUEPRINT_BIT: u8 = 10;
/// The planner lesson (`planner_lesson.rs`): a planner in the pack and no terraforming site marked yet.
pub const PLANNER_BIT: u8 = 11;
/// The map and pins lesson (`map_lesson.rs`): far from spawn, the world map never opened.
pub const MAP_BIT: u8 = 12;
/// The side tip `k` is bit `SIDE_BIT + k`.
pub const SIDE_BIT: u32 = 16;
/// Lessons added in save version 74 (no recipe, upgrade): a world saved before it counts them as taught.
pub const KNOWN_BEFORE_74: u32 = 1 << NO_RECIPE_BIT | 1 << UPGRADE_BIT;
/// The lesson added in save version 75 (planner): a world saved before it counts it as taught.
pub const KNOWN_BEFORE_75: u32 = 1 << PLANNER_BIT;
/// The lesson added in save version 76 (map and pins): a world saved before it counts it as taught.
pub const KNOWN_BEFORE_76: u32 = 1 << MAP_BIT;
/// Lessons added in save version 68, which a world saved before it counts as taught: its players are past them.
pub const KNOWN_BEFORE_68: u32 = 1 << LOW_POWER_BIT | 1 << (SIDE_BIT as usize + crate::hints::SLAG_SIDE);

const _: () = assert!(CAUSES as u8 <= LOW_POWER_BIT && LOW_POWER_BIT < NO_RECIPE_BIT && NO_RECIPE_BIT < UPGRADE_BIT);
const _: () = assert!(UPGRADE_BIT < BLUEPRINT_BIT && BLUEPRINT_BIT < PLANNER_BIT && PLANNER_BIT < MAP_BIT);
const _: () = assert!((MAP_BIT as u32) < SIDE_BIT);
const _: () = assert!(SIDE_BIT as usize + SIDE_HINTS.len() <= 32);

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
