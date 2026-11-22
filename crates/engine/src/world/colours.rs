//! Sparse per-cell per-face paint on `World` (`colour/mod.rs`, `chunk.rs`): core getters/setters,
//! legacy dyed-concrete migration (for saves `< 85`), and canonical save/hash serialisation (from `SAVE_VERSION` 85).

use crate::bytes::{ByteReader, ByteWriter};
use crate::chunk::CHUNK_VOLUME;
use crate::colour::{self, FacePaint};
use crate::math::{sort_small_by_key, IVec3};
use crate::worldgen::{WORLD_HEIGHT, WORLD_HEIGHT_CHUNKS};

use super::{chunk_of, local_of, World};

/// Save format version that began saving per-chunk per-face surface paint and player paint charges.
pub const PAINT_SAVED: u32 = 85;

impl World {
    /// Packed per-face paint at loaded cell `p` (`0` = unpainted).
    #[inline]
    pub fn get_paint(&self, p: IVec3) -> FacePaint {
        if !(0..WORLD_HEIGHT).contains(&p.y) {
            return 0;
        }
        self.chunks.get(&chunk_of(p)).map_or(0, |e| {
            let (x, y, z) = local_of(p);
            e.chunk.get_paint(x, y, z)
        })
    }

    /// Paint colour (`0..=16`) on `face` (`0..6`) of loaded cell `p`.
    #[inline]
    pub fn get_face_paint(&self, p: IVec3, face: usize) -> u8 {
        colour::get_face(self.get_paint(p), face)
    }

    /// Packed per-face paint anywhere in the world (loaded or stored); deterministic core code uses this.
    pub fn paint_anywhere(&self, p: IVec3) -> FacePaint {
        if !(0..WORLD_HEIGHT).contains(&p.y) {
            return 0;
        }
        let c = chunk_of(p);
        let (x, y, z) = local_of(p);
        if let Some(e) = self.chunks.get(&c) {
            return e.chunk.get_paint(x, y, z);
        }
        self.saved.get(&c).map_or(0, |chunk| chunk.get_paint(x, y, z))
    }

    /// Sets the packed per-face paint at `p` anywhere in the world (if `p` holds a paintable block).
    /// Returns `false` if out of bounds, not paintable, or unchanged.
    pub fn set_paint_anywhere(&mut self, p: IVec3, packed: FacePaint) -> bool {
        if !(0..WORLD_HEIGHT).contains(&p.y) {
            return false;
        }
        let b = self.block_anywhere_or_generate(p);
        if packed != 0 && !colour::is_paintable(b) {
            return false;
        }
        let c = chunk_of(p);
        let (x, y, z) = local_of(p);
        if let Some(entry) = self.chunks.get_mut(&c) {
            if entry.chunk.get_paint(x, y, z) == packed {
                return false;
            }
            entry.chunk.set_paint(x, y, z, packed);
            entry.chunk.modified = true;
            self.dirty.insert(c);
            self.mesh_queue_stale = true;
            return true;
        }
        if let Some(chunk) = self.saved.get_mut(&c) {
            if chunk.get_paint(x, y, z) == packed {
                return false;
            }
            chunk.set_paint(x, y, z, packed);
            return true;
        }
        let cached = self.generated.iter().position(|g| g.0 == c);
        let mut chunk = match cached {
            Some(i) => self.generated.remove(i).1,
            None => self.generator.generate(c),
        };
        if packed == 0 {
            return false;
        }
        chunk.set_paint(x, y, z, packed);
        chunk.modified = true;
        self.saved.insert(c, chunk);
        true
    }

    /// Paints a single surface `face` (`0..6`) of cell `p` with `colour` (`0..=16`).
    pub fn set_face_paint_anywhere(&mut self, p: IVec3, face: usize, col: u8) -> bool {
        if face >= 6 || col as usize > colour::COUNT {
            return false;
        }
        let cur = self.paint_anywhere(p);
        let next = colour::set_face(cur, face, col);
        if next == cur {
            return false;
        }
        self.set_paint_anywhere(p, next)
    }

    /// Migrates placed legacy dyed concrete blocks in stored/loaded chunks into plain `CONCRETE` with all six faces
    /// painted (used when loading saves from before `SAVE_VERSION` 85).
    pub fn migrate_legacy_concretes(&mut self) {
        for chunk in self.saved.values_mut() {
            chunk.migrate_legacy_concretes();
        }
        let mut dirty = Vec::new();
        for (&c, entry) in self.chunks.iter_mut() {
            entry.chunk.migrate_legacy_concretes();
            dirty.push(c);
        }
        self.dirty.extend(dirty);
        self.mesh_queue_stale = true;
    }

    /// Canonical bytes of every chunk with painted surfaces, sorted by chunk coord then cell index.
    pub fn write_paint(&self, w: &mut ByteWriter) {
        let loaded =
            self.chunks.iter().filter(|(_, e)| e.chunk.modified && e.chunk.has_paint()).map(|(&c, e)| (c, &e.chunk));
        let stored = self.saved.iter().filter(|(_, chunk)| chunk.has_paint()).map(|(&c, chunk)| (c, chunk));
        let mut painted: Vec<(IVec3, &crate::chunk::Chunk)> = loaded.chain(stored).collect();
        sort_small_by_key(&mut painted, |(c, _)| (c.x, c.y, c.z));
        w.count(painted.len());
        for (c, chunk) in painted {
            w.ivec3(c);
            let list = chunk.paints();
            w.count(list.len());
            for &(idx, packed) in list {
                w.u16(idx);
                w.u32(packed);
            }
        }
    }

    /// Reads per-chunk surface paint written by [`World::write_paint`].
    pub fn read_paint(&mut self, r: &mut ByteReader) -> Option<()> {
        if r.version < PAINT_SAVED {
            self.migrate_legacy_concretes();
            return Some(());
        }
        let chunks = r.count()?;
        for _ in 0..chunks {
            let c = r.ivec3()?;
            if !(0..WORLD_HEIGHT_CHUNKS).contains(&c.y) {
                return None;
            }
            let n = r.count()?;
            if n > CHUNK_VOLUME {
                return None;
            }
            let mut list = Vec::with_capacity(n);
            let mut prev: Option<u16> = None;
            for _ in 0..n {
                let idx = r.u16()?;
                let packed = r.u32()?;
                if usize::from(idx) >= CHUNK_VOLUME || packed == 0 || prev.is_some_and(|p| idx <= p) {
                    return None;
                }
                prev = Some(idx);
                list.push((idx, packed));
            }
            if let Some(chunk) = self.saved.get_mut(&c) {
                chunk.set_paint_sorted(list);
            }
        }
        Some(())
    }
}
