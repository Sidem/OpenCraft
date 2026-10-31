//! Docks are pieces of rail (user, 2026-10-30): a loading or unloading dock (`process/docks.rs`) stands on the cell
//! of a rail node and is that node, heading along its axis (`rail_yaw` answers for it), so tracks run into and out
//! of it like any node's and a train stops right on it. A dock goes only onto a straight rail node (`dock_fit`: a
//! heading along a grid axis, at most one track leaving each end, no signal, no train on it) and takes its place
//! keeping the tracks (`swap_rail_piece`); breaking it puts the node back with them (`Sim::dismantle`).
//!
//! Invariants: a dock's heading is its `dir` turned to a heading byte (`dir * 64`); a dock end holds one track.
//! To add another rail piece: a `Pick` that `docks()`, and its block in `swap_rail_piece`'s callers.

use crate::block::{BlockId, AIR, MACHINE_PART, RAIL};
use crate::inventory::Stack;
use crate::math::IVec3;
use crate::world::World;

use super::super::footprint::Footprint;
use super::super::links::Slot;
use super::super::Factory;
use super::{dir_of, Track};

#[cfg(test)]
mod tests;

/// Heading steps between grid axes.
const AXIS: u8 = 64;

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
