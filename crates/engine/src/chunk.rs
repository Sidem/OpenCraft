//! 32³ block storage. Homogeneous chunks (all air, all stone) cost no heap memory.

use crate::block::BlockId;

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
}

#[cfg(test)]
mod tests;
