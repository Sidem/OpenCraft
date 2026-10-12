//! Power history for the analytics screen: the grids' totals (could give, gave, asked for), the kW each kind of source
//! gave (`factory/power_mix.rs`) and the kW each kind of machine asked for, as one `Series` each. Read every
//! [`SAMPLE_TICKS`](super::SAMPLE_TICKS) ticks, averaged over the second by `close_second`.
//!
//! Rows (the order of [`PowerHistory::rows`]): [`TOTALS`] totals, then one per `Source`, then one per kind of machine
//! seen asking for power (`users`, in the order first seen).

use crate::block::BlockId;
use crate::factory::efficiency::PowerTotals;
use crate::factory::power_mix::{Mix, SOURCES};
use crate::factory::Factory;

use super::series::Series;

/// Rows before the sources: power the grids could give, power given, power asked for.
pub const TOTALS: usize = 3;

struct User {
    block: BlockId,
    /// This second's readings added up.
    sum: f32,
    series: Series,
}

#[derive(Default)]
pub struct PowerHistory {
    totals: [Series; TOTALS],
    sources: [Series; SOURCES],
    sums: [f32; TOTALS + SOURCES],
    n: u32,
    /// What each kind of source could have given at the latest reading.
    could: [u32; SOURCES],
    users: Vec<User>,
}

impl PowerHistory {
    /// Reads the factory's power now; `secs` is how long the world has been open (for a new user's blank past).
    pub fn sample(&mut self, factory: &Factory, secs: u32) {
        let PowerTotals { capacity, supply, demand } = factory.power_totals();
        let mix: [Mix; SOURCES] = factory.power_mix();
        let readings = [capacity, supply, demand].into_iter().chain(mix.iter().map(|m| m.given));
        for (sum, v) in self.sums.iter_mut().zip(readings) {
            *sum += v as f32;
        }
        self.could = mix.map(|m| m.could);
        factory.power_uses(|block, kw| {
            let i = match self.users.iter().position(|u| u.block == block) {
                Some(i) => i,
                None => {
                    self.users.push(User { block, sum: 0.0, series: Series::after(secs) });
                    self.users.len() - 1
                }
            };
            self.users[i].sum += kw as f32;
        });
        self.n += 1;
    }

    /// Ends the second: pushes each row's average reading.
    pub fn close_second(&mut self) {
        let n = self.n.max(1) as f32;
        for (series, sum) in self.totals.iter_mut().chain(self.sources.iter_mut()).zip(self.sums) {
            series.push(sum / n);
        }
        for u in &mut self.users {
            u.series.push(std::mem::take(&mut u.sum) / n);
        }
        (self.sums, self.n) = ([0.0; TOTALS + SOURCES], 0);
    }

    /// Every row, in order.
    pub fn rows(&self) -> impl Iterator<Item = &Series> {
        self.totals.iter().chain(self.sources.iter()).chain(self.users.iter().map(|u| &u.series))
    }

    /// The block of each user row (after the totals and sources), as raw ids.
    pub fn user_blocks(&self) -> Vec<u32> {
        self.users.iter().map(|u| u.block as u32).collect()
    }

    /// What each kind of source could give right now.
    pub fn could(&self) -> [u32; SOURCES] {
        self.could
    }
}
