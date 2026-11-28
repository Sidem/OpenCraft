//! Smoke emitters and stream-history tracking for chimneys, exhausts, locomotives, burning trees, rocket pads and
//! orbital bursts (presentation only, never saved).
//!
//! Each active or winding-down smoke source is tracked in [`SmokeTrack`] with a 32-bit sliding ON/OFF stream bitmask
//! synchronized to the GPU plume clock (`phase`): turning a machine on starts smoke at the chimney lip and rises
//! smoothly; shutting it off (or breaking it) cuts off the mouth while the detached cloud finishes flowing upward and
//! downwind; rapid on/off cycling produces discrete rising packets instead of flickering. Each visible emitter packs
//! into [`SMOKE_FLOATS`] (8) `u32` words (32 bytes) expanded on the GPU (`web/src/render/smoke.ts`).

use rustc_hash::FxHashMap;

use crate::chunk::CHUNK_SIZE;
use crate::math::{hash3, sort_small_by_key, unit, IVec3, Vec3};
use crate::view_cone::ViewCone;
use crate::world::DAYLIGHT;

use super::miner::MinerStatus;
use super::miner_model;
use super::static_view::ChunkIndex;
use super::{generator, Factory};

#[cfg(test)]
mod tests;

/// 32-bit words per GPU smoke emitter record: camera-relative xyz (f32), scale (f32), seed (f32), intensity or burst
/// age (f32), packed `light + kind * 256` (f32), and 32-bit stream history mask (u32).
pub const SMOKE_FLOATS: usize = 8;
/// At most this many active smoke emitters drawn in a frame (nearest first when exceeded).
pub const MAX_EMITTERS: usize = 512;
/// Standard chimney/exhaust smoke emitters further than this from the eye (blocks) are skipped.
pub const SMOKE_RANGE: f64 = 160.0;
/// Generous view-cone margin (blocks) so a downwind plume stays visible after its chimney leaves the viewport edge.
const SMOKE_REACH: f64 = 18.0;
/// Half the diagonal of a 32³ chunk plus overhang for coarse chunk-sphere culling.
const CHUNK_HALF_DIAG: f64 = 30.0;
/// Exact common period (in plume cycles) where `phase * rate_mul * 31.0` is an integer for every archetype.
const PHASE_WRAP: f64 = 100.0;
/// History slices per plume lifetime (bit 0 is live mouth state, bits 1..=31 are past slices).
const HISTORY_SLICES: f64 = 31.0;
const HISTORY_MASK: u32 = 0x7FFF_FFFF;

/// Smoke archetypes decoded by `web/src/render/smoke-glsl.ts`.
pub const KIND_CHIMNEY: f32 = 0.0;
pub const KIND_FIRE: f32 = 1.0;
pub const KIND_LOCO: f32 = 2.0;
pub const KIND_ROCKET: f32 = 3.0;
pub const KIND_BURST: f32 = 4.0;

/// Source key namespace tags for [`source_key`].
pub const TAG_PROCESSOR: u8 = 0;
pub const TAG_ROCKET: u8 = 1;
pub const TAG_GENERATOR: u8 = 2;
pub const TAG_MINER: u8 = 3;
pub const TAG_LOCO: u8 = 4;
pub const TAG_FIRE: u8 = 5;
pub const TAG_TOWN: u8 = 6;

/// Rate multiplier per archetype (must match `rateMul` in `web/src/render/smoke-glsl.ts`).
#[inline]
fn rate_mul(kind: f32) -> f64 {
    if kind > 2.5 {
        1.45
    } else if kind > 1.5 {
        1.28
    } else if kind > 0.5 {
        0.88
    } else {
        1.0
    }
}

/// Stable 64-bit identity for a continuous smoke emitter (`tag`, world anchor `pos`, sub-index `sub`).
pub fn source_key(tag: u8, pos: IVec3, sub: u8) -> u64 {
    let mut h = 0xcbf2_9ce4_8422_2325u64;
    let mix = |h: &mut u64, v: u64| *h = (*h ^ v).wrapping_mul(0x0000_0100_0000_01b3);
    mix(&mut h, u64::from(tag) | (u64::from(sub) << 8));
    mix(&mut h, pos.x as u32 as u64 | ((pos.y as u32 as u64) << 32));
    mix(&mut h, pos.z as u32 as u64);
    h
}

/// A seed in 0..1 for the source at `pos`, to stagger neighbouring plumes.
pub fn seed(pos: IVec3) -> f64 {
    unit(hash3(17, pos.x, pos.y, pos.z))
}

struct Stream {
    world_pos: Vec3,
    scale: f32,
    seed: f32,
    intensity: f32,
    kind: f32,
    /// Past 31 completed step-slices in bits `0..=30` (bit 0 = most recently completed slice).
    history: u32,
    /// Latch: whether this source was active at any point during the current in-progress slice.
    saw_on: bool,
    last_step: u64,
    active_frame: u64,
}

/// Presentation-only tracker for continuous smoke streams and this frame's packed GPU emitter buffer.
pub(crate) struct SmokeTrack {
    entries: FxHashMap<u64, Stream>,
    wind_speed: f32,
    pub wind_motion: bool,
    phase: f64,
    frame: u64,
    pub out: Vec<u32>,
}

impl Default for SmokeTrack {
    fn default() -> Self {
        Self {
            entries: FxHashMap::default(),
            wind_speed: 2.0,
            wind_motion: true,
            phase: 0.0,
            frame: 0,
            out: Vec::new(),
        }
    }
}

impl SmokeTrack {
    /// Advances the shared plume clock for `dt` seconds at local surface wind speed `wind_ms` (m/s) and clears the
    /// per-frame GPU output buffer.
    pub fn begin_frame(&mut self, dt: f64, wind_ms: f32) {
        self.out.clear();
        self.frame = self.frame.wrapping_add(1);
        let dt = dt.clamp(0.0, 0.1);
        let target = if self.wind_motion { wind_ms.max(0.0) } else { 0.0 };
        let ease = (1.0 - (-dt * 2.2).exp()) as f32;
        self.wind_speed += (target - self.wind_speed) * ease;
        let life = (5.4 / (1.0 + 0.085 * f64::from(self.wind_speed))).max(2.3);
        self.phase += dt / life;
    }

    /// Wrapped plume phase (in `0..100`) sent to the GPU shader so bitmask slices and puff ages stay locked in phase.
    pub fn gpu_phase(&self) -> f32 {
        (self.phase % PHASE_WRAP) as f32
    }

    /// Marks the continuous smoke source `key` as actively emitting this frame at `world_pos`.
    pub fn record(&mut self, key: u64, world_pos: Vec3, scale: f32, seed: f64, intensity: f32, kind: f32) {
        if scale <= 0.0 || intensity <= 0.0 {
            return;
        }
        let step = self.step_of(kind);
        let seed_f = seed.rem_euclid(1.0) as f32;
        match self.entries.get_mut(&key) {
            Some(s) => {
                s.world_pos = world_pos;
                s.scale = scale;
                s.intensity = intensity;
                s.kind = kind;
                s.active_frame = self.frame;
            }
            None => {
                self.entries.insert(
                    key,
                    Stream {
                        world_pos,
                        scale,
                        seed: seed_f,
                        intensity,
                        kind,
                        history: 0,
                        saw_on: true,
                        last_step: step,
                        active_frame: self.frame,
                    },
                );
            }
        }
    }

    /// Appends a one-shot radial burst (`orbit_fx.rs`, `KIND_BURST`), which already carries its own explicit age.
    pub fn push_burst(&mut self, rel: Vec3, scale: f32, seed: f64, age: f32) {
        if scale <= 0.0 || age <= 0.0 {
            return;
        }
        push_raw(&mut self.out, rel, scale, seed.rem_euclid(1.0) as f32, age, DAYLIGHT, KIND_BURST, u32::MAX);
    }

    /// Advances every tracked stream's 32-bit history, prunes finished tails, frustum-culls, lights and caps output.
    pub fn finish(
        &mut self,
        eye: Vec3,
        max_range: f64,
        cone: &ViewCone,
        mut light: impl FnMut(IVec3) -> u8,
        mut visible: impl FnMut(Vec3) -> bool,
    ) {
        let smoke_range2 = max_range.min(SMOKE_RANGE).powi(2);
        let rocket_range2 = max_range * max_range;
        let phase = self.phase;
        let frame = self.frame;
        let out = &mut self.out;

        // Light any one-shot bursts already appended this frame.
        light_packed(out, eye, &mut light);

        let mut last_light: Option<(IVec3, u8)> = None;
        self.entries.retain(|_, s| {
            let active_now = s.active_frame == frame;
            let cur_step = (phase * rate_mul(s.kind) * HISTORY_SLICES).floor() as u64;
            let delta = cur_step.saturating_sub(s.last_step);
            if delta > 0 {
                s.history = ((s.history << 1) | u32::from(s.saw_on)) & HISTORY_MASK;
                if delta > 1 {
                    let extra = (delta - 1).min(31) as u32;
                    if s.saw_on && active_now {
                        let fill = if extra >= 31 { HISTORY_MASK } else { (1u32 << extra) - 1 };
                        s.history = ((s.history << extra) | fill) & HISTORY_MASK;
                    } else {
                        s.history = if extra >= 31 { 0 } else { (s.history << extra) & HISTORY_MASK };
                    }
                }
                s.saw_on = active_now;
                s.last_step = cur_step;
            } else if active_now {
                s.saw_on = true;
            }
            let mask = ((s.history << 1) & !1u32) | u32::from(active_now);
            if mask == 0 {
                return false;
            }
            let rel = s.world_pos - eye;
            let d2 = rel.x * rel.x + rel.y * rel.y + rel.z * rel.z;
            let lim2 = if s.kind > 2.5 { rocket_range2 } else { smoke_range2 };
            if d2 <= lim2 && cone.sees(rel, SMOKE_REACH) && visible(s.world_pos) {
                let lit = read_light(s.world_pos.floor(), &mut last_light, &mut light);
                push_raw(out, rel, s.scale, s.seed, s.intensity, lit, s.kind, mask);
            }
            true
        });
        cap_emitters(out);
    }

    #[inline]
    fn step_of(&self, kind: f32) -> u64 {
        (self.phase * rate_mul(kind) * HISTORY_SLICES).floor() as u64
    }
}

/// Pushes one packed 8-word (32-byte) GPU smoke emitter record.
#[allow(clippy::too_many_arguments)]
pub fn push_raw(out: &mut Vec<u32>, rel: Vec3, scale: f32, seed: f32, intensity: f32, light: u8, kind: f32, mask: u32) {
    let lk = f32::from(light) + kind * 256.0;
    out.extend_from_slice(&[
        (rel.x as f32).to_bits(),
        (rel.y as f32).to_bits(),
        (rel.z as f32).to_bits(),
        scale.to_bits(),
        seed.to_bits(),
        intensity.to_bits(),
        lk.to_bits(),
        mask,
    ]);
}

/// Reads light at `cell`, borrowing brightest neighbour if inside a solid block, caching repeated lookups.
fn read_light(cell: IVec3, last: &mut Option<(IVec3, u8)>, light: &mut impl FnMut(IVec3) -> u8) -> u8 {
    const AROUND: [(i32, i32, i32); 6] = [(0, 1, 0), (1, 0, 0), (-1, 0, 0), (0, 0, 1), (0, 0, -1), (0, -1, 0)];
    if let Some((c, l)) = *last {
        if c == cell {
            return l;
        }
    }
    let lit = match light(cell) {
        0 => AROUND.iter().map(|&(x, y, z)| light(cell + IVec3::new(x, y, z))).fold(0, brighter),
        l => l,
    };
    *last = Some((cell, lit));
    lit
}

/// Updates the packed light byte in existing records (used for one-shot bursts before streams are emitted).
fn light_packed(emitters: &mut [u32], eye: Vec3, light: &mut impl FnMut(IVec3) -> u8) {
    let mut last: Option<(IVec3, u8)> = None;
    for rec in emitters.chunks_exact_mut(SMOKE_FLOATS) {
        let rel = Vec3::new(
            f64::from(f32::from_bits(rec[0])),
            f64::from(f32::from_bits(rec[1])),
            f64::from(f32::from_bits(rec[2])),
        );
        let lit = read_light((eye + rel).floor(), &mut last, light);
        let kind = (f32::from_bits(rec[6]) / 256.0).floor();
        rec[6] = (f32::from(lit) + kind * 256.0).to_bits();
    }
}

/// Keeps at most [`MAX_EMITTERS`] nearest emitters when a huge scene exceeds the budget.
pub fn cap_emitters(emitters: &mut Vec<u32>) {
    let count = emitters.len() / SMOKE_FLOATS;
    if count <= MAX_EMITTERS {
        return;
    }
    let mut order: Vec<(u32, usize)> = (0..count)
        .map(|i| {
            let base = i * SMOKE_FLOATS;
            let (x, y, z) = (
                f32::from_bits(emitters[base]),
                f32::from_bits(emitters[base + 1]),
                f32::from_bits(emitters[base + 2]),
            );
            let d2 = x * x + y * y + z * z;
            ((d2.min(1.0e9) * 4.0) as u32, i)
        })
        .collect();
    sort_small_by_key(&mut order, |e| e.0);
    order.truncate(MAX_EMITTERS);
    let mut kept = Vec::with_capacity(MAX_EMITTERS * SMOKE_FLOATS);
    for &(_, idx) in &order {
        let base = idx * SMOKE_FLOATS;
        kept.extend_from_slice(&emitters[base..base + SMOKE_FLOATS]);
    }
    *emitters = kept;
}

/// The brighter of two light bytes, sky and block light each.
fn brighter(a: u8, b: u8) -> u8 {
    (a & 15).max(b & 15) | (a >> 4).max(b >> 4) << 4
}

impl Factory {
    /// Records all active smoking factory machines within `range` of `eye` into `track` (without view-cone culling so
    /// turning away and back never resets an active chimney's stream history).
    pub(crate) fn record_smoke(&self, index: &ChunkIndex, track: &mut SmokeTrack, eye: Vec3, time: f64, range: f64) {
        let smoke_range = range.min(SMOKE_RANGE);
        let range2 = smoke_range * smoke_range;
        let rocket_range2 = range * range;
        let chunk_max2 = (range + CHUNK_HALF_DIAG).powi(2);
        let half = f64::from(CHUNK_SIZE) * 0.5;

        for (&c, m) in &index.chunks {
            if m.smoke_processors.is_empty() {
                continue;
            }
            let mid = (c * CHUNK_SIZE).as_vec3() + Vec3::new(half, half, half) - eye;
            if mid.x * mid.x + mid.y * mid.y + mid.z * mid.z > chunk_max2 {
                continue;
            }
            for &i in &m.smoke_processors {
                let p = &self.processors[i as usize];
                let world_centre = p.pos.as_vec3() + Vec3::new(0.5, 0.5, 0.5);
                let rel = world_centre - eye;
                let d2 = rel.x * rel.x + rel.y * rel.y + rel.z * rel.z;
                if d2 <= rocket_range2 {
                    p.record_smoke(track, world_centre, time, d2 <= range2);
                }
            }
        }
        for g in &self.generators {
            if !g.running() {
                continue;
            }
            let world_centre = g.pos.as_vec3() + Vec3::new(0.5, 0.5, 0.5);
            let rel = world_centre - eye;
            if rel.x * rel.x + rel.y * rel.y + rel.z * rel.z <= range2 {
                track.record(
                    source_key(TAG_GENERATOR, g.pos, 0),
                    world_centre + generator::CHIMNEY_MOUTH,
                    generator::SMOKE_SCALE,
                    seed(g.pos),
                    1.0,
                    KIND_CHIMNEY,
                );
            }
        }
        for m in &self.miners {
            if !m.is_coal() || m.status != MinerStatus::Running || m.draw_rate <= 0.01 {
                continue;
            }
            let world_centre = m.pos.as_vec3() + Vec3::new(0.5, 0.5, 0.5);
            let rel = world_centre - eye;
            if rel.x * rel.x + rel.y * rel.y + rel.z * rel.z <= range2 {
                let mouth = miner_model::chimney_mouth(m.drill, world_centre);
                track.record(
                    source_key(TAG_MINER, m.pos, 0),
                    mouth,
                    miner_model::SMOKE_SCALE,
                    seed(m.pos),
                    1.0,
                    KIND_CHIMNEY,
                );
            }
        }
        self.record_train_smoke(track, eye, smoke_range);
    }
}
