//! The ground station (Milestone 12, step 12.3): a 3×3×2 dish that downlinks what science satellites (`satellites.rs`,
//! kind `SCIENCE`) gather, as compute for its data grid.
//!
//! - It is a data producer (`compute > 0`, hung on a fibre node like a datacenter) and a power sink (`Energy::Optimizer`,
//!   `STATION_KW`): it gives `TF_PER_SATELLITE` of compute for every science satellite whose footprint covers it, up to
//!   `MAX_TF`, scaled by its power share. Under no satellite it gives nothing.
//! - What it gives is derived each tick from the satellites and the tick alone (`sense`, `Store::flow`), before the data
//!   balance (`Data::balance` reads [`Processor::offered_tf`]); nothing here is saved.
//!
//! To add another sky-fed data source: a spec like this with its own satellite kind.

use crate::block::{tex, GROUND_STATION};
use crate::satellites::{Satellites, SCIENCE};

use super::super::footprint::Footprint;
use super::model::{part, Look, Part};
use super::{Energy, Pick, ProcessSpec, ProcessTier, Processor, Status};

/// TF one science satellite downlinks to a station it covers.
pub const TF_PER_SATELLITE: u32 = 40;
/// The most a station gives, in TF (the spec's `compute`).
pub const MAX_TF: u32 = 100;
/// What it draws on its power grid, in kW.
pub const STATION_KW: u32 = 150;

pub const STATION_SPEC: ProcessSpec = ProcessSpec {
    block: GROUND_STATION,
    categories: &[],
    pick: Pick::ByInput,
    buffers: [0, 0, 0],
    side: 0,
    tiers: &[ProcessTier { energy: Energy::Optimizer, speed: 1000, fuel: 0, power: STATION_KW }],
    footprint: Footprint { size: [3, 3, 2], ports: &[] },
    verb: "Downlinking",
    products: "compute",
    waiting: "Idle",
    map_colour: 0x9be7ff,
    compute: MAX_TF as i32,
    parts: &PARTS,
};

impl Processor {
    /// The status line of a ground station (`None`: another processor).
    pub(super) fn ground_text(&self) -> Option<String> {
        if self.spec.block != GROUND_STATION {
            return None;
        }
        Some(match (self.status, self.store.flow) {
            (_, 0) => "No science satellite overhead".to_string(),
            (Status::NoPower, _) => "No power: hang it on a power pole".to_string(),
            (_, tf) => format!("Science satellites downlink {tf} of {MAX_TF} TF · hang it on a fibre node"),
        })
    }

    /// TF a ground station offers its data grid before power scaling (0: another processor, or no satellite overhead).
    pub(in crate::factory) fn offered_tf(&self) -> u32 {
        if self.spec.block == GROUND_STATION {
            self.store.flow
        } else {
            self.spec.compute.max(0) as u32
        }
    }
}

/// Measures the science satellites over every ground station. Called by `Factory::update` before the data balance.
pub(in crate::factory) fn sense(processors: &mut [Processor], stations: &[u32], sats: &Satellites, tick: u64) {
    for &i in stations {
        let p = &mut processors[i as usize];
        let overhead = sats.covering(SCIENCE, p.pos.x, p.pos.z, tick) as u32;
        p.store.flow = (overhead * TF_PER_SATELLITE).min(MAX_TF);
    }
}

/// A banded plinth, a pale body, a big dish tilted up on a short neck, a feed horn on a mast and a status lamp.
const PARTS: [Part; 6] = [
    part([0.0, -0.9, 0.0], [2.96, 0.2, 2.96], Look::Band(tex::FRAME)),
    part([0.0, -0.4, 0.0], [2.2, 0.8, 2.2], Look::Tex([tex::GROUND_SIDE, tex::GROUND_TOP, tex::FRAME])),
    part([0.0, 0.2, 0.0], [0.3, 0.5, 0.3], Look::Tex([tex::FRAME; 3])),
    part([0.0, 0.6, 0.0], [2.8, 0.14, 2.8], Look::Tex([tex::GROUND_TOP, tex::GROUND_TOP, tex::FRAME])),
    part([0.0, 0.95, 0.0], [0.08, 0.7, 0.08], Look::Tex([tex::FRAME; 3])),
    part([0.0, 1.35, 0.0], [0.2, 0.14, 0.2], Look::Lamp),
];

#[cfg(test)]
mod tests;
