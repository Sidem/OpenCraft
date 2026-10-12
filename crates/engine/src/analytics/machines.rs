//! How well each machine has worked lately: a running average (about [`WINDOW_SECS`] seconds) of its pace and of
//! what held it back, kept per machine anchor (machines are renumbered when others are removed, positions are not).
//!
//! Invariants: only machines seen in the latest `sample` are kept; a machine with nothing to do keeps its old
//! average but reads as idle. To word a new cause: `loss_name` and the `Cause` list in `factory/efficiency.rs`.

use rustc_hash::FxHashMap;

use crate::block::{BlockId, BLOCK_COUNT};
use crate::factory::efficiency::{Cause, Sample, CAUSES};
use crate::factory::Factory;
use crate::math::IVec3;

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
    block: BlockId,
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
            let e = self.map.entry(pos).or_insert(Entry {
                block,
                work: 1.0,
                lost: [0.0; CAUSES],
                ready: false,
                idle: true,
                stamp,
            });
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

    /// One row of [`TYPE_FIELDS`] numbers per kind of machine seen: `[block, working, at full pace, average percent
    /// of those working, idle, then one count per cause]` (a short machine counts under what costs it most).
    pub fn by_type(&self) -> Vec<u32> {
        let mut rows = vec![[0u32; TYPE_FIELDS]; BLOCK_COUNT];
        let mut percent_sum = vec![0u32; BLOCK_COUNT];
        for e in self.map.values().filter(|e| e.ready) {
            let b = e.block as usize;
            rows[b][0] = e.block as u32;
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
        let mut out = Vec::new();
        for (row, sum) in rows.iter_mut().zip(percent_sum) {
            if row[1] + row[4] == 0 {
                continue;
            }
            row[3] = sum.checked_div(row[1]).unwrap_or(0);
            out.extend_from_slice(row);
        }
        out
    }

    /// The `n` working machines furthest below full pace, worst first, as `[x, y, z, block, percent, cause]` each.
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
            .flat_map(|&(pct, p, e)| [p.x, p.y, p.z, e.block as i32, pct as i32, main_cause(e) as i32])
            .collect()
    }
}

/// Numbers per kind of machine in [`Machines::by_type`].
pub const TYPE_FIELDS: usize = 5 + CAUSES;

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
