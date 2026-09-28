//! Multi-block machines (docs/TECH_TREE.md section 7): a processor's footprint, w × d × h cells turned
//! by the way its placer faced (`dir`), and the ports on its sides. The anchor cell (the machine's `pos`)
//! holds its block and the other cells hold `MACHINE_PART` (`action/multiblock.rs` places and clears
//! them). Every cell maps to the machine in `Factory.at`, so targeting, panels, breaking and power work
//! from any cell; belts link only through ports (`links.rs`).
//!
//! Geometry: from the anchor the footprint runs `w` cells to the placer's right, `d` cells away from the
//! placer (along `dir`) and `h` up; its front faces the placer. Sides are named as the placer sees
//! them. A port is every bottom-layer face on its side, marked by a hatch (`process/model.rs`).
//!
//! Invariants: a 1×1×1 footprint has no ports and takes and gives on every side like any machine;
//! `cells` lists the anchor first.
//!
//! To give a processor a footprint: its spec's `footprint` (size and ports).

use crate::block::{self, BlockId};
use crate::math::{IVec3, Vec3};

use super::links::Slot;
use super::{opposite, Factory, DIRS};

const UP: IVec3 = IVec3::new(0, 1, 0);

/// A side of a machine, as its placer sees it.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Side {
    Front,
    Back,
    Left,
    Right,
}

/// What a port does: belts pointing into an inlet deliver; belts leading away from an outlet take the
/// main product, and from a side outlet the byproduct (`process/`).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Role {
    In,
    Out,
    Side,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Port {
    pub side: Side,
    pub role: Role,
}

#[derive(Clone, Copy)]
pub struct Footprint {
    /// Cells across (to the placer's right), deep (away from the placer) and high.
    pub size: [i32; 3],
    pub ports: &'static [Port],
}

/// One cell, taking and giving on every side.
pub const SINGLE: Footprint = Footprint { size: [1, 1, 1], ports: &[] };

impl Side {
    /// The direction (a `DIRS` index) this side faces on a machine placed facing `dir`.
    pub fn facing(self, dir: u8) -> u8 {
        match self {
            Side::Front => opposite(dir),
            Side::Back => dir,
            Side::Left => (dir + 3) % 4,
            Side::Right => (dir + 1) % 4,
        }
    }
}

impl Footprint {
    pub fn is_single(&self) -> bool {
        self.size == [1, 1, 1]
    }

    /// Every cell of a machine anchored at `anchor` and placed facing `dir`, the anchor first.
    pub fn cells(&self, anchor: IVec3, dir: u8) -> Vec<IVec3> {
        let (right, back) = (DIRS[(dir as usize + 1) % 4], DIRS[dir as usize]);
        let [w, d, h] = self.size;
        let mut cells = Vec::with_capacity((w * d * h) as usize);
        for y in 0..h {
            for k in 0..d {
                for i in 0..w {
                    cells.push(anchor + right * i + back * k + UP * y);
                }
            }
        }
        cells
    }

    /// The footprint's centre relative to its anchor cell's centre.
    pub fn centre(&self, dir: u8) -> Vec3 {
        let (right, back) = (DIRS[(dir as usize + 1) % 4], DIRS[dir as usize]);
        let [w, d, h] = self.size.map(|s| (s - 1) as f64 * 0.5);
        right.as_vec3() * w + back.as_vec3() * d + Vec3::new(0.0, h, 0.0)
    }

    /// The faces (cell and the `DIRS` index it faces out of) where a machine anchored at `anchor`, facing
    /// `dir`, takes (`Role::In`) or gives (`Role::Out`, `Role::Side`) items: a single cell's four sides, else
    /// its ports.
    pub fn faces(&self, anchor: IVec3, dir: u8, role: Role) -> Vec<(IVec3, u8)> {
        if self.is_single() {
            return (0..4).map(|s| (anchor, s)).collect();
        }
        let [w, d, _] = self.size;
        let (right, back) = (DIRS[(dir as usize + 1) % 4], DIRS[dir as usize]);
        let at = |i: i32, k: i32| anchor + right * i + back * k;
        let mut faces = Vec::new();
        for p in self.ports.iter().filter(|p| p.role == role) {
            let s = p.side.facing(dir);
            match p.side {
                Side::Front => faces.extend((0..w).map(|i| (at(i, 0), s))),
                Side::Back => faces.extend((0..w).map(|i| (at(i, d - 1), s))),
                Side::Left => faces.extend((0..d).map(|k| (at(0, k), s))),
                Side::Right => faces.extend((0..d).map(|k| (at(w - 1, k), s))),
            }
        }
        faces
    }

    /// Whether a machine anchored at `anchor`, facing `dir`, takes items into `cell` from the cell `from`
    /// beside it (a single cell takes from anywhere).
    pub fn takes(&self, anchor: IVec3, dir: u8, cell: IVec3, from: IVec3) -> bool {
        self.is_single()
            || self.faces(anchor, dir, Role::In).iter().any(|&(c, s)| c == cell && c + DIRS[s as usize] == from)
    }
}

/// The footprint of the machine `block` places, if it takes more than one cell.
pub fn of(block: BlockId) -> Option<&'static Footprint> {
    super::process::spec(block).map(|s| &s.footprint).filter(|f| !f.is_single())
}

/// Which of `cells` are in the way of placing a machine (anything but air or water), by the block
/// `block_at` finds there (`None`: not loaded, so in the way).
pub fn blocked(cells: &[IVec3], mut block_at: impl FnMut(IVec3) -> Option<BlockId>) -> Vec<bool> {
    cells.iter().map(|&c| !block_at(c).is_some_and(block::replaceable)).collect()
}

impl Factory {
    /// The anchor, block and cells of the multi-block machine with a cell at `pos`.
    pub fn footprint_at(&self, pos: IVec3) -> Option<(IVec3, BlockId, Vec<IVec3>)> {
        let Slot::Process(i) = *self.at.get(&pos)? else { return None };
        let p = &self.processors[i as usize];
        (!p.spec.footprint.is_single()).then(|| (p.pos, p.spec.block, p.cells()))
    }

    /// The block of the multi-block machine with a cell at `pos` (what a `MACHINE_PART` belongs to).
    pub fn block_at(&self, pos: IVec3) -> Option<BlockId> {
        self.footprint_at(pos).map(|f| f.1)
    }
}

#[cfg(test)]
mod tests;
