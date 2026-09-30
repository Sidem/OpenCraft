//! Time of day, derived from the core tick: a day lasts [`DAY_TICKS`] and a new world starts in the
//! morning. Because the tick is core state, the time is saved with the world, identical for every
//! co-op player, and needs no state of its own; the world pauses while the game is closed, and so
//! does the sun. The host turns the time into sky colours, sun and daylight (`web/src/render/sky.ts`).

use crate::TICK_RATE;

/// Ticks in one day: 20 minutes.
pub const DAY_TICKS: u64 = 20 * 60 * TICK_RATE as u64;
/// Where tick 0 falls in the day: 7:00.
const START: u64 = DAY_TICKS * 7 / 24;

/// The time of day at `tick` as a fraction: 0 midnight, 0.25 sunrise, 0.5 noon, 0.75 sunset.
pub fn time_of_day(tick: u64) -> f64 {
    ((tick + START) % DAY_TICKS) as f64 / DAY_TICKS as f64
}

/// Sunlight at `tick` in thousandths: 0 at night, 1000 at noon, a parabola between sunrise (6:00) and sunset
/// (18:00). Integer maths only, so every platform agrees (solar panels: `factory/process/solar.rs`).
pub fn sunlight(tick: u64) -> u32 {
    let (rise, len) = (DAY_TICKS / 4, DAY_TICKS / 2);
    let x = (tick + START) % DAY_TICKS;
    if x < rise || x >= rise + len {
        return 0;
    }
    let x = x - rise;
    (4000 * x * (len - x) / (len * len)) as u32
}

/// Which day `tick` falls on, counting from 1.
pub fn day_number(tick: u64) -> u64 {
    (tick + START) / DAY_TICKS + 1
}

#[cfg(test)]
mod tests;
