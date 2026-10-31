//! Docks are pieces of rail (user, 2026-10-30): a loading or unloading dock (`process/docks.rs`) stands on the cell
//! of a rail node and is that node, heading along its axis (`rail_yaw` answers for it), so tracks run into and out
//! of it like any node's and a train stops right on it. A dock goes only onto a straight rail node (`dock_fit`: a
//! heading along a grid axis, at most one track leaving each end, no signal, no train on it) and takes its place
//! keeping the tracks (`swap_rail_piece`); breaking it puts the node back with them (`Sim::dismantle`).
//!
//! Invariants: a dock's heading is its `dir` turned to a heading byte (`dir * 64`); a dock end holds one track.
//! To add another rail piece: a `Pick` that `docks()`, and its block in `swap_rail_piece`'s callers.

use crate::block::{replaceable, BlockId, AIR, MACHINE_PART, RAIL};
use crate::inventory::Stack;
use crate::math::IVec3;
use crate::world::World;

use super::super::footprint::Footprint;
use super::super::links::Slot;
use super::super::Factory;
use super::ground::{faults as rail_faults, node_stands, Fault};
use super::{dir_of, fit, Curve, Fit, Track};

#[cfg(test)]
mod tests;

/// Heading steps between grid axes.
const AXIS: u8 = 64;
/// How far above or below a straight track a cell aimed at still counts as on it (blocks).
const REACH: i32 = 2;

/// Whether a dock can go onto the cell `pos`, and why not.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum DockFit {
    Ok,
    /// No rail node there.
    NoNode,
    /// The node's track runs on a diagonal: docks stand north-south or east-west.
    Diagonal,
    /// More than one track leaves an end of it (a junction).
    Junction,
    Signal,
    /// A train is on a track that ends there.
    Train,
}

/// Why a dock cannot go into the middle of a straight track at a cell (`Factory::split_fit`).
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum SplitWhy {
    /// A train is on the track.
    Train,
    /// The cell is taken, or has no ground under it.
    Spot,
    /// One of the two stretches it would make does not fit (too short, too steep).
    Fit(Fit),
    /// One of them would not lie on the ground: the cells to fill or clear.
    Ground(Vec<(IVec3, Fault)>),
}

/// Whether `block` is a dock (a rail piece of its own).
pub fn is_dock(block: BlockId) -> bool {
    super::super::process::spec(block).is_some_and(|s| s.pick.docks())
}

impl Factory {
    /// The dock (a processor index) whose cell `pos` is, if one is.
    pub(crate) fn dock_piece(&self, pos: IVec3) -> Option<usize> {
        match self.at.get(&pos) {
            Some(&Slot::Process(i)) if self.processors[i as usize].spec.pick.docks() => Some(i as usize),
            _ => None,
        }
    }

    /// Whether a dock can go onto the cell `pos`.
    pub fn dock_fit(&self, pos: IVec3) -> DockFit {
        let Some(&Slot::Rail(i)) = self.at.get(&pos) else { return DockFit::NoNode };
        let rail = &self.rails[i as usize];
        if !rail.yaw.is_multiple_of(AXIS) {
            DockFit::Diagonal
        } else if rail.signal {
            DockFit::Signal
        } else if !self.one_track_each_end(pos, rail.yaw) {
            DockFit::Junction
        } else if self.trains.iter().any(|t| t.touches(pos)) {
            DockFit::Train
        } else {
            DockFit::Ok
        }
    }

    /// Whether a track from the node at `pos` facing `yaw` to `to` would leave by an end that already has one.
    pub(super) fn end_taken(&self, pos: IVec3, yaw: u8, to: IVec3) -> bool {
        let side = end_of(pos, yaw, to);
        self.tracks.iter().filter_map(|t| other_end(t, pos)).any(|q| end_of(pos, yaw, q) == side)
    }

    /// Puts `block` (a rail node or a dock) at `pos` in place of the rail piece there, with the same heading and
    /// tracks; returns what the old piece held. The world's block is the caller's (`Sim`). Nothing if `pos` holds no
    /// rail piece.
    pub fn swap_rail_piece(&mut self, world: &mut World, pos: IVec3, block: BlockId) -> Vec<Stack> {
        let Some(yaw) = self.rail_yaw(pos) else { return Vec::new() };
        let kept: Vec<Track> = self.tracks.iter().filter(|t| t.has(pos)).copied().collect();
        let held = self.remove(pos);
        let facing = if block == RAIL { yaw } else { yaw / AXIS };
        self.place(world, block, pos, facing, pos, 0);
        self.tracks.extend(kept);
        self.dirty = true;
        held
    }

    /// The straight tracks (both nodes heading along the same grid axis, on one line) that run through the column of
    /// `cell` strictly between their nodes, each with the two cells in that column the track's height falls
    /// between (one if it is level there). Nothing for a cell more than `REACH` above or below the track.
    pub fn tracks_through(&self, cell: IVec3) -> impl Iterator<Item = (Track, [IVec3; 2])> + '_ {
        self.tracks.iter().filter_map(move |&t| {
            let (ya, yb) = (self.rail_yaw(t.a)?, self.rail_yaw(t.b)?);
            if !ya.is_multiple_of(AXIS) || ya % 128 != yb % 128 {
                return None;
            }
            // North-south track runs along z at one x; east-west along x at one z.
            let pick = |p: IVec3| if ya % 128 == 0 { (p.z, p.x) } else { (p.x, p.z) };
            let ((at, line), (from, line_a), (to, line_b)) = (pick(cell), pick(t.a), pick(t.b));
            // Distance along the track from node a, and its length, both counted towards b.
            let (len, k) = ((to - from).abs(), (at - from) * (to - from).signum());
            if line != line_a || line_b != line_a || k <= 0 || k >= len {
                return None;
            }
            // The track's height in this column: a + rise * k / len, between `low` and the cell above.
            let rise = (t.b.y - t.a.y) * k;
            let low = t.a.y + rise.div_euclid(len);
            let high = low + i32::from(rise.rem_euclid(len) != 0);
            (cell.y >= low - REACH && cell.y <= high + REACH)
                .then_some((t, [IVec3::new(cell.x, low, cell.z), IVec3::new(cell.x, high, cell.z)]))
        })
    }

    /// Whether a dock can stand at `pos` (a cell `tracks_through` gave) in the middle of the track `t`, which it
    /// would cut in two: no train on it, a free cell with ground under it, and both new stretches fit and lie on
    /// the ground (`block` is the world's blocks).
    pub fn split_fit(&self, t: Track, pos: IVec3, mut block: impl FnMut(IVec3) -> BlockId) -> Result<(), SplitWhy> {
        let (Some(ya), Some(yb)) = (self.rail_yaw(t.a), self.rail_yaw(t.b)) else { return Err(SplitWhy::Spot) };
        if self.train_on(t.a, t.b) {
            return Err(SplitWhy::Train);
        }
        if !replaceable(block(pos)) || !node_stands(&mut block, pos) {
            return Err(SplitWhy::Spot);
        }
        let mut faults = Vec::new();
        for (a, ya, b, yb) in [(t.a, ya, pos, ya), (pos, ya, t.b, yb)] {
            match fit(a, ya, b, yb) {
                Fit::Ok => faults.extend(rail_faults(&Curve::new(a, ya, b, yb), &mut block)),
                other => return Err(SplitWhy::Fit(other)),
            }
        }
        if faults.is_empty() {
            Ok(())
        } else {
            Err(SplitWhy::Ground(faults))
        }
    }

    /// Cuts the track `t` in two at `pos` with `dock` standing there (the caller checked `split_fit`, and sets the
    /// world's block): both halves keep the heading of the track, so the dock has one track at each end.
    pub fn split_track(&mut self, world: &mut World, t: Track, pos: IVec3, dock: BlockId) {
        let yaw = self.rail_yaw(t.a).unwrap_or(0);
        self.tracks.retain(|&x| x != t);
        self.place(world, dock, pos, yaw / AXIS, pos, 0);
        self.tracks.extend([Track { a: t.a, b: pos }, Track { a: pos, b: t.b }]);
        self.dirty = true;
    }

    /// Saves before version 64 stood docks 2×2 beside the track: each keeps its anchor cell, holding what it held,
    /// and its other three cells (machine parts) are cleared. Such a dock is a rail piece with no track yet.
    pub(in crate::factory) fn shrink_old_docks(&self, world: &mut World) {
        const OLD: Footprint = Footprint { size: [2, 2, 1], ports: &[] };
        for p in self.processors.iter().filter(|p| p.spec.pick.docks()) {
            for c in OLD.cells(p.pos, p.dir).into_iter().skip(1) {
                if world.block_anywhere_or_generate(c) == MACHINE_PART {
                    world.set_block_anywhere(c, AIR);
                }
            }
        }
    }

    /// Whether at most one track leaves each end of the node at `pos` facing `yaw`.
    fn one_track_each_end(&self, pos: IVec3, yaw: u8) -> bool {
        let ends: Vec<bool> =
            self.tracks.iter().filter_map(|t| other_end(t, pos)).map(|q| end_of(pos, yaw, q)).collect();
        ends.iter().filter(|&&e| e).count() <= 1 && ends.iter().filter(|&&e| !e).count() <= 1
    }
}

/// The far end of `t` if it ends at `pos`.
fn other_end(t: &Track, pos: IVec3) -> Option<IVec3> {
    t.has(pos).then(|| if t.a == pos { t.b } else { t.a })
}

/// Which end of the node at `pos` facing `yaw` a track to `to` leaves by: true for the end the heading points to.
fn end_of(pos: IVec3, yaw: u8, to: IVec3) -> bool {
    let (x, z) = dir_of(yaw);
    let d = to - pos;
    x * f64::from(d.x) + z * f64::from(d.z) > 0.0
}
