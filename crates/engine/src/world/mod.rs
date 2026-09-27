//! Chunk storage and block access: loaded chunks, edited chunks kept while streamed out (`saved`),
//! and the accessors. Streaming and meshing live in `streaming.rs`; results reach the renderer
//! through the ordered `events` queue.
//!
//! Invariants: player-edited chunks are never lost; when they stream out they move to `saved` and
//! come back instead of being regenerated. `get_block` / `set_block` only see loaded chunks (the
//! render cache); `block_anywhere_or_generate` / `set_block_anywhere` work everywhere, which is what
//! deterministic core code must use (DEV_PLAN section 3.4). Those keep the last `GENERATED_CACHE`
//! chunks they generated (`generated`), exactly as generation made them, so core rules reading
//! around unloaded ground don't generate the same chunk over and over.

mod streaming;

use std::collections::VecDeque;

use rustc_hash::{FxHashMap, FxHashSet};

use crate::block::{BlockId, AIR, BEDROCK, LIQUID, SOLID, STONE};
use crate::bytes::{ByteReader, ByteWriter};
use crate::chunk::{Chunk, CHUNK_MASK, CHUNK_SHIFT, CHUNK_SIZE};
use crate::light::{Lighting, MARGIN};
use crate::math::{sort_small_by_key, IVec3, Vec3};
use crate::mesher::Mesher;
use crate::worldgen::{WorldGen, WORLD_HEIGHT, WORLD_HEIGHT_CHUNKS};

/// Generated chunks kept for reads outside loaded and edited chunks.
const GENERATED_CACHE: usize = 8;

pub struct MeshData {
    pub pos: IVec3,
    pub verts: Vec<u32>,
    pub opaque_quads: u32,
    pub cutout_quads: u32,
    pub liquid_quads: u32,
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
    /// The chunks other players stand in (`update_streaming`).
    others: Vec<IVec3>,
    view_radius: i32,
    mesher: Mesher,
    lighting: Lighting,
    air: Chunk,
    floor: Chunk,
    /// Recently generated, unedited chunks, oldest first (never shadows an edit: `chunks` and
    /// `saved` are asked first).
    generated: Vec<(IVec3, Chunk)>,
    pub events: VecDeque<Event>,
}

impl World {
    /// A world from the newest generator.
    #[cfg(test)]
    pub fn new(seed: u32, view_radius: i32) -> Self {
        Self::with_generator(WorldGen::new(seed), view_radius)
    }

    pub fn with_generator(generator: WorldGen, view_radius: i32) -> Self {
        Self {
            chunks: FxHashMap::default(),
            generator,
            saved: FxHashMap::default(),
            dirty: FxHashSet::default(),
            gen_queue: Vec::new(),
            mesh_queue: Vec::new(),
            mesh_queue_stale: true,
            center: None,
            focus: IVec3::ZERO,
            others: Vec::new(),
            view_radius: view_radius.max(2),
            mesher: Mesher::new(),
            lighting: Lighting::default(),
            air: Chunk::uniform(AIR),
            floor: Chunk::uniform(STONE),
            generated: Vec::new(),
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
        let c = chunk_of(p);
        if let Some((_, chunk)) = self.generated.iter().find(|g| g.0 == c) {
            return chunk.get(x, y, z);
        }
        let chunk = self.generator.generate(c);
        let b = chunk.get(x, y, z);
        if self.generated.len() >= GENERATED_CACHE {
            self.generated.remove(0);
        }
        self.generated.push((c, chunk));
        b
    }

    /// Like [`World::set_block`], but also works where no chunk is loaded (machines keep running
    /// while the player is away): the edit goes into the stored copy of that chunk. As with
    /// `set_block`, writing the block that is already there changes nothing and returns false.
    pub fn set_block_anywhere(&mut self, p: IVec3, b: BlockId) -> bool {
        self.edit_anywhere(p, b, true)
    }

    /// Like [`World::set_block_anywhere`], but a loaded chunk is remeshed later, within the streaming
    /// budget, instead of at once: for rules that change many blocks a tick (flowing water).
    pub fn set_block_anywhere_later(&mut self, p: IVec3, b: BlockId) -> bool {
        self.edit_anywhere(p, b, false)
    }

    fn edit_anywhere(&mut self, p: IVec3, b: BlockId, remesh_now: bool) -> bool {
        if p.y < 0 || p.y >= WORLD_HEIGHT {
            return false;
        }
        let c = chunk_of(p);
        if self.chunks.contains_key(&c) {
            return self.edit_loaded(p, b, remesh_now);
        }
        let (x, y, z) = local_of(p);
        if let Some(chunk) = self.saved.get_mut(&c) {
            if chunk.get(x, y, z) == b {
                return false;
            }
            chunk.set(x, y, z, b);
            return true;
        }
        let cached = self.generated.iter().position(|g| g.0 == c);
        let mut chunk = match cached {
            Some(i) if self.generated[i].1.get(x, y, z) == b => return false,
            Some(i) => self.generated.remove(i).1,
            None => self.generator.generate(c),
        };
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

    /// Restores the edited chunks `write_state` wrote, into a world with no edits yet. They go to
    /// storage and stream in like any edited chunk.
    pub fn read_state(&mut self, r: &mut ByteReader) -> Option<()> {
        for _ in 0..r.count()? {
            let c = r.ivec3()?;
            let mut chunk = Chunk::read_state(r)?;
            chunk.modified = true;
            if !(0..WORLD_HEIGHT_CHUNKS).contains(&c.y) || self.saved.insert(c, chunk).is_some() {
                return None;
            }
        }
        Some(())
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

    /// A loaded chunk (the render cache): for presentation such as the minimap, never core code.
    pub fn loaded_chunk(&self, c: IVec3) -> Option<&Chunk> {
        self.chunks.get(&c).map(|e| &e.chunk)
    }

    /// Unloaded space counts as solid so nothing falls through the world while it streams in.
    #[inline]
    pub fn is_solid(&self, x: i32, y: i32, z: i32) -> bool {
        self.get_block(IVec3::new(x, y, z)).is_none_or(|b| SOLID[b as usize])
    }

    /// Whether a loaded cell holds a liquid (bodies swim and items float in it).
    #[inline]
    pub fn is_water(&self, x: i32, y: i32, z: i32) -> bool {
        self.get_block(IVec3::new(x, y, z)).is_some_and(|b| LIQUID[b as usize])
    }

    pub fn is_loaded(&self, p: Vec3) -> bool {
        let c = chunk_of(p.floor());
        c.y < 0 || c.y >= WORLD_HEIGHT_CHUNKS || self.chunks.contains_key(&c)
    }

    /// Changes a block and immediately remeshes every chunk whose mesh can see it (up to 8, because
    /// AO samples across chunk borders). Chunks whose light it can change (within `MARGIN` blocks,
    /// and everything below, which its shadow reaches) remesh later. Core code uses `set_block_anywhere`.
    #[cfg(test)]
    pub fn set_block(&mut self, p: IVec3, b: BlockId) -> bool {
        self.edit_loaded(p, b, true)
    }

    fn edit_loaded(&mut self, p: IVec3, b: BlockId, remesh_now: bool) -> bool {
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
            if remesh_now {
                self.remesh(c);
            }
        }
        let m = MARGIN as i32;
        let (lo, hi) = (chunk_of(p - IVec3::new(m, 0, m)), chunk_of(p + IVec3::new(m, m, m)));
        for cz in lo.z..=hi.z {
            for cx in lo.x..=hi.x {
                for cy in 0..=hi.y.min(WORLD_HEIGHT_CHUNKS - 1) {
                    let c = IVec3::new(cx, cy, cz);
                    if self.chunks.contains_key(&c) {
                        self.dirty.insert(c);
                    }
                }
            }
        }
        self.mesh_queue_stale = true;
        true
    }

    /// Co-op resync: this world, just read from a host's snapshot, takes over `old`'s render cache
    /// (loaded chunks, queues, pending renderer events), so nothing streams in again. A loaded chunk
    /// whose blocks differ from this world's is replaced and remeshed with its neighbours; the rest
    /// keep their meshes.
    pub fn adopt_loaded(&mut self, old: World) {
        let World { chunks, dirty, gen_queue, center, focus, others, view_radius, events, .. } = old;
        (self.dirty, self.gen_queue, self.center, self.focus) = (dirty, gen_queue, center, focus);
        (self.others, self.view_radius, self.events) = (others, view_radius, events);
        let mut changed = Vec::new();
        for (p, mut e) in chunks {
            let fresh = match self.saved.remove(&p) {
                Some(chunk) => Some(chunk),
                None if e.chunk.modified => Some(self.generator.generate(p)),
                None => None,
            };
            if let Some(chunk) = fresh {
                if !chunk.same_blocks(&e.chunk) {
                    changed.push(p);
                }
                e.chunk = chunk;
            }
            self.chunks.insert(p, e);
        }
        for p in changed {
            for dz in -1..=1 {
                for y in 0..=p.y + 1 {
                    for dx in -1..=1 {
                        let q = IVec3::new(p.x + dx, y, p.z + dz);
                        if self.chunks.contains_key(&q) {
                            self.dirty.insert(q);
                        }
                    }
                }
            }
        }
        self.mesh_queue_stale = true;
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
