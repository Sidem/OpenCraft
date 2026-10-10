//! The laser blocks (Milestone 11, the Photonics tech): four 1×1×1 processors that carry power or data along a line of
//! sight (`factory/laser.rs` casts the beams and joins the grids). They draw nothing themselves and run no work
//! (`Energy::Beam`); each status says whether a beam links it (`Working`) or not (`NoInput`), set by `aim_beams`.
//!
//! - The emitter shoots out of its front, the side that faces whoever placed it (`factory::laser::emit_dir`).
//! - The receiver takes a beam from any side and joins the power grids of its pole and the emitter's.
//! - The data receiver takes a beam from any side and joins the data grids of the fibre nodes beside it and the emitter.
//! - The mirror turns a beam 90° (`factory::laser::reflect`); its facing picks which diagonal it stands on.
//!
//! To add a longer-range tier: a tier row here and a range per tier in `laser.rs`.

use crate::block::{tex, BlockId, DATA_RECEIVER, LASER_EMITTER, LASER_MIRROR, LASER_RECEIVER};

use super::super::footprint::SINGLE;
use super::model::{drum_x, drum_z, part, turned, Look, Part};
use super::{Energy, Pick, ProcessSpec, ProcessTier, Processor, Status};

const fn spec(block: BlockId, verb: &'static str, products: &'static str, parts: &'static [Part]) -> ProcessSpec {
    ProcessSpec {
        block,
        categories: &[],
        pick: Pick::ByInput,
        buffers: [0, 0, 0],
        side: 0,
        tiers: &[ProcessTier { energy: Energy::Beam, speed: 1000, fuel: 0, power: 0 }],
        footprint: SINGLE,
        verb,
        products,
        waiting: "Idle",
        map_colour: 0xff6a5a,
        compute: 0,
        parts,
    }
}

pub const EMITTER_SPEC: ProcessSpec = spec(LASER_EMITTER, "Beaming", "power", &EMITTER_PARTS);
pub const RECEIVER_SPEC: ProcessSpec = spec(LASER_RECEIVER, "Receiving", "power", &RECEIVER_PARTS);
pub const MIRROR_SPEC: ProcessSpec = spec(LASER_MIRROR, "Turning", "beams", &MIRROR_PARTS);
pub const DATA_RECEIVER_SPEC: ProcessSpec = spec(DATA_RECEIVER, "Receiving", "data", &DATA_PARTS);

impl Processor {
    /// The status line of a laser block (`None`: another processor).
    pub(super) fn laser_text(&self) -> Option<String> {
        if self.energy() != Energy::Beam {
            return None;
        }
        let on = self.status == Status::Working;
        Some(
            match (self.spec.block, on) {
                (LASER_EMITTER, true) => "Beaming to a receiver",
                (LASER_EMITTER, false) => "No receiver in line",
                (LASER_RECEIVER, true) => "Receiving power from an emitter",
                (DATA_RECEIVER, true) => "Receiving data from an emitter",
                (LASER_MIRROR, true) => "Turning a beam",
                (LASER_MIRROR, false) => "No beam reaches it",
                _ => "No beam",
            }
            .to_string(),
        )
    }
}

/// A plinth and a plate on its edge, turned 45° so a beam meeting it goes off at a right angle (double-sided).
const MIRROR_PARTS: [Part; 3] = [
    part([0.0, -0.4, 0.0], [0.9, 0.2, 0.9], Look::Band(tex::FRAME)),
    turned([0.0, 0.1, 0.0], [1.2, 0.8, 0.08], Look::Tex([tex::MIRROR; 3]), 0.5),
    part([0.0, -0.25, 0.0], [0.16, 0.1, 0.16], Look::Tex([tex::FRAME; 3])),
];

/// The data receiver: a plinth, a cabinet in the data colours and an antenna of three shrinking fins on a post
/// (beams come from any side), a status lamp.
const DATA_PARTS: [Part; 7] = [
    part([0.0, -0.4, 0.0], [0.9, 0.2, 0.9], Look::Band(tex::FRAME)),
    part([0.0, -0.15, 0.0], [0.7, 0.3, 0.7], Look::Tex([tex::DATA_TOP, tex::DATA_SIDE, tex::FRAME])),
    part([0.0, 0.2, 0.0], [0.08, 0.5, 0.08], Look::Tex([tex::FRAME; 3])),
    part([0.0, 0.08, 0.0], [0.6, 0.05, 0.6], Look::Tex([tex::DATA_TOP; 3])),
    part([0.0, 0.24, 0.0], [0.44, 0.05, 0.44], Look::Tex([tex::DATA_TOP; 3])),
    part([0.0, 0.4, 0.0], [0.28, 0.05, 0.28], Look::Tex([tex::DATA_TOP; 3])),
    part([0.25, -0.1, 0.36], [0.1, 0.08, 0.04], Look::Lamp),
];

/// Inside one cell, front towards +z: a plinth, a short dark housing, a barrel out of the front with a lens, a status lamp.
const EMITTER_PARTS: [Part; 5] = [
    part([0.0, -0.4, 0.0], [0.9, 0.2, 0.9], Look::Band(tex::FRAME)),
    part([0.0, -0.05, -0.08], [0.7, 0.5, 0.6], Look::Tex([tex::EMIT_TOP, tex::EMIT_SIDE, tex::FRAME])),
    part([0.0, 0.0, 0.3], [0.3, 0.3, 0.4], Look::Tex([tex::FRAME; 3])),
    part([0.0, 0.0, 0.5], [0.18, 0.18, 0.04], Look::Lamp),
    part([0.3, 0.3, -0.3], [0.1, 0.08, 0.1], Look::Lamp),
];

/// The power receiver: a plinth, a squat cabinet, and on a short post a round collector (two crossed parts; beams
/// come from any side) girdled by a copper ring, a status lamp.
const RECEIVER_PARTS: [Part; 7] = [
    part([0.0, -0.4, 0.0], [0.9, 0.2, 0.9], Look::Band(tex::FRAME)),
    part([0.0, -0.15, 0.0], [0.7, 0.3, 0.7], Look::Tex([tex::RECV_TOP, tex::RECV_SIDE, tex::FRAME])),
    part([0.0, 0.05, 0.0], [0.14, 0.12, 0.14], Look::Tex([tex::FRAME; 3])),
    drum_x([0.0, 0.3, 0.0], [0.5, 0.4, 0.5], Look::Tex([tex::RECV_TOP; 3])),
    drum_z([0.0, 0.3, 0.0], [0.5, 0.4, 0.5], Look::Tex([tex::RECV_TOP; 3])),
    part([0.0, 0.3, 0.0], [0.56, 0.06, 0.56], Look::Tex([tex::COPPER_INGOT; 3])),
    part([0.25, -0.1, 0.36], [0.1, 0.08, 0.04], Look::Lamp),
];
