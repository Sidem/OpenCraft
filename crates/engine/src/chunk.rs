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
    /// Sparse per-cell bare turf faces (`sim/sod.rs`: a bit per face grass hasn't grown over since an edit uncovered
    /// it), sorted by `index(x, y, z) as u16`; `None` when there are none (0 heap bytes).
    bare: Option<Vec<(u16, u8)>>,
    /// Sparse per-cell waterlog level (`world/waterlog.rs`: `1..=7` for `FLOW_1..=FLOW_7`, `8` for `WATER`),
    /// sorted by `index(x, y, z) as u16`; `None` when no cell in the chunk is waterlogged (0 heap bytes).
    wet: Option<Vec<(u16, u8)>>,
    /// Edited by the player since generation; kept when the chunk is streamed out.
    pub modified: bool,
}

impl Chunk {
    pub fn uniform(b: BlockId) -> Self {
        Self { data: Data::Uniform(b), shapes: None, paint: None, bare: None, wet: None, modified: false }
    }

    /// Takes ownership of a dense buffer, collapsing it to a uniform chunk when possible.
    pub fn from_blocks(blocks: Vec<BlockId>) -> Self {
        debug_assert_eq!(blocks.len(), CHUNK_VOLUME);
        let first = blocks[0];
        if blocks.iter().all(|&b| b == first) {
            return Self::uniform(first);
        }
        Self {
            data: Data::Dense(blocks.into_boxed_slice()),
            shapes: None,
            paint: None,
            bare: None,
            wet: None,
            modified: false,
        }
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
        if self.bare.is_some() {
            self.set_bare(x, y, z, 0);
        }
        if self.wet.is_some() {
            self.set_wet(x, y, z, 0);
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
        get_sparse(self.shapes.as_deref(), index(x, y, z))
    }

    /// Sets the shape modifier at `(x, y, z)` (`0` clears it), returning the previous shape.
    /// Promotes a uniform chunk to dense when adding a non-full shape so `as_uniform` stays exact.
    pub fn set_shape(&mut self, x: usize, y: usize, z: usize, shape: u8) -> u8 {
        if shape != 0 {
            self.make_dense();
        }
        set_sparse(&mut self.shapes, index(x, y, z), shape)
    }

    /// Bare turf faces at `(x, y, z)` (`0` = every face grown). O(1) fast path when the chunk has none.
    #[inline]
    pub fn get_bare(&self, x: usize, y: usize, z: usize) -> u8 {
        get_sparse(self.bare.as_deref(), index(x, y, z))
    }

    /// Sets the bare turf faces at `(x, y, z)` (`0` clears them), returning the previous mask.
    pub fn set_bare(&mut self, x: usize, y: usize, z: usize, mask: u8) -> u8 {
        set_sparse(&mut self.bare, index(x, y, z), mask)
    }

    /// Every cell with bare turf faces, sorted by cell index.
    #[inline]
    pub fn bares(&self) -> &[(u16, u8)] {
        self.bare.as_deref().unwrap_or(&[])
    }

    /// Replaces the chunk's bare-face table from a validated, sorted save list.
    pub fn set_bare_sorted(&mut self, list: Vec<(u16, u8)>) {
        self.bare = (!list.is_empty()).then_some(list);
    }

    /// Waterlog level at `(x, y, z)` (`0` = dry, `1..=7` = `FLOW_1..=FLOW_7`, `8` = `WATER`). O(1) fast path when dry.
    #[inline]
    pub fn get_wet(&self, x: usize, y: usize, z: usize) -> u8 {
        get_sparse(self.wet.as_deref(), index(x, y, z))
    }

    /// Sets the waterlog level at `(x, y, z)` (`0` clears it), returning the previous level.
    /// Promotes a uniform chunk to dense when adding a wet cell so `as_uniform` stays exact.
    pub fn set_wet(&mut self, x: usize, y: usize, z: usize, level: u8) -> u8 {
        if level != 0 {
            self.make_dense();
        }
        set_sparse(&mut self.wet, index(x, y, z), level)
    }

    /// Every waterlogged cell in the chunk, sorted by cell index.
    #[inline]
    pub fn wets(&self) -> &[(u16, u8)] {
        self.wet.as_deref().unwrap_or(&[])
    }

    /// Replaces the chunk's waterlog table from a validated, sorted save list.
    pub fn set_wet_sorted(&mut self, list: Vec<(u16, u8)>) {
        if !list.is_empty() {
            self.make_dense();
            self.wet = Some(list);
        } else {
            self.wet = None;
        }
    }

    fn make_dense(&mut self) {
        if let Data::Uniform(u) = self.data {
            self.data = Data::Dense(vec![u; CHUNK_VOLUME].into_boxed_slice());
        }
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
        get_sparse(self.paint.as_deref(), index(x, y, z))
    }

    /// Sets the packed per-face paint at `(x, y, z)` (`0` clears it), returning the previous value.
    pub fn set_paint(&mut self, x: usize, y: usize, z: usize, packed: u32) -> u32 {
        if packed != 0 {
            self.make_dense();
        }
        set_sparse(&mut self.paint, index(x, y, z), packed)
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

    /// Whether both hold the same blocks, shapes, paint, bare turf faces and waterlog levels, however each is stored.
    pub fn same_blocks(&self, other: &Chunk) -> bool {
        if self.shapes() != other.shapes()
            || self.paints() != other.paints()
            || self.bares() != other.bares()
            || self.wets() != other.wets()
        {
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

/// The value at cell `i` of a sparse table sorted by cell (`T::default()` when absent).
#[inline]
fn get_sparse<T: Copy + Default>(list: Option<&[(u16, T)]>, i: usize) -> T {
    let Some(list) = list else { return T::default() };
    list.binary_search_by_key(&(i as u16), |&(k, _)| k).map_or_else(|_| T::default(), |at| list[at].1)
}

/// Sets cell `i` of a sparse table (the default value removes it; an empty table becomes `None`); the old value.
fn set_sparse<T: Copy + Default + PartialEq>(table: &mut Option<Vec<(u16, T)>>, i: usize, value: T) -> T {
    let (key, none) = (i as u16, T::default());
    let list = table.get_or_insert_with(Vec::new);
    let old = match list.binary_search_by_key(&key, |&(k, _)| k) {
        Ok(at) if value == none => list.remove(at).1,
        Ok(at) => std::mem::replace(&mut list[at].1, value),
        Err(at) => {
            if value != none {
                list.insert(at, (key, value));
            }
            none
        }
    };
    if list.is_empty() {
        *table = None;
    }
    old
}

#[cfg(test)]
mod tests;
