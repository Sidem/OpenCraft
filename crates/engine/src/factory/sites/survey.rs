//! Surveys for the planner (queries over loaded chunks, never core state) and the one scan marking does
//! (`ground_range`, from the world as it is). See `factory/sites.rs`.

use super::{cut_takes, is_ground, shape, Column, Job};
use crate::block::{is_ore, AIR, LEAVES, LIQUID, LOG};
use crate::chunk::{CHUNK_SHIFT, CHUNK_SIZE};
use crate::math::IVec3;
use crate::world::World;
use crate::worldgen::{WORLD_HEIGHT, WORLD_HEIGHT_CHUNKS};
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
pub(super) fn ground_range(world: &mut World, lo: Column, hi: Column) -> (i32, i32) {
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
