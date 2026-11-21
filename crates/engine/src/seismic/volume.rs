//! 3D subsurface volume scanner for Seismic Prospecting (F-SEIS): samples a 48×48×H block volume centred on a
//! Seismic Thumper and extracts a compact, boundary-culled 3D feature voxel cloud (ores, caves, player tunnels,
//! aquifers and buried structures) plus the 48×48 terrain surface heightmap for `web/src/ui/seismic-volume.ts`.
//!
//! Invariants: pure read-only presentation query — never mutates core state or `Sim::state_hash`. Mk1 uses coarser
//! 2-block voxel clusters with subtle speckle and unidentified dense anomalies; Mk2 separates player tunnels,
//! structures and mineral families at 1-block resolution; Mk3 resolves exact ore identities.

use rustc_hash::FxHashMap;

use crate::block::{self, AIR};
use crate::chunk::Chunk;
use crate::factory::ThumperInfo;
use crate::math::{hash3, unit, IVec3};
use crate::worldgen::{SEA_LEVEL, WORLD_HEIGHT};
use crate::Game;

use super::scan::{classify_block, mask_for_tier};
use super::{
    CLASS_BEDROCK, CLASS_CAVE, CLASS_ORE_CARBON, CLASS_ORE_METAL, CLASS_ORE_MINERAL, CLASS_ORE_UNKNOWN, CLASS_ROCK,
    CLASS_SKY, CLASS_SOIL, CLASS_STRUCTURE, CLASS_TUNNEL, CLASS_WATER,
};

/// Horizontal footprint of the 3D subsurface volume (blocks per side, centred on the thumper).
pub const VOL_SIZE: usize = 48;
pub const VOL_HALF: i32 = (VOL_SIZE / 2) as i32;
/// Packed bytes per visible feature voxel: `[lx, y, lz, impedance, class, detail]`.
pub const VOL_VOXEL_BYTES: usize = 6;
/// Safety cap on exported feature voxels so 3D rendering stays locked at 60 FPS even over vast cave systems.
const MAX_VOL_VOXELS: usize = 12_000;
const VOL_HEADROOM: i32 = 4;

/// Cached 3D subsurface volume reading (`SeismicView::volume`).
pub struct VolumeReading {
    pub info: ThumperInfo,
    pub top_y: i32,
    pub player_dx: i32,
    pub player_y: i32,
    pub player_dz: i32,
    pub surface: Vec<i32>,
    pub voxels: Vec<u8>,
}

impl Game {
    /// Scans and caches the 48×48×H 3D subsurface volume under the Seismic Thumper at `cell`.
    pub(crate) fn seismic_volume_at(&mut self, cell: IVec3, pulse_seed: u32) -> u32 {
        let reading = self.scan_volume(cell, pulse_seed);
        let count = reading.as_ref().map_or(0, |r| (r.voxels.len() / VOL_VOXEL_BYTES) as u32);
        self.seismic.volume = reading;
        count
    }

    pub(crate) fn scan_volume(&mut self, cell: IVec3, pulse_seed: u32) -> Option<VolumeReading> {
        let info = self.sim.factory.thumper_at(cell)?;
        let effective_tier = if info.power_share < 500 { info.tier.saturating_sub(1) } else { info.tier };

        // 1. Sample 48×48 ground surface elevations.
        let mut surface = vec![SEA_LEVEL; VOL_SIZE * VOL_SIZE];
        let mut max_y = info.anchor.y + 2;
        for lz in 0..VOL_SIZE {
            let wz = info.anchor.z + lz as i32 - VOL_HALF;
            for lx in 0..VOL_SIZE {
                let wx = info.anchor.x + lx as i32 - VOL_HALF;
                let h = self.column_surface_y(wx, wz, info.anchor.y);
                surface[lz * VOL_SIZE + lx] = h;
                if h > max_y {
                    max_y = h;
                }
            }
        }
        let top_y = (max_y + VOL_HEADROOM).min(WORLD_HEIGHT - 1);
        let h_len = (top_y + 1) as usize;

        // 2. Classify the full 48×48×H volume (only regenerating original chunks where modified).
        let total = VOL_SIZE * VOL_SIZE * h_len;
        let mut cls_grid = vec![CLASS_SKY; total];
        let mut imp_grid = vec![0u8; total];
        let mut det_grid = vec![0u8; total];
        let mut orig_cache: FxHashMap<IVec3, Chunk> = FxHashMap::default();
        let noise_amp: f32 = match info.tier {
            0 => 0.14,
            1 => 0.055,
            _ => 0.018,
        };
        let brownout = (1.0 - (info.power_share as f32 / 1000.0).clamp(0.0, 1.0)) * 0.24;

        for lz in 0..VOL_SIZE {
            let wz = info.anchor.z + lz as i32 - VOL_HALF;
            for lx in 0..VOL_SIZE {
                let wx = info.anchor.x + lx as i32 - VOL_HALF;
                let surf = surface[lz * VOL_SIZE + lx];
                let col_base = (lz * VOL_SIZE + lx) * h_len;
                let max_col = (surf + 1).min(top_y);
                for y in 1..=max_col {
                    let p = IVec3::new(wx, y, wz);
                    let b = self.sim.world.block_anywhere_or_generate(p);
                    if y > surf && (b == AIR || block::is_tree(b) || block::is_sapling(b)) {
                        continue;
                    }
                    let (raw_imp, raw_cls, raw_det) =
                        classify_block(b, p, surf, &self.sim.factory, &mut self.sim.world, &mut orig_cache);
                    if matches!(raw_cls, CLASS_ROCK | CLASS_SOIL | CLASS_BEDROCK | CLASS_SKY) {
                        cls_grid[col_base + y as usize] = raw_cls;
                        continue;
                    }
                    let nval =
                        (unit(hash3(pulse_seed ^ 0x3D5E, wx, y, wz)) as f32 * 2.0 - 1.0) * (noise_amp + brownout);
                    let imp = (raw_imp + nval).clamp(0.02, 0.99);
                    let (cls, det) = mask_for_tier(raw_cls, raw_det, imp, effective_tier);
                    let idx = col_base + y as usize;
                    cls_grid[idx] = cls;
                    imp_grid[idx] = (imp * 255.0).round() as u8;
                    det_grid[idx] = det;
                }
            }
        }

        // 3. Extract boundary-culled feature voxels in priority order: ores & tunnels & structures first, then water & caves.
        let mut priority = Vec::with_capacity(4096 * VOL_VOXEL_BYTES);
        let mut secondary = Vec::with_capacity(4096 * VOL_VOXEL_BYTES);
        let step = if effective_tier == 0 { 2usize } else { 1usize };

        for lz in (0..VOL_SIZE).step_by(step) {
            for lx in (0..VOL_SIZE).step_by(step) {
                let surf = surface[lz * VOL_SIZE + lx];
                let max_col = surf.min(top_y);
                for y in (1..=max_col as usize).step_by(step) {
                    let (cls, imp, det) = if step == 1 {
                        let idx = (lz * VOL_SIZE + lx) * h_len + y;
                        (cls_grid[idx], imp_grid[idx], det_grid[idx])
                    } else {
                        sample_2x2x2(&cls_grid, &imp_grid, &det_grid, lx, y, lz, h_len)
                    };
                    if !is_feature_class(cls) {
                        continue;
                    }
                    // Hide open ocean/pond surface water if at the very top; keep underground aquifers & caves.
                    if cls == CLASS_WATER && y as i32 >= surf - 1 {
                        continue;
                    }
                    // Skip deep interior voxels completely surrounded by the exact same class (cuts draw load 4-6×).
                    if step == 1 && is_enclosed(&cls_grid, lx, y, lz, h_len, cls) {
                        continue;
                    }
                    let dst = if is_high_priority(cls) { &mut priority } else { &mut secondary };
                    dst.extend_from_slice(&[lx as u8, y as u8, lz as u8, imp, cls, det]);
                }
            }
        }

        let max_bytes = MAX_VOL_VOXELS * VOL_VOXEL_BYTES;
        if priority.len() > max_bytes {
            priority.truncate(max_bytes);
        }
        let rem = max_bytes.saturating_sub(priority.len());
        if secondary.len() > rem {
            // Evenly subsample secondary voxels (huge natural caves) so the whole 48×48 volume stays represented.
            let keep = rem / VOL_VOXEL_BYTES;
            let have = secondary.len() / VOL_VOXEL_BYTES;
            if keep > 0 && have > keep {
                let mut picked = Vec::with_capacity(keep * VOL_VOXEL_BYTES);
                for i in 0..keep {
                    let src = (i * have / keep) * VOL_VOXEL_BYTES;
                    picked.extend_from_slice(&secondary[src..src + VOL_VOXEL_BYTES]);
                }
                priority.extend_from_slice(&picked);
            }
        } else {
            priority.extend_from_slice(&secondary);
        }

        let p = self.body().pos.floor();
        Some(VolumeReading {
            info,
            top_y,
            player_dx: p.x - info.anchor.x,
            player_y: p.y,
            player_dz: p.z - info.anchor.z,
            surface,
            voxels: priority,
        })
    }
}

fn is_feature_class(cls: u8) -> bool {
    matches!(
        cls,
        CLASS_CAVE
            | CLASS_TUNNEL
            | CLASS_WATER
            | CLASS_STRUCTURE
            | CLASS_ORE_UNKNOWN
            | CLASS_ORE_METAL
            | CLASS_ORE_CARBON
            | CLASS_ORE_MINERAL
    )
}

fn is_high_priority(cls: u8) -> bool {
    matches!(
        cls,
        CLASS_TUNNEL | CLASS_STRUCTURE | CLASS_ORE_UNKNOWN | CLASS_ORE_METAL | CLASS_ORE_CARBON | CLASS_ORE_MINERAL
    )
}

fn is_enclosed(grid: &[u8], lx: usize, y: usize, lz: usize, h: usize, cls: u8) -> bool {
    if lx == 0 || lx + 1 >= VOL_SIZE || lz == 0 || lz + 1 >= VOL_SIZE || y <= 1 || y + 1 >= h {
        return false;
    }
    let at = |x: usize, yy: usize, z: usize| grid[(z * VOL_SIZE + x) * h + yy] == cls;
    at(lx - 1, y, lz)
        && at(lx + 1, y, lz)
        && at(lx, y - 1, lz)
        && at(lx, y + 1, lz)
        && at(lx, y, lz - 1)
        && at(lx, y, lz + 1)
}

/// Mk1 2×2×2 coarse cluster pick: prioritizes ores, then water, then caves so coarse scans remain legible.
fn sample_2x2x2(cls: &[u8], imp: &[u8], det: &[u8], lx: usize, y: usize, lz: usize, h: usize) -> (u8, u8, u8) {
    let (mut best_c, mut best_i, mut best_d, mut best_rank) = (CLASS_ROCK, 0u8, 0u8, 0u8);
    for dz in 0..2 {
        let z = (lz + dz).min(VOL_SIZE - 1);
        for dx in 0..2 {
            let x = (lx + dx).min(VOL_SIZE - 1);
            for dy in 0..2 {
                let yy = (y + dy).min(h - 1);
                let idx = (z * VOL_SIZE + x) * h + yy;
                let c = cls[idx];
                let rank = match c {
                    CLASS_ORE_UNKNOWN | CLASS_ORE_METAL | CLASS_ORE_CARBON | CLASS_ORE_MINERAL => 4,
                    CLASS_TUNNEL | CLASS_STRUCTURE => 3,
                    CLASS_WATER => 2,
                    CLASS_CAVE => 1,
                    _ => 0,
                };
                if rank > best_rank || (rank == best_rank && imp[idx] > best_i) {
                    (best_c, best_i, best_d, best_rank) = (c, imp[idx], det[idx], rank);
                }
            }
        }
    }
    (best_c, best_i, best_d)
}
