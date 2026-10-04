//! Terraforming sites (core state the factory owns, like `research`): areas of columns marked with
//! the planner, each with a job that digs down to, fills up to or flattens to its `level`.
//!
//! Invariants: ids come from a counter and are never reused (never a list index). A site is at most
//! `MAX_SITE` columns on a side and never overlaps another; a world has at most `MAX_SITES`. The cut
//! and fill ranges (`Site::high`, `Site::low`) are found once, when the site is made, from the world as
//! it is (`*_anywhere`, never from what happens to be loaded). Cell order (`Site::cell`): cut layers
//! from the top down, then fill layers from the bottom up; in a layer, rows along z, back and forth
//! along x. A fill cell counts only above its column's ground (`is_ground`), never into a cave.
//! `survey_site` is a query over loaded chunks, for the planner. Drone ports work sites (`drones/earthworks.rs`).
//! A tunnel (`sites/tunnel.rs`) is a site too, with `Job::Tunnel` and a `Tunnel` instead of an area: its cells are
//! the bore, `lo` / `hi` its columns and `low` / `high` the heights just below and at its top cell; sites clash
//! when their columns overlap, but a tunnel only when it also shares heights.
//! To add a job: a `Job` variant appended to `JOBS` (its index is saved) and its ranges in
//! `Job::cuts` / `Job::fills`.

use super::machine;
use crate::block::{BlockId, AIR, BEDROCK, LEAVES, LIQUID, LOG, SOLID};
use crate::bytes::{ByteReader, ByteWriter};
use crate::math::IVec3;
use crate::world::World;
use crate::worldgen::WORLD_HEIGHT;

mod survey;
mod tunnel;
use survey::ground_range;
pub use survey::{survey_site, SiteSurvey};
pub use tunnel::{survey_tunnel, touches_water, Tunnel, SECTIONS};

/// Most columns a site may have on a side.
pub const MAX_SITE: i32 = 64;
/// Most sites one world may have.
pub const MAX_SITES: usize = 32;

/// A column (x, z).
pub type Column = (i32, i32);

/// What a site's ground becomes.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Job {
    /// Everything above the level is cut away.
    Dig,
    /// Every column is built up to the level.
    Fill,
    /// Both: the ground ends level.
    Flatten,
    /// A bore from one block to another (`Site::tunnel`).
    Tunnel,
}

/// Jobs by their saved byte (append only).
const JOBS: [Job; 4] = [Job::Dig, Job::Fill, Job::Flatten, Job::Tunnel];

impl Job {
    /// The job whose byte (`job as u8`) this is.
    pub fn from_byte(byte: u8) -> Option<Job> {
        JOBS.get(byte as usize).copied()
    }

    /// Whether the job cuts and fills the area between `level` and the ground (a tunnel does neither: its own cells).
    fn cuts(self) -> bool {
        matches!(self, Job::Dig | Job::Flatten)
    }

    fn fills(self) -> bool {
        matches!(self, Job::Fill | Job::Flatten)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Site {
    pub id: u32,
    /// The corner columns, `lo` at or below `hi` on both axes, inclusive.
    pub lo: Column,
    pub hi: Column,
    /// The top of the ground when the job is done: the cut takes what is above it, the fill builds up
    /// to it.
    pub level: i32,
    pub job: Job,
    /// Found when the site was made: its highest block (air aside) and its lowest column ground.
    pub high: i32,
    pub low: i32,
    /// Cells from the start known to need no work (a cache for `drones/earthworks.rs`; never saved).
    pub done: u32,
    /// The bore, for `Job::Tunnel` sites (whose other fields it decides: see `Site::bore`).
    pub tunnel: Option<Tunnel>,
}

impl Site {
    /// The site that cuts `t`.
    fn bore(id: u32, t: Tunnel) -> Site {
        let (lo, hi, bottom, top) = t.bounds();
        Site { id, lo, hi, level: t.from.y, job: Job::Tunnel, high: top, low: bottom - 1, done: 0, tunnel: Some(t) }
    }

    pub fn cut_layers(&self) -> u32 {
        if self.job.cuts() {
            (self.high - self.level).max(0) as u32
        } else {
            0
        }
    }

    pub fn fill_layers(&self) -> u32 {
        if self.job.fills() {
            (self.level - self.low).max(0) as u32
        } else {
            0
        }
    }

    pub fn cells(&self) -> u32 {
        match &self.tunnel {
            Some(t) => t.cells(),
            None => (self.cut_layers() + self.fill_layers()) * self.area(),
        }
    }

    /// Whether cell `i` is filled rather than cut (the cut's cells come first).
    #[cfg(test)]
    pub fn is_fill(&self, i: u32) -> bool {
        i >= self.cut_layers() * self.area()
    }

    /// Cell `i` (below `cells()`): cut layers from `high` down, then fill layers from above `low` up;
    /// in a layer, row by row along z, each row along x, every other one backwards.
    pub fn cell(&self, i: u32) -> IVec3 {
        if let Some(t) = &self.tunnel {
            return t.cell(i);
        }
        let (area, w) = (self.area(), (self.hi.0 - self.lo.0 + 1) as u32);
        let (layer, k) = (i / area, i % area);
        let (row, c) = (k / w, k % w);
        let col = if row % 2 == 1 { w - 1 - c } else { c };
        let cut = self.cut_layers();
        let y = if layer < cut { self.high - layer as i32 } else { self.low + 1 + (layer - cut) as i32 };
        IVec3::new(self.lo.0 + col as i32, y, self.lo.1 + row as i32)
    }

    /// Whether the column of `pos` is inside the site.
    pub fn has_column(&self, pos: IVec3) -> bool {
        (self.lo.0..=self.hi.0).contains(&pos.x) && (self.lo.1..=self.hi.1).contains(&pos.z)
    }

    /// Whether `pos` is one of the site's cells (drones work only these).
    pub fn covers(&self, pos: IVec3) -> bool {
        match &self.tunnel {
            Some(t) => t.has_cell(pos),
            None => self.has_column(pos) && (self.span().0..=self.span().1).contains(&pos.y),
        }
    }

    /// Whether the cell at `pos` is cut away, not built.
    pub fn cuts_at(&self, pos: IVec3) -> bool {
        self.tunnel.is_some() || pos.y > self.level
    }

    /// Whether the planner's aim at block `pos` means this site: any block in an area's columns, or in a tunnel's
    /// box (a tunnel is then preferred, as it may run under an area).
    pub fn picks(&self, pos: IVec3) -> bool {
        self.has_column(pos) && (self.tunnel.is_none() || (self.low + 1..=self.high).contains(&pos.y))
    }

    fn area(&self) -> u32 {
        ((self.hi.0 - self.lo.0 + 1) * (self.hi.1 - self.lo.1 + 1)) as u32
    }

    /// The lowest and highest height of the site's cells (empty when the first is above the second).
    fn span(&self) -> (i32, i32) {
        match self.job {
            Job::Dig => (self.level + 1, self.high),
            Job::Fill => (self.low + 1, self.level),
            Job::Flatten => (self.low.min(self.level) + 1, self.high.max(self.level)),
            Job::Tunnel => (self.low + 1, self.high),
        }
    }

    /// Whether the two can't both stand: their columns overlap, and for a tunnel their heights too.
    fn clashes(&self, o: &Site) -> bool {
        let columns = self.lo.0 <= o.hi.0 && o.lo.0 <= self.hi.0 && self.lo.1 <= o.hi.1 && o.lo.1 <= self.hi.1;
        let heights = self.span().0 <= o.span().1 && o.span().0 <= self.span().1;
        columns && (heights || (self.tunnel.is_none() && o.tunnel.is_none()))
    }
}

/// The world's sites.
#[derive(Default)]
pub struct Sites {
    /// In the order they were made.
    pub list: Vec<Site>,
    next_id: u32,
}

impl Sites {
    /// Marks a site over the columns between corners `a` and `b` (any order) and returns its id, or
    /// `None` (nothing changes) when it is too big, overlaps another site, the world has `MAX_SITES`
    /// or the level leaves no room above or below it. Finds the cut and fill ranges now.
    pub fn mark(&mut self, world: &mut World, a: Column, b: Column, level: i32, job: Job) -> Option<u32> {
        let (lo, hi) = shape(a, b)?;
        if self.list.len() >= MAX_SITES || !(1..WORLD_HEIGHT - 1).contains(&level) || job == Job::Tunnel {
            return None;
        }
        let (high, low) = ground_range(world, lo, hi);
        self.add(Site { id: 0, lo, hi, level, job, high, low, done: 0, tunnel: None })
    }

    /// Marks the tunnel `Tunnel::new(from, to, size)` and returns its id, or `None` (nothing changes) when that
    /// is refused, it clashes with a site, or the world has `MAX_SITES`.
    pub fn mark_tunnel(&mut self, from: IVec3, to: IVec3, size: u8) -> Option<u32> {
        self.add(Site::bore(0, Tunnel::new(from, to, size)?))
    }

    /// Adds `site` under the next id unless it is too many or clashes with one.
    fn add(&mut self, mut site: Site) -> Option<u32> {
        if self.list.len() >= MAX_SITES || self.list.iter().any(|s| s.clashes(&site)) {
            return None;
        }
        site.id = self.next_id;
        self.next_id = self.next_id.wrapping_add(1);
        self.list.push(site);
        Some(site.id)
    }

    /// Removes site `id`; false if there is none.
    pub fn remove(&mut self, id: u32) -> bool {
        let before = self.list.len();
        self.list.retain(|s| s.id != id);
        self.list.len() < before
    }

    pub fn write_state(&self, w: &mut ByteWriter) {
        w.u32(self.next_id);
        w.count(self.list.len());
        for s in &self.list {
            w.u32(s.id);
            for v in [s.lo.0, s.lo.1, s.hi.0, s.hi.1, s.level] {
                w.i32(v);
            }
            w.u8(s.job as u8);
            w.i32(s.high);
            w.i32(s.low);
            if let Some(t) = &s.tunnel {
                for v in [t.from, t.to] {
                    w.ivec3(v);
                }
                w.u8(t.size);
            }
        }
    }

    /// Reads what `write_state` wrote; a site of the wrong shape, an unknown job or an id not below
    /// the counter is damage.
    pub fn read_state(r: &mut ByteReader) -> Option<Sites> {
        let next_id = r.u32()?;
        let n = r.count()?;
        if n > MAX_SITES {
            return None;
        }
        let mut list = Vec::with_capacity(n);
        for _ in 0..n {
            let id = r.u32()?;
            let (lo, hi) = ((r.i32()?, r.i32()?), (r.i32()?, r.i32()?));
            let (level, job) = (r.i32()?, Job::from_byte(r.u8()?)?);
            let (high, low) = (r.i32()?, r.i32()?);
            let tunnel = if job == Job::Tunnel {
                // Everything but the bore is derived from it; saved fields that disagree are damage.
                let (from, to) = (r.ivec3()?, r.ivec3()?);
                let t = Tunnel::new(from, to, r.u8()?).filter(|t| t.to == to)?;
                let b = Site::bore(id, t);
                if (b.lo, b.hi, b.level, b.high, b.low) != (lo, hi, level, high, low) {
                    return None;
                }
                Some(t)
            } else if shape(lo, hi) == Some((lo, hi)) {
                None
            } else {
                return None;
            };
            if id >= next_id {
                return None;
            }
            list.push(Site { id, lo, hi, level, job, high, low, done: 0, tunnel });
        }
        Some(Sites { list, next_id })
    }
}

/// Ground a column is built on: solid, and not part of a tree.
pub(super) fn is_ground(b: BlockId) -> bool {
    SOLID[b as usize] && b != LOG && b != LEAVES
}

/// Whether a cut takes `b`: everything but air, water, bedrock and machines (belts and pipes too).
pub fn cut_takes(b: BlockId) -> bool {
    b != AIR && !LIQUID[b as usize] && b != BEDROCK && machine(b).is_none()
}

/// The corners of a site between `a` and `b`, lowest first, if it is small enough.
pub(super) fn shape(a: Column, b: Column) -> Option<(Column, Column)> {
    let (lo, hi) = ((a.0.min(b.0), a.1.min(b.1)), (a.0.max(b.0), a.1.max(b.1)));
    let side = |l: i32, h: i32| h as i64 - l as i64 + 1;
    (side(lo.0, hi.0) <= MAX_SITE as i64 && side(lo.1, hi.1) <= MAX_SITE as i64).then_some((lo, hi))
}

#[cfg(test)]
mod tests;
