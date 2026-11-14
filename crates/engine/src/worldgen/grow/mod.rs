//! Generator 12's trees, grown rather than stamped from a layout: a species row (`kinds.rs`) drives a trunk that can
//! lean and narrow, limbs of log blocks that leave it at angles and droop, and leaf clusters at their ends
//! (`shapes.rs`). Generation (`place.rs`) and grown saplings (`sim/saplings.rs`) both call `grow`, so they look
//! alike.
//!
//! Invariants: a tree is a pure function of its kind, foot, height and the seed, built in its own box first, so
//! it never depends on what is around it. Every leaf it puts is within `LEAF_SUPPORT_STEPS` face steps of one of its
//! logs through its own leaves (so none decays as generated), and below its crown no leaf is within a block and a
//! half of its trunk (a trunk you can see and reach). Nothing reaches more than `REACH` blocks from the foot across.
//! To add a species: `kinds.rs`.

mod kinds;
mod place;
mod shapes;

use kinds::Kind;
pub use kinds::{kind_of_sapling, KINDS};

use crate::block::{BlockId, DIRT, GRASS};
use crate::math::{hash3, IVec3};
use crate::sim::LEAF_SUPPORT_STEPS;

/// The furthest any part of a tree reaches from its foot across (x or z).
pub const REACH: i32 = 9;
/// How far above its trunk's top a crown may reach.
const ABOVE: i32 = 6;

/// A tree of `kind` standing on the block at `ground` with a trunk `height` blocks tall, block by block: leaves
/// (which only go into air), then logs (which replace anything), then dirt under the trunk.
pub fn grow(kind: usize, ground: IVec3, height: i32, seed: u32, put: impl FnMut(IVec3, BlockId, bool)) {
    grow_as(&KINDS[kind], ground, height, seed, put);
}

/// A planted sapling's tree, young: the kind's shape on a trunk one block wide and at most `YOUNG_MAX` tall (the
/// lower of its heights from `roll`), so it fits a forester's patch and a garden.
pub fn grow_young(kind: usize, ground: IVec3, roll: u32, seed: u32, put: impl FnMut(IVec3, BlockId, bool)) {
    let least = KINDS[kind].height.0;
    let height = ((least * 3 / 5).max(4) + (roll % (least * 2 / 5).max(1) as u32) as i32).min(YOUNG_MAX);
    grow_as(&Kind { width: 1, ..KINDS[kind] }, ground, height, seed, put);
}

/// The tallest trunk a sapling grows.
pub const YOUNG_MAX: i32 = 10;

fn grow_as(k: &Kind, ground: IVec3, height: i32, seed: u32, mut put: impl FnMut(IVec3, BlockId, bool)) {
    let mut plot = Plot::new(ground, height);
    let base = shapes::build(k, &mut plot, height, hash3(seed ^ 0x6A0F, ground.x, ground.y, ground.z));
    plot.clear_bole(base);
    plot.prune();
    for (p, cell) in plot.cells() {
        if cell == LEAF {
            put(p, k.leaves, false);
        }
    }
    for (p, cell) in plot.cells() {
        if cell == WOOD {
            put(p, k.log, true);
        }
    }
    if k.ground == GRASS {
        put(ground, DIRT, true);
    }
}

/// A kind's trunk height on a spot whose hash is `h`.
pub fn height_of(kind: usize, h: u32) -> i32 {
    let k = &KINDS[kind];
    k.height.0 + (h % (k.height.1 + 1)) as i32
}

/// The highest block a tree of trunk `height` can reach above its foot.
pub fn top_of(height: i32) -> i32 {
    height + ABOVE
}

const AIR: u8 = 0;
const LEAF: u8 = 1;
const WOOD: u8 = 2;

/// The box a tree is built in: `REACH` across each way from the foot, from a few blocks under it to `ABOVE` over
/// its trunk's top.
pub(super) struct Plot {
    origin: IVec3,
    size: IVec3,
    cells: Vec<u8>,
    /// The trunk's centre at each height above the foot (x, z), for clearing the bole.
    trunk: Vec<(f32, f32, f32)>,
}

/// How far below the foot a trunk may root (into a slope).
const BELOW: i32 = 3;

impl Plot {
    fn new(ground: IVec3, height: i32) -> Self {
        let size = IVec3::new(2 * REACH + 1, BELOW + 1 + top_of(height), 2 * REACH + 1);
        let origin = ground - IVec3::new(REACH, BELOW, REACH);
        Plot { origin, size, cells: vec![AIR; (size.x * size.y * size.z) as usize], trunk: Vec::new() }
    }

    fn index(&self, p: IVec3) -> Option<usize> {
        let l = p - self.origin;
        let inside =
            (0..self.size.x).contains(&l.x) && (0..self.size.y).contains(&l.y) && (0..self.size.z).contains(&l.z);
        inside.then(|| ((l.y * self.size.z + l.z) * self.size.x + l.x) as usize)
    }

    /// The foot: the block the tree stands on.
    pub(super) fn foot(&self) -> IVec3 {
        self.origin + IVec3::new(REACH, BELOW, REACH)
    }

    pub(super) fn wood(&mut self, p: IVec3) {
        if let Some(i) = self.index(p) {
            self.cells[i] = WOOD;
        }
    }

    pub(super) fn leaf(&mut self, p: IVec3) {
        if let Some(i) = self.index(p) {
            if self.cells[i] == AIR {
                self.cells[i] = LEAF;
            }
        }
    }

    pub(super) fn is_wood(&self, p: IVec3) -> bool {
        self.index(p).is_some_and(|i| self.cells[i] == WOOD)
    }

    /// Records the trunk's centre at the next height up from the foot, and its half width.
    pub(super) fn trunk_at(&mut self, x: f32, z: f32, half: f32) {
        self.trunk.push((x, z, half));
    }

    /// Removes the leaves near the trunk below height `base` above the foot.
    fn clear_bole(&mut self, base: i32) {
        let foot = self.foot();
        for (dy, &(x, z, half)) in self.trunk.iter().enumerate().take(base.max(0) as usize) {
            let y = foot.y + 1 + dy as i32;
            let r = 1.5 + half;
            for dz in -REACH..=REACH {
                for dx in -REACH..=REACH {
                    let (px, pz) = ((foot.x + dx) as f32, (foot.z + dz) as f32);
                    if (px - x).powi(2) + (pz - z).powi(2) <= r * r {
                        let p = IVec3::new(foot.x + dx, y, foot.z + dz);
                        if let Some(i) = self.index(p) {
                            if self.cells[i] == LEAF {
                                self.cells[i] = AIR;
                            }
                        }
                    }
                }
            }
        }
    }

    /// Removes every leaf more than `LEAF_SUPPORT_STEPS` face steps from a log through leaves.
    fn prune(&mut self) {
        let mut dist = vec![u8::MAX; self.cells.len()];
        let mut frontier: Vec<usize> = Vec::new();
        for (i, &c) in self.cells.iter().enumerate() {
            if c == WOOD {
                dist[i] = 0;
                frontier.push(i);
            }
        }
        let (sx, sy, sz) = (self.size.x as usize, self.size.y as usize, self.size.z as usize);
        for step in 1..=LEAF_SUPPORT_STEPS as u8 {
            let mut next = Vec::new();
            for i in frontier {
                let (x, rest) = (i % sx, i / sx);
                let (z, y) = (rest % sz, rest / sz);
                let mut visit = |j: usize| {
                    if self.cells[j] == LEAF && dist[j] == u8::MAX {
                        dist[j] = step;
                        next.push(j);
                    }
                };
                if x > 0 {
                    visit(i - 1);
                }
                if x + 1 < sx {
                    visit(i + 1);
                }
                if z > 0 {
                    visit(i - sx);
                }
                if z + 1 < sz {
                    visit(i + sx);
                }
                if y > 0 {
                    visit(i - sx * sz);
                }
                if y + 1 < sy {
                    visit(i + sx * sz);
                }
            }
            frontier = next;
        }
        for (c, d) in self.cells.iter_mut().zip(dist) {
            if *c == LEAF && d == u8::MAX {
                *c = AIR;
            }
        }
    }

    /// Every cell that is not air, with its world position.
    fn cells(&self) -> impl Iterator<Item = (IVec3, u8)> + '_ {
        let (sx, sz) = (self.size.x as usize, self.size.z as usize);
        self.cells.iter().enumerate().filter(|(_, &c)| c != AIR).map(move |(i, &c)| {
            let (x, rest) = (i % sx, i / sx);
            (self.origin + IVec3::new(x as i32, (rest / sz) as i32, (rest % sz) as i32), c)
        })
    }
}

#[cfg(test)]
mod tests;
