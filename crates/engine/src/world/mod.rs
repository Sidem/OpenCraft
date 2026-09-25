//! Chunk storage and block access: loaded chunks, edited chunks kept while streamed out (`saved`),
//! and the accessors. Streaming and meshing live in `streaming.rs`; results reach the renderer
//! through the ordered `events` queue.
//!
//! Invariants: player-edited chunks are never lost; when they stream out they move to `saved` and
//! come back instead of being regenerated. `get_block` / `set_block` only see loaded chunks (the
//! render cache); `block_anywhere_or_generate` / `set_block_anywhere` work everywhere, which is what
//! deterministic core code must use (DEV_PLAN section 3.4).

mod streaming;

use std::collections::VecDeque;

use rustc_hash::{FxHashMap, FxHashSet};

use crate::block::{BlockId, AIR, BEDROCK, SOLID, STONE};
use crate::bytes::ByteWriter;
use crate::chunk::{Chunk, CHUNK_MASK, CHUNK_SHIFT, CHUNK_SIZE};
use crate::math::{sort_small_by_key, IVec3, Vec3};
use crate::mesher::Mesher;
use crate::worldgen::{WorldGen, WORLD_HEIGHT, WORLD_HEIGHT_CHUNKS};

pub struct MeshData {
    pub pos: IVec3,
    pub verts: Vec<u32>,
    pub opaque_quads: u32,
    pub cutout_quads: u32,
}

pub enum Event {
    Mesh(MeshData),
    Unload(IVec3),
}

#[inline]
pub fn chunk_of(p: IVec3) -> IVec3 {
    IVec3::new(p.x >> CHUNK_SHIFT, p.y >> CHUNK_SHIFT, p.z >> CHUNK_SHIFT)
}

pub struct World {
    chunks: FxHashMap<IVec3, Entry>,
    generator: WorldGen,
    /// Player-modified chunks that were streamed out; restored instead of regenerated.
    saved: FxHashMap<IVec3, Chunk>,
    dirty: FxHashSet<IVec3>,
    gen_queue: Vec<IVec3>,
    mesh_queue: Vec<IVec3>,
    mesh_queue_stale: bool,
    center: Option<(i32, i32)>,
    focus: IVec3,
    view_radius: i32,
    mesher: Mesher,
    air: Chunk,
    floor: Chunk,
    pub events: VecDeque<Event>,
}

impl World {
    pub fn new(seed: u32, view_radius: i32) -> Self {
        Self {
            chunks: FxHashMap::default(),
            generator: WorldGen::new(seed),
            saved: FxHashMap::default(),
            dirty: FxHashSet::default(),
            gen_queue: Vec::new(),
            mesh_queue: Vec::new(),
            mesh_queue_stale: true,
            center: None,
            focus: IVec3::ZERO,
            view_radius: view_radius.max(2),
            mesher: Mesher::new(),
            air: Chunk::uniform(AIR),
            floor: Chunk::uniform(STONE),
            events: VecDeque::new(),
        }
    }

    pub fn generator(&self) -> &WorldGen {
        &self.generator
    }

    pub fn generator_mut(&mut self) -> &mut WorldGen {
        &mut self.generator
    }

    /// The chunk exactly as world generation produced it, ignoring any edits.
    pub fn original_chunk(&mut self, c: IVec3) -> Chunk {
        self.generator.generate(c)
    }

    /// A block anywhere in the world, loaded or not. `None` means the chunk is neither loaded nor
    /// edited, i.e. it still holds exactly what generation produced there.
    pub fn block_anywhere(&self, p: IVec3) -> Option<BlockId> {
        if p.y < 0 || p.y >= WORLD_HEIGHT {
            return self.get_block(p);
        }
        let c = chunk_of(p);
        let (x, y, z) = local_of(p);
        match self.chunks.get(&c) {
            Some(e) => Some(e.chunk.get(x, y, z)),
            None => self.saved.get(&c).map(|chunk| chunk.get(x, y, z)),
        }
    }

    /// A block anywhere in the world. Where no copy exists, generates the chunk's original contents
    /// (without keeping them), so this can be slow outside loaded or edited chunks.
    pub fn block_anywhere_or_generate(&mut self, p: IVec3) -> BlockId {
        if let Some(b) = self.block_anywhere(p) {
            return b;
        }
        let (x, y, z) = local_of(p);
        self.generator.generate(chunk_of(p)).get(x, y, z)
    }

    /// Like [`World::set_block`], but also works where no chunk is loaded (machines keep running
    /// while the player is away): the edit goes into the stored copy of that chunk. As with
    /// `set_block`, writing the block that is already there changes nothing and returns false.
    pub fn set_block_anywhere(&mut self, p: IVec3, b: BlockId) -> bool {
        if p.y < 0 || p.y >= WORLD_HEIGHT {
            return false;
        }
        let c = chunk_of(p);
        if self.chunks.contains_key(&c) {
            return self.set_block(p, b);
        }
        let (x, y, z) = local_of(p);
        if let Some(chunk) = self.saved.get_mut(&c) {
            if chunk.get(x, y, z) == b {
                return false;
            }
            chunk.set(x, y, z, b);
            return true;
        }
        let mut chunk = self.generator.generate(c);
        if chunk.get(x, y, z) == b {
            return false;
        }
        chunk.set(x, y, z, b);
        chunk.modified = true;
        self.saved.insert(c, chunk);
        true
    }

    /// Core state: every edited chunk, loaded or stored, sorted by coordinate (x, y, z).
    pub fn write_state(&self, w: &mut ByteWriter) {
        let loaded = self.chunks.iter().filter(|(_, e)| e.chunk.modified).map(|(&c, e)| (c, &e.chunk));
        let mut edited: Vec<(IVec3, &Chunk)> = loaded.chain(self.saved.iter().map(|(&c, chunk)| (c, chunk))).collect();
        sort_small_by_key(&mut edited, |(c, _)| (c.x, c.y, c.z));
        w.count(edited.len());
        for (c, chunk) in edited {
            w.ivec3(c);
            chunk.write_state(w);
        }
    }

    pub fn set_view_radius(&mut self, r: i32) {
        self.view_radius = r.max(2);
        self.center = None;
    }

    /// Render distance in blocks.
    pub fn view_distance(&self) -> f64 {
        (self.view_radius * CHUNK_SIZE) as f64
    }

    pub fn get_block(&self, p: IVec3) -> Option<BlockId> {
        if p.y < 0 {
            return Some(BEDROCK);
        }
        if p.y >= WORLD_HEIGHT {
            return Some(AIR);
        }
        self.chunks.get(&chunk_of(p)).map(|e| {
            let (x, y, z) = local_of(p);
            e.chunk.get(x, y, z)
        })
    }

    /// Unloaded space counts as solid so nothing falls through the world while it streams in.
    #[inline]
    pub fn is_solid(&self, x: i32, y: i32, z: i32) -> bool {
        self.get_block(IVec3::new(x, y, z)).is_none_or(|b| SOLID[b as usize])
    }

    pub fn is_loaded(&self, p: Vec3) -> bool {
        let c = chunk_of(p.floor());
        c.y < 0 || c.y >= WORLD_HEIGHT_CHUNKS || self.chunks.contains_key(&c)
    }

    /// Changes a block and immediately remeshes every chunk whose mesh can see it
    /// (up to 8, because AO samples across chunk borders).
    pub fn set_block(&mut self, p: IVec3, b: BlockId) -> bool {
        if p.y < 0 || p.y >= WORLD_HEIGHT {
            return false;
        }
        let Some(entry) = self.chunks.get_mut(&chunk_of(p)) else { return false };
        let (x, y, z) = local_of(p);
        if entry.chunk.get(x, y, z) == b {
            return false;
        }
        entry.chunk.set(x, y, z, b);
        entry.chunk.modified = true;

        let mut affected: Vec<IVec3> = Vec::with_capacity(8);
        for dz in -1..=1 {
            for dy in -1..=1 {
                for dx in -1..=1 {
                    let c = chunk_of(p + IVec3::new(dx, dy, dz));
                    if self.chunks.contains_key(&c) && !affected.contains(&c) {
                        affected.push(c);
                    }
                }
            }
        }
        for c in affected {
            self.dirty.insert(c);
            self.remesh(c);
        }
        self.mesh_queue_stale = true;
        true
    }
}

struct Entry {
    chunk: Chunk,
    has_mesh: bool,
}

#[inline]
fn local_of(p: IVec3) -> (usize, usize, usize) {
    ((p.x & CHUNK_MASK) as usize, (p.y & CHUNK_MASK) as usize, (p.z & CHUNK_MASK) as usize)
}

#[cfg(test)]
mod tests;
