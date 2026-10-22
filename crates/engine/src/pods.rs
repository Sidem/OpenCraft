//! Drop pods in flight (Milestone 12, step 12.4): core state the factory owns (`Factory::pods`, saved since 46). A pod
//! launcher (`factory/process/pods.rs`) spends a drop pod item to send a load of cargo up; `FLIGHT_TICKS` later it comes down
//! on a landing pad, which is where `factory/process/pods.rs` delivers it. Nothing here knows about machines: a pod is
//! where it left, where it is going, its channel (the colour the launcher and the pads must share), when it lands and what it
//! carries. A pod whose pad is not ready (gone, out of comms cover or full) stays in flight and tries again each tick, so
//! cargo is never lost while a pad exists; at most `MAX_IN_FLIGHT` are up at once.
//!
//! Presentation (`falling`, `rising`) never changes the list; only `send` and `Pods::list` edits by the landing code do.

use crate::bytes::{ByteReader, ByteWriter};
use crate::inventory::Stack;
use crate::math::IVec3;

#[cfg(test)]
mod tests;

/// How many channels a launcher or pad can be tuned to: one per colour of the grid palette (`factory/grid_colour.rs`).
pub const CHANNELS: u8 = crate::block::tex::GRID_COLOURS as u8;
/// Ticks a pod takes from the launcher to its pad: 20 seconds.
pub const FLIGHT_TICKS: u64 = 1200;
/// Pods in the air at once, in all.
pub const MAX_IN_FLIGHT: usize = 64;
/// Stacks of cargo one pod carries.
pub const POD_STACKS: usize = 6;
/// Ticks of its last stretch in which a pod is drawn falling onto the pad.
pub const FALL_TICKS: u64 = 240;
/// Blocks above the pad where the fall begins.
pub const FALL_HEIGHT: f64 = 120.0;
/// Ticks after launch in which a pod is drawn rising out of its launcher, and how high it is by then.
pub const RISE_TICKS: u64 = 180;
pub const RISE_HEIGHT: f64 = 120.0;

/// One pod: where it left, the pad it is bound for, the channel it flies on, when it left and lands, and its cargo.
pub struct Pod {
    pub from: IVec3,
    pub to: IVec3,
    pub channel: u8,
    pub launched: u64,
    pub lands: u64,
    pub cargo: Vec<Stack>,
}

/// Every pod in flight, in launch order (so landing order too).
#[derive(Default)]
pub struct Pods {
    pub list: Vec<Pod>,
}

impl Pods {
    /// Whether one more pod can go up.
    pub fn has_room(&self) -> bool {
        self.list.len() < MAX_IN_FLIGHT
    }

    /// Sends a pod from `from` towards the pad at `to` on `channel`, launched at `tick`. Nothing happens when the sky is full
    /// (callers check `has_room`).
    pub fn send(&mut self, from: IVec3, to: IVec3, channel: u8, cargo: Vec<Stack>, tick: u64) {
        if self.has_room() {
            self.list.push(Pod { from, to, channel, launched: tick, lands: tick + FLIGHT_TICKS, cargo });
        }
    }

    /// How many pods are bound for pads on `channel`.
    pub fn inbound(&self, channel: u8) -> usize {
        self.list.iter().filter(|p| p.channel == channel).count()
    }

    /// Pods in the last stretch before landing at `time` (in ticks, fractions of a tick between frames): the pad each is bound
    /// for and how high above it it is, in blocks. A pod that is due but waits for a free pad hangs just above it.
    /// Presentation (the falling capsule).
    pub fn falling(&self, time: f64) -> Vec<(IVec3, f64)> {
        let high = |p: &Pod| {
            let left = (p.lands as f64 - time).max(0.0);
            (left <= FALL_TICKS as f64).then(|| FALL_HEIGHT * (left / FALL_TICKS as f64).powi(2))
        };
        self.list.iter().filter_map(|p| high(p).map(|h| (p.to, h))).collect()
    }

    /// Pods in the first stretch after launch at `time`: the launcher each left and how high above it it is, in blocks,
    /// speeding up as it climbs. Presentation (the rising capsule).
    pub fn rising(&self, time: f64) -> Vec<(IVec3, f64)> {
        let high = |p: &Pod| {
            let gone = time - p.launched as f64;
            (0.0..=RISE_TICKS as f64).contains(&gone).then(|| RISE_HEIGHT * (gone / RISE_TICKS as f64).powi(2))
        };
        self.list.iter().filter_map(|p| high(p).map(|h| (p.from, h))).collect()
    }

    /// Core state: the pods in flight.
    pub fn write_state(&self, w: &mut ByteWriter) {
        w.count(self.list.len());
        for p in &self.list {
            w.ivec3(p.from);
            w.ivec3(p.to);
            w.u8(p.channel);
            w.u64(p.launched);
            w.u64(p.lands);
            w.count(p.cargo.len());
            p.cargo.iter().for_each(|s| s.write_state(w));
        }
    }

    /// Reads what `write_state` wrote; too many pods, or too much cargo in one, is damage.
    pub fn read_state(r: &mut ByteReader) -> Option<Pods> {
        let n = r.count()?;
        if n > MAX_IN_FLIGHT {
            return None;
        }
        let mut pods = Pods::default();
        for _ in 0..n {
            let (from, to, channel) = (r.ivec3()?, r.ivec3()?, r.u8()? % CHANNELS);
            let (launched, lands) = (r.u64()?, r.u64()?);
            let stacks = r.count()?;
            if stacks > POD_STACKS {
                return None;
            }
            let cargo = (0..stacks).map(|_| Stack::read_state(r)).collect::<Option<Vec<_>>>()?;
            pods.list.push(Pod { from, to, channel, launched, lands, cargo });
        }
        Some(pods)
    }
}
