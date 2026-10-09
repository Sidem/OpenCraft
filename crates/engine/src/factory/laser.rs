//! Laser links (Milestone 11, the Photonics tech): an emitter and a receiver with a clear line between them join
//! two power grids (receiver) or two data grids (data receiver) across up to [`RANGE`] blocks.
//!
//! - An emitter (`process/laser.rs`, block 104) shoots along its front in a straight line. The first block on that line
//!   decides: air and glass let the beam through, a receiver (block 105 for power, 107 for data, from any side) ends it
//!   as a link, a mirror (block 106, horizontal beams only) turns it 90° and the line goes on from there (at most
//!   [`MAX_MIRRORS`] turns, [`RANGE`] blocks in all), anything else blocks it. `aim_beams` casts each emitter's path as
//!   a list of [`Ray`] segments (origin, step, cells looked at, what ended it). Power sees the power links as extra
//!   edges between the poles the two ends hang on (`Power::rebuild`): the two grids become one, and the receiving
//!   side's use costs a ninth more (`LOSS_DIVISOR`: the beam delivers 90%). Data links join the grids of the fibre
//!   nodes within reach of the two ends (`Data::rebuild`), with no loss.
//! - No ray is cast per tick. `aim_beams` runs when the factory is relinked (a machine placed or removed) and when a
//!   block changes on a ray (`Sim::block_changed` -> `beam_cut_check` marks the beams stale); a changed set of links
//!   relinks the factory once. Rays read the world with `block_anywhere_or_generate`, so loaded chunks never matter.
//! - Nothing is saved: the emitters and receivers are processors (their facing and status are saved with them), and the
//!   links are derived. `write_beams` draws the beam as a thin lit box (a blocked beam flickers up to what blocks it).
//!
//! To let another block pass the beam: add it to `transparent`.

use crate::block::{tex, BlockId, AIR, BLOCK_DEFS, DATA_RECEIVER, GLASS, LASER_EMITTER, LASER_MIRROR, LASER_RECEIVER};
use crate::math::{IVec3, Vec3};
use crate::world::World;

use super::links::Slot;
use super::process::{Processor, Status};
use super::render::push_box;
use super::{Factory, DIRS};

/// Blocks a beam can cross, over all its segments.
pub const RANGE: i32 = 128;
/// Mirrors one beam can turn at.
pub const MAX_MIRRORS: usize = 8;
/// The receiving side's use is this many ninths again: 10% of what is delivered is lost on the way.
pub const LOSS_DIVISOR: u32 = 9;

/// What ended a ray.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum End {
    /// A receiver or data receiver (processor index).
    Linked(u32),
    /// A mirror (processor index): the path goes on in the next segment.
    Turned(u32),
    /// A block that is neither a receiver nor transparent: its cell and id.
    Blocked(IVec3, BlockId),
    /// Nothing within [`RANGE`].
    Open,
}

/// One straight stretch of an emitter's path (the path's last segment says how it ended).
#[derive(Clone, Copy, Debug)]
pub struct Ray {
    /// The emitter's processor index.
    pub emitter: u32,
    pub from: IVec3,
    pub step: IVec3,
    /// Cells looked at past `from`: the last one is the receiver, the mirror, the blocker or the end of the range.
    pub len: i32,
    pub end: End,
}

impl Ray {
    /// Whether a block change at `pos` could change this ray.
    pub fn covers(&self, pos: IVec3) -> bool {
        let d = pos - self.from;
        let k = d.x * self.step.x + d.y * self.step.y + d.z * self.step.z;
        (1..=self.len).contains(&k) && d == IVec3::new(self.step.x * k, self.step.y * k, self.step.z * k)
    }
}

/// The beams, derived by `aim_beams`.
#[derive(Default)]
pub(crate) struct Beams {
    /// Every emitter's segments, in path order.
    pub rays: Vec<Ray>,
    /// Linked pairs (emitter, receiver or data receiver) as processor indices.
    pub links: Vec<(u32, u32)>,
    /// A block changed on a ray: `aim_beams` must run before the next tick.
    pub stale: bool,
}

/// The way an emitter shoots: out of its front, which faces back towards whoever placed it.
pub fn emit_dir(p: &Processor) -> IVec3 {
    DIRS[((p.dir + 2) % 4) as usize]
}

fn transparent(block: BlockId) -> bool {
    block == AIR || block == GLASS
}

/// The direction a beam travelling `step` leaves a mirror turned `dir`: an even facing swaps the horizontal
/// components (east becomes south), an odd one swaps and flips them (east becomes north). `None` for a vertical beam.
pub fn reflect(step: IVec3, dir: u8) -> Option<IVec3> {
    if step.y != 0 {
        return None;
    }
    Some(if dir.is_multiple_of(2) { IVec3::new(step.z, 0, step.x) } else { IVec3::new(-step.z, 0, -step.x) })
}

impl Factory {
    /// Recasts every emitter's path; relinks the factory when the set of links changed. Runs when `dirty` or `stale`.
    pub(super) fn aim_beams(&mut self, world: &mut World) {
        self.beams.stale = false;
        let mut rays = Vec::new();
        for (i, p) in self.processors.iter().enumerate() {
            if p.spec.block == LASER_EMITTER {
                self.cast(world, i as u32, &mut rays);
            }
        }
        let links: Vec<(u32, u32)> =
            rays.iter().filter_map(|r| if let End::Linked(j) = r.end { Some((r.emitter, j)) } else { None }).collect();
        let laser = |b| matches!(b, LASER_EMITTER | LASER_RECEIVER | DATA_RECEIVER | LASER_MIRROR);
        for p in self.processors.iter_mut().filter(|p| laser(p.spec.block)) {
            p.status = Status::NoInput;
        }
        let turned = rays.iter().filter_map(|r| if let End::Turned(m) = r.end { Some(m) } else { None });
        for i in links.iter().flat_map(|&(e, r)| [e, r]).chain(turned) {
            self.processors[i as usize].status = Status::Working;
        }
        if links != self.beams.links {
            self.dirty = true;
        }
        self.beams.rays = rays;
        self.beams.links = links;
    }

    /// A block changed at `pos`: beams whose line crosses it must be recast.
    pub(crate) fn beam_cut_check(&mut self, pos: IVec3) {
        if self.beams.rays.iter().any(|r| r.covers(pos)) {
            self.beams.stale = true;
        }
    }

    /// The linked pairs that carry power (`data` false: they end at a receiver) or data (a data receiver).
    pub(super) fn beam_links(&self, data: bool) -> Vec<(u32, u32)> {
        let ends_in_data = |&&(_, r): &&(u32, u32)| (self.processors[r as usize].spec.block == DATA_RECEIVER) == data;
        self.beams.links.iter().filter(ends_in_data).copied().collect()
    }

    /// Casts `emitter`'s path into `out`: segments up to a receiver, a blocker or the end of the range, a mirror
    /// ending each one that goes on.
    fn cast(&self, world: &mut World, emitter: u32, out: &mut Vec<Ray>) {
        let p = &self.processors[emitter as usize];
        let (mut from, mut step) = (p.pos, emit_dir(p));
        let (mut left, mut turns) = (RANGE, 0);
        loop {
            let mut ray = Ray { emitter, from, step, len: left, end: End::Open };
            let mut bend = None;
            for k in 1..=left {
                let cell = from + IVec3::new(step.x * k, step.y * k, step.z * k);
                let block = world.block_anywhere_or_generate(cell);
                let Some(&Slot::Process(j)) = self.at.get(&cell) else {
                    if !transparent(block) {
                        (ray.len, ray.end) = (k, End::Blocked(cell, block));
                        break;
                    }
                    continue;
                };
                if matches!(block, LASER_RECEIVER | DATA_RECEIVER) {
                    (ray.len, ray.end) = (k, End::Linked(j));
                } else if let Some(s) = reflect(step, self.processors[j as usize].dir).filter(|_| block == LASER_MIRROR)
                {
                    (ray.len, ray.end) = (k, End::Turned(j));
                    bend = Some((cell, s));
                } else {
                    (ray.len, ray.end) = (k, End::Blocked(cell, block));
                }
                break;
            }
            match bend {
                Some((cell, s)) if ray.len < left && turns < MAX_MIRRORS => {
                    out.push(ray);
                    (from, step, left, turns) = (cell, s, left - ray.len, turns + 1);
                }
                Some((cell, _)) => {
                    // No range left, or too many turns: the path ends at this mirror.
                    ray.end = if ray.len < left { End::Blocked(cell, LASER_MIRROR) } else { End::Open };
                    return out.push(ray);
                }
                None => return out.push(ray),
            }
        }
    }

    /// The whole path of `emitter`: its segments, and the blocks it has gone through to the end.
    fn path(&self, emitter: u32) -> (Vec<&Ray>, i32) {
        let segments: Vec<&Ray> = self.beams.rays.iter().filter(|r| r.emitter == emitter).collect();
        let total = segments.iter().map(|r| r.len).sum();
        (segments, total)
    }

    /// ", round 2 mirrors" for a path of `segments` (empty without turns).
    fn via(segments: usize) -> String {
        match segments {
            0 | 1 => String::new(),
            2 => " (round a mirror)".to_string(),
            n => format!(" (round {} mirrors)", n - 1),
        }
    }

    /// The beam line of a laser block `i` for its readout (`None`: another machine).
    pub(super) fn beam_line(&self, i: usize) -> Option<String> {
        let block = self.processors[i].spec.block;
        let joined = |e: u32, r: u32| !self.dirty && self.data.joined.contains(&(e, r));
        match block {
            LASER_EMITTER => {
                let (segments, total) = self.path(i as u32);
                let last = segments.last()?;
                let via = Self::via(segments.len());
                Some(match last.end {
                    End::Linked(r) if self.processors[r as usize].spec.block == DATA_RECEIVER => {
                        let ok = if joined(i as u32, r) {
                            "it joins the two data grids".to_string()
                        } else {
                            format!("it and the emitter each need a fibre node within {} blocks", super::fibre::REACH)
                        };
                        format!("Beam to a data receiver {total} blocks away{via}: {ok}")
                    }
                    End::Linked(_) => format!(
                        "Beam to a receiver {total} blocks away{via}: a tenth of what it carries is lost on the way"
                    ),
                    End::Blocked(_, b) => format!(
                        "Beam blocked by {} {total} blocks away{via}: clear the way or aim it elsewhere",
                        BLOCK_DEFS[b as usize].name
                    ),
                    End::Turned(_) | End::Open => {
                        format!("Nothing in line within {RANGE} blocks{via}: aim it at a receiver")
                    }
                })
            }
            LASER_RECEIVER | DATA_RECEIVER => {
                let last = self.beams.rays.iter().find(|r| r.end == End::Linked(i as u32));
                Some(match last {
                    Some(r) => {
                        let (segments, total) = self.path(r.emitter);
                        let data = block == DATA_RECEIVER && !joined(r.emitter, i as u32);
                        let note = if data { ", but both ends need a fibre node within reach" } else { "" };
                        format!("Fed by an emitter {total} blocks away{}{note}", Self::via(segments.len()))
                    }
                    None => "No beam reaches it: aim an emitter at it with a clear line".to_string(),
                })
            }
            LASER_MIRROR => {
                let beams = self.beams.rays.iter().filter(|r| r.end == End::Turned(i as u32)).count();
                let (a, b) =
                    if self.processors[i].dir.is_multiple_of(2) { ("south", "north") } else { ("north", "south") };
                let sides = format!("Joins its west and {a} sides, and its east and {b} sides");
                Some(if beams > 0 {
                    format!("Turning {beams} beam{}. {sides}", if beams > 1 { "s" } else { "" })
                } else {
                    sides
                })
            }
            _ => None,
        }
    }

    /// The beams as thin lit boxes within `range` of the eye; a blocked beam flickers up to its blocker.
    pub(super) fn write_beams(&self, out: &mut Vec<f32>, eye: Vec3, time: f64, range: f64) {
        let flicker = (time * 9.0) as i64 % 3 != 0;
        for r in &self.beams.rays {
            let reach = match r.end {
                End::Linked(_) | End::Turned(_) => r.len,
                End::Blocked(..) if flicker => r.len,
                _ => continue,
            };
            let gap = reach - 1;
            if gap < 1 {
                continue;
            }
            let d = Vec3::new(r.step.x as f64, r.step.y as f64, r.step.z as f64);
            let mid = r.from.as_vec3() + Vec3::new(0.5, 0.5, 0.5) + d * ((1 + gap) as f64 / 2.0) - eye;
            if (mid.x * mid.x + mid.y * mid.y + mid.z * mid.z).sqrt() > range + gap as f64 * 0.5 {
                continue;
            }
            let yaw = d.x.atan2(-d.z) as f32;
            push_box(out, mid, yaw, [0.08, 0.08, gap as f32], 0.0, [tex::BEAM; 3], false);
        }
    }
}

#[cfg(test)]
mod tests;
