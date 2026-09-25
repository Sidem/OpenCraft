//! Chunk storage and streaming around the player.
//!
//! Work is split into small units (generate one chunk / mesh one chunk) that the host calls under a
//! per-frame time budget, nearest-first. Results reach the renderer through an ordered event queue.
//!
//! Invariants: player-edited chunks are never lost; when they stream out they move to `saved` and
//! come back instead of being regenerated. `get_block` / `set_block` only see loaded chunks;
//! `block_anywhere` / `set_block_anywhere` work everywhere, which is what deterministic core code
//! must use (DEV_PLAN section 3.4).

use std::collections::VecDeque;

use rustc_hash::{FxHashMap, FxHashSet};

use crate::block::{BlockId, AIR, BEDROCK, SOLID, STONE};
use crate::chunk::{Chunk, CHUNK_MASK, CHUNK_SHIFT, CHUNK_SIZE};
use crate::math::{IVec3, Vec3};
use crate::mesher::{neighbor_index, Mesher};
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

    /// Like [`World::set_block`], but also works where no chunk is loaded (machines keep running
    /// while the player is away): the edit goes into the stored copy of that chunk.
    pub fn set_block_anywhere(&mut self, p: IVec3, b: BlockId) -> bool {
        if p.y < 0 || p.y >= WORLD_HEIGHT {
            return false;
        }
        let c = chunk_of(p);
        if self.chunks.contains_key(&c) {
            return self.set_block(p, b);
        }
        let mut chunk = match self.saved.remove(&c) {
            Some(chunk) => chunk,
            None => self.generator.generate(c),
        };
        let (x, y, z) = local_of(p);
        chunk.set(x, y, z, b);
        chunk.modified = true;
        self.saved.insert(c, chunk);
        true
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

    /// Re-centres streaming on the player: unloads distant chunks and rebuilds the generation queue.
    pub fn update_streaming(&mut self, player: Vec3) {
        let b = player.floor();
        let (px, pz) = (b.x >> CHUNK_SHIFT, b.z >> CHUNK_SHIFT);
        self.focus = IVec3::new(px, (b.y >> CHUNK_SHIFT).clamp(0, WORLD_HEIGHT_CHUNKS - 1), pz);
        if self.center == Some((px, pz)) {
            return;
        }
        self.center = Some((px, pz));
        let r2 = self.view_radius * self.view_radius;

        let far: Vec<IVec3> = self.chunks.keys().filter(|p| ring_dist2(p.x - px, p.z - pz, 2) > r2).copied().collect();
        for p in far {
            let e = self.chunks.remove(&p).expect("key collected above");
            if e.chunk.modified {
                self.saved.insert(p, e.chunk);
            }
            if e.has_mesh {
                self.events.push_back(Event::Unload(p));
            }
            self.dirty.remove(&p);
        }
        self.generator.retain_columns(|cx, cz| ring_dist2(cx - px, cz - pz, 2) <= r2);

        // A chunk is generated if any of its neighbours is inside the view radius (so that
        // every visible chunk has all the neighbour data it needs for meshing).
        self.gen_queue.clear();
        let r = self.view_radius + 1;
        for dz in -r..=r {
            for dx in -r..=r {
                if ring_dist2(dx, dz, 1) > r2 {
                    continue;
                }
                for cy in 0..WORLD_HEIGHT_CHUNKS {
                    let p = IVec3::new(px + dx, cy, pz + dz);
                    if !self.chunks.contains_key(&p) {
                        self.gen_queue.push(p);
                    }
                }
            }
        }
        let focus = self.focus;
        self.gen_queue.sort_unstable_by_key(|&p| std::cmp::Reverse(priority(focus, p)));
        self.mesh_queue_stale = true;
    }

    /// Collects dirty chunks that can be meshed now, farthest first (popped from the back).
    pub fn begin_work(&mut self) {
        if !self.mesh_queue_stale {
            return;
        }
        self.mesh_queue_stale = false;
        let Some((cx, cz)) = self.center else { return };
        let r2 = self.view_radius * self.view_radius;
        let mut q = std::mem::take(&mut self.mesh_queue);
        q.clear();
        q.extend(
            self.dirty
                .iter()
                .filter(|p| ring_dist2(p.x - cx, p.z - cz, 0) <= r2 && self.neighbors_loaded(**p))
                .copied(),
        );
        let focus = self.focus;
        q.sort_unstable_by_key(|&p| std::cmp::Reverse(priority(focus, p)));
        self.mesh_queue = q;
    }

    /// Performs one unit of streaming work, nearest first. Returns false when idle.
    pub fn work_step(&mut self) -> bool {
        while self.gen_queue.last().is_some_and(|p| self.chunks.contains_key(p)) {
            self.gen_queue.pop();
        }
        while self.mesh_queue.last().is_some_and(|p| !self.dirty.contains(p)) {
            self.mesh_queue.pop();
        }
        let gen_next = self.gen_queue.last().map(|&p| priority(self.focus, p));
        let mesh_next = self.mesh_queue.last().map(|&p| priority(self.focus, p));
        let mesh = match (gen_next, mesh_next) {
            (None, None) => return false,
            (Some(g), Some(m)) => m <= g,
            (None, Some(_)) => true,
            (Some(_), None) => false,
        };
        if mesh {
            let p = self.mesh_queue.pop().expect("checked above");
            self.remesh(p);
        } else {
            let p = self.gen_queue.pop().expect("checked above");
            let chunk = self.saved.remove(&p).unwrap_or_else(|| self.generator.generate(p));
            self.chunks.insert(p, Entry { chunk, has_mesh: false });
            self.dirty.insert(p);
            self.mesh_queue_stale = true;
        }
        true
    }

    fn neighbors_loaded(&self, p: IVec3) -> bool {
        for dz in -1..=1 {
            for dy in -1..=1 {
                for dx in -1..=1 {
                    let q = p + IVec3::new(dx, dy, dz);
                    if q.y >= 0 && q.y < WORLD_HEIGHT_CHUNKS && !self.chunks.contains_key(&q) {
                        return false;
                    }
                }
            }
        }
        true
    }

    /// Meshes `p` if all neighbours are present; emits a mesh event when the visible result changes.
    fn remesh(&mut self, p: IVec3) -> bool {
        let mut refs: [&Chunk; 27] = [&self.air; 27];
        for dz in -1..=1 {
            for dy in -1..=1 {
                for dx in -1..=1 {
                    let q = p + IVec3::new(dx, dy, dz);
                    refs[neighbor_index(dx, dy, dz)] = if q.y < 0 {
                        &self.floor
                    } else if q.y >= WORLD_HEIGHT_CHUNKS {
                        &self.air
                    } else {
                        match self.chunks.get(&q) {
                            Some(e) => &e.chunk,
                            None => return false,
                        }
                    };
                }
            }
        }
        let out = self.mesher.mesh(&refs);
        self.dirty.remove(&p);
        let entry = self.chunks.get_mut(&p).expect("centre chunk is loaded");
        let quads = out.opaque_quads + out.cutout_quads;
        if quads == 0 && !entry.has_mesh {
            return true;
        }
        entry.has_mesh = quads > 0;
        self.events.push_back(Event::Mesh(MeshData {
            pos: p,
            verts: out.verts,
            opaque_quads: out.opaque_quads,
            cutout_quads: out.cutout_quads,
        }));
        true
    }

    /// True once everything within `radius` chunks of the player is generated and meshed.
    pub fn area_ready(&self, radius: i32) -> bool {
        let f = self.focus;
        for dz in -radius..=radius {
            for dx in -radius..=radius {
                for cy in 0..WORLD_HEIGHT_CHUNKS {
                    let p = IVec3::new(f.x + dx, cy, f.z + dz);
                    if !self.chunks.contains_key(&p) || self.dirty.contains(&p) {
                        return false;
                    }
                }
            }
        }
        true
    }

    pub fn loaded_count(&self) -> usize {
        self.chunks.len()
    }

    pub fn pending_count(&self) -> usize {
        self.gen_queue.len()
    }

    pub fn dirty_count(&self) -> usize {
        self.dirty.len()
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

/// Squared horizontal distance in chunks, after shrinking each axis by `shrink` chunks.
#[inline]
fn ring_dist2(dx: i32, dz: i32, shrink: i32) -> i32 {
    let ax = (dx.abs() - shrink).max(0);
    let az = (dz.abs() - shrink).max(0);
    ax * ax + az * az
}

#[inline]
fn priority(focus: IVec3, p: IVec3) -> i32 {
    let d = p - focus;
    d.x * d.x + d.z * d.z + d.y * d.y
}

#[cfg(test)]
mod tests;
