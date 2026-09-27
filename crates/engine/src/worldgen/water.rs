//! Version 3 water (Milestone 5): the sea and ponds as generation places them. Generated water is still;
//! it only moves once something next to it changes (flowing water, a later step).
//!
//! - The sea: every column whose ground lies below `SEA_LEVEL` holds water up to it; sand covers the
//!   ground under water and up to one block above the sea (`surface_v3`).
//! - Ponds: at most one per `POND_CELL`² cell, on flat plains or lowlands well above the sea. A pond is a
//!   bowl carved into `height_at` (`shape_ponds`) and filled to its level (the lowest of 16 rim samples,
//!   minus 1). A bank ring around the bowl is raised to that level, so the water stays in whatever the
//!   ground between the samples does.
//! - Watertight: no cave is carved within 2 blocks of water ([`WaterGuard`]).
//!
//! Invariants: a pure function of the seed and the column; versions 1 and 2 never call into this file.
//! A pond stays inside its own cell (`POND_MARGIN`), so a column only asks its own cell; pond lookups are
//! cached per cell, which never changes results. To tune: the constants below.

use crate::block::{BlockId, SAND};
use crate::math::{hash2, unit};

use super::strata::DIRS16;
use super::{Biome, WorldGen, SEA_LEVEL};

/// Ponds: one chance per cell of this many blocks square.
const POND_CELL: i32 = 96;
const POND_CHANCE: f64 = 0.5;
/// Bowl radius range, how deep the middle goes below the level, and the raised bank's width.
const POND_RADIUS: (i32, i32) = (5, 9);
const POND_DEPTH: (i32, i32) = (2, 4);
const BANK: i32 = 2;
/// A pond's centre keeps this far from its cell's edges, so everything it shapes stays in the cell.
const POND_MARGIN: i32 = POND_RADIUS.1 + BANK + 2;
/// No pond within this many blocks of spawn.
const POND_SPAWN_CLEARING: i32 = 32;
/// Most height difference allowed among a pond's centre and rim samples (flat ground only).
const POND_FLATNESS: i32 = 3;
/// No cave within this many blocks of water.
const CAVE_GUARD: i32 = 2;

/// A pond: centre, bowl radius and depth, and the height of its top water block.
#[derive(Clone, Copy, Debug)]
pub(super) struct Pond {
    pub x: i32,
    pub z: i32,
    pub r: i32,
    depth: i32,
    pub level: i32,
}

/// Around one chunk column (a margin of `CAVE_GUARD` on every side): each column's ground height and
/// top water block (`i32::MIN` when dry). Built only where water is near.
pub(super) struct WaterGuard {
    cells: Vec<(i32, i32)>,
}

/// The guard grid's width: the 32 columns plus the margin on both sides.
pub(super) const GUARD_SPAN: usize = 32 + 2 * CAVE_GUARD as usize;

impl WaterGuard {
    /// `cells` holds (ground, water top) for the `GUARD_SPAN`² columns, row by row; `None` if none is wet.
    pub fn new(cells: Vec<(i32, i32)>) -> Option<WaterGuard> {
        cells.iter().any(|c| c.1 != i32::MIN).then_some(WaterGuard { cells })
    }

    /// Whether a cave cell at column (x, z) of the chunk column and height y would come within
    /// `CAVE_GUARD` blocks of water.
    pub fn near(&self, x: usize, z: usize, y: i32) -> bool {
        let span = 2 * CAVE_GUARD as usize;
        (z..=z + span).any(|gz| {
            self.cells[gz * GUARD_SPAN + x..=gz * GUARD_SPAN + x + span]
                .iter()
                .any(|&(h, top)| top >= y - CAVE_GUARD && h < y + CAVE_GUARD)
        })
    }
}

impl WorldGen {
    /// The terrain height `h` after a pond reshapes it: a bowl inside its radius, a bank raised to its
    /// level around that.
    pub(super) fn shape_ponds(&self, x: i32, z: i32, h: i32) -> i32 {
        let Some(p) = self.pond_near(x, z) else { return h };
        let (d2, r2) = ((x - p.x).pow(2) + (z - p.z).pow(2), p.r * p.r);
        if d2 < r2 {
            h.min(p.level - 1 - p.depth * (r2 - d2) / r2)
        } else if d2 < (p.r + BANK).pow(2) {
            h.max(p.level)
        } else {
            h
        }
    }

    /// The top water block of the column at (x, z), whose ground is at `h`, or `i32::MIN` when dry.
    pub(super) fn water_top(&self, x: i32, z: i32, h: i32) -> i32 {
        if let Some(p) = self.pond_near(x, z) {
            if (x - p.x).pow(2) + (z - p.z).pow(2) < p.r * p.r && h < p.level {
                return p.level;
            }
        }
        if h < SEA_LEVEL {
            SEA_LEVEL
        } else {
            i32::MIN
        }
    }

    /// Version 3's (top, filler) pair: sand under water and on the shore, else `surface_v2`'s.
    pub(super) fn surface_v3(pair: (BlockId, BlockId), h: i32, wet: bool) -> (BlockId, BlockId) {
        if wet || h <= SEA_LEVEL + 1 {
            (SAND, SAND)
        } else {
            pair
        }
    }

    /// The pond of the cell holding (x, z), if its bowl or bank reaches that column.
    pub(super) fn pond_near(&self, x: i32, z: i32) -> Option<Pond> {
        let p = self.pond_in(x.div_euclid(POND_CELL), z.div_euclid(POND_CELL))?;
        let reach = p.r + BANK;
        ((x - p.x).abs() < reach && (z - p.z).abs() < reach).then_some(p)
    }

    /// The pond of cell (gx, gz), if it has one (cached).
    pub(super) fn pond_in(&self, gx: i32, gz: i32) -> Option<Pond> {
        if let Some((cell, p)) = self.last_pond.get() {
            if cell == (gx, gz) {
                return p;
            }
        }
        let cached = self.ponds.borrow().get(&(gx, gz)).copied();
        let p = cached.unwrap_or_else(|| {
            let p = self.find_pond(gx, gz);
            self.ponds.borrow_mut().insert((gx, gz), p);
            p
        });
        self.last_pond.set(Some(((gx, gz), p)));
        p
    }

    /// Decides cell (gx, gz)'s pond from hashes and the unshaped terrain.
    fn find_pond(&self, gx: i32, gz: i32) -> Option<Pond> {
        if unit(hash2(self.seed ^ 0x90DD, gx, gz)) >= POND_CHANCE {
            return None;
        }
        let h = hash2(self.seed ^ 0x90DE, gx, gz);
        let span = (POND_CELL - 2 * POND_MARGIN) as u32;
        let x = gx * POND_CELL + POND_MARGIN + (h % span) as i32;
        let z = gz * POND_CELL + POND_MARGIN + ((h >> 8) % span) as i32;
        let r = POND_RADIUS.0 + ((h >> 16) % (POND_RADIUS.1 - POND_RADIUS.0 + 1) as u32) as i32;
        let depth = POND_DEPTH.0 + ((h >> 24) % (POND_DEPTH.1 - POND_DEPTH.0 + 1) as u32) as i32;
        if x * x + z * z < (POND_SPAWN_CLEARING + r + BANK).pow(2) {
            return None;
        }
        let h0 = self.base_height(x, z);
        if !matches!(self.biome_at(x, z, h0), Biome::Plains | Biome::Lowlands) {
            return None;
        }
        let rim = DIRS16.map(|(dx, dz)| self.base_height(x + dx * (r + 1) / 1000, z + dz * (r + 1) / 1000));
        let (lo, hi) = (rim.iter().copied().fold(h0, i32::min), rim.iter().copied().fold(h0, i32::max));
        (lo > SEA_LEVEL + 3 && hi - lo <= POND_FLATNESS).then_some(Pond { x, z, r, depth, level: lo - 1 })
    }
}

#[cfg(test)]
mod tests;
