//! Streaming and meshing, the render cache side of `World`: re-centring on the player, the
//! generation and mesh queues, one unit of work per `work_step`, and remeshing edited chunks.
//!
//! Chunks load within the view radius of the local player and within `OTHERS_RADIUS` of every other
//! player the authority moves (their bodies and the items near them need ground); only the local
//! player's are meshed. Nearest chunks first (`priority`, to the nearest player). A chunk is meshed
//! only once all 26 neighbours are loaded, because AO and face culling read across borders. None of
//! this is core state (DEV_PLAN 3.4).

use crate::chunk::{Chunk, CHUNK_SHIFT};
use crate::math::{IVec3, Vec3};
use crate::mesher::neighbor_index;
use crate::worldgen::WORLD_HEIGHT_CHUNKS;

use super::{Entry, Event, MeshData, World};

/// Chunks loaded around each other player, in chunks (plus the neighbours generation needs).
pub const OTHERS_RADIUS: i32 = 1;

impl World {
    /// Re-centres streaming on the local player and the `others`: unloads distant chunks and
    /// rebuilds the generation queue.
    pub fn update_streaming(&mut self, player: Vec3, others: &[Vec3]) {
        self.focus = focus_of(player);
        let (px, pz) = (self.focus.x, self.focus.z);
        let foci: Vec<IVec3> = others.iter().map(|&p| focus_of(p)).collect();
        if self.center == Some((px, pz)) && foci == self.others {
            return;
        }
        self.center = Some((px, pz));
        self.others = foci;
        let r2 = self.view_radius * self.view_radius;
        let others = &self.others;
        let keep = |cx: i32, cz: i32| {
            ring_dist2(cx - px, cz - pz, 2) <= r2
                || others.iter().any(|o| ring_dist2(cx - o.x, cz - o.z, 2) <= OTHERS_RADIUS * OTHERS_RADIUS)
        };

        let far: Vec<IVec3> = self.chunks.keys().filter(|p| !keep(p.x, p.z)).copied().collect();
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
        self.generator.retain_columns(keep);

        // A chunk is generated if any of its neighbours is inside the view radius (so that
        // every visible chunk has all the neighbour data it needs for meshing). Rings of several
        // players may overlap; `work_step` skips a chunk that is already loaded.
        self.gen_queue.clear();
        let rings = self.others.iter().map(|o| (o.x, o.z, OTHERS_RADIUS)).chain([(px, pz, self.view_radius)]);
        for (cx, cz, radius) in rings {
            let r = radius + 1;
            for dz in -r..=r {
                for dx in -r..=r {
                    if ring_dist2(dx, dz, 1) > radius * radius {
                        continue;
                    }
                    for cy in 0..WORLD_HEIGHT_CHUNKS {
                        let p = IVec3::new(cx + dx, cy, cz + dz);
                        if !self.chunks.contains_key(&p) {
                            self.gen_queue.push(p);
                        }
                    }
                }
            }
        }
        sort_nearest_last(&mut self.gen_queue, self.focus, &self.others);
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
        sort_nearest_last(&mut q, self.focus, &[]);
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
        let gen_next = self.gen_queue.last().map(|&p| nearest(self.focus, &self.others, p));
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

/// The chunk a player at `p` stands in, its height clamped into the world.
fn focus_of(p: Vec3) -> IVec3 {
    let b = p.floor();
    IVec3::new(b.x >> CHUNK_SHIFT, (b.y >> CHUNK_SHIFT).clamp(0, WORLD_HEIGHT_CHUNKS - 1), b.z >> CHUNK_SHIFT)
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

/// Sorts a work queue so the chunk nearest a player comes last (popped first). The one sort both
/// queues share: each distinct sort closure adds 5–9 KB of wasm.
fn sort_nearest_last(queue: &mut [IVec3], focus: IVec3, others: &[IVec3]) {
    queue.sort_unstable_by_key(|&p| std::cmp::Reverse(nearest(focus, others, p)));
}

/// `priority` to the nearest player: the local one at `focus` or one of the `others`. Kept out of line:
/// inlined into the sort's comparisons it costs about 9 KB of wasm.
#[inline(never)]
fn nearest(focus: IVec3, others: &[IVec3], p: IVec3) -> i32 {
    others.iter().fold(priority(focus, p), |d, &o| d.min(priority(o, p)))
}
