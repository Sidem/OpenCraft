//! Laser power links (Milestone 11, the Photonics tech): an emitter and a receiver with a clear line between them join
//! two power grids across any distance up to [`RANGE`] blocks.
//!
//! - An emitter (`process/laser.rs`, block 104) shoots along its front in a straight line. The first block on that line
//!   decides: air and glass let the beam through, a receiver (block 105, from any side) ends it as a link, anything else
//!   blocks it. `aim_beams` casts one ray per emitter and keeps it as a [`Ray`] (origin, step, cells looked at, what ended
//!   it). Power sees the links as extra edges between the poles the two ends hang on (`Power::rebuild`): the two grids
//!   become one, and the receiving side's use costs a ninth more (`LOSS_DIVISOR`: the beam delivers 90%).
//! - No ray is cast per tick. `aim_beams` runs when the factory is relinked (a machine placed or removed) and when a
//!   block changes on a ray (`Sim::block_changed` -> `beam_cut_check` marks the beams stale); a changed set of links
//!   relinks the factory once. Rays read the world with `block_anywhere_or_generate`, so loaded chunks never matter.
//! - Nothing is saved: the emitters and receivers are processors (their facing and status are saved with them), and the
//!   links are derived. `write_beams` draws the beam as a thin lit box (a blocked beam flickers up to what blocks it).
//!
//! To let another block pass the beam: add it to `transparent`. A mirror (later) would end a ray and start another.

use crate::block::{tex, BlockId, AIR, BLOCK_DEFS, GLASS, LASER_EMITTER, LASER_RECEIVER};
use crate::math::{IVec3, Vec3};
use crate::world::World;

use super::links::Slot;
use super::process::{Processor, Status};
use super::render::push_box;
use super::{Factory, DIRS};

/// Blocks a beam can cross.
pub const RANGE: i32 = 128;
/// The receiving side's use is this many ninths again: 10% of what is delivered is lost on the way.
pub const LOSS_DIVISOR: u32 = 9;

/// What ended a ray.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum End {
    /// A receiver (processor index).
    Linked(u32),
    /// A block that is neither a receiver nor transparent: its cell and id.
    Blocked(IVec3, BlockId),
    /// Nothing within [`RANGE`].
    Open,
}

/// One emitter's line.
#[derive(Clone, Copy, Debug)]
pub struct Ray {
    /// The emitter's processor index.
    pub emitter: u32,
    pub from: IVec3,
    pub step: IVec3,
    /// Cells looked at past `from`: the last one is the receiver, the blocker or the end of the range.
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
    pub rays: Vec<Ray>,
    /// Linked pairs (emitter, receiver) as processor indices.
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

impl Factory {
    /// Recasts every emitter's ray; relinks the factory when the set of links changed. Runs when `dirty` or `stale`.
    pub(super) fn aim_beams(&mut self, world: &mut World) {
        self.beams.stale = false;
        let mut rays = Vec::new();
        for (i, p) in self.processors.iter().enumerate() {
            if p.spec.block == LASER_EMITTER {
                rays.push(self.cast(world, i as u32));
            }
        }
        let links: Vec<(u32, u32)> =
            rays.iter().filter_map(|r| if let End::Linked(j) = r.end { Some((r.emitter, j)) } else { None }).collect();
        for p in self.processors.iter_mut().filter(|p| matches!(p.spec.block, LASER_EMITTER | LASER_RECEIVER)) {
            p.status = Status::NoInput;
        }
        for &(e, r) in &links {
            self.processors[e as usize].status = Status::Working;
            self.processors[r as usize].status = Status::Working;
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

    fn cast(&self, world: &mut World, emitter: u32) -> Ray {
        let p = &self.processors[emitter as usize];
        let (from, step) = (p.pos, emit_dir(p));
        let mut ray = Ray { emitter, from, step, len: RANGE, end: End::Open };
        for k in 1..=RANGE {
            let cell = from + IVec3::new(step.x * k, step.y * k, step.z * k);
            let block = world.block_anywhere_or_generate(cell);
            if block == LASER_RECEIVER {
                if let Some(&Slot::Process(j)) = self.at.get(&cell) {
                    (ray.len, ray.end) = (k, End::Linked(j));
                    break;
                }
            }
            if !transparent(block) {
                (ray.len, ray.end) = (k, End::Blocked(cell, block));
                break;
            }
        }
        ray
    }

    /// The beam line of the emitter or receiver `i` for its readout (`None`: another machine).
    pub(super) fn beam_line(&self, i: usize) -> Option<String> {
        match self.processors[i].spec.block {
            LASER_EMITTER => {
                let ray = self.beams.rays.iter().find(|r| r.emitter as usize == i)?;
                Some(match ray.end {
                    End::Linked(_) => format!(
                        "Beam to a receiver {} blocks away: a tenth of what it carries is lost on the way",
                        ray.len
                    ),
                    End::Blocked(_, block) => format!(
                        "Beam blocked by {} {} blocks away: clear the way or aim it elsewhere",
                        BLOCK_DEFS[block as usize].name, ray.len
                    ),
                    End::Open => format!("Nothing in line within {RANGE} blocks: aim it at a receiver"),
                })
            }
            LASER_RECEIVER => {
                let ray = self.beams.rays.iter().find(|r| r.end == End::Linked(i as u32));
                Some(match ray {
                    Some(r) => format!("Fed by an emitter {} blocks away", r.len),
                    None => "No beam reaches it: aim an emitter at it with a clear line".to_string(),
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
                End::Linked(_) => r.len,
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
