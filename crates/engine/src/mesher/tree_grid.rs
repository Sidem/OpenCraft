//! The blocks around a chunk that its trees are drawn from: the chunk and `REACH` blocks of border on every side,
//! copied from the 27 chunks the mesher gets. Branches (`logs.rs`) read their neighbours' thickness and links this far
//! out; leaf cards (`foliage.rs`) read how open their surroundings are.
//! Invariant: whatever reads the grid reads it relative to a cell and in the same order in any chunk, so two chunks
//! drawing the two halves of one branch agree on its every number. An edit can change a tree's drawing up to
//! `REACH` blocks away (the world remeshes that far: `world/streaming.rs` `mesh_reach`).

use super::N;
use crate::block::{is_leaves, is_log, BlockId, AIR};
use crate::chunk::{index, Chunk};

/// How far, in blocks, the blocks around a chunk shape its trees: a log's thickness looks up to 4 blocks above it and
/// 2 aside, and the far half of a branch is its neighbour's.
pub const REACH: usize = 5;

const G: usize = N + 2 * REACH;
const R: i32 = REACH as i32;

pub struct TreeGrid {
    blocks: Vec<BlockId>,
    /// Whether each cell is part of a tree (a log or leaves), for the reads that ask only that.
    tree: Vec<bool>,
}

impl Default for TreeGrid {
    fn default() -> Self {
        Self { blocks: vec![AIR; G * G * G], tree: vec![false; G * G * G] }
    }
}

/// The step in the grid's flat index for one block along y and z (x is 1).
pub const STEP_Y: usize = G * G;
pub const STEP_Z: usize = G;

impl TreeGrid {
    /// The grid's blocks from the 27 chunks (`super::neighbor_index` order).
    #[inline(never)]
    pub fn fill(&mut self, n: &[&Chunk; 27]) {
        let split = |g: usize| {
            let v = g as i32 - R;
            ((v.div_euclid(N as i32) + 1) as usize, v.rem_euclid(N as i32) as usize)
        };
        for gy in 0..G {
            let (cy, ly) = split(gy);
            for gz in 0..G {
                let (cz, lz) = split(gz);
                let row = (gy * G + gz) * G;
                for (cx, from, to, lx) in [(0, 0, REACH, N - REACH), (1, REACH, REACH + N, 0), (2, REACH + N, G, 0)] {
                    let c = n[cx + cy * 3 + cz * 9];
                    let dst = &mut self.blocks[row + from..row + to];
                    match c.dense() {
                        Some(d) => {
                            let s = index(lx, ly, lz);
                            dst.copy_from_slice(&d[s..s + (to - from)]);
                        }
                        None => dst.fill(c.get(0, 0, 0)),
                    }
                }
            }
        }
        for (t, &b) in self.tree.iter_mut().zip(&self.blocks) {
            *t = is_log(b) || is_leaves(b);
        }
    }

    /// The flat index of chunk-local `(x, y, z)`, which must lie in the grid.
    #[inline]
    pub fn index(x: i32, y: i32, z: i32) -> usize {
        (((y + R) as usize * G) + (z + R) as usize) * G + (x + R) as usize
    }

    /// Whether the cell at a flat index is part of a tree.
    #[inline]
    pub fn tree_at(&self, i: usize) -> bool {
        self.tree[i]
    }

    /// The block at chunk-local `(x, y, z)` (from `-REACH` to `N + REACH - 1`); air beyond the grid.
    #[inline]
    pub fn at(&self, x: i32, y: i32, z: i32) -> BlockId {
        let (gx, gy, gz) = (x + R, y + R, z + R);
        if gx < 0 || gy < 0 || gz < 0 || gx >= G as i32 || gy >= G as i32 || gz >= G as i32 {
            return AIR;
        }
        self.blocks[((gy as usize * G) + gz as usize) * G + gx as usize]
    }
}
