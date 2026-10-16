//! One river reach or lake as a shape, and what it says about a column (`Sample`). Version 7's reaches are
//! straight with even pools held by weirs and its lakes discs; version 8's reaches follow a winding course
//! (`course.rs`) whose pools end in a drop the water runs down, and its lakes have a shore (`shore.rs`).
//!
//! Invariants: pool levels never rise downstream; the first and last equal the nodes' levels at the ends. A column
//! farther than `REACH` from the shape is never shaped by it.

use super::super::shore::Shore;
use super::course::Course;
use super::{Node, BANK, LAKE_DEPTH, MAX_RAMP, REACH, RIVER_DEPTH, SPAWN_DRY, WEIR_WIDTH};

/// Version 8: how far a width wobbles either way (a share of it), with the column's wobble noise.
const WOBBLE: f64 = 0.3;
/// Version 8: banks keep the upper pool's height this far past a drop, so it can't spill sideways round it.
const BANK_LAG: f64 = 6.0;
/// Version 8: the bed rises to a lip this far before a drop, and a plunge pool deepens it this far after one.
const LIP: f64 = 3.0;
const PLUNGE: f64 = 4.0;

/// A river reach (a node to its downstream) or, with both ends equal, a lake.
pub(super) struct Seg {
    pub a: (f64, f64),
    pub b: (f64, f64),
    pub len: f64,
    pub wa: f64,
    pub wb: f64,
    depth: f64,
    /// Water level of each pool from the upstream end; the first and last equal the nodes' levels.
    pub levels: Vec<i32>,
    /// Version 7: every pool's length. Version 8: where each pool starts along the course (the first at 0).
    pool_len: f64,
    ends: Vec<f64>,
    /// Version 8: a reach's course, a lake's shore.
    pub course: Option<Course>,
    shore: Option<Shore>,
}

/// What a segment says about one column.
#[derive(Clone, Copy)]
pub(super) struct Sample {
    /// Distance from the water's edge (negative inside), the water's half-width there, the distance to its axis and
    /// the channel's depth in the middle.
    pub sd: f64,
    pub width: f64,
    pub axis: f64,
    pub depth: f64,
    /// The pool's water level, and the weir's top when on one (`i32::MIN` if not; version 7).
    pub level: i32,
    pub weir: i32,
    /// The level the bank rises above, the upper pool's just past a drop (version 8).
    pub bank: i32,
    /// Version 8: the bed rises to a lip before a drop; how much deeper a plunge pool is after one.
    pub lip: bool,
    pub plunge: i32,
}

impl Seg {
    /// Version 7's straight reach from `a` to `b` with even pools, or with no `b` a round lake.
    pub fn new(a: &Node, b: Option<&Node>, levels: Vec<i32>) -> Seg {
        let (ax, az) = (a.x as f64, a.z as f64);
        let (bx, bz) = b.map_or((ax, az), |b| (b.x as f64, b.z as f64));
        let len = ((bx - ax).powi(2) + (bz - az).powi(2)).sqrt();
        let pool_len = len / levels.len() as f64;
        let (wa, wb, depth) = match b {
            Some(b) => (a.width, b.width, RIVER_DEPTH),
            None => (a.width, a.width, LAKE_DEPTH),
        };
        Seg {
            a: (ax, az),
            b: (bx, bz),
            len,
            wa,
            wb,
            depth,
            levels,
            pool_len,
            ends: Vec::new(),
            course: None,
            shore: None,
        }
    }

    /// Version 8's reach along `course`, its pools starting at `ends`.
    pub fn winding(a: &Node, b: &Node, course: Course, ends: Vec<f64>, levels: Vec<i32>) -> Seg {
        let seg = Seg::new(a, Some(b), levels);
        Seg { len: course.len(), ends, course: Some(course), ..seg }
    }

    /// Version 8's lake at `n` (its width the full radius) with `shore`.
    pub fn lake(n: &Node, shore: Shore) -> Seg {
        Seg { shore: Some(shore), ..Seg::new(n, None, vec![n.level]) }
    }

    /// Whether the segment can shape any column of the rectangle (`REACH` beyond it).
    pub fn reaches(&self, min: (f64, f64), max: (f64, f64)) -> bool {
        let (lo, hi) = match &self.course {
            Some(c) => (c.lo, c.hi),
            None => {
                ((self.a.0.min(self.b.0), self.a.1.min(self.b.1)), (self.a.0.max(self.b.0), self.a.1.max(self.b.1)))
            }
        };
        lo.0 - REACH < max.0 && hi.0 + REACH > min.0 && lo.1 - REACH < max.1 && hi.1 + REACH > min.1
    }

    /// Whether the segment's water keeps `SPAWN_DRY` from spawn.
    pub fn clear_of_spawn(&self) -> bool {
        let widest = self.wa.max(self.wb) * if self.course.is_some() { 1.0 + WOBBLE } else { 1.0 };
        let d = match &self.course {
            Some(c) => c.nearest(0.0, 0.0, f64::MAX).map_or(f64::MAX, |n| n.0),
            None => {
                let ((ax, az), (dx, dz)) = (self.a, (self.b.0 - self.a.0, self.b.1 - self.a.1));
                let t =
                    if self.len > 0.0 { (-(ax * dx + az * dz) / (self.len * self.len)).clamp(0.0, 1.0) } else { 0.0 };
                ((ax + dx * t).powi(2) + (az + dz * t).powi(2)).sqrt()
            }
        };
        d > SPAWN_DRY + widest
    }

    /// What the segment says about column (px, pz); `wobble` is the column's wobble noise (version 8, about -1..1).
    pub fn sample(&self, px: f64, pz: f64, wobble: f64) -> Sample {
        if let Some(shore) = &self.shore {
            let (dx, dz) = (px - self.a.0, pz - self.a.1);
            let axis = (dx * dx + dz * dz).sqrt();
            if axis > self.wa * 2.0 + BANK + MAX_RAMP {
                // Too far for the shore to matter (`river_sample` drops it); skip its trigonometry.
                return self.pooled(axis, self.wa, 0);
            }
            let width = self.wa * shore.reach(dx, dz) * (1.0 + 0.5 * WOBBLE * wobble);
            return self.pooled(axis, width, 0);
        }
        let Some(course) = &self.course else { return self.straight(px, pz) };
        // Farther than this from the axis, no column is shaped (`river_sample` keeps `sd` under `BANK + MAX_RAMP`).
        let limit = BANK + MAX_RAMP + self.wa.max(self.wb) * (1.0 + WOBBLE);
        let Some((axis, s)) = course.nearest(px, pz, limit) else { return self.pooled(f64::MAX, 1.0, 0) };
        let width = (self.wa + (self.wb - self.wa) * s / self.len) * (1.0 + WOBBLE * wobble);
        let pool = |s: f64| self.ends.partition_point(|&e| e <= s).max(1) - 1;
        let i = pool(s);
        let mut out = self.pooled(axis, width, i);
        out.bank = self.levels[pool(s - BANK_LAG)];
        out.lip = i + 1 < self.ends.len() && self.ends[i + 1] - s < LIP;
        if i > 0 && s - self.ends[i] < PLUNGE {
            out.plunge = (self.levels[i - 1] - self.levels[i] - 1).clamp(0, 3);
        }
        out
    }

    /// A sample in pool `i` with no weir, lip or plunge.
    fn pooled(&self, axis: f64, width: f64, i: usize) -> Sample {
        let level = self.levels[i];
        Sample {
            sd: axis - width,
            width,
            axis,
            depth: self.depth,
            level,
            weir: i32::MIN,
            bank: level,
            lip: false,
            plunge: 0,
        }
    }

    /// Version 7's straight reach or round lake.
    fn straight(&self, px: f64, pz: f64) -> Sample {
        let (dx, dz) = (self.b.0 - self.a.0, self.b.1 - self.a.1);
        let t = if self.len > 0.0 {
            (((px - self.a.0) * dx + (pz - self.a.1) * dz) / (self.len * self.len)).clamp(0.0, 1.0)
        } else {
            0.0
        };
        let (cx, cz) = (self.a.0 + dx * t, self.a.1 + dz * t);
        let axis = ((px - cx).powi(2) + (pz - cz).powi(2)).sqrt();
        let width = self.wa + (self.wb - self.wa) * t;
        let s = t * self.len;
        let i = ((s / self.pool_len) as usize).min(self.levels.len() - 1);
        let on_weir = i > 0 && self.levels[i - 1] != self.levels[i] && s - i as f64 * self.pool_len < WEIR_WIDTH;
        let weir = if on_weir { self.levels[i - 1] } else { i32::MIN };
        Sample { weir, ..self.pooled(axis, width, i) }
    }

    /// The (x, z) columns along the axis, about every `step` blocks (tests).
    #[cfg(test)]
    pub fn axis(&self, step: f64) -> Vec<(i32, i32)> {
        if let Some(c) = &self.course {
            let every = (step / 6.0).round().max(1.0) as usize;
            return c.points().iter().step_by(every).map(|&(x, z)| (x as i32, z as i32)).collect();
        }
        let n = (self.len / step).ceil().max(1.0) as i32;
        (0..=n)
            .map(|i| {
                let t = i as f64 / n as f64;
                ((self.a.0 + (self.b.0 - self.a.0) * t) as i32, (self.a.1 + (self.b.1 - self.a.1) * t) as i32)
            })
            .collect()
    }
}
