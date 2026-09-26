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
    /// Edited by the player since generation; kept when the chunk is streamed out.
    pub modified: bool,
}

impl Chunk {
    pub fn uniform(b: BlockId) -> Self {
        Self { data: Data::Uniform(b), modified: false }
    }

    /// Takes ownership of a dense buffer, collapsing it to a uniform chunk when possible.
    pub fn from_blocks(blocks: Vec<BlockId>) -> Self {
        debug_assert_eq!(blocks.len(), CHUNK_VOLUME);
        let first = blocks[0];
        if blocks.iter().all(|&b| b == first) {
            return Self::uniform(first);
        }
        Self { data: Data::Dense(blocks.into_boxed_slice()), modified: false }
    }

    #[inline]
    pub fn get(&self, x: usize, y: usize, z: usize) -> BlockId {
        match &self.data {
            Data::Uniform(b) => *b,
            Data::Dense(d) => d[index(x, y, z)],
        }
    }

    pub fn set(&mut self, x: usize, y: usize, z: usize, b: BlockId) {
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

    /// Whether both hold the same blocks, however each is stored.
    pub fn same_blocks(&self, other: &Chunk) -> bool {
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
                w.u8(*b);
                return;
            }
            Data::Dense(d) => d,
        };
        let mut start = 0;
        for i in 1..=CHUNK_VOLUME {
            if i == CHUNK_VOLUME || blocks[i] != blocks[start] {
                w.u16((i - start) as u16);
                w.u8(blocks[start]);
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
