//! Power history for the analytics screen: the grids' totals (could give, gave, asked for), the kW each kind of source
//! gave (`factory/power_mix.rs`) and the kW each kind of machine asked for, as one `Series` each, for the whole factory
//! and again for each grid with a machine on it (up to [`MAX_GRIDS`], first seen first). Read every
//! [`SAMPLE_TICKS`](super::SAMPLE_TICKS) ticks, averaged over the second by `close_second`.
//!
//! Rows (the order of `rows`): [`TOTALS`] totals, then one per `Source`, then one per kind of machine seen asking for
//! power (`users`, in the order first seen). A grid is known by its `GridReading::key` (where its oldest pole stands);
//! one that loses its last machine, or whose oldest pole goes, starts a new history.

use crate::block::BlockId;
use crate::factory::efficiency::PowerTotals;
use crate::factory::power_mix::{GridReading, Mix, SOURCES};
use crate::factory::Factory;

use super::series::Series;

/// Rows before the sources: power the grids could give, power given, power asked for.
pub const TOTALS: usize = 3;
/// The most grids that keep a history of their own (each costs as much memory as the whole factory's rows).
pub const MAX_GRIDS: usize = 8;
/// Numbers per grid in [`PowerHistory::grids`].
pub const GRID_FIELDS: usize = 9;

#[derive(Default)]
pub struct PowerHistory {
    all: Rows,
    grids: Vec<Grid>,
}

impl PowerHistory {
    /// Reads the factory's power now; `secs` is how long the world has been open (for a new row's blank past).
    pub fn sample(&mut self, factory: &Factory, secs: u32) {
        let PowerTotals { capacity, supply, demand } = factory.power_totals();
        self.all.add([capacity, supply, demand], &factory.power_mix());
        self.grids.iter_mut().for_each(|g| g.seen = false);
        // Which tracked grid each of the factory's grids is, by its number this relink.
        let mut tracked: Vec<Option<usize>> = Vec::new();
        for r in factory.power_grids() {
            let at = match self.grids.iter().position(|g| g.now.key == r.key) {
                Some(at) => at,
                None if self.grids.len() < MAX_GRIDS => {
                    self.grids.push(Grid { now: r, rows: Rows::after(secs), seen: false });
                    self.grids.len() - 1
                }
                None => continue,
            };
            let g = &mut self.grids[at];
            (g.now, g.seen) = (r, true);
            g.rows.add([r.capacity, r.supply, r.demand], &r.mix);
            tracked.resize(tracked.len().max(r.grid + 1), None);
            tracked[r.grid] = Some(at);
        }
        let grids = &mut self.grids;
        let all = &mut self.all;
        factory.power_uses(|grid, block, kw| {
            all.add_use(block, kw, secs);
            if let Some(&Some(at)) = tracked.get(grid) {
                grids[at].rows.add_use(block, kw, secs);
            }
        });
        // Grids gone since the last reading lose their history (the tracked numbers above are by position).
        self.grids.retain(|g| g.seen);
    }

    /// Ends the second: pushes each row's average reading.
    pub fn close_second(&mut self) {
        self.all.close_second();
        self.grids.iter_mut().for_each(|g| g.rows.close_second());
    }

    /// Every row of the whole factory (`grid` -1) or of tracked grid `grid` (an index into `grids`), in order; empty
    /// for a grid no longer tracked.
    pub fn rows(&self, grid: i32) -> Vec<&Series> {
        self.scope(grid).map_or_else(Vec::new, Rows::rows)
    }

    /// The block of each user row (after the totals and sources), as raw ids.
    pub fn user_blocks(&self, grid: i32) -> Vec<u32> {
        self.scope(grid).map_or_else(Vec::new, |r| r.users.iter().map(|u| u.block as u32).collect())
    }

    /// What each kind of source could give right now.
    pub fn could(&self, grid: i32) -> [u32; SOURCES] {
        self.scope(grid).map_or([0; SOURCES], |r| r.could)
    }

    /// The tracked grids at the latest reading, [`GRID_FIELDS`] numbers each: `[x, y, z` of the oldest pole (its key),
    /// `colour as 0xRRGGBB, poles, machines, could give, gave, asked for]` (kW, last tick).
    pub fn grids(&self) -> Vec<i32> {
        let mut out = Vec::with_capacity(self.grids.len() * GRID_FIELDS);
        for g in &self.grids {
            let r = &g.now;
            let n = |v: u32| v as i32;
            out.extend_from_slice(&[r.key.x, r.key.y, r.key.z, n(r.rgb), n(r.poles), n(r.machines)]);
            out.extend_from_slice(&[n(r.capacity), n(r.supply), n(r.demand)]);
        }
        out
    }

    /// The tracked grids' colour names, one a line, in `grids` order.
    pub fn grid_names(&self) -> String {
        self.grids.iter().map(|g| g.now.colour).collect::<Vec<_>>().join("\n")
    }

    fn scope(&self, grid: i32) -> Option<&Rows> {
        match usize::try_from(grid) {
            Ok(i) => self.grids.get(i).map(|g| &g.rows),
            Err(_) => Some(&self.all),
        }
    }
}

/// One grid's latest reading and its rows.
struct Grid {
    now: GridReading,
    rows: Rows,
    seen: bool,
}

/// The rows of one scope (the whole factory or one grid).
#[derive(Default)]
struct Rows {
    totals: [Series; TOTALS],
    sources: [Series; SOURCES],
    sums: [f32; TOTALS + SOURCES],
    n: u32,
    /// What each kind of source could have given at the latest reading.
    could: [u32; SOURCES],
    users: Vec<User>,
}

struct User {
    block: BlockId,
    /// This second's readings added up.
    sum: f32,
    series: Series,
}

impl Rows {
    /// Rows that start `secs` into the world, their past reading as nothing (a grid built later).
    fn after(secs: u32) -> Rows {
        Rows {
            totals: std::array::from_fn(|_| Series::after(secs)),
            sources: std::array::from_fn(|_| Series::after(secs)),
            ..Rows::default()
        }
    }

    fn add(&mut self, totals: [u32; TOTALS], mix: &[Mix; SOURCES]) {
        let readings = totals.into_iter().chain(mix.iter().map(|m| m.given));
        for (sum, v) in self.sums.iter_mut().zip(readings) {
            *sum += v as f32;
        }
        self.could = mix.map(|m| m.could);
        self.n += 1;
    }

    fn add_use(&mut self, block: BlockId, kw: u32, secs: u32) {
        let i = match self.users.iter().position(|u| u.block == block) {
            Some(i) => i,
            None => {
                self.users.push(User { block, sum: 0.0, series: Series::after(secs) });
                self.users.len() - 1
            }
        };
        self.users[i].sum += kw as f32;
    }

    fn close_second(&mut self) {
        let n = self.n.max(1) as f32;
        for (series, sum) in self.totals.iter_mut().chain(self.sources.iter_mut()).zip(self.sums) {
            series.push(sum / n);
        }
        for u in &mut self.users {
            u.series.push(std::mem::take(&mut u.sum) / n);
        }
        (self.sums, self.n) = ([0.0; TOTALS + SOURCES], 0);
    }

    fn rows(&self) -> Vec<&Series> {
        self.totals.iter().chain(self.sources.iter()).chain(self.users.iter().map(|u| &u.series)).collect()
    }
}
