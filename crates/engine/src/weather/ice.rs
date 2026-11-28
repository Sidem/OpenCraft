//! Ice (FREEZE): real mineable `block::ICE` blocks forming strictly as a single top layer over still water below
//! freezing, and melting back to water (or disappearing in inventory) with an exponential half-life above 0 °C.
//!
//! - **Freezing:** only still `WATER` whose cell directly above is `AIR` freezes (`should_freeze_cell`), so deep lakes
//!   keep liquid water underneath their single top ice sheet. Inland water (`y > SEA_LEVEL`) freezes at or below
//!   [`FREEZE_TEMP`] (−1 °C); salty sea (`y <= SEA_LEVEL`) freezes only at or below [`SEA_FREEZE_TEMP`] (−10 °C).
//! - **Melting:** any `ICE` block in the world melts back to `WATER` with half-life [`WORLD_MELT_HALF_LIFE`] whenever
//!   local air temperature (`Reading::at`) is above [`MELT_TEMP`] (0 °C), via `sim/timers.rs` (`TimerKind::IceMelt`).
//! - **Inventory melting:** `ICE_ITEM` held by a player whose body stands in air above 0 °C decays with half-life
//!   [`INVENTORY_MELT_HALF_LIFE`] (`melt_inventory_count`, driven deterministically via `Action::MeltInventoryIce`).
//!
//! Pure rules and maths only; world edits go through `sim/timers.rs` and `ice_sweep.rs`.
//! To tune: the temperature thresholds and half-lives below.

use crate::block::{BlockId, AIR, FLOW_1, FLOW_7, WATER};
use crate::math::Rng;
use crate::weather::{Atmosphere, Climate, Reading};
use crate::worldgen::{WorldGen, SEA_LEVEL};

/// Air temperature (hundredths of °C) at or below which still inland water (`y > SEA_LEVEL`) freezes.
pub const FREEZE_TEMP: i32 = -100;
/// Air temperature (hundredths of °C) at or below which still sea water (`y <= SEA_LEVEL`) freezes.
pub const SEA_FREEZE_TEMP: i32 = -1000;
/// Air temperature (hundredths of °C) strictly above which ice melts (0.00 °C).
pub const MELT_TEMP: i32 = 0;
/// Half-life (seconds) of an `ICE` block placed or standing in air above 0 °C.
pub const WORLD_MELT_HALF_LIFE: f64 = 120.0;
/// Half-life (seconds) of `ICE_ITEM` carried in a player's inventory while standing in air above 0 °C.
pub const INVENTORY_MELT_HALF_LIFE: f64 = 120.0;
/// Interval (seconds) between inventory ice-melt checks (`ice_sweep.rs`).
pub const INVENTORY_MELT_STEP_SECS: f64 = 1.0;
/// Minimum seconds a thin `1/8` (`FLOW_1`) puddle stays wet before evaporation can begin, and its half-life after.
pub const PUDDLE_EVAP_MIN_SECS: f64 = 15.0;
pub const PUDDLE_EVAP_HALF_LIFE: f64 = 90.0;

/// Whether still water at height `y` freezes at local air temperature `temp` (hundredths of °C).
#[inline]
pub fn can_freeze(temp: i32, y: i32) -> bool {
    if y <= SEA_LEVEL {
        temp <= SEA_FREEZE_TEMP
    } else {
        temp <= FREEZE_TEMP
    }
}

/// Whether the weather at `(x, y, z)` is cold enough to freeze still water: both the current air temperature and the
/// day's climate mean must be freezing, so a brief chilly dawn on a warm spring day does not snap-freeze ponds.
pub fn freezes_at(air: &Atmosphere, gen: &WorldGen, x: i32, y: i32, z: i32) -> bool {
    let temp = Reading::at(air, gen, x, y, z).temp;
    let day_mean = Climate::at(gen, x, z).day_mean(y, air.tick) * 10;
    can_freeze(temp, y) && can_freeze(day_mean, y)
}

/// Whether ice melts at local air temperature `temp` (hundredths of °C): strictly above 0 °C.
#[inline]
pub fn can_melt(temp: i32) -> bool {
    temp > MELT_TEMP
}

/// Whether the cell holding `block` with `above` directly on top of it at `(x, y, z)` should freeze into `block::ICE`.
/// Strictly top-layer water (`WATER` directly, or `FLOW_7` when borrowing 1/8 from a neighbour in `sim/timers.rs`)
/// directly under `AIR`, so water beneath ice or water never freezes.
pub fn should_freeze_at(
    air: &Atmosphere,
    gen: &WorldGen,
    block: BlockId,
    above: BlockId,
    x: i32,
    y: i32,
    z: i32,
) -> bool {
    (block == WATER || block == FLOW_7) && above == AIR && freezes_at(air, gen, x, y, z)
}

/// Whether a thin `1/8` (`FLOW_1`) puddle above sea level with open `AIR` above it evaporates at `temp` (> 0 °C).
#[inline]
pub fn can_evaporate_puddle(block: BlockId, above: BlockId, y: i32, temp: i32) -> bool {
    y > SEA_LEVEL && block == FLOW_1 && above == AIR && can_melt(temp)
}

/// How many of `count` ice items melt during an interval of `dt_secs` seconds under exponential decay with `half_life`.
/// Memoryless: each item independently melts with probability `1 - 2^(-dt_secs / half_life)`.
pub fn melt_inventory_count(count: u32, half_life: f64, dt_secs: f64, rng: &mut Rng) -> u32 {
    if count == 0 || half_life <= 0.0 || dt_secs <= 0.0 {
        return 0;
    }
    let p = 1.0 - 2.0_f64.powf(-dt_secs / half_life);
    let mut melted = 0;
    for _ in 0..count {
        if rng.next_f64() < p {
            melted += 1;
        }
    }
    melted
}

#[cfg(test)]
mod tests;
