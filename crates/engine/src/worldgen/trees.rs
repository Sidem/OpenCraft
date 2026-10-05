//! Trees: where one stands (`Tree`, chosen per cell in `WorldGen::trees_near`) and how one is built block by block
//! (`tree_blocks`, shared with grown saplings in `sim/saplings.rs` so they look alike). Version 1 depends on both.

use crate::block::{BlockId, AIR, DIRT, LEAVES, LOG};
use crate::chunk::{index, CHUNK_SIZE};
use crate::math::{hash3, IVec3};

/// A tree to stamp: its trunk's foot column, the ground it stands on and its trunk's height.
pub(super) struct Tree {
    pub x: i32,
    pub z: i32,
    pub ground: i32,
    pub trunk: i32,
}

pub(super) fn stamp_tree(t: &Tree, base: IVec3, seed: u32, b: &mut [BlockId]) {
    tree_blocks(IVec3::new(t.x, t.ground, t.z), t.trunk, seed, |p, id, replace_solid| {
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

/// A tree standing on the block at `ground` with a trunk `trunk` blocks tall, block by block: leaves
/// (which only go into air), then the logs and the dirt under them (which replace anything). Shared
/// by generation and grown saplings (`sim/saplings.rs`), so they look alike. Version 1 depends on it.
pub fn tree_blocks(ground: IVec3, trunk: i32, seed: u32, mut put: impl FnMut(IVec3, BlockId, bool)) {
    let (x, z) = (ground.x, ground.z);
    let top = ground.y + trunk;
    // Canopy: two wide layers, then two narrow ones; corners are randomly trimmed.
    for y in (top - 2)..=(top + 1) {
        let r: i32 = if y < top { 2 } else { 1 };
        for dz in -r..=r {
            for dx in -r..=r {
                let corner = dx.abs() == r && dz.abs() == r;
                if corner && (y == top + 1 || hash3(seed ^ 0x1EAF, x + dx, y, z + dz) & 1 == 0) {
                    continue;
                }
                put(IVec3::new(x + dx, y, z + dz), LEAVES, false);
            }
        }
    }
    for y in (ground.y + 1)..=top {
        put(IVec3::new(x, y, z), LOG, true);
    }
    put(ground, DIRT, true);
}
