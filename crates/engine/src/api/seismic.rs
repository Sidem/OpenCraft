//! Seismic prospecting API for the host (`web/src/ui/seismic.ts`): thin forwarders to `seismic/`.

use wasm_bindgen::prelude::*;

use crate::block::BlockId;
use crate::math::IVec3;
use crate::seismic::{ore_block_of, ANOMALY_FIELDS, CELL_BYTES, SLICE_WIDTH, VOL_SIZE, VOL_VOXEL_BYTES};
use crate::worldgen::SEA_LEVEL;
use crate::Game;

#[wasm_bindgen]
impl Game {
    /// Metadata for the Seismic Thumper at `(x, y, z)` (empty if none stands there):
    /// `[anchor_x, anchor_y, anchor_z, dir, tier, power_share_0_1000, kw, width, top_y, surface_at_thumper, status, sea_level, player_s, player_y, player_perp, snr_percent]`.
    pub fn seismic_info(&mut self, x: i32, y: i32, z: i32) -> Vec<i32> {
        let Some(h) = self.seismic_header(IVec3::new(x, y, z)) else { return Vec::new() };
        let i = h.info;
        vec![
            i.anchor.x,
            i.anchor.y,
            i.anchor.z,
            i32::from(i.dir),
            i32::from(i.tier),
            i.power_share as i32,
            i.kw as i32,
            SLICE_WIDTH as i32,
            h.top_y,
            h.surface_at_thumper,
            i32::from(i.status),
            SEA_LEVEL,
            h.player_s,
            h.player_y,
            h.player_perp,
            h.snr_percent,
        ]
    }

    /// Runs one 2D seismic slice for the thumper at `(x, y, z)` shifted `perp_offset` blocks perpendicular (`-32..=31`)
    /// and optionally rotated 90° (`flip_axis`). Returns `[height, dir, perp_offset, surface_at_center, player_s, player_perp]`,
    /// or empty if no thumper stands there.
    pub fn seismic_pulse(
        &mut self,
        x: i32,
        y: i32,
        z: i32,
        pulse_seed: u32,
        perp_offset: i32,
        flip_axis: bool,
    ) -> Vec<i32> {
        if !self.seismic_pulse_at(IVec3::new(x, y, z), pulse_seed, perp_offset, flip_axis) {
            return Vec::new();
        }
        let Some(r) = self.seismic.reading.as_ref() else { return Vec::new() };
        vec![r.top_y + 1, i32::from(r.dir), r.perp_offset, r.surface_at_thumper, r.player_s, r.player_perp]
    }

    /// Cached 2D cross-section bytes (`64 × height × seismic_cell_bytes()`): `[impedance, gradient, class, detail]`.
    pub fn seismic_cells(&self) -> Vec<u8> {
        self.seismic.reading.as_ref().map_or_else(Vec::new, |r| r.cells.clone())
    }

    /// Cached ground surface elevations (64 integers) from the latest 2D slice.
    pub fn seismic_surface(&self) -> Vec<i32> {
        self.seismic.reading.as_ref().map_or_else(Vec::new, |r| r.surface.to_vec())
    }

    /// Cached subsurface highlights (`seismic_anomaly_fields()` ints each: `[class, s_index, y, strength, detail, span]`).
    pub fn seismic_anomalies(&self) -> Vec<i32> {
        self.seismic.reading.as_ref().map_or_else(Vec::new, |r| r.anomalies.clone())
    }

    /// Scans and caches a 48×48×H 3D subsurface volume under the Seismic Thumper at `(x, y, z)`.
    /// Returns `[vol_size, top_y, anchor_x, anchor_y, anchor_z, dir, tier, sea_level, player_dx, player_y, player_dz, voxel_count]`.
    pub fn seismic_volume_scan(&mut self, x: i32, y: i32, z: i32, pulse_seed: u32) -> Vec<i32> {
        let count = self.seismic_volume_at(IVec3::new(x, y, z), pulse_seed);
        let Some(v) = self.seismic.volume.as_ref() else { return Vec::new() };
        vec![
            VOL_SIZE as i32,
            v.top_y,
            v.info.anchor.x,
            v.info.anchor.y,
            v.info.anchor.z,
            i32::from(v.info.dir),
            i32::from(v.info.tier),
            SEA_LEVEL,
            v.player_dx,
            v.player_y,
            v.player_dz,
            count as i32,
        ]
    }

    /// Cached 3D feature voxels (`voxel_count × seismic_volume_bytes()`): `[lx, y, lz, impedance, class, detail]`.
    pub fn seismic_volume_voxels(&self) -> Vec<u8> {
        self.seismic.volume.as_ref().map_or_else(Vec::new, |v| v.voxels.clone())
    }

    /// Cached 48×48 terrain surface elevations (`lz * 48 + lx`) for the 3D volume cutaway roof.
    pub fn seismic_volume_surface(&self) -> Vec<i32> {
        self.seismic.volume.as_ref().map_or_else(Vec::new, |v| v.surface.clone())
    }

    pub fn seismic_cell_bytes(&self) -> u32 {
        CELL_BYTES as u32
    }

    pub fn seismic_volume_bytes(&self) -> u32 {
        VOL_VOXEL_BYTES as u32
    }

    pub fn seismic_anomaly_fields(&self) -> u32 {
        ANOMALY_FIELDS as u32
    }

    /// Maps a 1-based mineral `detail` byte from a Mk3 seismic cell back to its ore `BlockId` (0 if none).
    pub fn seismic_ore_block(&self, detail: u8) -> BlockId {
        ore_block_of(detail)
    }
}
