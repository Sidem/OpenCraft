//! Seismic prospecting query (F-SEIS): computes a 64-block-wide vertical cross-section from the surface down to
//! bedrock along a placed Seismic Thumper's facing (`factory/process/thumper.rs`).
//!
//! Invariants: pure presentation query — never mutates core state, never creates deposit tracking records and never
//! moves `Sim::state_hash`. Reads true world blocks via `World::block_anywhere_or_generate`, distinguishes player-dug
//! tunnels and structures via `scan.rs`, and applies tier-scaled spatial blur and coherent seismic noise (`Mk1` →
//! `Mk2` → `Mk3`). To tune resolution: adjust `TIER_PARAMS` below.

use rustc_hash::FxHashMap;

use crate::block::{self, BlockId, AIR};
use crate::chunk::Chunk;
use crate::factory::{ThumperInfo, DIRS};
use crate::math::{hash3, unit, IVec3};
use crate::prospect::FILTER_ORES;
use crate::worldgen::{SEA_LEVEL, WORLD_HEIGHT};
use crate::Game;

mod scan;
#[cfg(test)]
mod tests;
mod volume;

use scan::{classify_block, detect_anomalies, mask_for_tier};
pub use volume::{VolumeReading, VOL_SIZE, VOL_VOXEL_BYTES};

/// Cross-section width in blocks (`s ∈ [-SLICE_HALF, SLICE_HALF)` along `DIRS[dir]`).
pub const SLICE_WIDTH: usize = 64;
pub const SLICE_HALF: i32 = (SLICE_WIDTH / 2) as i32;
/// Bytes per cell in [`SeismicReading::cells`]: `[impedance, gradient, class, detail]`.
pub const CELL_BYTES: usize = 4;
/// Fields per detected highlight in [`SeismicReading::anomalies`]: `[class, s_index, y, strength_0_100, detail, span]`.
pub const ANOMALY_FIELDS: usize = 6;
/// Sky blocks kept above the highest ground surface along the slice.
const SKY_HEADROOM: i32 = 5;

/// Signal class byte (`cells[i * 4 + 2]`).
pub const CLASS_SKY: u8 = 0;
pub const CLASS_ROCK: u8 = 1;
pub const CLASS_SOIL: u8 = 2;
pub const CLASS_CAVE: u8 = 3;
pub const CLASS_TUNNEL: u8 = 4;
pub const CLASS_WATER: u8 = 5;
pub const CLASS_STRUCTURE: u8 = 6;
pub const CLASS_BEDROCK: u8 = 7;
pub const CLASS_ORE_UNKNOWN: u8 = 8;
pub const CLASS_ORE_METAL: u8 = 9;
pub const CLASS_ORE_CARBON: u8 = 10;
pub const CLASS_ORE_MINERAL: u8 = 11;

struct TierParams {
    blur_x: i32,
    blur_y: i32,
    noise_amp: f32,
}

/// Mk1: rugged blurred seismogram; Mk2: tight phased array; Mk3: crisp tomography.
const TIER_PARAMS: [TierParams; 3] = [
    TierParams { blur_x: 2, blur_y: 1, noise_amp: 0.16 },
    TierParams { blur_x: 1, blur_y: 1, noise_amp: 0.062 },
    TierParams { blur_x: 0, blur_y: 0, noise_amp: 0.02 },
];

/// Presentation-only cache of the latest seismic 2D slice and 3D volume computed for the UI (`Game::seismic`).
#[derive(Default)]
pub struct SeismicView {
    pub reading: Option<SeismicReading>,
    pub volume: Option<VolumeReading>,
}

/// Complete seismic cross-section computed on demand for one thumper pulse.
pub struct SeismicReading {
    pub top_y: i32,
    pub surface_at_thumper: i32,
    pub dir: u8,
    pub perp_offset: i32,
    pub player_s: i32,
    pub player_perp: i32,
    pub surface: [i32; SLICE_WIDTH],
    pub cells: Vec<u8>,
    pub anomalies: Vec<i32>,
}

/// Metadata header for a placed thumper (`api/seismic.rs`).
pub struct SeismicHeader {
    pub info: ThumperInfo,
    pub top_y: i32,
    pub surface_at_thumper: i32,
    pub player_s: i32,
    pub player_y: i32,
    pub player_perp: i32,
    pub snr_percent: i32,
}

/// Maps a 1-based `detail` ore id back to its `BlockId` (`0` if invalid/unknown).
pub fn ore_block_of(detail: u8) -> BlockId {
    detail.checked_sub(1).and_then(|i| FILTER_ORES.get(i as usize).copied()).unwrap_or(AIR)
}

impl Game {
    /// Header metadata for a placed Seismic Thumper at `cell` (any cell of its 2×2×2 footprint).
    pub(crate) fn seismic_header(&mut self, cell: IVec3) -> Option<SeismicHeader> {
        let info = self.sim.factory.thumper_at(cell)?;
        let (surface, top_y) = self.seismic_surface_heights(info);
        let axis = DIRS[info.dir as usize];
        let perp = DIRS[(info.dir as usize + 1) % 4];
        let p = self.body().pos.floor();
        let rel = p - info.anchor;
        let player_s = rel.x * axis.x + rel.z * axis.z;
        let player_perp = (rel.x * perp.x + rel.z * perp.z).abs();
        let snr_percent = snr_for(info.tier, info.power_share);
        Some(SeismicHeader {
            info,
            top_y,
            surface_at_thumper: surface[SLICE_HALF as usize],
            player_s,
            player_y: p.y,
            player_perp,
            snr_percent,
        })
    }

    /// Computes and caches a seismic cross-section for the thumper at `cell`, shifted `perp_offset` blocks
    /// perpendicular (`-32..=31`) and optionally rotated 90° (`flip_axis`). Returns whether a thumper was found.
    pub(crate) fn seismic_pulse_at(&mut self, cell: IVec3, pulse_seed: u32, perp_offset: i32, flip_axis: bool) -> bool {
        let reading = self.seismic_scan_slice(cell, pulse_seed, perp_offset, flip_axis);
        let ok = reading.is_some();
        self.seismic.reading = reading;
        ok
    }

    /// Computes a full 64 × (`top_y` + 1) vertical seismic cross-section through the thumper at `cell`.
    #[cfg(test)]
    pub(crate) fn seismic_scan(&mut self, cell: IVec3, pulse_seed: u32) -> Option<SeismicReading> {
        self.seismic_scan_slice(cell, pulse_seed, 0, false)
    }

    /// Computes a 64 × (`top_y` + 1) vertical seismic cross-section shifted `perp_offset` blocks perpendicular to the
    /// active axis (`dir`, or rotated 90° when `flip_axis` is true).
    pub(crate) fn seismic_scan_slice(
        &mut self,
        cell: IVec3,
        pulse_seed: u32,
        perp_offset: i32,
        flip_axis: bool,
    ) -> Option<SeismicReading> {
        let info = self.sim.factory.thumper_at(cell)?;
        let dir = if flip_axis { (info.dir + 1) % 4 } else { info.dir };
        let perp_off = perp_offset.clamp(-SLICE_HALF, SLICE_HALF - 1);
        let axis = DIRS[dir as usize];
        let perp = DIRS[(dir as usize + 1) % 4];
        let origin = IVec3::new(info.anchor.x + perp.x * perp_off, info.anchor.y, info.anchor.z + perp.z * perp_off);

        let (surface, top_y) = self.seismic_surface_heights_at(origin, dir);
        let h = (top_y + 1) as usize;
        let n = SLICE_WIDTH * h;

        let mut raw_imp = vec![0.0f32; n];
        let mut raw_cls = vec![CLASS_SKY; n];
        let mut raw_det = vec![0u8; n];
        let mut orig_cache: FxHashMap<IVec3, Chunk> = FxHashMap::default();

        for (s, &surf) in surface.iter().enumerate() {
            let offset = s as i32 - SLICE_HALF;
            let wx = origin.x + axis.x * offset;
            let wz = origin.z + axis.z * offset;
            for y in 0..=top_y {
                let idx = y as usize * SLICE_WIDTH + s;
                let p = IVec3::new(wx, y, wz);
                let b = self.sim.world.block_anywhere_or_generate(p);
                if y > surf && (b == AIR || block::is_tree(b) || block::is_sapling(b)) {
                    continue;
                }
                let (imp, cls, det) =
                    classify_block(b, p, surf, &self.sim.factory, &mut self.sim.world, &mut orig_cache);
                raw_imp[idx] = imp;
                raw_cls[idx] = cls;
                raw_det[idx] = det;
            }
        }

        let params = &TIER_PARAMS[usize::from(info.tier).min(TIER_PARAMS.len() - 1)];
        let power_frac = (info.power_share as f32 / 1000.0).clamp(0.0, 1.0);
        let brownout = (1.0 - power_frac) * 0.26;
        let effective_tier = if info.power_share < 500 { info.tier.saturating_sub(1) } else { info.tier };

        // Step 1: spatial blur and deterministic seismic speckle over subsurface cells.
        let mut blurred = vec![0.0f32; n];
        for y in 0..h {
            for (s, &surf_y) in surface.iter().enumerate() {
                let idx = y * SLICE_WIDTH + s;
                if raw_cls[idx] == CLASS_SKY {
                    continue;
                }
                if raw_cls[idx] == CLASS_BEDROCK {
                    blurred[idx] = 1.0;
                    continue;
                }
                let v = blur_cell(&raw_imp, &raw_cls, s, y, h, params);
                let depth_frac = ((surf_y - y as i32).max(0) as f32 / (top_y.max(1) as f32)).clamp(0.0, 1.0);
                let amp = (params.noise_amp + brownout) * (1.0 + 0.35 * depth_frac);
                let coarse = signed_noise(pulse_seed ^ 0x5E15, s as i32 / 2, y as i32 / 2, origin.x ^ perp_off);
                let fine = signed_noise(pulse_seed ^ 0xA91C, s as i32, y as i32, origin.z ^ perp_off);
                blurred[idx] = (v + (coarse * 0.65 + fine * 0.35) * amp).clamp(0.02, 0.99);
            }
        }

        // Step 2: pack `[impedance, gradient, tier_class, tier_detail]`.
        let mut cells = vec![0u8; n * CELL_BYTES];
        for y in 0..h {
            for s in 0..SLICE_WIDTH {
                let idx = y * SLICE_WIDTH + s;
                let base = idx * CELL_BYTES;
                if raw_cls[idx] == CLASS_SKY {
                    cells[base + 1] = 128;
                    continue;
                }
                let imp = blurred[idx];
                let above = if y + 1 < h { blurred[(y + 1) * SLICE_WIDTH + s] } else { 0.0 };
                let grad = ((imp - above) * 2.4).clamp(-1.0, 1.0);
                let (cls, det) = mask_for_tier(raw_cls[idx], raw_det[idx], imp, effective_tier);
                cells[base] = (imp * 255.0).round() as u8;
                cells[base + 1] = ((grad * 127.0) + 128.0).round().clamp(0.0, 255.0) as u8;
                cells[base + 2] = cls;
                cells[base + 3] = det;
            }
        }

        let anomalies = detect_anomalies(&cells, &surface, h);
        let p = self.body().pos.floor();
        let rel = p - origin;
        let player_s = rel.x * axis.x + rel.z * axis.z;
        let player_perp = (rel.x * perp.x + rel.z * perp.z).abs();
        Some(SeismicReading {
            top_y,
            surface_at_thumper: surface[SLICE_HALF as usize],
            dir,
            perp_offset: perp_off,
            player_s,
            player_perp,
            surface,
            cells,
            anomalies,
        })
    }

    fn seismic_surface_heights(&mut self, info: ThumperInfo) -> ([i32; SLICE_WIDTH], i32) {
        self.seismic_surface_heights_at(info.anchor, info.dir)
    }

    fn seismic_surface_heights_at(&mut self, origin: IVec3, dir: u8) -> ([i32; SLICE_WIDTH], i32) {
        let axis = DIRS[dir as usize];
        let mut surface = [SEA_LEVEL; SLICE_WIDTH];
        let mut max_y = origin.y + 2;
        for (s, slot) in surface.iter_mut().enumerate() {
            let offset = s as i32 - SLICE_HALF;
            let h = self.column_surface_y(origin.x + axis.x * offset, origin.z + axis.z * offset, origin.y);
            *slot = h;
            if h > max_y {
                max_y = h;
            }
        }
        (surface, (max_y + SKY_HEADROOM).min(WORLD_HEIGHT - 1))
    }

    pub(super) fn column_surface_y(&mut self, wx: i32, wz: i32, hint_y: i32) -> i32 {
        let est = self.sim.world.generator().height_at(wx, wz).max(SEA_LEVEL).max(hint_y);
        let start = (est + 18).min(WORLD_HEIGHT - 1);
        for y in (1..=start).rev() {
            let b = self.sim.world.block_anywhere_or_generate(IVec3::new(wx, y, wz));
            if b != AIR
                && !block::is_tree(b)
                && !block::is_sapling(b)
                && self.sim.factory.anchor_of(IVec3::new(wx, y, wz)).is_none()
            {
                return y;
            }
        }
        1
    }
}

fn blur_cell(raw_imp: &[f32], raw_cls: &[u8], s: usize, y: usize, h: usize, params: &TierParams) -> f32 {
    let idx = y * SLICE_WIDTH + s;
    if params.blur_x == 0 && params.blur_y == 0 {
        let c = raw_imp[idx];
        let l = if s > 0 && raw_cls[idx - 1] != CLASS_SKY { raw_imp[idx - 1] } else { c };
        let r = if s + 1 < SLICE_WIDTH && raw_cls[idx + 1] != CLASS_SKY { raw_imp[idx + 1] } else { c };
        return c * 0.88 + (l + r) * 0.06;
    }
    let (mut sum, mut wsum) = (0.0f32, 0.0f32);
    for dy in -params.blur_y..=params.blur_y {
        let ny = y as i32 + dy;
        if ny < 0 || ny >= h as i32 {
            continue;
        }
        for dx in -params.blur_x..=params.blur_x {
            let nx = s as i32 + dx;
            if nx < 0 || nx >= SLICE_WIDTH as i32 {
                continue;
            }
            let nidx = ny as usize * SLICE_WIDTH + nx as usize;
            if raw_cls[nidx] == CLASS_SKY || raw_cls[nidx] == CLASS_BEDROCK {
                continue;
            }
            let w = 1.0 / (1.0 + (dx * dx + dy * dy) as f32 * 0.85);
            sum += raw_imp[nidx] * w;
            wsum += w;
        }
    }
    if wsum > 0.0 {
        sum / wsum
    } else {
        raw_imp[idx]
    }
}

fn snr_for(tier: u8, power_share: u32) -> i32 {
    if power_share == 0 {
        return 0;
    }
    let base = match tier {
        0 => 64.0,
        1 => 86.0,
        _ => 97.0,
    };
    let p = (power_share as f32 / 1000.0).clamp(0.0, 1.0);
    (base * (0.35 + 0.65 * p)).round() as i32
}

fn signed_noise(seed: u32, x: i32, y: i32, z: i32) -> f32 {
    (unit(hash3(seed, x, y, z)) as f32) * 2.0 - 1.0
}
