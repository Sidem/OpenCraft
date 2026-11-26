//! Bare turf faces on `World` (core state, stored sparsely in `chunk.rs`; the rules are `sim/sod.rs`): getters and
//! setters that work in loaded and unloaded chunks alike, and their save bytes (from `SAVE_VERSION` 90).
//!
//! Invariants: a cell's mask is a bit per face (`mesher::FACES` order) and lives only on turf (`block::is_turf`);
//! changing a cell's block clears it (`Chunk::set`). A change in a loaded chunk remeshes it.

use crate::block::is_turf;
use crate::bytes::{ByteReader, ByteWriter};
use crate::chunk::{Chunk, CHUNK_VOLUME};
use crate::math::{sort_small_by_key, IVec3};
use crate::worldgen::{WORLD_HEIGHT, WORLD_HEIGHT_CHUNKS};

use super::{chunk_of, local_of, World};

/// Every face bit in use.
const ALL_FACES: u8 = 0b11_1111;

impl World {
    /// The bare turf faces at `p` anywhere in the world (loaded or stored), `0` when every face is grown.
    pub fn bare_anywhere(&self, p: IVec3) -> u8 {
        if !(0..WORLD_HEIGHT).contains(&p.y) {
            return 0;
        }
        let (c, (x, y, z)) = (chunk_of(p), local_of(p));
        match self.chunks.get(&c) {
            Some(e) => e.chunk.get_bare(x, y, z),
            None => self.saved.get(&c).map_or(0, |chunk| chunk.get_bare(x, y, z)),
        }
    }

    /// Sets the bare turf faces at `p` anywhere in the world; `false` when out of bounds, not turf, or unchanged.
    pub fn set_bare_anywhere(&mut self, p: IVec3, mask: u8) -> bool {
        if !(0..WORLD_HEIGHT).contains(&p.y) || mask & !ALL_FACES != 0 || mask == self.bare_anywhere(p) {
            return false;
        }
        if mask != 0 && !is_turf(self.block_anywhere_or_generate(p)) {
            return false;
        }
        let (c, (x, y, z)) = (chunk_of(p), local_of(p));
        if let Some(entry) = self.chunks.get_mut(&c) {
            entry.chunk.set_bare(x, y, z, mask);
            entry.chunk.modified = true;
            self.dirty.insert(c);
            self.mesh_queue_stale = true;
            return true;
        }
        if let Some(chunk) = self.saved.get_mut(&c) {
            chunk.set_bare(x, y, z, mask);
            return true;
        }
        // Turf in a chunk never edited (`block_anywhere_or_generate` above cached its generation).
        let cached = self.generated.iter().position(|g| g.0 == c);
        let mut chunk = match cached {
            Some(i) => self.generated.remove(i).1,
            None => self.generator.generate(c),
        };
        chunk.set_bare(x, y, z, mask);
        chunk.modified = true;
        self.saved.insert(c, chunk);
        true
    }

    /// Canonical bytes of every chunk with bare turf faces, sorted by chunk coord then cell index.
    pub fn write_bare(&self, w: &mut ByteWriter) {
        let loaded = self.chunks.iter().filter(|(_, e)| e.chunk.modified).map(|(&c, e)| (c, &e.chunk));
        let mut bare: Vec<(IVec3, &Chunk)> = loaded
            .chain(self.saved.iter().map(|(&c, chunk)| (c, chunk)))
            .filter(|(_, ch)| !ch.bares().is_empty())
            .collect();
        sort_small_by_key(&mut bare, |(c, _)| (c.x, c.y, c.z));
        w.count(bare.len());
        for (c, chunk) in bare {
            w.ivec3(c);
            w.count(chunk.bares().len());
            for &(idx, mask) in chunk.bares() {
                w.u16(idx);
                w.u8(mask);
            }
        }
    }

    /// Reads what [`World::write_bare`] wrote (saves from version 90).
    pub fn read_bare(&mut self, r: &mut ByteReader) -> Option<()> {
        for _ in 0..r.count()? {
            let c = r.ivec3()?;
            let n = r.count()?;
            if !(0..WORLD_HEIGHT_CHUNKS).contains(&c.y) || n > CHUNK_VOLUME {
                return None;
            }
            let mut list = Vec::with_capacity(n);
            for _ in 0..n {
                let (idx, mask) = (r.u16()?, r.u8()?);
                let sorted = list.last().is_none_or(|&(prev, _)| idx > prev);
                if usize::from(idx) >= CHUNK_VOLUME || mask == 0 || mask & !ALL_FACES != 0 || !sorted {
                    return None;
                }
                list.push((idx, mask));
            }
            if let Some(chunk) = self.saved.get_mut(&c) {
                chunk.set_bare_sorted(list);
            }
        }
        Some(())
    }
}
