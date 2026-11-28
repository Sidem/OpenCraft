//! Spatial chunk index and static/dynamic split for factory rendering (`static_boxes.rs`).
//!
//! `ChunkIndex` groups every machine with cached static geometry (belts, poles, cables, processors, routers, pipework,
//! rail nodes, fibre nodes, plus binned rail tracks, fibre lines and power wires) by 32³ chunk, along with a 64-bit
//! structural fingerprint per chunk. Rebuilt only when `Factory::layout_gen` or `WireProps` changes; steady frames pay
//! zero cost for static geometry and cull dynamic machines (riding belt items, near tier studs, animated processor parts)
//! at chunk granularity. Presentation helper only: reads `Factory`, never mutates core state.

use rustc_hash::FxHashMap;

use crate::chunk::CHUNK_SIZE;
use crate::math::{IVec3, Vec3};
use crate::view_cone::ViewCone;
use crate::world::chunk_of;

use super::pipes::Part;
use super::power::wire;
use super::rail::write_track;
use super::render::{models, models_in_view, INSTANCE_FLOATS, MODEL_REACH};
use super::{router_model, tier_mark, Factory, Machine};

/// Half the diagonal of a 32³ chunk plus a small overhang margin, for coarse chunk-sphere culling.
const CHUNK_HALF_DIAG: f64 = 30.0;
/// How far a belt's riding items reach from its cell centre (lifts and drops carry them a cell up or down).
const BELT_REACH: f64 = 2.0;
const FNV_OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;

#[inline]
fn mix(h: &mut u64, v: u64) {
    *h = (*h ^ v).wrapping_mul(FNV_PRIME);
}

#[inline]
fn mix_ivec3(h: &mut u64, p: IVec3) {
    mix(h, p.x as u32 as u64 | ((p.y as u32 as u64) << 32));
    mix(h, p.z as u32 as u64);
}

fn hash_floats(xs: &[f32]) -> u64 {
    let mut h = FNV_OFFSET;
    mix(&mut h, xs.len() as u64);
    for &x in xs {
        mix(&mut h, u64::from(x.to_bits()));
    }
    h
}

/// Every static-capable machine and binned span box living in one 32³ chunk.
#[derive(Default)]
pub(crate) struct ChunkMembers {
    pub(crate) factory_fp: u64,
    pub(crate) wire_fp: u64,
    pub(crate) belts: Vec<u32>,
    pub(crate) stud_belts: Vec<u32>,
    pub(crate) poles: Vec<u32>,
    pub(crate) collar_poles: Vec<u32>,
    pub(crate) processors: Vec<u32>,
    pub(crate) smoke_processors: Vec<u32>,
    pub(crate) storages: Vec<u32>,
    pub(crate) routers: Vec<u32>,
    pub(crate) pipework: Vec<u32>,
    pub(crate) dynamic_pipes: Vec<u32>,
    pub(crate) rails: Vec<u32>,
    pub(crate) nodes: Vec<u32>,
    /// Factory span boxes (rail tracks, fibre lines), chunk-local and unlit (included in X-ray).
    pub(crate) span_boxes: Vec<f32>,
    /// Power wire and prop boxes, chunk-local and unlit (excluded from X-ray).
    pub(crate) wire_boxes: Vec<f32>,
}

impl ChunkMembers {
    pub(crate) fn fingerprint(&self) -> u64 {
        self.factory_fp ^ self.wire_fp.rotate_left(31)
    }

    pub(crate) fn has_static(&self) -> bool {
        !(self.belts.is_empty()
            && self.poles.is_empty()
            && self.processors.is_empty()
            && self.storages.is_empty()
            && self.routers.is_empty()
            && self.pipework.is_empty()
            && self.rails.is_empty()
            && self.nodes.is_empty()
            && self.span_boxes.is_empty()
            && self.wire_boxes.is_empty())
    }

    fn clear_factory(&mut self) {
        self.factory_fp = 0;
        self.belts.clear();
        self.stud_belts.clear();
        self.poles.clear();
        self.collar_poles.clear();
        self.processors.clear();
        self.smoke_processors.clear();
        self.storages.clear();
        self.routers.clear();
        self.pipework.clear();
        self.dynamic_pipes.clear();
        self.rails.clear();
        self.nodes.clear();
        self.span_boxes.clear();
    }

    fn clear_wires(&mut self) {
        self.wire_fp = 0;
        self.wire_boxes.clear();
    }
}

/// Chunk-bucketed index of all factory machines and wire/track/fibre spans.
#[derive(Default)]
pub(crate) struct ChunkIndex {
    pub(crate) chunks: FxHashMap<IVec3, ChunkMembers>,
}

impl ChunkIndex {
    pub(crate) fn clear(&mut self) {
        self.chunks.clear();
    }

    /// Rebuilds the factory-side machine lists, span boxes (tracks & fibre) and factory fingerprints.
    pub(crate) fn rebuild_factory(&mut self, f: &Factory) {
        for m in self.chunks.values_mut() {
            m.clear_factory();
        }
        for (i, b) in f.belts.iter().enumerate() {
            let m = self.chunks.entry(chunk_of(b.pos)).or_default();
            m.belts.push(i as u32);
            if b.has_studs() {
                m.stud_belts.push(i as u32);
            }
            mix_ivec3(&mut m.factory_fp, b.pos);
            let flags = u64::from(b.dir)
                | (u64::from(b.shape as u8) << 4)
                | (u64::from(b.tier) << 8)
                | (u64::from(b.curve_from.unwrap_or(255)) << 16)
                | (u64::from(b.lift_below) << 24)
                | (u64::from(b.lift_above) << 25)
                | (u64::from(b.fed_by_belt) << 26)
                | (u64::from(b.fed_by_port) << 27)
                | (u64::from(b.hands_to_belt()) << 28)
                | (u64::from(b.ends_in_opening()) << 29);
            mix(&mut m.factory_fp, flags);
        }
        for (i, p) in f.poles.iter().enumerate() {
            let m = self.chunks.entry(chunk_of(p.pos)).or_default();
            m.poles.push(i as u32);
            if !p.is_cable() && !p.is_long() {
                m.collar_poles.push(i as u32);
            }
            mix_ivec3(&mut m.factory_fp, p.pos);
            let mut bits =
                u64::from(p.tier) | (u64::from(p.wall) << 8) | (u64::from(f.power.pole_layer(i as u32)) << 16);
            if p.is_cable() {
                const AROUND: [(i32, i32, i32); 6] =
                    [(1, 0, 0), (-1, 0, 0), (0, 1, 0), (0, -1, 0), (0, 0, 1), (0, 0, -1)];
                for (k, &(x, y, z)) in AROUND.iter().enumerate() {
                    if f.at.contains_key(&(p.pos + IVec3::new(x, y, z))) {
                        bits |= 1u64 << (32 + k);
                    }
                }
            }
            mix(&mut m.factory_fp, bits);
        }
        for (i, p) in f.processors.iter().enumerate() {
            let m = self.chunks.entry(chunk_of(p.pos)).or_default();
            m.processors.push(i as u32);
            if p.has_smoke() {
                m.smoke_processors.push(i as u32);
            }
            mix_ivec3(&mut m.factory_fp, p.pos);
            let bits = u64::from(p.spec.block)
                | (u64::from(p.dir) << 16)
                | (u64::from(p.tier) << 20)
                | (u64::from(p.channel()) << 24)
                | (u64::from(p.is_compact()) << 32);
            mix(&mut m.factory_fp, bits);
        }
        for (i, s) in f.storages.iter().enumerate() {
            let m = self.chunks.entry(chunk_of(s.pos)).or_default();
            m.storages.push(i as u32);
            mix_ivec3(&mut m.factory_fp, s.pos);
            mix(&mut m.factory_fp, u64::from(s.tier));
        }
        for (i, r) in f.routers.iter().enumerate() {
            let m = self.chunks.entry(chunk_of(r.pos)).or_default();
            m.routers.push(i as u32);
            mix_ivec3(&mut m.factory_fp, r.pos);
            mix(&mut m.factory_fp, u64::from(r.dir) | (u64::from(r.kind as u8) << 4));
        }
        for (i, p) in f.pipework.iter().enumerate() {
            let m = self.chunks.entry(chunk_of(p.pos)).or_default();
            m.pipework.push(i as u32);
            if p.part != Part::Pipe {
                m.dynamic_pipes.push(i as u32);
            }
            mix_ivec3(&mut m.factory_fp, p.pos);
            let bits = u64::from(p.part as u8)
                | (u64::from(p.tier) << 4)
                | (u64::from(p.facing) << 8)
                | (u64::from(p.fluid as u8) << 12)
                | (u64::from(p.arms) << 16);
            mix(&mut m.factory_fp, bits);
        }
        for (i, r) in f.rails.iter().enumerate() {
            let m = self.chunks.entry(chunk_of(r.pos)).or_default();
            m.rails.push(i as u32);
            mix_ivec3(&mut m.factory_fp, r.pos);
            mix(&mut m.factory_fp, u64::from(r.yaw) | (u64::from(r.signal) << 8) | (u64::from(r.links == 0) << 9));
        }
        for (i, n) in f.nodes.iter().enumerate() {
            let m = self.chunks.entry(chunk_of(n.pos)).or_default();
            m.nodes.push(i as u32);
            mix_ivec3(&mut m.factory_fp, n.pos);
        }
        let mut scratch = Vec::new();
        f.emit_tracks_and_fibre(&mut scratch, &mut self.chunks);
        for m in self.chunks.values_mut() {
            if !m.span_boxes.is_empty() {
                mix(&mut m.factory_fp, hash_floats(&m.span_boxes));
            }
        }
        self.chunks.retain(|_, m| m.has_static());
    }

    /// Replaces every chunk's power-wire and prop boxes using `emit` (which pushes boxes relative to a caller base
    /// through [`bin_boxes`]) and refreshes each chunk's `wire_fp`.
    pub(crate) fn rebuild_wires(&mut self, emit: impl FnOnce(&mut FxHashMap<IVec3, ChunkMembers>)) {
        for m in self.chunks.values_mut() {
            m.clear_wires();
        }
        emit(&mut self.chunks);
        for m in self.chunks.values_mut() {
            if !m.wire_boxes.is_empty() {
                m.wire_fp = hash_floats(&m.wire_boxes);
            }
        }
        self.chunks.retain(|_, m| m.has_static());
    }
}

/// Bins box records generated relative to `base` into the chunk holding each box's world centre, storing chunk-local
/// centres in `span_boxes` (`into_wires = false`) or `wire_boxes` (`into_wires = true`).
pub(crate) fn bin_boxes(scratch: &[f32], base: Vec3, chunks: &mut FxHashMap<IVec3, ChunkMembers>, into_wires: bool) {
    for rec in scratch.chunks_exact(INSTANCE_FLOATS) {
        let world = base + Vec3::new(f64::from(rec[0]), f64::from(rec[1]), f64::from(rec[2]));
        let c = chunk_of(world.floor());
        let origin = (c * CHUNK_SIZE).as_vec3();
        let m = chunks.entry(c).or_default();
        let dst = if into_wires { &mut m.wire_boxes } else { &mut m.span_boxes };
        let start = dst.len();
        dst.extend_from_slice(rec);
        dst[start] = (world.x - origin.x) as f32;
        dst[start + 1] = (world.y - origin.y) as f32;
        dst[start + 2] = (world.z - origin.z) as f32;
    }
}

impl Factory {
    /// Emits all static rail tracks and fibre lines into chunk-local `span_boxes` with full `f64` precision.
    fn emit_tracks_and_fibre(&self, scratch: &mut Vec<f32>, chunks: &mut FxHashMap<IVec3, ChunkMembers>) {
        for t in &self.tracks {
            if let Some(curve) = self.curve_of(t) {
                let base = (chunk_of(t.a) * CHUNK_SIZE).as_vec3();
                scratch.clear();
                write_track(scratch, &curve, base, f64::INFINITY);
                bin_boxes(scratch, base, chunks, false);
            }
        }
        let top = |pos: IVec3, h: f64| pos.as_vec3() + Vec3::new(0.5, h, 0.5);
        let mut emit_fibre = |a: Vec3, b: Vec3| {
            let base = (chunk_of(a.floor()) * CHUNK_SIZE).as_vec3();
            scratch.clear();
            wire(scratch, a - base, b - base, crate::block::tex::FIBRE);
            bin_boxes(scratch, base, chunks, false);
        };
        for &(i, j) in &self.data.links {
            emit_fibre(top(self.nodes[i as usize].pos, 0.9), top(self.nodes[j as usize].pos, 0.9));
        }
        for (p, node) in self.processors.iter().zip(&self.data.process_node) {
            let Some(node) = *node else { continue };
            let at = self.nodes[node as usize].pos;
            let dist2 = |c: &IVec3| {
                let d = at - *c;
                d.x * d.x + d.y * d.y + d.z * d.z
            };
            let cell = p.cells().into_iter().min_by_key(dist2).unwrap_or(p.pos);
            emit_fibre(top(at, 0.9), top(cell, 0.7));
        }
    }

    /// Writes all static factory boxes for one chunk `m` relative to `origin` (`chunk * 32`).
    pub(crate) fn write_chunk_static(&self, m: &ChunkMembers, origin: Vec3, out: &mut Vec<f32>) {
        let rel = |p: IVec3| p.as_vec3() + Vec3::new(0.5, 0.5, 0.5) - origin;
        for &i in &m.belts {
            let b = &self.belts[i as usize];
            b.frame_static(out, rel(b.pos));
        }
        for &i in &m.poles {
            let p = &self.poles[i as usize];
            let r = rel(p.pos);
            if p.is_cable() {
                self.write_cable_one(out, i as usize, r);
            } else {
                p.write_static(out, r);
                self.write_pole_accent(out, i as usize, r);
            }
        }
        for &i in &m.processors {
            let p = &self.processors[i as usize];
            p.write_static(out, rel(p.pos));
        }
        for &i in &m.storages {
            let s = &self.storages[i as usize];
            s.write_static(out, rel(s.pos));
        }
        for &i in &m.routers {
            let r = &self.routers[i as usize];
            router_model::draw_static(r, out, rel(r.pos));
        }
        for &i in &m.pipework {
            let p = &self.pipework[i as usize];
            p.write_static(out, rel(p.pos));
        }
        for &i in &m.rails {
            let r = &self.rails[i as usize];
            r.model(out, rel(r.pos), 0.0);
        }
        for &i in &m.nodes {
            let n = &self.nodes[i as usize];
            n.model(out, rel(n.pos), 0.0);
        }
        out.extend_from_slice(&m.span_boxes);
    }

    /// Writes only the dynamic and near-camera factory box instances within `range` of `eye` and inside `cone`, using
    /// `index` to skip out-of-range or unseen chunks and static-only machines.
    pub(crate) fn write_dynamic_instances(
        &self,
        index: &ChunkIndex,
        out: &mut Vec<f32>,
        eye: Vec3,
        time: f64,
        range: f64,
        cone: &ViewCone,
    ) {
        let range2 = range * range;
        let chunk_max2 = (range + CHUNK_HALF_DIAG).powi(2);
        let near_max2 = (tier_mark::DETAIL + CHUNK_HALF_DIAG).powi(2);
        let half = f64::from(CHUNK_SIZE) * 0.5;
        for (&c, m) in &index.chunks {
            let mid = (c * CHUNK_SIZE).as_vec3() + Vec3::new(half, half, half) - eye;
            let dist2 = mid.x * mid.x + mid.y * mid.y + mid.z * mid.z;
            if dist2 > chunk_max2 || !cone.sees(mid, CHUNK_HALF_DIAG + MODEL_REACH) {
                continue;
            }
            for &i in &m.belts {
                let b = &self.belts[i as usize];
                if b.items.is_empty() {
                    continue;
                }
                let rel = b.pos.as_vec3() + Vec3::new(0.5, 0.5, 0.5) - eye;
                if rel.x * rel.x + rel.y * rel.y + rel.z * rel.z <= range2 && cone.sees(rel, BELT_REACH) {
                    b.write_items(out, rel);
                }
            }
            if dist2 <= near_max2 {
                for &i in &m.stud_belts {
                    let b = &self.belts[i as usize];
                    b.write_studs(out, b.pos.as_vec3() + Vec3::new(0.5, 0.5, 0.5) - eye);
                }
                for &i in &m.collar_poles {
                    let p = &self.poles[i as usize];
                    p.write_dynamic(out, p.pos.as_vec3() + Vec3::new(0.5, 0.5, 0.5) - eye);
                }
                for &i in &m.routers {
                    let r = &self.routers[i as usize];
                    router_model::draw_dynamic(r, out, r.pos.as_vec3() + Vec3::new(0.5, 0.5, 0.5) - eye, time);
                }
            }
            for &i in &m.processors {
                let p = &self.processors[i as usize];
                let rel = p.pos.as_vec3() + Vec3::new(0.5, 0.5, 0.5) - eye;
                if rel.x * rel.x + rel.y * rel.y + rel.z * rel.z <= range2 {
                    p.write_dynamic(out, rel, time);
                }
            }
            for &i in &m.storages {
                let s = &self.storages[i as usize];
                let rel = s.pos.as_vec3() + Vec3::new(0.5, 0.5, 0.5) - eye;
                if rel.x * rel.x + rel.y * rel.y + rel.z * rel.z <= range2 {
                    s.write_dynamic(out, rel);
                }
            }
            for &i in &m.dynamic_pipes {
                let p = &self.pipework[i as usize];
                let rel = p.pos.as_vec3() + Vec3::new(0.5, 0.5, 0.5) - eye;
                if rel.x * rel.x + rel.y * rel.y + rel.z * rel.z <= range2 {
                    p.write_dynamic(out, rel, time);
                }
            }
        }
        models_in_view(&self.miners, out, eye, time, range, cone);
        models_in_view(&self.generators, out, eye, time, range, cone);
        models_in_view(&self.labs, out, eye, time, range, cone);
        // Not culled: a quarry's drill hangs down to the depth it digs, far past `MODEL_REACH`; quarries are few.
        models(&self.quarries, out, eye, time, range);
        models_in_view(&self.sensors, out, eye, time, range, cone);
        self.write_beams(out, eye, time, range);
        self.write_train_models(out, eye, time, range);
    }
}
