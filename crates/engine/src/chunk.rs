//! 32³ block storage for one chunk. Homogeneous chunks (all air, all stone) cost no heap memory;
//! writing a different block makes them dense. `modified` marks player edits, which `World` keeps
//! when the chunk streams out. Memory layout: see [`index`].

use crate::block::BlockId;
use crate::bytes::{ByteReader, ByteWriter};

pub const CHUNK_SIZE: i32 = 32;
pub const CHUNK_SHIFT: i32 = 5;
pub const CHUNK_MASK: i32 = CHUNK_SIZE - 1;
pub const CHUNK_VOLUME: usize = (CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE) as usize;

/// Memory layout is x-fastest, then z, then y, so an x-row is a contiguous 32-byte run.
#[inline]
pub const fn index(x: usize, y: usize, z: usize) -> usize {
    (y << 10) | (z << 5) | x
}

#[derive(Clone)]
enum Data {
    Uniform(BlockId),
    Dense(Box<[BlockId]>),
}

#[derive(Clone)]
pub struct Chunk {
    data: Data,
    /// Sparse per-cell shape modifiers (`shape/mod.rs`), sorted by `index(x, y, z) as u16`; `None` when
    /// every cell in the chunk is a full cube (0 heap bytes for natural chunks).
    shapes: Option<Vec<(u16, u8)>>,
    /// Sparse per-cell per-face paint (`colour/mod.rs`, 6 faces × 5 bits packed in `u32`), sorted by
    /// `index(x, y, z) as u16`; `None` when no surface in the chunk is painted (0 heap bytes).
    paint: Option<Vec<(u16, u32)>>,
    /// Edited by the player since generation; kept when the chunk is streamed out.
    pub modified: bool,
}

impl Chunk {
    pub fn uniform(b: BlockId) -> Self {
        Self { data: Data::Uniform(b), shapes: None, paint: None, modified: false }
    }

    /// Takes ownership of a dense buffer, collapsing it to a uniform chunk when possible.
    pub fn from_blocks(blocks: Vec<BlockId>) -> Self {
        debug_assert_eq!(blocks.len(), CHUNK_VOLUME);
        let first = blocks[0];
        if blocks.iter().all(|&b| b == first) {
            return Self::uniform(first);
        }
        Self { data: Data::Dense(blocks.into_boxed_slice()), shapes: None, paint: None, modified: false }
    }

    #[inline]
    pub fn get(&self, x: usize, y: usize, z: usize) -> BlockId {
        match &self.data {
            Data::Uniform(b) => *b,
            Data::Dense(d) => d[index(x, y, z)],
        }
    }

    pub fn set(&mut self, x: usize, y: usize, z: usize, b: BlockId) {
        if self.shapes.is_some() {
            self.set_shape(x, y, z, 0);
        }
        if self.paint.is_some() {
            self.set_paint(x, y, z, 0);
        }
        if let Data::Uniform(u) = self.data {
            if u == b {
                return;
            }
            self.data = Data::Dense(vec![u; CHUNK_VOLUME].into_boxed_slice());
        }
        if let Data::Dense(d) = &mut self.data {
            d[index(x, y, z)] = b;
        }
    }

    /// Shape modifier at `(x, y, z)` (`0` = full cube). O(1) fast path when the chunk has no shapes.
    #[inline]
    pub fn get_shape(&self, x: usize, y: usize, z: usize) -> u8 {
        let Some(list) = self.shapes.as_deref() else { return 0 };
        let key = index(x, y, z) as u16;
        list.binary_search_by_key(&key, |&(i, _)| i).map_or(0, |at| list[at].1)
    }

    /// Sets the shape modifier at `(x, y, z)` (`0` clears it), returning the previous shape.
    /// Promotes a uniform chunk to dense when adding a non-full shape so `as_uniform` stays exact.
    pub fn set_shape(&mut self, x: usize, y: usize, z: usize, shape: u8) -> u8 {
        let key = index(x, y, z) as u16;
        if shape != 0 {
            if let Data::Uniform(u) = self.data {
                self.data = Data::Dense(vec![u; CHUNK_VOLUME].into_boxed_slice());
            }
        }
        let Some(list) = self.shapes.as_mut() else {
            if shape == 0 {
                return 0;
            }
            self.shapes = Some(vec![(key, shape)]);
            return 0;
        };
        let old = match list.binary_search_by_key(&key, |&(i, _)| i) {
            Ok(at) => {
                let prev = list[at].1;
                if shape == 0 {
                    list.remove(at);
                } else {
                    list[at].1 = shape;
                }
                prev
            }
            Err(at) => {
                if shape != 0 {
                    list.insert(at, (key, shape));
                }
                0
            }
        };
        if list.is_empty() {
            self.shapes = None;
        }
        old
    }

    #[inline]
    pub fn has_shapes(&self) -> bool {
        self.shapes.is_some()
    }

    #[inline]
    pub fn shapes(&self) -> &[(u16, u8)] {
        self.shapes.as_deref().unwrap_or(&[])
    }

    /// Replaces the chunk's shape table from a validated, sorted save list.
    pub fn set_shapes_sorted(&mut self, list: Vec<(u16, u8)>) {
        if list.is_empty() {
            self.shapes = None;
            return;
        }
        if let Data::Uniform(u) = self.data {
            self.data = Data::Dense(vec![u; CHUNK_VOLUME].into_boxed_slice());
        }
        self.shapes = Some(list);
    }

    /// Packed per-face paint at `(x, y, z)` (`0` = unpainted). O(1) fast path when the chunk has no paint.
    #[inline]
    pub fn get_paint(&self, x: usize, y: usize, z: usize) -> u32 {
        let Some(list) = self.paint.as_deref() else { return 0 };
        let key = index(x, y, z) as u16;
        list.binary_search_by_key(&key, |&(i, _)| i).map_or(0, |at| list[at].1)
    }

    /// Sets the packed per-face paint at `(x, y, z)` (`0` clears it), returning the previous value.
    pub fn set_paint(&mut self, x: usize, y: usize, z: usize, packed: u32) -> u32 {
        let key = index(x, y, z) as u16;
        if packed != 0 {
            if let Data::Uniform(u) = self.data {
                self.data = Data::Dense(vec![u; CHUNK_VOLUME].into_boxed_slice());
            }
        }
        let Some(list) = self.paint.as_mut() else {
            if packed == 0 {
                return 0;
            }
            self.paint = Some(vec![(key, packed)]);
            return 0;
        };
        let old = match list.binary_search_by_key(&key, |&(i, _)| i) {
            Ok(at) => {
                let prev = list[at].1;
                if packed == 0 {
                    list.remove(at);
                } else {
                    list[at].1 = packed;
                }
                prev
            }
            Err(at) => {
                if packed != 0 {
                    list.insert(at, (key, packed));
                }
                0
            }
        };
        if list.is_empty() {
            self.paint = None;
        }
        old
    }

    #[inline]
    pub fn has_paint(&self) -> bool {
        self.paint.is_some()
    }

    #[inline]
    pub fn paints(&self) -> &[(u16, u32)] {
        self.paint.as_deref().unwrap_or(&[])
    }

    /// Replaces the chunk's paint table from a validated, sorted save list.
    pub fn set_paint_sorted(&mut self, list: Vec<(u16, u32)>) {
        if list.is_empty() {
            self.paint = None;
            return;
        }
        if let Data::Uniform(u) = self.data {
            self.data = Data::Dense(vec![u; CHUNK_VOLUME].into_boxed_slice());
        }
        self.paint = Some(list);
    }

    /// Rewrites every legacy dyed concrete block in this chunk to plain `CONCRETE` painted on all six faces
    /// (called once when loading saves from before `SAVE_VERSION` 85).
    pub fn migrate_legacy_concretes(&mut self) {
        let legacy = match self.data {
            Data::Uniform(b) => crate::colour::legacy_concrete_colour(b),
            Data::Dense(_) => None,
        };
        if let Some(col) = legacy {
            let packed = crate::colour::all_faces(col);
            self.data = Data::Dense(vec![crate::block::CONCRETE; CHUNK_VOLUME].into_boxed_slice());
            self.paint = Some((0..CHUNK_VOLUME as u16).map(|i| (i, packed)).collect());
            return;
        }
        let mut updates = Vec::new();
        if let Data::Dense(d) = &mut self.data {
            for (i, b) in d.iter_mut().enumerate() {
                if let Some(col) = crate::colour::legacy_concrete_colour(*b) {
                    *b = crate::block::CONCRETE;
                    updates.push((i as u16, crate::colour::all_faces(col)));
                }
            }
        }
        for (idx, packed) in updates {
            let (x, z, y) = ((idx & 31) as usize, ((idx >> 5) & 31) as usize, (idx >> 10) as usize);
            self.set_paint(x, y, z, packed);
        }
    }

    /// Whether both hold the same blocks, shapes and paint, however each is stored.
    pub fn same_blocks(&self, other: &Chunk) -> bool {
        if self.shapes() != other.shapes() || self.paints() != other.paints() {
            return false;
        }
        match (&self.data, &other.data) {
            (Data::Uniform(a), Data::Uniform(b)) => a == b,
            (Data::Dense(a), Data::Dense(b)) => a == b,
            (Data::Uniform(u), Data::Dense(d)) | (Data::Dense(d), Data::Uniform(u)) => d.iter().all(|b| b == u),
        }
    }

    #[inline]
    pub fn as_uniform(&self) -> Option<BlockId> {
        match self.data {
            Data::Uniform(b) => Some(b),
            Data::Dense(_) => None,
        }
    }

    #[inline]
    pub fn dense(&self) -> Option<&[BlockId]> {
        match &self.data {
            Data::Uniform(_) => None,
            Data::Dense(d) => Some(d),
        }
    }

    /// The blocks in [`index`] order as runs of (length `u16`, block), so the same contents give the
    /// same bytes whether stored uniform or dense.
    pub fn write_state(&self, w: &mut ByteWriter) {
        let blocks = match &self.data {
            Data::Uniform(b) => {
                w.u16(CHUNK_VOLUME as u16);
                w.block(*b);
                return;
            }
            Data::Dense(d) => d,
        };
        let mut start = 0;
        for i in 1..=CHUNK_VOLUME {
            if i == CHUNK_VOLUME || blocks[i] != blocks[start] {
                w.u16((i - start) as u16);
                w.block(blocks[start]);
                start = i;
            }
        }
    }

    /// Reads what `write_state` wrote. The chunk comes back unmodified; the caller marks edits.
    pub fn read_state(r: &mut ByteReader) -> Option<Chunk> {
        let mut blocks: Vec<BlockId> = Vec::with_capacity(CHUNK_VOLUME);
        while blocks.len() < CHUNK_VOLUME {
            let (n, b) = (r.u16()? as usize, r.block()?);
            if n == 0 || blocks.len() + n > CHUNK_VOLUME {
                return None;
            }
            blocks.resize(blocks.len() + n, b);
        }
        Some(Chunk::from_blocks(blocks))
    }
}

#[cfg(test)]
mod tests;
