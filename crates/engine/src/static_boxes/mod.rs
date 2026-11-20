//! Per-chunk static machine geometry cache (presentation only; owned by `Game`, never in `Sim` or the state hash).
//!
//! Caches static machine box instances (belt frames with shader-driven scroll, poles, towers, cables, processor hulls,
//! router gatehouses, pipework, rail nodes/tracks, fibre nodes/lines and power wires/props) per 32³ chunk in chunk-local
//! coordinates, pre-lit via `World::light_at_tracked` and pre-split into cube-only and round-only buffers. The host
//! uploads each chunk once into its own GPU buffer (`web/src/render/boxes.ts`) and draws visible chunks with a per-chunk
//! `u_offset` uniform and frustum culling against tight chunk-local bounds.
//!
//! Invariants:
//! - On a steady frame (no layout edit, no wire prop change, no light change, camera in the same chunk), `update` does
//!   zero geometry work, zero lighting lookups and emits zero GPU upload events.
//! - Structural edits (`Factory::layout_gen`) rebuild only chunks whose 64-bit structural fingerprint changed.
//! - Lighting updates (`World::light_version`) relight cached buffers in place without re-running machine models; chunks
//!   whose initial lighting hit `World::light_budget` retry at `LIGHTS_PER_FRAME` per frame without spinning at borders.

use std::collections::VecDeque;

use rustc_hash::{FxHashMap, FxHashSet};

use crate::chunk::CHUNK_SIZE;
use crate::factory::{self, is_round, ChunkIndex, INSTANCE_FLOATS};
use crate::math::{sort_small_by_key, IVec3, Vec3};
use crate::wire_props::WireProps;
use crate::world::{chunk_of, World};
use crate::worldgen::WORLD_HEIGHT;

#[cfg(test)]
mod tests;

/// Slack added to `view_distance` when streaming static box chunks in/out, covering chunk half-diagonal plus camera
/// movement within `last_eye_chunk` so chunks never pop at the view edge.
const STREAM_SLACK: f64 = 64.0;

/// Pending upload or removal for the host renderer (`api/render.rs`, `web/src/render/mesh-events.ts`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum StaticBoxEvent {
    Upsert(IVec3),
    Unload(IVec3),
}

/// One cached 32³ chunk of static boxes in chunk-local coordinates (relative to `pos * 32`).
pub(crate) struct CachedChunk {
    /// Non-round boxes: `..factory_cube_end` from `Factory` (included in X-ray), `factory_cube_end..` power wires.
    pub(crate) cubes: Vec<f32>,
    pub(crate) factory_cube_end: usize,
    /// Round bodies (all from `Factory`).
    pub(crate) round: Vec<f32>,
    /// Conservative chunk-local AABB `[min_x, min_y, min_z, max_x, max_y, max_z]` for frustum culling.
    pub(crate) bounds: [f32; 6],
    fingerprint: u64,
    /// Chunks queried when lighting this chunk and their `chunk_light_gen` at that time.
    light_deps: Vec<(IVec3, u32)>,
}

impl CachedChunk {
    fn instance_count(&self) -> usize {
        (self.cubes.len() + self.round.len()) / INSTANCE_FLOATS
    }
}

pub(crate) struct StaticBoxes {
    pub(crate) index: ChunkIndex,
    chunks: FxHashMap<IVec3, CachedChunk>,
    last_layout_gen: u64,
    last_wire_version: u64,
    last_light_version: u64,
    last_eye_chunk: Option<IVec3>,
    last_view_dist: i32,
    last_strategy: bool,
    last_explored: usize,
    budget_retry: FxHashSet<IVec3>,
    dirty: FxHashSet<IVec3>,
    pub(crate) events: VecDeque<StaticBoxEvent>,
    pub(crate) cur: Option<StaticBoxEvent>,
    pub(crate) total_instances: usize,
}

impl Default for StaticBoxes {
    fn default() -> Self {
        Self {
            index: ChunkIndex::default(),
            chunks: FxHashMap::default(),
            last_layout_gen: u64::MAX,
            last_wire_version: u64::MAX,
            last_light_version: u64::MAX,
            last_eye_chunk: None,
            last_view_dist: -1,
            last_strategy: false,
            last_explored: 0,
            budget_retry: FxHashSet::default(),
            dirty: FxHashSet::default(),
            events: VecDeque::new(),
            cur: None,
            total_instances: 0,
        }
    }
}

impl StaticBoxes {
    /// Clears all cached chunks (queuing `Unload` events for the host) and forces a full rebuild (co-op snapshot resync).
    pub(crate) fn reset(&mut self) {
        for &pos in self.chunks.keys() {
            self.events.push_back(StaticBoxEvent::Unload(pos));
        }
        self.chunks.clear();
        self.index.clear();
        self.budget_retry.clear();
        self.dirty.clear();
        self.cur = None;
        self.total_instances = 0;
        self.last_layout_gen = u64::MAX;
        self.last_wire_version = u64::MAX;
        self.last_light_version = u64::MAX;
        self.last_eye_chunk = None;
    }

    /// Reads a cached chunk by its chunk coordinate (for `api/render.rs` zero-copy pointers).
    pub(crate) fn chunk(&self, pos: IVec3) -> Option<&CachedChunk> {
        self.chunks.get(&pos)
    }

    /// Syncs the spatial index, streams chunks in/out for `eye` and `strategy`, checks lighting staleness and builds
    /// dirty chunks. Steady frames return immediately after a few integer comparisons.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn update(
        &mut self,
        factory: &factory::Factory,
        wire_props: &WireProps,
        world: &mut World,
        eye: Vec3,
        strategy: bool,
        explored_len: usize,
        is_explored: impl Fn(i32, i32) -> bool,
    ) {
        let mut index_changed = false;
        if factory.layout_gen != self.last_layout_gen {
            self.last_layout_gen = factory.layout_gen;
            self.index.rebuild_factory(factory);
            index_changed = true;
        }
        if wire_props.version() != self.last_wire_version {
            self.last_wire_version = wire_props.version();
            self.index.rebuild_wires(|chunks| wire_props.emit_binned(chunks));
            index_changed = true;
        }

        let eye_chunk = chunk_of(eye.floor());
        let view_dist = world.view_distance().ceil() as i32;
        let explored_key = if strategy { explored_len } else { 0 };
        let view_changed = self.last_eye_chunk != Some(eye_chunk)
            || self.last_view_dist != view_dist
            || self.last_strategy != strategy
            || self.last_explored != explored_key;

        if index_changed || view_changed {
            self.last_eye_chunk = Some(eye_chunk);
            self.last_view_dist = view_dist;
            self.last_strategy = strategy;
            self.last_explored = explored_key;
            let max_d2 = (f64::from(view_dist) + STREAM_SLACK).powi(2);
            let eye_mid = (eye_chunk * CHUNK_SIZE).as_vec3() + Vec3::new(16.0, 16.0, 16.0);
            let in_range = |c: IVec3| {
                let d = (c * CHUNK_SIZE).as_vec3() + Vec3::new(16.0, 16.0, 16.0) - eye_mid;
                d.x * d.x + d.y * d.y + d.z * d.z <= max_d2 && (!strategy || is_explored(c.x, c.z))
            };
            let drop: Vec<IVec3> =
                self.chunks.keys().copied().filter(|&c| !self.index.chunks.contains_key(&c) || !in_range(c)).collect();
            for c in drop {
                if let Some(old) = self.chunks.remove(&c) {
                    self.total_instances -= old.instance_count();
                }
                self.budget_retry.remove(&c);
                self.dirty.remove(&c);
                self.events.push_back(StaticBoxEvent::Unload(c));
            }
            for (&c, m) in &self.index.chunks {
                if !in_range(c) {
                    continue;
                }
                if self.chunks.get(&c).is_none_or(|cached| cached.fingerprint != m.fingerprint()) {
                    self.dirty.insert(c);
                }
            }
        }

        if world.light_version() != self.last_light_version {
            self.last_light_version = world.light_version();
            for (&c, cached) in &self.chunks {
                if self.dirty.contains(&c) || self.budget_retry.contains(&c) {
                    continue;
                }
                let stale = cached.light_deps.iter().any(|&(dep, gen)| {
                    world.chunk_light_gen(dep) != gen || (gen == 0 && !world.is_unlit_sentinel(dep))
                });
                if stale {
                    self.dirty.insert(c);
                }
            }
        }

        if !self.budget_retry.is_empty() && world.light_budget() > 0 {
            let mut retries: Vec<IVec3> = self.budget_retry.iter().copied().collect();
            sort_small_by_key(&mut retries, |c| dist2_chunk(*c, eye_chunk));
            let take = usize::from(world.light_budget()).min(retries.len());
            for c in retries.into_iter().take(take) {
                self.budget_retry.remove(&c);
                if self.chunks.contains_key(&c) {
                    self.dirty.insert(c);
                }
            }
        }

        if self.dirty.is_empty() {
            return;
        }
        let mut to_build: Vec<IVec3> = self.dirty.drain().collect();
        if to_build.len() > 2 {
            sort_small_by_key(&mut to_build, |c| dist2_chunk(*c, eye_chunk));
        }
        let mut scratch = Vec::new();
        for c in to_build {
            self.rebuild_chunk(c, factory, world, &mut scratch);
        }
    }

    /// Rebuilds (or relights in place if the geometry fingerprint is unchanged) one chunk `c`.
    fn rebuild_chunk(&mut self, c: IVec3, factory: &factory::Factory, world: &mut World, scratch: &mut Vec<f32>) {
        let Some(m) = self.index.chunks.get(&c) else {
            if let Some(old) = self.chunks.remove(&c) {
                self.total_instances -= old.instance_count();
                self.budget_retry.remove(&c);
                self.events.push_back(StaticBoxEvent::Unload(c));
            }
            return;
        };
        let origin = (c * CHUNK_SIZE).as_vec3();
        let fp = m.fingerprint();

        // Fast path: geometry unchanged, only lighting changed -> relight cached buffers in place.
        if let Some(cached) = self.chunks.get_mut(&c).filter(|cached| cached.fingerprint == fp) {
            let (deps, hit_budget) = light_buffers(&mut cached.cubes, &mut cached.round, origin, world);
            let changed = deps != cached.light_deps;
            cached.light_deps = deps;
            if hit_budget {
                self.budget_retry.insert(c);
            } else {
                self.budget_retry.remove(&c);
            }
            if changed {
                self.events.push_back(StaticBoxEvent::Upsert(c));
            }
            return;
        }

        scratch.clear();
        factory.write_chunk_static(m, origin, scratch);
        let factory_end = scratch.len();
        scratch.extend_from_slice(&m.wire_boxes);
        if scratch.is_empty() {
            if let Some(old) = self.chunks.remove(&c) {
                self.total_instances -= old.instance_count();
                self.budget_retry.remove(&c);
                self.events.push_back(StaticBoxEvent::Unload(c));
            }
            return;
        }

        let (mut cubes, mut round) = (Vec::with_capacity(scratch.len()), Vec::new());
        let mut bounds =
            [f32::INFINITY, f32::INFINITY, f32::INFINITY, f32::NEG_INFINITY, f32::NEG_INFINITY, f32::NEG_INFINITY];
        for rec in scratch[..factory_end].chunks_exact(INSTANCE_FLOATS) {
            expand_bounds(&mut bounds, rec);
            if is_round(rec) {
                round.extend_from_slice(rec);
            } else {
                cubes.extend_from_slice(rec);
            }
        }
        let factory_cube_end = cubes.len();
        for rec in scratch[factory_end..].chunks_exact(INSTANCE_FLOATS) {
            expand_bounds(&mut bounds, rec);
            cubes.extend_from_slice(rec);
        }

        let (light_deps, hit_budget) = light_buffers(&mut cubes, &mut round, origin, world);
        let entry = CachedChunk { cubes, factory_cube_end, round, bounds, fingerprint: fp, light_deps };
        let new_count = entry.instance_count();
        if let Some(old) = self.chunks.insert(c, entry) {
            self.total_instances -= old.instance_count();
        }
        self.total_instances += new_count;
        if hit_budget {
            self.budget_retry.insert(c);
        } else {
            self.budget_retry.remove(&c);
        }
        self.events.push_back(StaticBoxEvent::Upsert(c));
    }

    /// Appends static factory records (excluding power wires) within `radius` of `eye` into `out` and `round` (for
    /// `xray.rs`), converting chunk-local centres to camera-relative. Returns whether `max` was hit.
    pub(crate) fn copy_xray_near(
        &self,
        eye: Vec3,
        radius: f32,
        max: usize,
        out: &mut Vec<f32>,
        round: &mut Vec<f32>,
    ) -> bool {
        let r2 = radius * radius;
        let chunk_reach2 = (f64::from(radius) + 32.0).powi(2);
        for (&c, cached) in &self.chunks {
            let origin = (c * CHUNK_SIZE).as_vec3();
            let mid = origin + Vec3::new(16.0, 16.0, 16.0) - eye;
            if mid.x * mid.x + mid.y * mid.y + mid.z * mid.z > chunk_reach2 {
                continue;
            }
            let off = origin - eye;
            let (ox, oy, oz) = (off.x as f32, off.y as f32, off.z as f32);
            for (slice, is_r) in [(&cached.cubes[..cached.factory_cube_end], false), (&cached.round[..], true)] {
                for rec in slice.chunks_exact(INSTANCE_FLOATS) {
                    let (rx, ry, rz) = (rec[0] + ox, rec[1] + oy, rec[2] + oz);
                    if rx * rx + ry * ry + rz * rz > r2 {
                        continue;
                    }
                    if out.len() >= max * INSTANCE_FLOATS {
                        return true;
                    }
                    let start = out.len();
                    out.extend_from_slice(rec);
                    (out[start], out[start + 1], out[start + 2]) = (rx, ry, rz);
                    if is_r {
                        let rs = round.len();
                        round.extend_from_slice(rec);
                        (round[rs], round[rs + 1], round[rs + 2]) = (rx, ry, rz);
                    }
                }
            }
        }
        false
    }
}

fn dist2_chunk(a: IVec3, b: IVec3) -> i64 {
    let (dx, dy, dz) = (i64::from(a.x - b.x), i64::from(a.y - b.y), i64::from(a.z - b.z));
    dx * dx + dy * dy + dz * dz
}

/// Expands conservative chunk-local AABB `b` to cover box record `rec` regardless of rotation.
fn expand_bounds(b: &mut [f32; 6], rec: &[f32]) {
    let r = 0.5 * (rec[4] * rec[4] + rec[5] * rec[5] + rec[6] * rec[6]).sqrt();
    for i in 0..3 {
        b[i] = b[i].min(rec[i] - r);
        b[i + 3] = b[i + 3].max(rec[i] + r);
    }
}

/// Lights `cubes` and `round` (chunk-local centres relative to `origin`) and returns `(light_deps, hit_budget)`.
fn light_buffers(cubes: &mut [f32], round: &mut [f32], origin: Vec3, world: &mut World) -> (Vec<(IVec3, u32)>, bool) {
    let mut hit_budget = false;
    let mut deps: Vec<(IVec3, u32)> = Vec::new();
    let mut last_dep = IVec3::new(i32::MIN, i32::MIN, i32::MIN);
    let mut sample = |cell: IVec3| -> u8 {
        let (lit, budget) = world.light_at_tracked(cell);
        hit_budget |= budget;
        if (0..WORLD_HEIGHT).contains(&cell.y) {
            let cc = chunk_of(cell);
            if cc != last_dep {
                last_dep = cc;
                if !deps.iter().any(|&(k, _)| k == cc) {
                    deps.push((cc, world.chunk_light_gen(cc)));
                }
            }
        }
        lit
    };
    factory::light_boxes(cubes, origin, &mut sample);
    factory::light_boxes(round, origin, &mut sample);
    (deps, hit_budget)
}
