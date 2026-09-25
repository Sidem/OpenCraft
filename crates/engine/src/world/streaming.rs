//! Streaming and meshing, the render cache side of `World`: re-centring on the player, the
//! generation and mesh queues, one unit of work per `work_step`, and remeshing edited chunks.
//!
//! Nearest chunks first (`priority`). A chunk is meshed only once all 26 neighbours are loaded,
//! because AO and face culling read across borders. None of this is core state (DEV_PLAN 3.4).

use crate::chunk::{Chunk, CHUNK_SHIFT};
use crate::math::{IVec3, Vec3};
use crate::mesher::neighbor_index;
use crate::worldgen::WORLD_HEIGHT_CHUNKS;

use super::{Entry, Event, MeshData, World};

impl World {
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
    pub(super) fn remesh(&mut self, p: IVec3) -> bool {
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
