//! The Weather Station (ATMOS, tech Meteorology): a 1×1 mast, 3 high, with an instrument screen, a rain gauge, an
//! anemometer and a vane. On power (`Energy::Optimizer`, a plain power sink: working while its grid gives any power)
//! it measures the atmosphere (`weather/`) for `RADIUS` blocks around; the host shows those readings on the world map
//! and under the minimap (`api/weather.rs`). It changes nothing in the world; nothing of its own is saved.
//!
//! To change its reach or draw: `RADIUS`, `STATION_KW`.

use crate::block::{tex, WEATHER_STATION};
use crate::math::IVec3;

use super::super::footprint::Footprint;
use super::super::Factory;
use super::model::{part, round, Look, Part};
use super::{Energy, Pick, ProcessSpec, ProcessTier, Processor, Status};

/// Blocks around a working station whose weather it measures.
pub const RADIUS: i32 = 640;
/// What it draws on its power grid, in kW.
pub const STATION_KW: u32 = 20;

pub const WEATHER_STATION_SPEC: ProcessSpec = ProcessSpec {
    block: WEATHER_STATION,
    categories: &[],
    pick: Pick::ByInput,
    buffers: [0, 0, 0],
    side: 0,
    tiers: &[ProcessTier { energy: Energy::Optimizer, speed: 1000, fuel: 0, power: STATION_KW }],
    footprint: Footprint { size: [1, 1, 3], ports: &[] },
    verb: "Measuring",
    products: "weather",
    waiting: "Idle",
    map_colour: 0x7fd4ff,
    compute: 0,
    parts: &PARTS,
};

impl Processor {
    /// The status line of a weather station (`None`: another processor).
    pub(super) fn weather_station_text(&self) -> Option<String> {
        if self.spec.block != WEATHER_STATION {
            return None;
        }
        Some(match self.status {
            Status::Working => format!("Measuring the weather within {RADIUS} blocks: see the map (M)"),
            Status::Off => "Switched off".to_string(),
            _ => "No power: hang it on a power pole".to_string(),
        })
    }
}

impl Factory {
    /// Blocks around a working weather station whose weather it measures.
    pub const WEATHER_RADIUS: i32 = RADIUS;

    /// Whether a working weather station measures column (`x`, `z`).
    pub fn weather_covers(&self, x: i32, z: i32) -> bool {
        let r2 = i64::from(RADIUS).pow(2);
        self.weather_stations().any(|(p, on)| on && i64::from(p.x - x).pow(2) + i64::from(p.z - z).pow(2) <= r2)
    }

    /// Every weather station's anchor, and whether it measures now (powered and not switched off).
    pub fn weather_stations(&self) -> impl Iterator<Item = (IVec3, bool)> + '_ {
        self.kinds.weather.iter().map(|&i| {
            let p = &self.processors[i as usize];
            (p.pos, p.status == Status::Working)
        })
    }
}

const WHITE: Look = Look::Tex([tex::PIGMENT_WHITE; 3]);
const POLE: Look = Look::Tex([tex::FRAME; 3]);
const STEEL: Look = Look::Tex([tex::STEEL; 3]);
const BRASS: Look = Look::Tex([tex::MAT_BRASS; 3]);

/// 3 high (±1.5): a concrete pad, the mast, a white louvred screen with the status lamp, a rain gauge, and at the top
/// two crossed arms with three anemometer cups and a wind vane.
const PARTS: [Part; 11] = [
    part([0.0, -1.42, 0.0], [0.92, 0.16, 0.92], Look::Tex([tex::CONCRETE; 3])),
    part([0.0, 0.0, 0.0], [0.1, 2.7, 0.1], POLE),
    part([0.0, -0.55, 0.0], [0.56, 0.5, 0.56], WHITE),
    part([0.0, -0.27, 0.0], [0.64, 0.06, 0.64], WHITE),
    part([0.2, -0.62, 0.29], [0.08, 0.08, 0.02], Look::Lamp),
    round([0.3, -1.17, -0.3], [0.16, 0.34, 0.16], STEEL),
    part([0.0, 1.05, 0.0], [0.8, 0.05, 0.05], POLE),
    round([0.38, 1.13, 0.0], [0.13, 0.1, 0.13], BRASS),
    round([-0.19, 1.13, 0.33], [0.13, 0.1, 0.13], BRASS),
    round([-0.19, 1.13, -0.33], [0.13, 0.1, 0.13], BRASS),
    part([0.0, 1.38, -0.12], [0.03, 0.16, 0.42], POLE),
];

#[cfg(test)]
mod tests;
