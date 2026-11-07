//! A forester's patch as geometry (no state): the flat ground in front of it, its columns in the order the crane looks
//! them over, its plots and bounds, and the survey that says how much of it is flat.
//!
//! Sizes: `SIZES` rows ahead and columns to each side, one per tier; each contains the one before, so a forester works
//! the largest of its tier's sizes whose every column is flat ([`survey`]), and none at all when even the first is
//! not (it says so, and placing refuses it). Plots lie on a fixed lattice (every `PLOT_EVERY` cells ahead and across,
//! at least `PLOT_INSET` in from the far and side edges), so a bigger patch keeps the trees a smaller one planted.
//! To change a size: `SIZES` (a row per tier of `FORESTER_SPEC.tiers`, tested).

use crate::block::{is_leaves, is_log, is_sapling, BlockId, AIR, DIRT, GRASS};
use crate::math::IVec3;

use super::super::super::DIRS;

/// Rows ahead and columns to each side of the patch, by size (Mk1, Mk2, Mk3).
pub const SIZES: [(i32, i32); 3] = [(11, 5), (14, 8), (17, 11)];
/// Layers from the forester's own level up (a tall pine fits).
pub const PATCH_HEIGHT: i32 = 14;
/// Plots lie every this many cells ahead and across, at least this far in from the far and side edges.
const PLOT_EVERY: i32 = 3;
const PLOT_INSET: i32 = 2;

/// A forester's patch: where it stands, the way it faces (a `DIRS` index) and its size (an index into `SIZES`).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Patch {
    pub pos: IVec3,
    pub dir: u8,
    pub size: u8,
}

/// What a survey of the ground found: the size it can work (`None`: not even the first is flat), and how many
/// columns stop the next size (the first when it has none), with the first of them for a marker.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct Survey {
    pub size: Option<u8>,
    pub uneven: u16,
    pub blocker: Option<IVec3>,
}

impl Patch {
    /// Rows ahead and columns to each side.
    pub fn extent(self) -> (i32, i32) {
        SIZES[usize::from(self.size).min(SIZES.len() - 1)]
    }

    /// How many columns it has.
    pub fn columns(self) -> u16 {
        let (ahead, half) = self.extent();
        (ahead * (2 * half + 1)) as u16
    }

    /// Plots in it.
    pub fn plots(self) -> u16 {
        (0..self.columns()).filter(|&c| self.is_plot(c)).count() as u16
    }

    /// Column `c`'s foot: its cell level with the forester. Columns run row by row away from it, back and forth.
    pub fn column(self, c: u16) -> IVec3 {
        let (ahead, side) = self.place(c);
        self.cell(ahead, side, self.pos.y)
    }

    /// Column `c` as cells ahead and to the right.
    pub fn place(self, c: u16) -> (i32, i32) {
        let (_, half) = self.extent();
        let width = 2 * half + 1;
        let (row, k) = (i32::from(c) / width, i32::from(c) % width);
        (row + 1, if row % 2 == 1 { half - k } else { k - half })
    }

    /// Whether column `c` is a plot it plants.
    pub fn is_plot(self, c: u16) -> bool {
        let (ahead, side) = self.place(c);
        self.is_plot_at(ahead, side)
    }

    /// Whether the column `ahead` cells in front and `side` to the right is a plot.
    pub fn is_plot_at(self, ahead: i32, side: i32) -> bool {
        let (rows, half) = self.extent();
        ahead % PLOT_EVERY == 0
            && side % PLOT_EVERY == 0
            && ahead <= rows - PLOT_INSET
            && side.abs() <= half - PLOT_INSET
    }

    /// The highest layer.
    pub fn top(self) -> i32 {
        self.pos.y + PATCH_HEIGHT - 1
    }

    #[cfg(test)]
    pub fn contains(self, p: IVec3) -> bool {
        let ((d, r), rel) = (self.axes(), p - self.pos);
        let (ahead, side) = (rel.x * d.x + rel.z * d.z, rel.x * r.x + rel.z * r.z);
        let (rows, half) = self.extent();
        (1..=rows).contains(&ahead) && side.abs() <= half && (self.pos.y..=self.top()).contains(&p.y)
    }

    /// The lowest and highest corner cells.
    pub fn bounds(self) -> (IVec3, IVec3) {
        let (rows, half) = self.extent();
        let (a, b) = (self.cell(1, -half, self.pos.y), self.cell(rows, half, self.top()));
        (IVec3::new(a.x.min(b.x), a.y, a.z.min(b.z)), IVec3::new(a.x.max(b.x), b.y, a.z.max(b.z)))
    }

    /// The foot of every column that is not flat ground, read with `block`.
    pub fn uneven(self, mut block: impl FnMut(IVec3) -> BlockId) -> Vec<IVec3> {
        (0..self.columns()).map(|c| self.column(c)).filter(|&foot| !flat(&mut block, foot)).collect()
    }

    /// Ahead and to the right.
    pub fn axes(self) -> (IVec3, IVec3) {
        (DIRS[self.dir as usize % 4], DIRS[(self.dir as usize + 1) % 4])
    }

    /// The cell `ahead` cells in front and `side` to the right of the forester, at height `y`.
    pub fn cell(self, ahead: i32, side: i32, y: i32) -> IVec3 {
        let (d, r) = self.axes();
        IVec3::new(self.pos.x + d.x * ahead + r.x * side, y, self.pos.z + d.z * ahead + r.z * side)
    }
}

/// Surveys the ground in front of a forester at `pos` facing `dir` for sizes up to `tier`'s, reading blocks with `block`
/// (the core's `block_anywhere_or_generate`, or loaded chunks for the placing preview).
pub fn survey(pos: IVec3, dir: u8, tier: u8, mut block: impl FnMut(IVec3) -> BlockId) -> Survey {
    let top = usize::from(tier).min(SIZES.len() - 1);
    let big = Patch { pos, dir, size: top as u8 };
    // Per size: columns in it (and not in a smaller one) that are not flat, and the first of them.
    let mut uneven = [0u16; SIZES.len()];
    let mut first = [None; SIZES.len()];
    for c in 0..big.columns() {
        let (ahead, side) = big.place(c);
        let ring = SIZES.iter().position(|&(rows, half)| ahead <= rows && side.abs() <= half).unwrap_or(top);
        let foot = big.cell(ahead, side, pos.y);
        if !flat(&mut block, foot) {
            uneven[ring] += 1;
            first[ring] = first[ring].or(Some(foot));
        }
    }
    let ok = (0..=top).take_while(|&k| uneven[k] == 0).last();
    let next = ok.map_or(0, |k| k + 1);
    let (uneven, blocker) = if next <= top { (uneven[next], first[next]) } else { (0, None) };
    Survey { size: ok.map(|k| k as u8), uneven, blocker }
}

/// Whether the column whose foot is `foot` is flat ground a tree can stand on: grass or dirt under it, and the foot and
/// the cell above it open (air, or a tree's logs, leaves or a sapling, which the forester deals with).
fn flat(block: &mut impl FnMut(IVec3) -> BlockId, foot: IVec3) -> bool {
    let open = |b: BlockId| b == AIR || is_log(b) || is_leaves(b) || is_sapling(b);
    let up = IVec3::new(0, 1, 0);
    matches!(block(foot - up), DIRT | GRASS) && open(block(foot)) && open(block(foot + up))
}
