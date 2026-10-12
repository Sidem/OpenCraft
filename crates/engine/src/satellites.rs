//! Satellites in orbit (Milestone 12, step 12.2): core state the factory owns (`Factory::orbit`, like its research).
//! A rocket from the launch site (`factory/process/launch.rs`) carries one satellite and puts it in orbit; nothing about
//! it is stored but where and when it left, because **a position is a pure function of the tick**: integer maths on
//! milli-blocks, never the frame, the camera or a loaded chunk.
//!
//! The ground is infinite, so an orbit wraps: every `WRAP` (4096) blocks the sky repeats, in both directions. A satellite
//! leaves straight above its pad and flies one of `HEADINGS` at `SPEED` (6 blocks a second, a lap in 11 minutes). Headings
//! are tilted a little off the axes, so successive tracks sweep different ground, and the launch count picks the next one.
//! A satellite covers a place while the place is within its kind's `reach` blocks of the point below it (wrapped
//! distance). At most `MAX_PER_KIND` of a kind stay up; one more retires the oldest.
//!
//! To add a kind: a row of `KINDS` (its index is saved), its item in `item/later.rs`, recipes in `recipes/orbit.rs`.
//! Presentation (`sky_directions`) and queries never change the list; only `launch` does, called by the launch site.

use crate::bytes::{ByteReader, ByteWriter};
use crate::item::{ItemId, SURVEY_SATELLITE};
use crate::math::IVec3;

#[cfg(test)]
mod tests;

/// Blocks after which the sky repeats, both ways (the orbits are on a torus of this size).
pub const WRAP: i32 = 4096;
/// Milli-blocks in `WRAP`.
const WRAP_MM: i64 = WRAP as i64 * 1000;
/// Most satellites of one kind in orbit; another launch retires the oldest of the kind.
pub const MAX_PER_KIND: usize = 24;
/// How high satellites look to hang above the ground when drawn in the sky, in blocks.
pub const SKY_HEIGHT: f64 = 400.0;
/// Farthest ground distance, in blocks, at which a satellite is drawn above the horizon.
pub const SKY_RANGE: f64 = 2400.0;

/// Ground speed in milli-blocks a tick of the eight headings (each about 100 mm a tick: 6 blocks a second).
const HEADINGS: [(i64, i64); 8] =
    [(96, 28), (28, 96), (-28, 96), (-96, 28), (-96, -28), (-28, -96), (28, -96), (96, -28)];

/// One kind of satellite: what it is called, the item a rocket carries, and how far its footprint reaches.
pub struct Kind {
    pub name: &'static str,
    pub item: ItemId,
    /// Blocks from the point below the satellite within which a place is covered.
    pub reach: i32,
}

/// The kinds, by saved index. What each one gives is the step after this one (12.3).
pub const KINDS: [Kind; 1] = [Kind { name: "Survey", item: SURVEY_SATELLITE, reach: 768 }];

/// The kind of satellite item `item` is, if it is one.
pub fn kind_of(item: ItemId) -> Option<u8> {
    KINDS.iter().position(|k| k.item == item).map(|i| i as u8)
}

/// One satellite: its kind, the heading it flies, and where and when it left (a place on the ground, at a tick).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Satellite {
    pub kind: u8,
    pub heading: u8,
    pub launched: u64,
    pub from: [i32; 2],
}

impl Satellite {
    /// The ground point below it at `tick`, in milli-blocks, each in `0..WRAP_MM`.
    pub fn position(&self, tick: u64) -> [i64; 2] {
        let dt = tick.saturating_sub(self.launched) as i64;
        let (vx, vz) = HEADINGS[self.heading as usize % HEADINGS.len()];
        [
            (self.from[0] as i64 * 1000 + vx * dt).rem_euclid(WRAP_MM),
            (self.from[1] as i64 * 1000 + vz * dt).rem_euclid(WRAP_MM),
        ]
    }

    /// Whether the place (a column, in blocks) is inside its footprint at `tick`.
    pub fn covers(&self, x: i32, z: i32, tick: u64) -> bool {
        let [dx, dz] = self.offset(x, z, tick);
        let reach = KINDS[self.kind as usize].reach as i64 * 1000;
        dx * dx + dz * dz <= reach * reach
    }

    /// From the place (a column, in blocks) to the point below the satellite, in milli-blocks, the short way round.
    pub fn offset(&self, x: i32, z: i32, tick: u64) -> [i64; 2] {
        let [px, pz] = self.position(tick);
        [short(px - x as i64 * 1000), short(pz - z as i64 * 1000)]
    }
}

/// Every satellite in orbit, in launch order.
#[derive(Default)]
pub struct Satellites {
    list: Vec<Satellite>,
    /// Launches ever made: picks the next heading, and never goes down when a satellite retires.
    launched: u32,
    /// The latest tick the factory ran, for readouts (derived, not saved).
    pub now: u64,
}

impl Satellites {
    /// Puts a satellite of the kind `item` is in orbit, straight above `pad` at `tick`. Nothing for any other item.
    pub fn launch(&mut self, item: ItemId, pad: IVec3, tick: u64) {
        let Some(kind) = kind_of(item) else { return };
        if self.count(kind) >= MAX_PER_KIND {
            let oldest = self.list.iter().position(|s| s.kind == kind).unwrap_or(0);
            self.list.remove(oldest);
        }
        let heading = (self.launched % HEADINGS.len() as u32) as u8;
        self.list.push(Satellite { kind, heading, launched: tick, from: [pad.x, pad.z] });
        self.launched += 1;
    }

    #[cfg(test)]
    pub fn list(&self) -> &[Satellite] {
        &self.list
    }

    /// How many satellites of `kind` are up.
    pub fn count(&self, kind: u8) -> usize {
        self.list.iter().filter(|s| s.kind == kind).count()
    }

    /// How many satellites of `kind` cover the place (a column, in blocks) at `tick`.
    pub fn covering(&self, kind: u8, x: i32, z: i32, tick: u64) -> usize {
        self.list.iter().filter(|s| s.kind == kind && s.covers(x, z, tick)).count()
    }

    /// Unit directions towards the satellites above the horizon, as seen from the place (a column and the eye's height),
    /// four floats each: x, y, z and the kind. They hang `SKY_HEIGHT` blocks up and show within `SKY_RANGE` blocks.
    pub fn sky_directions(&self, x: f64, z: f64, tick: u64) -> Vec<f32> {
        let mut out = Vec::new();
        for s in &self.list {
            let [dx, dz] = s.offset(x.round() as i32, z.round() as i32, tick);
            let (dx, dz) = (dx as f64 / 1000.0, dz as f64 / 1000.0);
            let ground = dx.hypot(dz);
            if ground > SKY_RANGE {
                continue;
            }
            let len = (ground * ground + SKY_HEIGHT * SKY_HEIGHT).sqrt();
            out.extend([(dx / len) as f32, (SKY_HEIGHT / len) as f32, (dz / len) as f32, s.kind as f32]);
        }
        out
    }

    /// Core state: the launch count, then each satellite.
    pub fn write_state(&self, w: &mut ByteWriter) {
        w.u32(self.launched);
        w.count(self.list.len());
        for s in &self.list {
            w.u8(s.kind);
            w.u8(s.heading);
            w.u64(s.launched);
            w.i32(s.from[0]);
            w.i32(s.from[1]);
        }
    }

    /// Reads what `write_state` wrote; a kind or heading that does not exist, or too many satellites, is damage.
    pub fn read_state(r: &mut ByteReader) -> Option<Satellites> {
        let mut sats = Satellites { launched: r.u32()?, ..Satellites::default() };
        let n = r.count()?;
        if n > MAX_PER_KIND * KINDS.len() {
            return None;
        }
        for _ in 0..n {
            let (kind, heading) = (r.u8()?, r.u8()?);
            if kind as usize >= KINDS.len() || heading as usize >= HEADINGS.len() {
                return None;
            }
            sats.list.push(Satellite { kind, heading, launched: r.u64()?, from: [r.i32()?, r.i32()?] });
        }
        Some(sats)
    }
}

/// `d` (a difference of two points in `0..WRAP_MM`) the short way round: in `-WRAP_MM / 2..=WRAP_MM / 2`.
fn short(d: i64) -> i64 {
    let d = d.rem_euclid(WRAP_MM);
    if d > WRAP_MM / 2 {
        d - WRAP_MM
    } else {
        d
    }
}
