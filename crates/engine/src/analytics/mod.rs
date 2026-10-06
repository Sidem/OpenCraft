//! Electricity and production analytics, for the analytics screen and the machines' efficiency readouts. The
//! screen graphs the power the grids could give, gave and were asked for, and how much of each item the machines
//! made a minute, over the last minutes or hours; a machine's efficiency is how much of its full pace it ran at.
//!
//! Presentation only (DEV_PLAN 3.4): `Game` feeds it after every core tick (`record`, and `Produced` events from
//! `events.rs`), it reads the factory and never changes it, and it is not saved: history starts when the world is
//! opened. Time is counted in core ticks, so the graphs follow game time, not the wall clock.
//!
//! - `series.rs`: a value over time at two resolutions. `machines.rs`: each machine's running average.
//! - Every [`SAMPLE_TICKS`] ticks it reads the power totals and every machine; every second it closes a bucket: the
//!   average power, and each item's trailing rate (items a minute over the last [`RATE_SECS`] seconds).
//!
//! To graph another quantity: a `Series` here, pushed in `close_second`, and a row in `export`.

mod machines;
mod series;
#[cfg(test)]
mod tests;

use crate::factory::efficiency::PowerTotals;
use crate::factory::Factory;
use crate::item::ItemId;
use crate::TICK_RATE;

pub use machines::Machines;
use series::Series;

/// Ticks between readings of the power grids and the machines (6 a second).
const SAMPLE_TICKS: u32 = 10;
/// Seconds an item's rate is averaged over (items come in lumps, so a single second would jump about).
const RATE_SECS: usize = 10;
/// Rows before the items in `export`: power the grids could give, power used, power asked for.
pub const POWER_ROWS: usize = 3;

struct Item {
    id: ItemId,
    /// Made this second, and in each of the last seconds (a ring).
    now: u32,
    recent: [u32; RATE_SECS],
    series: Series,
}

#[derive(Default)]
pub struct Analytics {
    pub machines: Machines,
    power: [Series; POWER_ROWS],
    /// Sum and count of this second's power readings (capacity, used, demanded).
    power_sum: [f32; POWER_ROWS],
    power_n: u32,
    items: Vec<Item>,
    /// Ticks into the current second, and seconds recorded so far.
    ticks: u32,
    secs: u32,
}

impl Analytics {
    /// Counts `count` of `item` made this second.
    pub fn produced(&mut self, item: ItemId, count: u32) {
        let i = match self.items.iter().position(|it| it.id == item) {
            Some(i) => i,
            None => {
                // Its past reads as zeros, so every row covers the same time.
                self.items.push(Item { id: item, now: 0, recent: [0; RATE_SECS], series: Series::after(self.secs) });
                self.items.len() - 1
            }
        };
        self.items[i].now += count;
    }

    /// Called after each core tick: reads the factory now and then, and closes a second every `TICK_RATE` ticks.
    pub fn record(&mut self, factory: &Factory) {
        if self.ticks.is_multiple_of(SAMPLE_TICKS) {
            let PowerTotals { capacity, supply, demand } = factory.power_totals();
            for (sum, v) in self.power_sum.iter_mut().zip([capacity, supply, demand]) {
                *sum += v as f32;
            }
            self.power_n += 1;
            self.machines.sample(factory);
        }
        self.ticks += 1;
        if self.ticks >= TICK_RATE {
            self.close_second();
        }
    }

    fn close_second(&mut self) {
        self.ticks = 0;
        let n = self.power_n.max(1) as f32;
        for (series, sum) in self.power.iter_mut().zip(self.power_sum) {
            series.push(sum / n);
        }
        (self.power_sum, self.power_n) = ([0.0; POWER_ROWS], 0);
        let slot = self.secs as usize % RATE_SECS;
        for it in &mut self.items {
            it.recent[slot] = std::mem::take(&mut it.now);
            // Items a minute: the last `RATE_SECS` seconds' count, scaled up.
            it.series.push((it.recent.iter().sum::<u32>() * 60 / RATE_SECS as u32) as f32);
        }
        self.secs += 1;
    }

    /// The item of each row after the power rows, as raw ids.
    pub fn item_ids(&self) -> Vec<u32> {
        self.items.iter().map(|it| it.id.0 as u32).collect()
    }

    /// The last `secs` seconds of every row (power in kW, then each item's rate a minute), as `[points, rows, then
    /// `points` values per row, oldest first]`. `points` is at most what was asked and at most the values the window
    /// holds; values before the history begins are `NAN`.
    pub fn export(&self, secs: u32, points: usize) -> Vec<f32> {
        let rows: Vec<&Series> = self.power.iter().chain(self.items.iter().map(|it| &it.series)).collect();
        let mut out = vec![0.0, rows.len() as f32];
        let mut got = 0;
        for s in &rows {
            got = s.window(secs, points, &mut out);
        }
        out[0] = got as f32;
        out
    }

    /// The newest value of every row, in the same order as `export`.
    pub fn latest(&self) -> Vec<f32> {
        self.power.iter().chain(self.items.iter().map(|it| &it.series)).map(Series::latest).collect()
    }
}
