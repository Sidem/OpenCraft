//! Where the light-giving blocks (lamps, torches, panel lights: `BlockDef::light` above 0) stand in each meshed chunk,
//! for the halos drawn round them (`halos.rs`). Part of the render cache: rebuilt whenever a chunk meshes (every edit
//! remeshes its chunk), dropped when the chunk unloads, never saved or hashed. Only chunks holding one are kept.

use super::World;
use crate::block::{BlockId, DEFS};
use crate::chunk::{Chunk, CHUNK_SHIFT};
use crate::math::IVec3;

impl World {
    /// The light-giving blocks of every meshed chunk that holds one, by chunk (presentation: `halos.rs`).
    pub fn glows(&self) -> impl Iterator<Item = (&IVec3, &Vec<(IVec3, BlockId)>)> {
        self.glows.iter()
    }
}

/// The light-giving blocks of `chunk` (at chunk position `at`) as their world cell and block.
pub fn scan(chunk: &Chunk, at: IVec3) -> Vec<(IVec3, BlockId)> {
    let origin = IVec3::new(at.x << CHUNK_SHIFT, at.y << CHUNK_SHIFT, at.z << CHUNK_SHIFT);
    let Some(blocks) = chunk.dense() else {
        // A chunk of one block is never all lamps in play; a uniform chunk gives no halos.
        return Vec::new();
    };
    let mut out = Vec::new();
    for (i, &b) in blocks.iter().enumerate() {
        if GIVES_LIGHT[b as usize] {
            // `chunk::index` is (y << 10) | (z << 5) | x.
            let local = IVec3::new((i & 31) as i32, (i >> 10) as i32, ((i >> 5) & 31) as i32);
            out.push((origin + local, b));
        }
    }
    out
}

/// Per block id: whether it gives light (a table, so the scan of a chunk's 32,768 cells stays a lookup each).
const GIVES_LIGHT: [bool; 256] = {
    let mut t = [false; 256];
    let mut i = 0;
    while i < DEFS.len() {
        t[i] = DEFS[i].light != 0;
        i += 1;
    }
    t
};
