//! Where the light-giving blocks (lamps, torches, panel lights: `BlockDef::light` above 0) stand in each meshed chunk,
//! for the halos drawn round them (`halos.rs`). Part of the render cache: rebuilt whenever a chunk meshes (every edit
//! remeshes its chunk), dropped when the chunk unloads, never saved or hashed. Only chunks holding one are kept.

use super::World;
use crate::block::{BlockId, SHINES};
use crate::chunk::{Chunk, CHUNK_SHIFT, HOLDS_LIGHT};
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
    // A chunk of one block is never all lamps in play: a uniform chunk gives no halos. Most chunks hold no light at
    // all, which the chunk already knows (`Chunk::holds`, cached).
    let Some(blocks) = chunk.dense().filter(|_| chunk.holds(HOLDS_LIGHT)) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for (i, &b) in blocks.iter().enumerate() {
        if SHINES[b as usize] {
            // `chunk::index` is (y << 10) | (z << 5) | x.
            let local = IVec3::new((i & 31) as i32, (i >> 10) as i32, ((i >> 5) & 31) as i32);
            out.push((origin + local, b));
        }
    }
    out
}
