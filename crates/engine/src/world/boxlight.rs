//! Light for things drawn as instanced boxes (machines, belt items, dropped items, other players):
//! `World::light_at` gives the sky and block light of a cell, so lamps and torches light them and
//! shade darkens them like the terrain beside them. The chunk mesher's light field (`light.rs`) is
//! only alive while a chunk meshes, so a cell's chunk is lit again here on demand and its interior kept in a
//! small cache (`LIGHT_CHUNKS`).
//!
//! Render cache only, derived from blocks, never core state. A chunk's entry is dropped when a block
//! changes within light's reach (`World::set_block`, the same chunks it marks dirty), and an entry for a
//! chunk that couldn't be lit yet (a neighbour isn't loaded) when more chunks load. Until it can be lit,
//! a cell reads as plain daylight ([`DAYLIGHT`]), which is how boxes looked before they had light.

use crate::chunk::{index, CHUNK_VOLUME};
use crate::math::IVec3;
use crate::mesher::PAD;
use crate::worldgen::WORLD_HEIGHT;

use super::streaming::{chunks_above, neighbourhood};
use super::{chunk_of, local_of, World};

/// Chunks kept lit: the machines within view span a handful.
const LIGHT_CHUNKS: usize = 24;
/// Full sky light and no block light, in `light.rs`'s byte (sky low nibble, block high nibble).
pub const DAYLIGHT: u8 = 15;

/// The lit cells of a chunk (`chunk::index` order), or `None` if it can't be lit yet.
pub(super) type Lit = Option<Box<[u8]>>;

impl World {
    /// The light byte (sky in the low nibble, block light in the high one, each 0..=15) of the cell at `p`.
    pub fn light_at(&mut self, p: IVec3) -> u8 {
        if p.y < 0 || p.y >= WORLD_HEIGHT {
            return DAYLIGHT;
        }
        let c = chunk_of(p);
        let at = match self.light_cache.iter().position(|(k, _)| *k == c) {
            Some(i) => i,
            None => {
                let lit = self.light_chunk(c);
                if self.light_cache.len() >= LIGHT_CHUNKS {
                    self.light_cache.remove(0);
                }
                self.light_cache.push((c, lit));
                self.light_cache.len() - 1
            }
        };
        let (x, y, z) = local_of(p);
        self.light_cache[at].1.as_ref().map_or(DAYLIGHT, |cells| cells[index(x, y, z)])
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
