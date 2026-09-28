//! Terraforming sites (core state the factory owns, like `research`): areas of columns marked with
//! the planner, each with a job that digs down to, fills up to or flattens to its `level`.
//!
//! Invariants: ids come from a counter and are never reused (never a list index). A site is at most
//! `MAX_SITE` columns on a side and never overlaps another; a world has at most `MAX_SITES`. The cut
//! and fill ranges (`Site::high`, `Site::low`) are found once, when the site is made, from the world as
//! it is (`*_anywhere`, never from what happens to be loaded). Cell order (`Site::cell`): cut layers
//! from the top down, then fill layers from the bottom up; in a layer, rows along z, back and forth
//! along x. A fill cell counts only above its column's ground (`is_ground`), never into a cave.
//! `survey_site` is a query over loaded chunks, for the planner. Nothing works a site yet (step 6.3).
//! To add a job: a `Job` variant appended to `JOBS` (its index is saved) and its ranges in
//! `Job::cuts` / `Job::fills`.

use super::machine;
use crate::block::{is_ore, BlockId, AIR, BEDROCK, LEAVES, LIQUID, LOG, SOLID};
use crate::bytes::{ByteReader, ByteWriter};
use crate::chunk::{CHUNK_SHIFT, CHUNK_SIZE};
use crate::math::IVec3;
use crate::world::World;
use crate::worldgen::{WORLD_HEIGHT, WORLD_HEIGHT_CHUNKS};

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
}

/// Jobs by their saved byte (append only).
const JOBS: [Job; 3] = [Job::Dig, Job::Fill, Job::Flatten];

impl Job {
    /// The job whose byte (`job as u8`) this is.
    pub fn from_byte(byte: u8) -> Option<Job> {
        JOBS.get(byte as usize).copied()
    }

    fn cuts(self) -> bool {
        self != Job::Fill
    }

    fn fills(self) -> bool {
        self != Job::Dig
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
}

impl Site {
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
        (self.cut_layers() + self.fill_layers()) * self.area()
    }

    /// Whether cell `i` is filled rather than cut (the cut's cells come first).
    #[cfg_attr(not(test), expect(dead_code, reason = "excavators work sites from step 6.3"))]
    pub fn is_fill(&self, i: u32) -> bool {
        i >= self.cut_layers() * self.area()
    }

    /// Cell `i` (below `cells()`): cut layers from `high` down, then fill layers from above `low` up;
    /// in a layer, row by row along z, each row along x, every other one backwards.
    #[cfg_attr(not(test), expect(dead_code, reason = "excavators work sites from step 6.3"))]
    pub fn cell(&self, i: u32) -> IVec3 {
        let (area, w) = (self.area(), (self.hi.0 - self.lo.0 + 1) as u32);
        let (layer, k) = (i / area, i % area);
        let (row, c) = (k / w, k % w);
        let col = if row % 2 == 1 { w - 1 - c } else { c };
        let cut = self.cut_layers();
        let y = if layer < cut { self.high - layer as i32 } else { self.low + 1 + (layer - cut) as i32 };
        IVec3::new(self.lo.0 + col as i32, y, self.lo.1 + row as i32)
    }

    fn area(&self) -> u32 {
        ((self.hi.0 - self.lo.0 + 1) * (self.hi.1 - self.lo.1 + 1)) as u32
    }

    fn overlaps(&self, lo: Column, hi: Column) -> bool {
        self.lo.0 <= hi.0 && lo.0 <= self.hi.0 && self.lo.1 <= hi.1 && lo.1 <= self.hi.1
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
        let full = self.list.len() >= MAX_SITES || self.list.iter().any(|s| s.overlaps(lo, hi));
        if full || !(1..WORLD_HEIGHT - 1).contains(&level) {
            return None;
        }
        let (high, low) = ground_range(world, lo, hi);
        let id = self.next_id;
        self.next_id = self.next_id.wrapping_add(1);
        self.list.push(Site { id, lo, hi, level, job, high, low });
        Some(id)
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
            if id >= next_id || shape(lo, hi) != Some((lo, hi)) {
                return None;
            }
            list.push(Site { id, lo, hi, level, job, high, low });
        }
        Some(Sites { list, next_id })
    }
}

/// What a job over some columns would move, among loaded chunks (a query, for the planner).
#[derive(Clone, Copy, Default, PartialEq, Eq, Debug)]
pub struct SiteSurvey {
    /// Blocks the cut takes (ground, trees, ore and loose things; bedrock and machines stay).
    pub cut: u32,
    /// Cells the fill builds.
    pub fill: u32,
    /// Of the cut: ore blocks (cut like hand mining) and tree blocks (logs, leaves).
    pub ore: u32,
    pub trees: u32,
    /// Water cells in the cut or the fill.
    pub water: u32,
    /// Columns not loaded here, left out of the counts.
    pub unseen: u32,
}

impl SiteSurvey {
    /// Ground left over after the fill, to carry away (negative: ground to bring in).
    pub fn spoil(&self) -> i32 {
        (self.cut - self.ore - self.trees) as i32 - self.fill as i32
    }
}

/// What `job` to `level` over the columns between `a` and `b` would move, among loaded chunks (a
/// query: never creates core state). `None` for an area too big to be a site.
pub fn survey_site(world: &World, a: Column, b: Column, level: i32, job: Job) -> Option<SiteSurvey> {
    let (lo, hi) = shape(a, b)?;
    let mut s = SiteSurvey::default();
    for z in lo.1..=hi.1 {
        for x in lo.0..=hi.0 {
            match survey_column(world, x, z, level, job) {
                Some(c) => {
                    s.cut += c.cut;
                    s.fill += c.fill;
                    s.ore += c.ore;
                    s.trees += c.trees;
                    s.water += c.water;
                }
                None => s.unseen += 1,
            }
        }
    }
    Some(s)
}

/// Ground a column is built on: solid, and not part of a tree.
fn is_ground(b: BlockId) -> bool {
    SOLID[b as usize] && b != LOG && b != LEAVES
}

/// Whether a cut takes `b`: everything but air, water, bedrock and machines (belts and pipes too).
fn cut_takes(b: BlockId) -> bool {
    b != AIR && !LIQUID[b as usize] && b != BEDROCK && machine(b).is_none()
}

/// The corners of a site between `a` and `b`, lowest first, if it is small enough.
fn shape(a: Column, b: Column) -> Option<(Column, Column)> {
    let (lo, hi) = ((a.0.min(b.0), a.1.min(b.1)), (a.0.max(b.0), a.1.max(b.1)));
    let side = |l: i32, h: i32| h as i64 - l as i64 + 1;
    (side(lo.0, hi.0) <= MAX_SITE as i64 && side(lo.1, hi.1) <= MAX_SITE as i64).then_some((lo, hi))
}

/// The top block of the highest chunk in chunk column (cx, cz) that isn't all air (`air` says whether
/// a chunk is; `None`: unknown), or -1 if they all are.
fn sky_floor(cx: i32, cz: i32, mut air: impl FnMut(IVec3) -> Option<bool>) -> Option<i32> {
    for cy in (0..WORLD_HEIGHT_CHUNKS).rev() {
        if !air(IVec3::new(cx, cy, cz))? {
            return Some(cy * CHUNK_SIZE + CHUNK_SIZE - 1);
        }
    }
    Some(-1)
}

/// The highest block (air aside) and the lowest column ground in the site, from the world as it is.
/// Chunk column by chunk column, skipping the sky, so the few chunks it reads stay in the world's small
/// cache of generated ones.
fn ground_range(world: &mut World, lo: Column, hi: Column) -> (i32, i32) {
    let (mut high, mut low) = (0, WORLD_HEIGHT);
    for cz in (lo.1 >> CHUNK_SHIFT)..=(hi.1 >> CHUNK_SHIFT) {
        for cx in (lo.0 >> CHUNK_SHIFT)..=(hi.0 >> CHUNK_SHIFT) {
            let from = sky_floor(cx, cz, |c| Some(world.is_air_anywhere(c))).unwrap_or(-1);
            let (z0, x0) = ((cz << CHUNK_SHIFT).max(lo.1), (cx << CHUNK_SHIFT).max(lo.0));
            let (z1, x1) =
                ((cz << CHUNK_SHIFT | (CHUNK_SIZE - 1)).min(hi.1), (cx << CHUNK_SHIFT | (CHUNK_SIZE - 1)).min(hi.0));
            for z in z0..=z1 {
                for x in x0..=x1 {
                    let mut top = None;
                    let mut ground = 0;
                    for y in (0..=from).rev() {
                        let b = world.block_anywhere_or_generate(IVec3::new(x, y, z));
                        if b != AIR {
                            top.get_or_insert(y);
                        }
                        if is_ground(b) {
                            ground = y;
                            break;
                        }
                    }
                    high = high.max(top.unwrap_or(0));
                    low = low.min(ground);
                }
            }
        }
    }
    (high, low)
}

/// One loaded column's share of a survey; `None` if part of it isn't loaded.
fn survey_column(world: &World, x: i32, z: i32, level: i32, job: Job) -> Option<SiteSurvey> {
    let air = |c| world.loaded_chunk(c).map(|chunk| chunk.as_uniform() == Some(AIR));
    let mut y = sky_floor(x >> CHUNK_SHIFT, z >> CHUNK_SHIFT, air)?;
    let mut s = SiteSurvey::default();
    let mut ground = None;
    while y >= 0 && (y > level || ground.is_none()) {
        let b = world.get_block(IVec3::new(x, y, z))?;
        let water = LIQUID[b as usize];
        if y > level && job.cuts() {
            if cut_takes(b) {
                s.cut += 1;
                s.ore += is_ore(b) as u32;
                s.trees += (b == LOG || b == LEAVES) as u32;
            }
            s.water += water as u32;
        } else if y <= level && ground.is_none() && job.fills() {
            s.water += water as u32;
        }
        if ground.is_none() && is_ground(b) {
            ground = Some(y);
        }
        y -= 1;
    }
    if job.fills() {
        s.fill = (level - ground.unwrap_or(0)).max(0) as u32;
    }
    Some(s)
}

#[cfg(test)]
mod tests;
