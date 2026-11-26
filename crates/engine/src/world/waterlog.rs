//! Waterlogged cells on `World` (core state, stored sparsely in `chunk.rs`; the flow rules are in `sim/water.rs`):
//! getters and setters that work in loaded and unloaded chunks alike, and their canonical save bytes (from `SAVE_VERSION` 91).
//!
//! Invariants: a cell's waterlog level is `0` (dry), `1..=7` (`FLOW_1..=FLOW_7`), or `WET_SOURCE` (`8`, `WATER`), and
//! lives only on `block::waterloggable` blocks; changing a cell's block clears it (`Chunk::set`). A change in a loaded
//! chunk marks the chunk and any border neighbour (`chunks_around(p, 1)`) dirty (remeshing at once or later in the
//! streaming budget) so water corners and face culling across chunk seams stay consistent.
//!
//! To allow another block to be waterlogged: add its id to `block::WATERLOGGABLE` (`block/mod.rs`), nothing more.

use crate::block::{waterloggable, LIQUID, WET_SOURCE};
use crate::bytes::{ByteReader, ByteWriter};
use crate::chunk::{Chunk, CHUNK_VOLUME};
use crate::math::{sort_small_by_key, IVec3};
use crate::worldgen::{WORLD_HEIGHT, WORLD_HEIGHT_CHUNKS};

use super::{chunk_of, local_of, streaming, World};

impl World {
    /// The waterlog level at `p` anywhere in the world (loaded or stored): `0` when dry, `1..=7` for flowing water,
    /// `WET_SOURCE` (`8`) for a water source.
    pub fn wet_anywhere(&self, p: IVec3) -> u8 {
        if !(0..WORLD_HEIGHT).contains(&p.y) {
            return 0;
        }
        let (c, (x, y, z)) = (chunk_of(p), local_of(p));
        match self.chunks.get(&c) {
            Some(e) => e.chunk.get_wet(x, y, z),
            None => self.saved.get(&c).map_or(0, |chunk| chunk.get_wet(x, y, z)),
        }
    }

    /// Sets the waterlog level at `p` anywhere in the world and immediately remeshes loaded chunks that see it.
    /// Returns `false` when out of bounds, invalid level, not waterloggable, or unchanged.
    pub fn set_wet_anywhere(&mut self, p: IVec3, level: u8) -> bool {
        self.edit_wet_anywhere(p, level, true)
    }

    /// Like [`World::set_wet_anywhere`], but loaded chunks remesh later in the streaming budget: for `sim/water.rs`.
    pub fn set_wet_anywhere_later(&mut self, p: IVec3, level: u8) -> bool {
        self.edit_wet_anywhere(p, level, false)
    }

    fn edit_wet_anywhere(&mut self, p: IVec3, level: u8, remesh_now: bool) -> bool {
        if !(0..WORLD_HEIGHT).contains(&p.y) || level > WET_SOURCE || level == self.wet_anywhere(p) {
            return false;
        }
        if level != 0 && !waterloggable(self.block_anywhere_or_generate(p)) {
            return false;
        }
        let (c, (x, y, z)) = (chunk_of(p), local_of(p));
        if let Some(entry) = self.chunks.get_mut(&c) {
            entry.chunk.set_wet(x, y, z, level);
            entry.chunk.modified = true;
            for nb in streaming::chunks_around(p, 1) {
                if self.chunks.contains_key(&nb) {
                    self.dirty.insert(nb);
                    if remesh_now {
                        self.remesh(nb);
                    }
                }
            }
            self.mesh_queue_stale = true;
            return true;
        }
        if let Some(chunk) = self.saved.get_mut(&c) {
            chunk.set_wet(x, y, z, level);
            return true;
        }
        let cached = self.generated.iter().position(|g| g.0 == c);
        let mut chunk = match cached {
            Some(i) => self.generated.remove(i).1,
            None => self.generator.generate(c),
        };
        chunk.set_wet(x, y, z, level);
        chunk.modified = true;
        self.saved.insert(c, chunk);
        true
    }

    /// Whether a loaded cell holds water (a liquid block or a waterlogged block; bodies swim and items float in it).
    #[inline]
    pub fn is_water(&self, x: i32, y: i32, z: i32) -> bool {
        let p = IVec3::new(x, y, z);
        if !(0..WORLD_HEIGHT).contains(&p.y) {
            return false;
        }
        self.chunks.get(&chunk_of(p)).is_some_and(|e| {
            let (lx, ly, lz) = local_of(p);
            LIQUID[e.chunk.get(lx, ly, lz) as usize] || e.chunk.get_wet(lx, ly, lz) != 0
        })
    }

    /// Canonical bytes of every chunk with waterlogged cells, sorted by chunk coord then cell index.
    pub fn write_wet(&self, w: &mut ByteWriter) {
        let loaded = self.chunks.iter().filter(|(_, e)| e.chunk.modified).map(|(&c, e)| (c, &e.chunk));
        let mut wet: Vec<(IVec3, &Chunk)> = loaded
            .chain(self.saved.iter().map(|(&c, chunk)| (c, chunk)))
            .filter(|(_, ch)| !ch.wets().is_empty())
            .collect();
        sort_small_by_key(&mut wet, |(c, _)| (c.x, c.y, c.z));
        w.count(wet.len());
        for (c, chunk) in wet {
            w.ivec3(c);
            w.count(chunk.wets().len());
            for &(idx, level) in chunk.wets() {
                w.u16(idx);
                w.u8(level);
            }
        }
    }

    /// Reads what [`World::write_wet`] wrote (saves from version 91).
    pub fn read_wet(&mut self, r: &mut ByteReader) -> Option<()> {
        let mut prev_chunk: Option<(i32, i32, i32)> = None;
        for _ in 0..r.count()? {
            let c = r.ivec3()?;
            let ck = (c.x, c.y, c.z);
            let n = r.count()?;
            if !(0..WORLD_HEIGHT_CHUNKS).contains(&c.y)
                || prev_chunk.is_some_and(|p| ck <= p)
                || n == 0
                || n > CHUNK_VOLUME
            {
                return None;
            }
            prev_chunk = Some(ck);
            let mut list = Vec::with_capacity(n);
            for _ in 0..n {
                let (idx, level) = (r.u16()?, r.u8()?);
                let sorted = list.last().is_none_or(|&(prev, _)| idx > prev);
                if usize::from(idx) >= CHUNK_VOLUME || level == 0 || level > WET_SOURCE || !sorted {
                    return None;
                }
                list.push((idx, level));
            }
            let chunk = self.saved.get_mut(&c)?;
            chunk.set_wet_sorted(list);
        }
        Some(())
    }
}
