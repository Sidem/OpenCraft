//! The laser emitter and receiver (Milestone 11, the Photonics tech): two 1×1×1 processors that carry power between
//! grids along a straight line of sight (`factory/laser.rs` casts the beams and joins the grids). They draw nothing
//! themselves and run no work (`Energy::Beam`); each hangs on a pole like any machine, and its status says whether a
//! beam links it (`Working`) or not (`NoInput`), set by `aim_beams`.
//!
//! - The emitter shoots out of its front, the side that faces whoever placed it (`factory::laser::emit_dir`).
//! - The receiver takes a beam from any side.
//!
//! To add a longer-range tier: a tier row here and a range per tier in `laser.rs`.

use crate::block::{tex, BlockId, LASER_EMITTER, LASER_RECEIVER};

use super::super::footprint::SINGLE;
use super::model::{part, Look, Part};
use super::{Energy, Pick, ProcessSpec, ProcessTier, Processor, Status};

const fn spec(block: BlockId, verb: &'static str, parts: &'static [Part]) -> ProcessSpec {
    ProcessSpec {
        block,
        categories: &[],
        pick: Pick::ByInput,
        buffers: [0, 0, 0],
        side: 0,
        tiers: &[ProcessTier { energy: Energy::Beam, speed: 1000, fuel: 0, power: 0 }],
        footprint: SINGLE,
        verb,
        products: "power",
        waiting: "Idle",
        map_colour: 0xff6a5a,
        compute: 0,
        parts,
    }
}

pub const EMITTER_SPEC: ProcessSpec = spec(LASER_EMITTER, "Beaming", &EMITTER_PARTS);
pub const RECEIVER_SPEC: ProcessSpec = spec(LASER_RECEIVER, "Receiving", &RECEIVER_PARTS);

impl Processor {
    /// The status line of an emitter or receiver (`None`: another processor).
    pub(super) fn laser_text(&self) -> Option<String> {
        if self.energy() != Energy::Beam {
            return None;
        }
        let emitter = self.spec.block == LASER_EMITTER;
        Some(match (self.status, emitter) {
            (Status::Working, true) => "Beaming power to a receiver".to_string(),
            (Status::Working, false) => "Receiving power from an emitter".to_string(),
            (_, true) => "No receiver in line".to_string(),
            (_, false) => "No beam".to_string(),
        })
    }
}

/// Inside one cell, front towards +z: a plinth, a short dark housing, a barrel out of the front with a lens, a status lamp.
const EMITTER_PARTS: [Part; 5] = [
    part([0.0, -0.4, 0.0], [0.9, 0.2, 0.9], Look::Band(tex::FRAME)),
    part([0.0, -0.05, -0.08], [0.7, 0.5, 0.6], Look::Tex([tex::EMIT_TOP, tex::EMIT_SIDE, tex::FRAME])),
    part([0.0, 0.0, 0.3], [0.3, 0.3, 0.4], Look::Tex([tex::FRAME; 3])),
    part([0.0, 0.0, 0.5], [0.18, 0.18, 0.04], Look::Lamp),
    part([0.3, 0.3, -0.3], [0.1, 0.08, 0.1], Look::Lamp),
];

/// A plinth, a squat cabinet, a dish-like collector on top and a status lamp.
const RECEIVER_PARTS: [Part; 4] = [
    part([0.0, -0.4, 0.0], [0.9, 0.2, 0.9], Look::Band(tex::FRAME)),
    part([0.0, -0.1, 0.0], [0.7, 0.4, 0.7], Look::Tex([tex::RECV_TOP, tex::RECV_SIDE, tex::FRAME])),
    part([0.0, 0.25, 0.0], [0.86, 0.1, 0.86], Look::Tex([tex::RECV_TOP; 3])),
    part([0.3, 0.0, 0.36], [0.1, 0.08, 0.1], Look::Lamp),
];
