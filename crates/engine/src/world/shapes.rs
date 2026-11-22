//! Sparse per-cell block shapes on `World` (`shape/mod.rs`, `chunk.rs`): core getters/setters,
//! sub-voxel collision box queries (`cell_boxes`), and canonical save/hash serialisation (from `SAVE_VERSION` 84).
//!
//! Invariants: worldgen produces only full cubes (`shape == 0`), so an unedited chunk always has shape 0.
//! `set_shaped_anywhere` keeps `Chunk::modified = true` and invalidates meshes and light whenever either
//! the block id or the shape byte changes.

use crate::block::{BlockId, SOLID};
use crate::bytes::{ByteReader, ByteWriter};
use crate::chunk::CHUNK_VOLUME;
use crate::math::{sort_small_by_key, IVec3};
use crate::shape::{self, CellBoxes, Corner};
use crate::worldgen::{WORLD_HEIGHT, WORLD_HEIGHT_CHUNKS};

use super::{chunk_of, local_of, World};

/// Save format version that began saving per-chunk block shapes after cleared ground cover.
pub const SHAPES_SAVED: u32 = 84;

impl World {
    /// Shape modifier at loaded cell `p` (`shape::FULL` outside loaded chunks or for normal cubes).
    #[inline]
    pub fn get_shape(&self, p: IVec3) -> u8 {
        if !(0..WORLD_HEIGHT).contains(&p.y) {
            return shape::FULL;
        }
        self.chunks.get(&chunk_of(p)).map_or(shape::FULL, |e| {
            let (x, y, z) = local_of(p);
            e.chunk.get_shape(x, y, z)
        })
    }

    /// Shape modifier anywhere in the world (loaded or stored); deterministic core code uses this.
    pub fn shape_anywhere(&self, p: IVec3) -> u8 {
        if !(0..WORLD_HEIGHT).contains(&p.y) {
            return shape::FULL;
        }
        let c = chunk_of(p);
        let (x, y, z) = local_of(p);
        if let Some(e) = self.chunks.get(&c) {
            return e.chunk.get_shape(x, y, z);
        }
        self.saved.get(&c).map_or(shape::FULL, |chunk| chunk.get_shape(x, y, z))
    }

    /// Auto-connected corner state of a stair or slope at loaded cell `p`.
    #[inline]
    pub fn cell_corner(&self, p: IVec3, s: u8) -> Corner {
        shape::corner_of(s, p, |q| self.get_shape(q))
    }

    /// Exact collision AABBs for cell `(x, y, z)`. Unloaded cells count as a full solid cube (matching
    /// [`World::is_solid`]); unshaped solid cells fast-path to one unit cube.
    #[inline]
    pub fn cell_boxes(&self, x: i32, y: i32, z: i32) -> CellBoxes {
        let p = IVec3::new(x, y, z);
        let Some(b) = self.get_block(p) else {
            return CellBoxes::full(p);
        };
        if !SOLID[b as usize] {
            return CellBoxes::EMPTY;
        }
        let s = self.get_shape(p);
        if s == shape::FULL {
            return CellBoxes::full(p);
        }
        shape::boxes_for(p, s, self.cell_corner(p, s))
    }

    /// Sets both block id and shape modifier at `p` anywhere in the world (loaded or unloaded).
    /// Returns `false` if out of bounds or neither block nor shape changed.
    pub fn set_shaped_anywhere(&mut self, p: IVec3, b: BlockId, s: u8) -> bool {
        if !(0..WORLD_HEIGHT).contains(&p.y) || !shape::is_valid(s) {
            return false;
        }
        let s = if shape::is_shapeable(b) { s } else { shape::FULL };
        let c = chunk_of(p);
        if let Some(entry) = self.chunks.get_mut(&c) {
            let (x, y, z) = local_of(p);
            let old_b = entry.chunk.get(x, y, z);
            let old_s = entry.chunk.get_shape(x, y, z);
            if old_b == b && old_s == s {
                return false;
            }
            entry.chunk.set(x, y, z, b);
            if s != shape::FULL {
                entry.chunk.set_shape(x, y, z, s);
            }
            entry.chunk.modified = true;
            self.invalidate_shaped(p, old_b, old_s, b, s, true);
            return true;
        }
        let (x, y, z) = local_of(p);
        if let Some(chunk) = self.saved.get_mut(&c) {
            let old_b = chunk.get(x, y, z);
            let old_s = chunk.get_shape(x, y, z);
            if old_b == b && old_s == s {
                return false;
            }
            chunk.set(x, y, z, b);
            if s != shape::FULL {
                chunk.set_shape(x, y, z, s);
            }
            return true;
        }
        let cached = self.generated.iter().position(|g| g.0 == c);
        let mut chunk = match cached {
            Some(i) if self.generated[i].1.get(x, y, z) == b && s == shape::FULL => return false,
            Some(i) => self.generated.remove(i).1,
            None => self.generator.generate(c),
        };
        if chunk.get(x, y, z) == b && s == shape::FULL {
            return false;
        }
        chunk.set(x, y, z, b);
        if s != shape::FULL {
            chunk.set_shape(x, y, z, s);
        }
        chunk.modified = true;
        self.saved.insert(c, chunk);
        true
    }

    /// Changes only the shape modifier of the block already at `p` (must be shapeable).
    pub fn set_shape_anywhere(&mut self, p: IVec3, s: u8) -> bool {
        let b = self.block_anywhere_or_generate(p);
        if !shape::is_shapeable(b) {
            return false;
        }
        self.set_shaped_anywhere(p, b, s)
    }

    /// Canonical bytes of every chunk with non-full shapes, sorted by chunk coord then cell index.
    pub fn write_shapes(&self, w: &mut ByteWriter) {
        let loaded =
            self.chunks.iter().filter(|(_, e)| e.chunk.modified && e.chunk.has_shapes()).map(|(&c, e)| (c, &e.chunk));
        let stored = self.saved.iter().filter(|(_, chunk)| chunk.has_shapes()).map(|(&c, chunk)| (c, chunk));
        let mut shaped: Vec<(IVec3, &crate::chunk::Chunk)> = loaded.chain(stored).collect();
        sort_small_by_key(&mut shaped, |(c, _)| (c.x, c.y, c.z));
        w.count(shaped.len());
        for (c, chunk) in shaped {
            w.ivec3(c);
            let list = chunk.shapes();
            w.count(list.len());
            for &(idx, s) in list {
                w.u16(idx);
                w.u8(s);
            }
        }
    }

    /// Restores chunk shapes written by [`Self::write_shapes`] into `self.saved`.
    pub fn read_shapes(&mut self, r: &mut ByteReader) -> Option<()> {
        if r.version < SHAPES_SAVED {
            return Some(());
        }
        let mut prev_chunk: Option<(i32, i32, i32)> = None;
        for _ in 0..r.count()? {
            let c = r.ivec3()?;
            let ck = (c.x, c.y, c.z);
            if !(0..WORLD_HEIGHT_CHUNKS).contains(&c.y) || prev_chunk.is_some_and(|p| ck <= p) {
                return None;
            }
            prev_chunk = Some(ck);
            let n = r.count()?;
            if n == 0 || n > CHUNK_VOLUME {
                return None;
            }
            let mut list = Vec::with_capacity(n);
            let mut prev_idx: Option<u16> = None;
            for _ in 0..n {
                let idx = r.u16()?;
                let s = r.u8()?;
                if usize::from(idx) >= CHUNK_VOLUME
                    || s == shape::FULL
                    || !shape::is_valid(s)
                    || prev_idx.is_some_and(|p| idx <= p)
                {
                    return None;
                }
                prev_idx = Some(idx);
                list.push((idx, s));
            }
            let chunk = self.saved.get_mut(&c)?;
            chunk.set_shapes_sorted(list);
        }
        Some(())
    }
}
