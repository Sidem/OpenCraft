//! Trees: where one stands (`Tree`, chosen per cell in `WorldGen::trees_near`) and how one is built block by block
//! (`tree_blocks`, shared with grown saplings in `sim/saplings.rs` so they look alike). Version 1 depends on both.
//!
//! A tree is a row of `SPECIES` (blocks, shape, trunk height, the ground it needs) and a `Shape` that lays its blocks;
//! versions before 9 only ever grow the oak. To add a species: a row, an index constant, a `Shape` arm if it needs a new
//! shape, a line in `pick`, and (for one with leaves) its sapling in `block/forest.rs` and `species_of_sapling`.

use super::Biome;
use crate::block::{
    BlockId, AIR, BASALT, BIRCH_LEAVES, BIRCH_LOG, BIRCH_SAPLING, CACTUS, DIRT, GRASS, LEAVES, LOG, PINE_LEAVES,
    PINE_SAPLING, SAND, SNAG_LOG,
};
use crate::chunk::{index, CHUNK_SIZE};
use crate::math::{hash3, IVec3};

/// A tree to stamp: its trunk's foot column, the ground it stands on, its trunk's height and its species.
pub(super) struct Tree {
    pub x: i32,
    pub z: i32,
    pub ground: i32,
    pub trunk: i32,
    pub species: usize,
}

#[derive(Clone, Copy)]
pub(super) enum Shape {
    /// Two wide layers and two narrow ones, corners trimmed at random (the oak, version 1's tree).
    Round,
    /// Low and broad: three layers up to three blocks out.
    Wide,
    /// Rings of needles that narrow to a point.
    Cone,
    /// A small round crown on a tall trunk.
    Crown,
    /// A bare column, no leaves.
    Column,
    /// A bare trunk with a stub of a branch.
    Snag,
}

pub(super) struct Species {
    pub log: BlockId,
    pub leaves: BlockId,
    pub shape: Shape,
    /// The trunk is `trunk_min` plus 0 to `spread - 1` blocks.
    pub trunk_min: i32,
    pub spread: u32,
    /// The surface block it grows on (generation only) and what is put under the trunk (none: the ground stays).
    pub ground: BlockId,
    pub foot: Option<BlockId>,
}

pub(super) const OAK: usize = 0;
pub(super) const BROAD: usize = 1;
pub(super) const PINE: usize = 2;
pub(super) const BIRCH: usize = 3;
pub(super) const CACTUS_PLANT: usize = 4;
pub(super) const SNAG: usize = 5;

pub(super) const SPECIES: [Species; 6] = [
    Species { log: LOG, leaves: LEAVES, shape: Shape::Round, trunk_min: 4, spread: 3, ground: GRASS, foot: Some(DIRT) },
    Species { log: LOG, leaves: LEAVES, shape: Shape::Wide, trunk_min: 3, spread: 2, ground: GRASS, foot: Some(DIRT) },
    Species {
        log: LOG,
        leaves: PINE_LEAVES,
        shape: Shape::Cone,
        trunk_min: 6,
        spread: 3,
        ground: GRASS,
        foot: Some(DIRT),
    },
    Species {
        log: BIRCH_LOG,
        leaves: BIRCH_LEAVES,
        shape: Shape::Crown,
        trunk_min: 5,
        spread: 3,
        ground: GRASS,
        foot: Some(DIRT),
    },
    Species { log: CACTUS, leaves: AIR, shape: Shape::Column, trunk_min: 2, spread: 3, ground: SAND, foot: None },
    Species { log: SNAG_LOG, leaves: AIR, shape: Shape::Snag, trunk_min: 3, spread: 4, ground: BASALT, foot: None },
];

/// The species a planted sapling grows into.
pub(crate) fn species_of_sapling(sapling: BlockId) -> usize {
    match sapling {
        BIRCH_SAPLING => BIRCH,
        PINE_SAPLING => PINE,
        _ => OAK,
    }
}

/// A species' trunk heights: the least, and how many heights it has.
pub(crate) fn trunk_range(species: usize) -> (i32, u32) {
    (SPECIES[species].trunk_min, SPECIES[species].spread)
}

/// Generator version 9: the species of a tree candidate in `biome`, from the cell's hash `h`.
pub(super) fn pick(biome: Biome, h: u32) -> usize {
    let roll = (h >> 24) % 10;
    match biome {
        Biome::Plains => [OAK, OAK, OAK, OAK, OAK, OAK, OAK, BIRCH, BIRCH, BIRCH][roll as usize],
        Biome::Lowlands => [BROAD, BROAD, BROAD, BROAD, BROAD, BROAD, OAK, OAK, OAK, BIRCH][roll as usize],
        Biome::Highlands => [PINE, PINE, PINE, PINE, PINE, PINE, PINE, PINE, PINE, OAK][roll as usize],
        Biome::Desert => CACTUS_PLANT,
        Biome::BasaltFields => SNAG,
    }
}

/// Chances a candidate cell holds a tree of a bare species, in thousandths (the others follow the forest noise).
pub(super) fn bare_density(species: usize) -> Option<u32> {
    match species {
        CACTUS_PLANT => Some(70),
        SNAG => Some(50),
        _ => None,
    }
}

pub(super) fn stamp_tree(t: &Tree, base: IVec3, seed: u32, b: &mut [BlockId]) {
    tree_blocks(IVec3::new(t.x, t.ground, t.z), t.trunk, t.species, seed, |p, id, replace_solid| {
        let l = p - base;
        if !(0..CHUNK_SIZE).contains(&l.x) || !(0..CHUNK_SIZE).contains(&l.y) || !(0..CHUNK_SIZE).contains(&l.z) {
            return;
        }
        let i = index(l.x as usize, l.y as usize, l.z as usize);
        if replace_solid || b[i] == AIR {
            b[i] = id;
        }
    });
}

/// A tree of `species` standing on the block at `ground` with a trunk `trunk` blocks tall, block by block: leaves
/// (which only go into air), then the logs and the dirt under them (which replace anything). Shared
/// by generation and grown saplings (`sim/saplings.rs`), so they look alike. Version 1 depends on it for the oak.
pub fn tree_blocks(ground: IVec3, trunk: i32, species: usize, seed: u32, mut put: impl FnMut(IVec3, BlockId, bool)) {
    let sp = &SPECIES[species];
    let (x, z) = (ground.x, ground.z);
    let top = ground.y + trunk;
    // Leaves of a layer at height `y` out to `r`, skipping the cells `skip(dx, dz)` names.
    let mut layer = |y: i32, r: i32, skip: &dyn Fn(i32, i32) -> bool| {
        for dz in -r..=r {
            for dx in -r..=r {
                if !skip(dx, dz) {
                    put(IVec3::new(x + dx, y, z + dz), sp.leaves, false);
                }
            }
        }
    };
    let coin = |dx: i32, y: i32, dz: i32| hash3(seed ^ 0x1EAF, x + dx, y, z + dz) & 1 == 0;
    match sp.shape {
        Shape::Round => {
            // Canopy: two wide layers, then two narrow ones; corners are randomly trimmed.
            for y in (top - 2)..=(top + 1) {
                let r: i32 = if y < top { 2 } else { 1 };
                layer(y, r, &|dx, dz| dx.abs() == r && dz.abs() == r && (y == top + 1 || coin(dx, y, dz)));
            }
        }
        Shape::Wide => {
            layer(top - 1, 3, &|dx, dz| dx * dx + dz * dz > 10 || (dx * dx + dz * dz > 6 && coin(dx, top - 1, dz)));
            layer(top, 3, &|dx, dz| dx * dx + dz * dz > 9 || (dx * dx + dz * dz > 5 && coin(dx, top, dz)));
            layer(top + 1, 2, &|dx, dz| dx * dx + dz * dz > 5);
        }
        Shape::Cone => {
            for y in (ground.y + 3)..=(top + 2) {
                let r = match top + 2 - y {
                    0 => 0,
                    1 | 2 => 1,
                    3 | 4 => 2,
                    _ => 3,
                };
                layer(y, r, &|dx, dz| {
                    dx * dx + dz * dz > r * r + 1 || (r == 3 && dx * dx + dz * dz > 6 && coin(dx, y, dz))
                });
            }
        }
        Shape::Crown => {
            layer(top - 1, 2, &|dx, dz| dx.abs() == 2 && dz.abs() == 2);
            layer(top, 2, &|dx, dz| dx.abs() == 2 && dz.abs() == 2);
            layer(top + 1, 1, &|dx, dz| dx != 0 && dz != 0);
        }
        Shape::Column | Shape::Snag => {}
    }
    for y in (ground.y + 1)..=top {
        put(IVec3::new(x, y, z), sp.log, true);
    }
    if let Shape::Snag = sp.shape {
        let h = hash3(seed ^ 0x54A6, x, ground.y, z);
        let (dx, dz) = [(1, 0), (-1, 0), (0, 1), (0, -1)][(h & 3) as usize];
        put(IVec3::new(x + dx, top - 1, z + dz), sp.log, true);
    }
    if let Some(foot) = sp.foot {
        put(ground, foot, true);
    }
}

#[cfg(test)]
mod tests;
