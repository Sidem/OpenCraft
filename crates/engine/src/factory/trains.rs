//! Trains (Milestone 9, step 9.4b): locomotives that run on the track of `rail.rs`. A train is a stretch of the
//! track graph: `path` lists the directed edges `(from node, to node)` it covers, tail first, and `head` is how far
//! its front is along the last edge, in millimetres. Distances along an edge are arc lengths of its curve
//! (`Factory::edge_mm`), so speed is the same on straights and bends.
//!
//! Each tick (`step_trains`) a train moves `STEP_MM`. Reaching a node it takes the track that leads on most
//! straight (`next_from`: within 60 degrees of its heading, the straightest first, ties by cell); with none it
//! reverses (the tail becomes the head), so a lone line shuttles between its ends. Edges it has left are
//! dropped from the tail of `path`. Trains are placed on and picked up from a node (`Action::PlaceTrain`,
//! `TakeTrain`); a node removed from under a train gives the locomotive back (`Factory::remove`), and a track
//! with a train on it cannot be cut.
//!
//! Wagons (`cars.rs`) couple on behind the locomotive and share one cargo buffer; the train is `len_mm` long. A
//! train with wagons that reaches a node beside a dock stops with its front there (`idle` counts the ticks since it
//! last traded) and trades with the dock (`docks.rs`) until `DWELL_TICKS` pass without a trade, then drives on.
//!
//! A train may carry a schedule of docks (`schedule.rs`): it then stops only at the next one on it and `steer`s
//! toward it at junctions; without one it takes `next_from` and stops at every dock.
//!
//! Invariants: every edge of a path is a laid track and `path` joins end to end; `head` is within the last edge;
//! `path` covers the whole train; only `+ - * / sqrt` and the curve's heading table decide where a train goes,
//! so every peer agrees. To extend: stopping and steering are `advance`; `model.rs` draws the cars.

mod cars;
mod docks;
mod model;
mod schedule;

use crate::bytes::{ByteReader, ByteWriter};
use crate::math::IVec3;

use super::buffer::Buffer;
use super::rail::Curve;
use super::Factory;

/// A locomotive's and a wagon's length, and what a train moves each tick (150 mm a tick is 9 blocks a second), in
/// millimetres.
pub const LOCO_MM: i64 = 2500;
pub const CAR_MM: i64 = 2500;
const STEP_MM: i64 = 150;
/// Within this many blocks of a node, a click picks a train up or couples a wagon to it.
const PICKUP_BLOCKS: f64 = 8.0;
/// A heading may turn this far between the track a train arrives on and the one it takes on (cosine of 60 degrees).
const MIN_AHEAD: f64 = 0.5;
/// Wagons to a train, and the slots of each (a storage box's).
pub const MAX_CARS: u8 = 6;
pub const CAR_SLOTS: usize = 24;
/// A stopped train drives on after this many ticks without a trade (5 seconds).
const DWELL_TICKS: u32 = 300;

pub struct Train {
    /// Directed edges, tail first.
    pub path: Vec<(IVec3, IVec3)>,
    /// The front's distance along the last edge (mm).
    pub head: i64,
    /// Wagons behind the locomotive; `cargo` has `CAR_SLOTS` slots for each.
    pub cars: u8,
    pub cargo: Buffer,
    /// While stopped at a dock (its front at the end of the last edge): ticks since the last trade.
    pub idle: Option<u32>,
    /// The docks it serves in turn, as anchor cells (`schedule.rs`), and which is next.
    pub schedule: Vec<IVec3>,
    pub next: u8,
}

/// What putting a locomotive on a node, or coupling a wagon beside it, would do.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Spot {
    Ok,
    /// No track leaves it (long enough for a locomotive).
    NoTrack,
    /// A train is already on a track here.
    Occupied,
    /// (Wagons) no train is near the node.
    NoTrain,
    /// (Wagons) the train has `MAX_CARS` already.
    Full,
    /// (Wagons) there is no track behind the train to stand a wagon on.
    NoRoom,
}

fn key(p: IVec3) -> (i32, i32, i32) {
    (p.x, p.y, p.z)
}

impl Train {
    fn new(path: Vec<(IVec3, IVec3)>, head: i64) -> Train {
        Train { path, head, cars: 0, cargo: Buffer::new(0), idle: None, schedule: Vec::new(), next: 0 }
    }

    /// Locomotive and wagons, in millimetres.
    pub fn len_mm(&self) -> i64 {
        LOCO_MM + i64::from(self.cars) * CAR_MM
    }

    fn touches(&self, p: IVec3) -> bool {
        self.path.iter().any(|&(a, b)| a == p || b == p)
    }
}

impl Factory {
    /// The length of the track between nodes `a` and `b` in millimetres (the same either way round).
    pub(super) fn edge_mm(&self, a: IVec3, b: IVec3) -> Option<i64> {
        let (lo, hi) = if key(a) <= key(b) { (a, b) } else { (b, a) };
        let curve = Curve::new(lo, self.rail_yaw(lo)?, hi, self.rail_yaw(hi)?);
        Some((curve.length() * 1000.0).round() as i64)
    }

    /// The curve of the track leaving `from` for `to`.
    fn run(&self, from: IVec3, to: IVec3) -> Option<Curve> {
        Some(Curve::new(from, self.rail_yaw(from)?, to, self.rail_yaw(to)?))
    }

    /// The nodes joined to `at` by track, in cell order.
    fn neighbours(&self, at: IVec3) -> Vec<IVec3> {
        let mut out: Vec<IVec3> =
            self.tracks.iter().filter(|t| t.a == at || t.b == at).map(|t| if t.a == at { t.b } else { t.a }).collect();
        out.sort_by_key(|&p| key(p));
        out
    }

    /// Where a train that came from `prev` to `at` goes on: the track leading on most straight, if any.
    pub(super) fn next_from(&self, prev: IVec3, at: IVec3) -> Option<IVec3> {
        let ahead = self.run(prev, at)?.heading(1.0);
        let mut best: Option<(f64, IVec3)> = None;
        for q in self.neighbours(at).into_iter().filter(|&q| q != prev) {
            let (x, z) = self.run(at, q)?.heading(0.0);
            let dot = ahead.0 * x + ahead.1 * z;
            if dot >= MIN_AHEAD && best.is_none_or(|b| dot > b.0) {
                best = Some((dot, q));
            }
        }
        best.map(|b| b.1)
    }

    /// Whether a train is on the track between `a` and `b`.
    pub fn train_on(&self, a: IVec3, b: IVec3) -> bool {
        self.trains.iter().any(|t| t.path.iter().any(|&(p, q)| (p, q) == (a, b) || (p, q) == (b, a)))
    }

    /// What a locomotive put on the node at `pos` would do.
    pub fn train_spot(&self, pos: IVec3) -> Spot {
        if self.trains.iter().any(|t| t.touches(pos)) {
            Spot::Occupied
        } else if self.launch(pos).is_none() {
            Spot::NoTrack
        } else {
            Spot::Ok
        }
    }

    /// The first neighbour of `pos` whose track holds a locomotive.
    fn launch(&self, pos: IVec3) -> Option<IVec3> {
        self.rail_yaw(pos)?;
        self.neighbours(pos).into_iter().find(|&q| self.edge_mm(pos, q).is_some_and(|l| l >= LOCO_MM))
    }

    /// Puts a locomotive on the node at `pos`, rear at the node, facing along its first track. False (nothing
    /// done) unless `train_spot` says `Ok`.
    pub fn place_train(&mut self, pos: IVec3) -> bool {
        if self.train_spot(pos) != Spot::Ok {
            return false;
        }
        let Some(to) = self.launch(pos) else { return false };
        self.trains.push(Train::new(vec![(pos, to)], LOCO_MM));
        true
    }

    /// Moves every train one tick: a stopped one trades with its dock, the others drive.
    pub(super) fn step_trains(&mut self) {
        if self.trains.is_empty() {
            return;
        }
        let mut trains = std::mem::take(&mut self.trains);
        for t in &mut trains {
            if t.idle.is_some() {
                self.trade(t);
            } else {
                self.advance(t, STEP_MM);
            }
        }
        self.trains = trains;
    }

    fn advance(&self, t: &mut Train, mm: i64) {
        let Some(&(a, b)) = t.path.last() else { return };
        let Some(len) = self.edge_mm(a, b) else { return };
        let before = t.head;
        t.head += mm;
        if before < len && t.head >= len && t.cars > 0 && self.stop_dock(t, b).is_some() {
            t.head = len;
            t.idle = Some(0);
            return;
        }
        if t.head > len {
            match self.steer(t, a, b) {
                Some(c) => {
                    t.head -= len;
                    t.path.push((b, c));
                }
                None => {
                    t.head = len;
                    self.reverse(t);
                }
            }
        }
        self.trim(t);
    }

    /// Drops the edges the train has left behind (those whose far end is more than a locomotive back).
    fn trim(&self, t: &mut Train) {
        while t.path.len() > 1 {
            let last = t.path.len() - 1;
            let mid: i64 = t.path[1..last].iter().map(|&(a, b)| self.edge_mm(a, b).unwrap_or(0)).sum();
            if t.head + mid < t.len_mm() {
                break;
            }
            t.path.remove(0);
        }
    }

    /// The train turns round: its tail becomes the head.
    fn reverse(&self, t: &mut Train) {
        let before: i64 = t.path[..t.path.len() - 1].iter().map(|&(a, b)| self.edge_mm(a, b).unwrap_or(0)).sum();
        let first = t.path[0];
        let tail = t.head + before - t.len_mm();
        let first_len = self.edge_mm(first.0, first.1).unwrap_or(0);
        t.path = t.path.iter().rev().map(|&(a, b)| (b, a)).collect();
        t.head = first_len - tail;
    }

    /// Drops trains whose track is gone (a damaged save).
    pub(super) fn prune_trains(&mut self) {
        let keep: Vec<bool> = self
            .trains
            .iter()
            .map(|t| {
                let ends = t.path.iter().all(|&(a, b)| self.track_between(a, b));
                let joined = t.path.windows(2).all(|w| w[0].1 == w[1].0);
                let inside =
                    t.path.last().and_then(|&(a, b)| self.edge_mm(a, b)).is_some_and(|l| (0..=l).contains(&t.head));
                let covered = self.tail_room(t).is_some_and(|room| room >= 0);
                let stopped = t.idle.is_none() || t.path.last().and_then(|&(a, b)| self.edge_mm(a, b)) == Some(t.head);
                !t.path.is_empty() && ends && joined && inside && covered && stopped
            })
            .collect();
        let mut keep = keep.into_iter();
        self.trains.retain(|_| keep.next().unwrap_or(false));
    }

    pub(super) fn write_trains(&self, w: &mut ByteWriter) {
        w.count(self.trains.len());
        for t in &self.trains {
            w.count(t.path.len());
            for &(a, b) in &t.path {
                w.ivec3(a);
                w.ivec3(b);
            }
            w.u32(t.head as u32);
            w.u8(t.cars);
            t.cargo.write_state(w);
            w.bool(t.idle.is_some());
            w.u32(t.idle.unwrap_or(0));
            w.count(t.schedule.len());
            t.schedule.iter().for_each(|&d| w.ivec3(d));
            w.u8(t.next);
        }
    }

    /// Saves from before version 33 hold locomotives only (no wagons, never stopped), before 34 no schedules.
    pub(super) fn read_trains(&mut self, r: &mut ByteReader) -> Option<()> {
        for _ in 0..r.count()? {
            let mut path = Vec::new();
            for _ in 0..r.count()? {
                path.push((r.ivec3()?, r.ivec3()?));
            }
            let mut t = Train::new(path, i64::from(r.u32()?));
            if r.version >= 33 {
                t.cars = r.u8()?;
                if t.cars > MAX_CARS {
                    return None;
                }
                t.cargo = Buffer::read_state(r, usize::from(t.cars) * CAR_SLOTS)?;
                let (stopped, idle) = (r.bool()?, r.u32()?);
                t.idle = stopped.then_some(idle);
            }
            if r.version >= 34 {
                let stops = r.count()?;
                if stops > schedule::MAX_STOPS {
                    return None;
                }
                for _ in 0..stops {
                    t.schedule.push(r.ivec3()?);
                }
                t.next = r.u8()?;
                if usize::from(t.next) >= stops.max(1) {
                    t.next = 0;
                }
            }
            self.trains.push(t);
        }
        self.prune_trains();
        Some(())
    }
}

#[cfg(test)]
mod tests;
