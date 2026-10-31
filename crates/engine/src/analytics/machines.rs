//! How well each machine has worked lately: a running average (about [`WINDOW_SECS`] seconds) of its pace and of
//! what held it back, kept per machine anchor (machines are renumbered when others are removed, positions are not).
//!
//! Invariants: only machines seen in the latest `sample` are kept; a machine with nothing to do keeps its old
//! average but reads as idle. To word a new cause: `loss_name` and the `Cause` list in `factory/efficiency.rs`.

use rustc_hash::FxHashMap;

use crate::block::BlockId;
use crate::factory::efficiency::{Cause, Sample, CAUSES};
use crate::factory::{tiers, Factory};
use crate::math::{sort_small_by_key, IVec3};

/// Seconds the average looks back over, roughly.
pub const WINDOW_SECS: u32 = 10;
/// Samples a second (`analytics/mod.rs` samples every 10th tick).
pub const SAMPLES_PER_SEC: u32 = 6;
const ALPHA: f32 = 1.0 / (WINDOW_SECS * SAMPLES_PER_SEC) as f32;
/// A machine at or above this share of full pace counts as running at full pace.
const FULL: f32 = 0.985;
/// Losses under this share are left out of the readout.
const MINOR: f32 = 0.03;

struct Entry {
    /// What it is: the item of its tier (`Miner Mk3`), or its block's for a machine without tiers.
    kind: u16,
    /// Average share of full pace (0 to 1), and the average share lost to each cause.
    work: f32,
    lost: [f32; CAUSES],
    /// Whether it has had anything to do yet, and whether it has now.
    ready: bool,
    idle: bool,
    stamp: u32,
}

#[derive(Default)]
pub struct Machines {
    map: FxHashMap<IVec3, Entry>,
    stamp: u32,
}

impl Machines {
    /// Reads every machine's latest sample into its average.
    pub fn sample(&mut self, factory: &Factory) {
        self.stamp = self.stamp.wrapping_add(1);
        let stamp = self.stamp;
        factory.samples(|block, pos, s| {
            let kind = kind_at(factory, block, pos);
            let e = self.map.entry(pos).or_insert(Entry {
                kind,
                work: 1.0,
                lost: [0.0; CAUSES],
                ready: false,
                idle: true,
                stamp,
            });
            // An upgrade in place keeps the entry: it is the same machine at a higher tier.
            e.kind = kind;
            e.stamp = stamp;
            e.idle = s.is_none();
            if let Some(Sample { work, short }) = s {
                let w = work as f32 / 1000.0;
                let mut lost = [0.0; CAUSES];
                if let Some(c) = short {
                    lost[c as usize] = 1.0 - w;
                }
                // The first reading starts the average, so a new machine reads right at once.
                let a = if e.ready { ALPHA } else { 1.0 };
                e.work += (w - e.work) * a;
                for (avg, now) in e.lost.iter_mut().zip(lost) {
                    *avg += (now - *avg) * a;
                }
                e.ready = true;
            }
        });
        self.map.retain(|_, e| e.stamp == stamp);
    }

    /// Average share of full pace in percent, or `None` for a machine with nothing to do (or none here).
    pub fn percent(&self, pos: IVec3) -> Option<u32> {
        self.map.get(&pos).filter(|e| e.ready && !e.idle).map(|e| (e.work * 100.0).round() as u32)
    }

    /// The readout for the machine at `pos`: its pace and what cost it the rest (empty if it is not tracked).
    pub fn line(&self, pos: IVec3) -> String {
        let Some(e) = self.map.get(&pos) else { return String::new() };
        if e.idle || !e.ready {
            return "Efficiency: idle, nothing to do".to_string();
        }
        let pct = (e.work * 100.0).round() as u32;
        if e.work >= FULL {
            return "Efficiency 100%: full speed".to_string();
        }
        let mut text = format!("Efficiency {pct}% (last {WINDOW_SECS} s)");
        let mut left = e.lost;
        let mut sep = ": lost ";
        for _ in 0..2 {
            let (c, &share) = left.iter().enumerate().max_by(|a, b| a.1.total_cmp(b.1)).expect("causes");
            if share < MINOR {
                break;
            }
            text += &format!("{sep}{}% to {}", (share * 100.0).round() as u32, loss_name(CAUSE_LIST[c]));
            (left[c], sep) = (0.0, ", ");
        }
        text
    }

    /// Counts of the machines that have something to do: `[working, at full pace, then one count per cause]`, each
    /// short machine counted under what costs it most.
    pub fn summary(&self) -> [u32; CAUSES + 2] {
        let mut out = [0; CAUSES + 2];
        for e in self.map.values().filter(|e| e.ready && !e.idle) {
            out[0] += 1;
            if e.work >= FULL {
                out[1] += 1;
                continue;
            }
            out[2 + main_cause(e)] += 1;
        }
        out
    }

    /// One row of [`TYPE_FIELDS`] numbers per kind of machine seen, by item id (each tier its own row):
    /// `[kind, working, at full pace, average percent of those working, idle, then one count per cause]` (a short
    /// machine counts under what costs it most), in item id order.
    pub fn by_type(&self) -> Vec<u32> {
        let mut rows: Vec<[u32; TYPE_FIELDS]> = Vec::new();
        let mut percent_sum: Vec<u32> = Vec::new();
        for e in self.map.values().filter(|e| e.ready) {
            let b = match rows.iter().position(|r| r[0] == u32::from(e.kind)) {
                Some(b) => b,
                None => {
                    let mut row = [0; TYPE_FIELDS];
                    row[0] = u32::from(e.kind);
                    rows.push(row);
                    percent_sum.push(0);
                    rows.len() - 1
                }
            };
            if e.idle {
                rows[b][4] += 1;
                continue;
            }
            rows[b][1] += 1;
            percent_sum[b] += (e.work * 100.0).round() as u32;
            if e.work >= FULL {
                rows[b][2] += 1;
            } else {
                rows[b][5 + main_cause(e)] += 1;
            }
        }
        for (row, sum) in rows.iter_mut().zip(percent_sum) {
            row[3] = sum.checked_div(row[1]).unwrap_or(0);
        }
        sort_small_by_key(&mut rows, |r| r[0]);
        rows.concat()
    }

    /// The `n` working machines furthest below full pace, worst first, as `[x, y, z, kind, percent, cause]` each
    /// (`kind` as in `by_type`).
    pub fn worst(&self, n: usize) -> Vec<i32> {
        let mut found: Vec<(u32, IVec3, &Entry)> = Vec::with_capacity(n + 1);
        for (&pos, e) in self.map.iter().filter(|(_, e)| e.ready && !e.idle && e.work < FULL) {
            let pct = (e.work * 100.0).round() as u32;
            // Ties go by position, so the list does not depend on the map's order.
            let key = |(p, q, _): &(u32, IVec3, &Entry)| (*p, q.x, q.y, q.z);
            let at = found.partition_point(|f| key(f) < (pct, pos.x, pos.y, pos.z));
            if at < n {
                found.insert(at, (pct, pos, e));
                found.truncate(n);
            }
        }
        found
            .iter()
            .flat_map(|&(pct, p, e)| [p.x, p.y, p.z, i32::from(e.kind), pct as i32, main_cause(e) as i32])
            .collect()
    }

    /// Every machine of `kind` (an item id, as in `by_type`) that `by_type` counts (working or idle), as
    /// [`KIND_FIELDS`] numbers each: `[x, y, z, percent, cause]`. Percent is -1 for an idle machine; cause is -1 for one
    /// idle or at full pace, else what costs it most. For the world map's "Show on map" (`web/src/ui/map-focus.ts`);
    /// order is not meaningful.
    pub fn of_kind(&self, kind: u16) -> Vec<i32> {
        let mut out = Vec::new();
        for (p, e) in self.map.iter().filter(|(_, e)| e.ready && e.kind == kind) {
            let (pct, cause) = match (e.idle, e.work >= FULL) {
                (true, _) => (-1, -1),
                (false, true) => ((e.work * 100.0).round() as i32, -1),
                (false, false) => ((e.work * 100.0).round() as i32, main_cause(e) as i32),
            };
            out.extend_from_slice(&[p.x, p.y, p.z, pct, cause]);
        }
        out
    }
}

/// Numbers per kind of machine in [`Machines::by_type`].
pub const TYPE_FIELDS: usize = 5 + CAUSES;
/// Numbers per machine in [`Machines::of_kind`].
pub const KIND_FIELDS: usize = 5;

/// What the analytics call the machine of `block` at `pos`: the item of its tier, so each Mk is a row of its own.
pub(crate) fn kind_at(factory: &Factory, block: BlockId, pos: IVec3) -> u16 {
    factory.tiered_at(pos).and_then(|(b, t)| tiers::item_of(b, t)).map_or(u16::from(block), |i| i.0)
}

/// The cause that cost the machine most of its pace (an index into the causes).
fn main_cause(e: &Entry) -> usize {
    e.lost.iter().enumerate().max_by(|a, b| a.1.total_cmp(b.1)).map_or(0, |(c, _)| c)
}

/// The causes in `Cause as usize` order.
const CAUSE_LIST: [Cause; CAUSES] =
    [Cause::Power, Cause::Input, Cause::Output, Cause::Fuel, Cause::Water, Cause::Deposit, Cause::Blocked];

/// What a loss reads as after "lost 25% to ...".
fn loss_name(c: Cause) -> &'static str {
    match c {
        Cause::Power => "low power",
        Cause::Input => "waiting for input",
        Cause::Output => "a full output",
        Cause::Fuel => "no fuel",
        Cause::Water => "no water",
        Cause::Deposit => "a thin or empty deposit",
        Cause::Blocked => "being blocked",
    }
}
