//! Factory analytics, for the analytics hub screen and the machines' efficiency readouts: the power the grids could
//! give, gave and were asked for (and from which sources, to which machines), how much of each item the machines made
//! and used up a minute, and how much of its full pace each machine ran at.
//!
//! Presentation only (DEV_PLAN 3.4): `Game` feeds it after every core tick (`record`, and `Produced` / `Consumed`
//! events from `events.rs`), it reads the factory and never changes it, and it is not saved: history starts when the
//! world is opened. Time is counted in core ticks, so the graphs follow game time, not the wall clock.
//!
//! - `series.rs`: a value over time at two resolutions. `power.rs`: the power rows. `machines.rs`: each machine's
//!   running average. `groups.rs`: the family each item belongs to.
//! - Every [`SAMPLE_TICKS`] ticks it reads the power and every machine; every second it closes a bucket: the average
//!   power, and each item's trailing rates (items a minute over the last [`RATE_SECS`] seconds, made and used).
//!
//! To graph another quantity: a `Series` here or in `power.rs`, pushed in `close_second`, and a row in its export.

mod groups;
mod machines;
mod power;
mod series;
#[cfg(test)]
mod tests;

use crate::factory::power_mix::{Source, ALL as SOURCE_LIST};
use crate::factory::Factory;
use crate::item::ItemId;
use crate::TICK_RATE;

pub use groups::GROUPS;
#[cfg(test)]
pub(crate) use machines::kind_at;
pub use machines::{Machines, KIND_FIELDS, TYPE_FIELDS};
use power::PowerHistory;
use series::Series;

/// Ticks between readings of the power grids and the machines (6 a second).
const SAMPLE_TICKS: u32 = 10;
/// Seconds an item's rate is averaged over (items come in lumps, so a single second would jump about).
const RATE_SECS: usize = 10;

/// One flow of an item (made, or used up): this second's count, the last seconds' counts, and the rate over time.
struct Flow {
    now: u32,
    recent: [u32; RATE_SECS],
    total: u32,
    series: Series,
}

impl Flow {
    fn after(secs: u32) -> Flow {
        Flow { now: 0, recent: [0; RATE_SECS], total: 0, series: Series::after(secs) }
    }

    /// Closes second number `secs`: its count joins the ring and the trailing rate a minute is pushed.
    fn close(&mut self, secs: u32) {
        let count = std::mem::take(&mut self.now);
        self.recent[secs as usize % RATE_SECS] = count;
        self.total = self.total.saturating_add(count);
        self.series.push((self.recent.iter().sum::<u32>() * 60 / RATE_SECS as u32) as f32);
    }
}

struct Item {
    id: ItemId,
    group: u8,
    made: Flow,
    used: Flow,
}

#[derive(Default)]
pub struct Analytics {
    pub machines: Machines,
    power: PowerHistory,
    items: Vec<Item>,
    /// Ticks into the current second, and seconds recorded so far.
    ticks: u32,
    secs: u32,
}

impl Analytics {
    /// Counts `count` of `item` made this second.
    pub fn produced(&mut self, item: ItemId, count: u32) {
        let i = self.item_index(item);
        self.items[i].made.now += count;
    }

    /// Counts `count` of `item` used up this second.
    pub fn consumed(&mut self, item: ItemId, count: u32) {
        let i = self.item_index(item);
        self.items[i].used.now += count;
    }

    /// Called after each core tick: reads the factory now and then, and closes a second every `TICK_RATE` ticks.
    pub fn record(&mut self, factory: &Factory) {
        if self.ticks.is_multiple_of(SAMPLE_TICKS) {
            self.power.sample(factory, self.secs);
            self.machines.sample(factory);
        }
        self.ticks += 1;
        if self.ticks >= TICK_RATE {
            self.close_second();
        }
    }

    fn close_second(&mut self) {
        self.ticks = 0;
        self.power.close_second();
        for it in &mut self.items {
            it.made.close(self.secs);
            it.used.close(self.secs);
        }
        self.secs += 1;
    }

    /// The position of `item`'s record, added on first sight with a past of zeros so every row covers the same time.
    fn item_index(&mut self, item: ItemId) -> usize {
        if let Some(i) = self.items.iter().position(|it| it.id == item) {
            return i;
        }
        let (group, secs) = (groups::group_of(item), self.secs);
        self.items.push(Item { id: item, group, made: Flow::after(secs), used: Flow::after(secs) });
        self.items.len() - 1
    }

    /// The items with graph rows, as raw ids (`export_items` has two rows each, in this order).
    pub fn item_ids(&self) -> Vec<u32> {
        self.items.iter().map(|it| it.id.0 as u32).collect()
    }

    /// The group of each item in `item_ids` order (an index into [`GROUPS`]).
    pub fn item_groups(&self) -> Vec<u32> {
        self.items.iter().map(|it| it.group as u32).collect()
    }

    /// Per item, in `item_ids` order: `[made a minute, used a minute, made in total, used in total]` (the totals count
    /// since the world was opened).
    pub fn item_now(&self) -> Vec<f32> {
        let flows =
            |it: &Item| [it.made.series.latest(), it.used.series.latest(), it.made.total as f32, it.used.total as f32];
        self.items.iter().flat_map(flows).collect()
    }

    /// The last `secs` seconds of every item's rates as `[points, rows, then points values per row, oldest first]`;
    /// the rows are made, then used, for each item in `item_ids` order.
    pub fn export_items(&self, secs: u32, points: usize) -> Vec<f32> {
        export(self.items.iter().flat_map(|it| [&it.made.series, &it.used.series]), secs, points)
    }

    /// The same for the power rows (kW): could give, given, asked for, then one per source (`power_sources` order),
    /// then one per kind of machine in `power_users` order.
    pub fn export_power(&self, secs: u32, points: usize) -> Vec<f32> {
        export(self.power.rows(), secs, points)
    }

    /// The newest value of every power row, in the same order.
    pub fn power_now(&self) -> Vec<f32> {
        self.power.rows().map(Series::latest).collect()
    }

    /// What each power source could give right now (kW), in source order.
    pub fn power_could(&self) -> Vec<u32> {
        self.power.could().to_vec()
    }

    /// The block of each machine-kind row after the totals and sources in `export_power`.
    pub fn power_users(&self) -> Vec<u32> {
        self.power.user_blocks()
    }
}

/// The names, one a line, of the power sources in row order.
pub fn power_sources() -> String {
    SOURCE_LIST.map(Source::name).join("\n")
}

/// The colour of each power source, as `0xRRGGBB`.
pub fn power_source_colours() -> Vec<u32> {
    SOURCE_LIST.iter().map(|s| s.colour()).collect()
}

/// `[points, rows, then points values per row]` for `rows`, each averaged into at most `points` values over `secs`.
fn export<'a>(rows: impl Iterator<Item = &'a Series>, secs: u32, points: usize) -> Vec<f32> {
    let rows: Vec<&Series> = rows.collect();
    let mut out = vec![0.0, rows.len() as f32];
    let mut got = 0;
    for s in &rows {
        got = s.window(secs, points, &mut out);
    }
    out[0] = got as f32;
    out
}
