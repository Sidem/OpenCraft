//! Light for things drawn as instanced boxes (machines, belt items, dropped items, other players):
//! `World::light_at` gives the sky and block light of a cell, so lamps and torches light them and
//! shade darkens them like the terrain beside them. The chunk mesher's light field (`light.rs`) is
//! only alive while a chunk meshes, so a cell's chunk is lit again here on demand and its interior kept in a
//! small cache (`LIGHT_CHUNKS`).
//!
//! Render cache only, derived from blocks, never core state. A chunk's entry goes stale when a block
//! changes within light's reach (`World::set_block`, the same chunks it marks dirty for the mesher): it is
//! lit again in the streaming budget, when nothing needs meshing (`World::work_step`), and reads as it was until
//! then. An entry for a chunk that couldn't be lit yet (a neighbour isn't loaded) is dropped when more chunks load. Until it can be lit,
//! a cell reads as plain daylight ([`DAYLIGHT`]), which is how boxes looked before they had light.

use crate::block::BlockId;
use crate::chunk::{index, CHUNK_VOLUME};
use crate::light;
use crate::math::IVec3;
use crate::mesher::PAD;
use crate::worldgen::{WORLD_HEIGHT, WORLD_HEIGHT_CHUNKS};

use super::streaming::{chunks_above, neighbourhood};
use super::{chunk_of, local_of, World};

/// Chunks kept lit (32 KB each). Every chunk holding a box in view must fit, or each frame lights them all
/// again (about 1.6 ms a chunk): a big factory in view spans dozens.
const LIGHT_CHUNKS: usize = 128;
/// Full sky light and no block light, in `light.rs`'s byte (sky low nibble, block high nibble).
pub const DAYLIGHT: u8 = 15;

/// The lit cells of a chunk (`chunk::index` order), or `None` if it can't be lit yet.
pub(super) type Lit = Option<Box<[u8]>>;

/// One cached chunk and when `light_at` last read it (`World::light_clock`). A `stale` entry is still
/// read, and lit again by `World::work_step` when it has nothing else to do.
pub(super) struct Entry {
    pub(super) chunk: IVec3,
    pub(super) lit: Lit,
    used: u64,
    stale: bool,
}

impl World {
    /// The light byte (sky in the low nibble, block light in the high one, each 0..=15) of the cell at `p`.
    pub fn light_at(&mut self, p: IVec3) -> u8 {
        if p.y < 0 || p.y >= WORLD_HEIGHT {
            return DAYLIGHT;
        }
        let c = chunk_of(p);
        self.light_clock += 1;
        let at = match self.light_cache.iter().position(|e| e.chunk == c) {
            Some(i) => i,
            None => {
                let lit = self.light_chunk(c);
                if self.light_cache.len() >= LIGHT_CHUNKS {
                    let stalest = (0..self.light_cache.len()).min_by_key(|&i| self.light_cache[i].used);
                    self.light_cache.swap_remove(stalest.unwrap_or(0));
                }
                self.light_cache.push(Entry { chunk: c, lit, used: 0, stale: false });
                self.light_cache.len() - 1
            }
        };
        let entry = &mut self.light_cache[at];
        entry.used = self.light_clock;
        let (x, y, z) = local_of(p);
        entry.lit.as_ref().map_or(DAYLIGHT, |cells| cells[index(x, y, z)])
    }

    /// Block `p` changed from `old` to `new`: the chunks its light can reach are meshed again and their
    /// cached light goes stale. The reach is `light::reach` (nowhere for most changes, 15 blocks for
    /// leaves, and then only down to the sky column's end and a reach below it). A stale entry keeps
    /// showing until `relight_one` lights it again, so a burst of changes (leaves decaying) costs the
    /// frame nothing; lighting a chunk takes about 2 ms.
    pub(super) fn light_changed(&mut self, p: IVec3, old: BlockId, new: BlockId) {
        let (m, sky_only) = light::reach(old, new);
        if m == 0 {
            return;
        }
        let bottom = if sky_only { self.sky_column_end(p) - m } else { 0 };
        let (lo, hi) = (chunk_of(p - IVec3::new(m, 0, m)), chunk_of(p + IVec3::new(m, m, m)));
        let mut reached = Vec::new();
        for cz in lo.z..=hi.z {
            for cx in lo.x..=hi.x {
                for cy in chunk_of(IVec3::new(p.x, bottom.max(0), p.z)).y..=hi.y.min(WORLD_HEIGHT_CHUNKS - 1) {
                    let c = IVec3::new(cx, cy, cz);
                    if self.chunks.contains_key(&c) {
                        self.dirty.insert(c);
                        reached.push(c);
                    }
                }
            }
        }
        for e in self.light_cache.iter_mut().filter(|e| reached.contains(&e.chunk)) {
            e.stale = true;
        }
    }

    /// The height where the sky column through `p` ends: the lowest y reached from `p` going down through
    /// blocks that let it through (loaded chunks only).
    fn sky_column_end(&self, p: IVec3) -> i32 {
        let mut y = p.y;
        while y > 0 && self.get_block(IVec3::new(p.x, y - 1, p.z)).is_some_and(light::passes_sky) {
            y -= 1;
        }
        y
    }

    /// Lights the most recently read stale chunk again (one unit of `World::work_step`). False if none is.
    pub(super) fn relight_one(&mut self) -> bool {
        let Some(at) = (0..self.light_cache.len())
            .filter(|&i| self.light_cache[i].stale)
            .max_by_key(|&i| self.light_cache[i].used)
        else {
            return false;
        };
        let lit = self.light_chunk(self.light_cache[at].chunk);
        let e = &mut self.light_cache[at];
        e.stale = false;
        if lit.is_some() {
            e.lit = lit;
        }
        true
    }

    /// Lights chunk `c` with the mesher's light code and copies out its interior.
    fn light_chunk(&mut self, c: IVec3) -> Lit {
        let refs = neighbourhood(&self.chunks, &self.air, &self.floor, c)?;
        let (above, count) = chunks_above(&self.chunks, &self.air, c)?;
        self.lighting.light(&refs, &std::array::from_fn(|i| &above[i][..count]));
        let mut cells = vec![0u8; CHUNK_VOLUME].into_boxed_slice();
        for y in 0..32 {
            for z in 0..32 {
                let from = ((y + 1) * PAD + z + 1) * PAD + 1;
                let to = index(0, y, z);
                cells[to..to + 32].copy_from_slice(&self.lighting.pad[from..from + 32]);
            }
        }
        Some(cells)
    }
}

#[cfg(test)]
mod tests;
