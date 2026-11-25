//! Depletion forecast (`F-DEPL`): computes remaining reserve percentage, units, active miner count and time to dry
//! for any worked deposit (`Factory::deposit_forecast`). Pure read-only query shared by the miner panel/HUD
//! (`miner.rs`) and the map tooltips (`minimap/marks.rs`, `minimap/describe.rs`).

use crate::deposits::DepositKey;
use crate::math::IVec3;

use super::describe::{fmt_duration, fmt_int};
use super::Factory;

/// Remaining percentage at or below which a deposit is flagged as running low.
pub const LOW_PCT: u8 = 15;
/// Remaining seconds at or below which a worked deposit is flagged as running low (15 minutes).
pub const LOW_SECONDS: u64 = 15 * 60;

/// Live depletion forecast for a deposit that has at least one miner on it.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct DepositForecast {
    /// Whole percent of original blocks remaining (0..=100).
    pub pct_left: u8,
    /// Estimated ore units remaining in the deposit.
    pub units_left: u64,
    /// Estimated seconds until worked out at the combined live draw of all miners on it (`None` if idle).
    pub secs_left: Option<u64>,
    /// Total miners standing on this deposit (running or idle).
    pub miners: u16,
}

impl DepositForecast {
    /// Whether the deposit is running low enough that the player should plan the next one.
    pub fn is_low(self) -> bool {
        self.pct_left <= LOW_PCT || self.secs_left.is_some_and(|s| s <= LOW_SECONDS)
    }

    /// Short suffix for titles, e.g. `"~14m left"` or `"12% left"`.
    pub fn short_tag(self) -> String {
        match self.secs_left {
            Some(s) => format!("~{} left", fmt_duration(s as f64)),
            None => format!("{}% left", self.pct_left),
        }
    }

    /// Full forecast line for the miner panel, HUD and map tooltip.
    pub fn detail_line(self) -> String {
        let count = if self.miners == 1 { "1 miner".into() } else { format!("{} miners", self.miners) };
        let base = match self.secs_left {
            Some(s) => format!(
                "{} units ({}%) left · ~{} at current draw ({count})",
                fmt_int(self.units_left),
                self.pct_left,
                fmt_duration(s as f64)
            ),
            None => format!("{} units ({}%) left · idle ({count})", fmt_int(self.units_left), self.pct_left),
        };
        if self.is_low() {
            format!("{base} · running low — plan your next deposit")
        } else {
            base
        }
    }
}

impl Factory {
    /// Depletion forecast for `key` if it is tracked, not yet exhausted, and has at least one miner on it.
    pub fn deposit_forecast(&self, key: &DepositKey) -> Option<DepositForecast> {
        let st = self.deposits.get(key)?;
        if st.exhausted() {
            return None;
        }
        let mut miners = 0u16;
        let mut total_draw = 0.0f64;
        for m in &self.miners {
            if m.deposit == Some(*key) {
                miners = miners.saturating_add(1);
                total_draw += m.draw_rate;
            }
        }
        if miners == 0 {
            return None;
        }
        let units = st.remaining_units();
        let pct = ((st.remaining_blocks as u64 * 100) / st.initial_blocks.max(1) as u64).min(100) as u8;
        let secs_left = (total_draw > 0.01).then(|| (units / total_draw).round().max(1.0) as u64);
        Some(DepositForecast { pct_left: pct, units_left: units as u64, secs_left, miners })
    }

    /// Depletion forecast for the deposit that the miner at `pos` is standing on, if any.
    pub fn miner_forecast(&self, pos: IVec3) -> Option<DepositForecast> {
        let m = self.miners.iter().find(|m| m.pos == pos)?;
        self.deposit_forecast(&m.deposit?)
    }
}
