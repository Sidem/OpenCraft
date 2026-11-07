//! The forester (Forestry, an early red tech; Mk2 and Mk3 by kits): a one-block electric processor that keeps the flat
//! grass or dirt in front of it (the way its placer faced) as a wood lot ([`Patch`]), worked by a knuckle-boom crane with
//! a harvester head, as a real one works a stand:
//!
//! - Look: it checks its patch's columns row by row, back and forth (`COLUMNS_PER_TICK` a tick), for a trunk standing
//!   on the ground (a tree to fell) or an empty plot it can plant. Each round starts with a survey of the ground
//!   (`patch::survey`): the largest of its tier's sizes that is all flat is its patch; none flat and it stops,
//!   `Blocked`, and says how many cells are in the way. A round with nothing to do and it rests `REST_SECONDS`.
//! - A tree: the crane swings out (Reach), the head's arms grip the trunk at its foot (Grip), its chain saw cuts it
//!   (Cut: the trunk leaves the world whole, with the leaves only it held: `fell.rs`), the head lays the stem down
//!   (Fell) and feeds it through, sawing it into logs one at a time (Buck), gathers them (Gather), swings them home
//!   (Haul) and drops them on its log bunk (Unload: into its output, waiting there for room). Every
//!   `LEAVES_PER_SAPLING` leaves give it a sapling of their kind (the rate leaves drop them, without the luck), kept in
//!   its input up to `SAPLING_SLOTS` stacks; more are lost.
//! - A plot (`Patch::is_plot`) left empty gets a sapling: the crane fetches one out (Fetch) and plants it (Plant),
//!   straight after its tree comes home; `sim/saplings.rs` grows it. Wild trees anywhere on the patch are felled too.
//! - Tiers: each works a bigger patch (`patch::SIZES`, if the ground allows) at the tier's speed for its power, so a log
//!   costs the same energy at every tier, more than it gives burned in the best generator: a wood source, never a
//!   power plant (`tests`). Power is drawn only while the crane moves.
//!
//! Invariants: it edits only the trunks standing in its patch, the leaves those alone held, and its plots, through the
//! `*_anywhere` accessors, reporting each edit in `changed`, so it runs the same in unloaded chunks. Its job is saved
//! ([`Forest`]); the crane's pose is drawn from the job and its progress (`view.rs`). Work counts in thousandths of a
//! tick at full power, as every processor's, times the tier's speed.
//!
//! To tune its pace: the constants (`tests::each_tier_gives_a_slow_steady_supply` measures it).

use crate::block::{BlockId, FORESTER};
use crate::bytes::{ByteReader, ByteWriter};
use crate::math::IVec3;
use crate::sim::SimEvent;
use crate::world::World;

use super::super::footprint::SINGLE;
use super::super::power::NOT_WIRED;
use super::{Energy, Extra, Pick, ProcessSpec, ProcessTier, Processor, Status};

mod crane;
mod fell;
mod patch;
mod scene;
mod view;
mod work;

pub use patch::{survey, Patch, Survey, SIZES};
pub(super) use view::draw_crane;

use view::PARTS;
use work::Crane;

#[cfg(test)]
mod tests;

/// kW while the crane works, at Mk1; each tier's power grows with its speed.
pub const FORESTER_KW: u32 = 20;
/// Seconds of work at full power at Mk1: swinging out to a tree or plot, gripping, sawing through, laying the stem
/// down, a log bucked off it, gathering the logs, swinging them home, dropping them, planting.
const REACH_SECONDS: f64 = 4.0;
const GRIP_SECONDS: f64 = 1.0;
const CUT_SECONDS: f64 = 2.5;
const FELL_SECONDS: f64 = 2.0;
const BUCK_SECONDS: f64 = 5.5;
const GATHER_SECONDS: f64 = 1.5;
const HAUL_SECONDS: f64 = 4.0;
const UNLOAD_SECONDS: f64 = 1.0;
const PLANT_SECONDS: f64 = 1.5;
/// Seconds it rests after a round with nothing to do, or waits before surveying uneven ground again.
const REST_SECONDS: f64 = 15.0;
/// Columns it looks over in one tick.
const COLUMNS_PER_TICK: u16 = 4;
/// Input stacks of saplings it keeps.
const SAPLING_SLOTS: usize = 2;

pub const FORESTER_SPEC: ProcessSpec = ProcessSpec {
    block: FORESTER,
    categories: &[],
    pick: Pick::Forest,
    buffers: [SAPLING_SLOTS, 0, 2],
    side: 0,
    tiers: &[
        ProcessTier { energy: Energy::Electric, speed: 1000, fuel: 0, power: FORESTER_KW },
        ProcessTier { energy: Energy::Electric, speed: 2000, fuel: 0, power: 2 * FORESTER_KW },
        ProcessTier { energy: Energy::Electric, speed: 3000, fuel: 0, power: 3 * FORESTER_KW },
    ],
    footprint: SINGLE,
    verb: "Felling",
    products: "logs",
    waiting: "Waiting for its trees to grow",
    map_colour: 0x4f7a3a,
    compute: 0,
    parts: &PARTS,
};

/// What the crane is doing. Saves store `job as u8`: append, never reorder.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum Job {
    #[default]
    Look,
    Reach,
    Grip,
    Cut,
    Fell,
    Buck,
    Gather,
    Haul,
    Unload,
    Fetch,
    Plant,
    Rest,
    Blocked,
}

const JOBS: [Job; 13] = [
    Job::Look,
    Job::Reach,
    Job::Grip,
    Job::Cut,
    Job::Fell,
    Job::Buck,
    Job::Gather,
    Job::Haul,
    Job::Unload,
    Job::Fetch,
    Job::Plant,
    Job::Rest,
    Job::Blocked,
];

impl Job {
    /// Whether the crane moves (and so draws power) in this job.
    pub fn moves(self) -> bool {
        !matches!(self, Job::Look | Job::Rest | Job::Blocked)
    }
}

/// A place the head can be: cells ahead and to the right of the forester; (0, 0) is home, over its log bunk.
pub type Spot = (i8, i8);
pub const HOME: Spot = (0, 0);

#[derive(Default)]
pub struct Forest {
    pub job: Job,
    /// The next column `Look` checks (`Patch::column`).
    pub col: u16,
    /// The tree it fells or the plot it plants.
    pub tree: Spot,
    /// Where the head is between moves, and where the move under way started.
    pub rest: Spot,
    pub from: Spot,
    /// Rest and Blocked: ticks left.
    pub step: u16,
    /// Whether this round of looking found work.
    pub busy: bool,
    /// Leaves stripped towards the next sapling.
    pub leaves: u8,
    /// Trees felled, for the readout.
    pub felled: u32,
    /// Logs in the stem it holds, logs bucked off it so far, and their block.
    pub stem: u8,
    pub cut: u8,
    pub log: BlockId,
    /// The last survey of its ground.
    pub ground: Survey,
}

impl Forest {
    /// The patch it works now (the first size while its ground is uneven).
    pub fn patch(&self, pos: IVec3, dir: u8) -> Patch {
        Patch { pos, dir, size: self.ground.size.unwrap_or(0) }
    }

    pub(super) fn write_state(&self, w: &mut ByteWriter) {
        w.u8(self.job as u8);
        w.u16(self.col);
        for s in [self.tree, self.rest, self.from] {
            write_spot(w, s);
        }
        w.u16(self.step);
        w.bool(self.busy);
        w.u8(self.leaves);
        w.u32(self.felled);
        w.u8(self.stem);
        w.u8(self.cut);
        w.u8(self.log);
        w.u8(self.ground.size.unwrap_or(u8::MAX));
        w.u16(self.ground.uneven);
        w.bool(self.ground.blocker.is_some());
        let b = self.ground.blocker.unwrap_or(IVec3::ZERO);
        for n in [b.x, b.y, b.z] {
            w.i32(n);
        }
    }

    pub(super) fn read_state(r: &mut ByteReader) -> Option<Forest> {
        let job = *JOBS.get(r.u8()? as usize)?;
        let col = r.u16()?;
        let (tree, rest, from) = (read_spot(r)?, read_spot(r)?, read_spot(r)?);
        let (step, busy, leaves, felled) = (r.u16()?, r.bool()?, r.u8()?, r.u32()?);
        let (stem, cut, log) = (r.u8()?, r.u8()?, r.u8()?);
        let size = Some(r.u8()?).filter(|&s| s != u8::MAX);
        let (uneven, blocked) = (r.u16()?, r.bool()?);
        let b = IVec3::new(r.i32()?, r.i32()?, r.i32()?);
        let ground = Survey { size, uneven, blocker: blocked.then_some(b) };
        let biggest = Patch { pos: IVec3::ZERO, dir: 0, size: SIZES.len() as u8 - 1 };
        let ok = col < biggest.columns() && size.is_none_or(|s| usize::from(s) < SIZES.len()) && cut <= stem;
        ok.then_some(Forest { job, col, tree, rest, from, step, busy, leaves, felled, stem, cut, log, ground })
    }
}

fn write_spot(w: &mut ByteWriter, (ahead, side): Spot) {
    w.u8(ahead as u8);
    w.u8(side as u8);
}

fn read_spot(r: &mut ByteReader) -> Option<Spot> {
    Some((r.u8()? as i8, r.u8()? as i8))
}

impl Processor {
    /// One tick of a forester at `share` of full power (nothing for other processors).
    pub(crate) fn forest(
        &mut self,
        world: &mut World,
        share: u32,
        changed: &mut Vec<(IVec3, BlockId)>,
        events: &mut Vec<SimEvent>,
    ) {
        let share = share * self.stats().speed / 1000;
        let Processor { pos, dir, tier, extra, input, out, progress, made, status, .. } = self;
        let Extra::Forest(forest) = extra else { return };
        let (pos, dir, tier) = (*pos, *dir, *tier);
        let mut c = Crane { pos, dir, tier, world, input, out, progress, made, changed, events, share };
        *status = forest.tick(&mut c);
    }

    /// Whether its crane would work this tick if powered.
    pub(super) fn forest_wants_power(&self) -> bool {
        let Extra::Forest(f) = &self.extra else { return false };
        f.job.moves() && !(f.job == Job::Unload && self.status == Status::OutputFull)
    }

    /// The status line of a forester (`None`: another processor).
    pub(super) fn forest_text(&self) -> Option<String> {
        let Extra::Forest(f) = &self.extra else { return None };
        let saplings: u32 = self.input.slots.iter().map(|s| s.count).sum();
        let doing = match (self.status, f.job) {
            (_, Job::Blocked) => {
                let n = f.ground.uneven;
                let cells = if n == 1 { "1 cell is" } else { "cells are" };
                let count = if n == 1 { String::new() } else { format!("{n} ") };
                format!("Needs flat grass or dirt in front: {count}{cells} in the way (marked red)")
            }
            (Status::NoPower, _) => NOT_WIRED.to_string(),
            (Status::OutputFull, _) => "Output full: put a belt leading away, or take the logs".to_string(),
            (_, Job::Rest) if saplings == 0 => "Waiting for trees · no saplings to plant: put some in it".to_string(),
            (_, Job::Rest) => self.spec.waiting.to_string(),
            (_, Job::Look) => "Looking over its patch".to_string(),
            (_, Job::Reach) => "Swinging out to a tree".to_string(),
            (_, Job::Grip) => "Gripping a trunk".to_string(),
            (_, Job::Cut) => "Sawing through the trunk".to_string(),
            (_, Job::Fell) => "Laying the stem down".to_string(),
            (_, Job::Buck) => "Cutting the stem into logs".to_string(),
            (_, Job::Gather) => "Gathering the logs".to_string(),
            (_, Job::Haul) => "Bringing the logs in".to_string(),
            (_, Job::Unload) => "Unloading logs".to_string(),
            (_, Job::Fetch) => "Taking a sapling out".to_string(),
            (_, Job::Plant) => "Planting a sapling".to_string(),
        };
        let (tree, sapling) =
            (if f.felled == 1 { "tree" } else { "trees" }, if saplings == 1 { "sapling" } else { "saplings" });
        let mut text = format!("{doing} · {} {tree} felled, {saplings} {sapling}", f.felled);
        let (patch, most) = (f.patch(self.pos, self.dir), Patch { size: self.tier, ..f.patch(self.pos, self.dir) });
        if f.job != Job::Blocked && patch.size < most.size {
            let ((rows, half), (big_rows, big_half)) = (patch.extent(), most.extent());
            let (w, big_w) = (2 * half + 1, 2 * big_half + 1);
            text += &format!(" · patch {w}×{rows} of {big_w}×{big_rows}: flatten more ground to use it all");
        }
        Some(text)
    }
}
