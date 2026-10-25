//! Item models of the power poles, so an icon reads as a pole and not a striped cube: a post with a crossarm and two
//! insulators (a tier wears its stripe near the foot, as the placed pole does: `factory/pole.rs`), and the
//! long-distance pole as a small lattice tower. Add a const and a `parts` row.

use super::{part, Part};
use crate::block::{tex, LONG_POLE, POLE};
use crate::item::{ItemId, POLE_MK2, POLE_MK3, POLE_MK4};

const ITEM_POLE: ItemId = ItemId::block(POLE);
const ITEM_LONG_POLE: ItemId = ItemId::block(LONG_POLE);

/// A post on a footing with a crossarm: back to front for the icon, the tier's band round the post's foot.
const fn pole(tier: u8) -> [Part; 8] {
    let frame = tex::FRAME;
    let band = if tier == 0 { frame } else { tex::stripe(tier) };
    [
        part([0.0, -0.46, 0.0], [0.5, 0.08, 0.5], frame),
        part([0.0, 0.0, 0.0], [0.14, 0.92, 0.14], frame),
        part([0.0, -0.3, 0.0], [0.2, 0.16, 0.2], band),
        part([0.0, 0.3, 0.0], [0.86, 0.08, 0.12], frame),
        part([-0.34, 0.41, 0.0], [0.12, 0.14, 0.12], tex::FLASK_GLASS),
        part([0.34, 0.41, 0.0], [0.12, 0.14, 0.12], tex::FLASK_GLASS),
        part([-0.34, 0.475, 0.0], [0.08, 0.04, 0.08], tex::COPPER_WIRE),
        part([0.34, 0.475, 0.0], [0.08, 0.04, 0.08], tex::COPPER_WIRE),
    ]
}

const POLE_MK1_MODEL: [Part; 8] = pole(0);
const POLE_MK2_MODEL: [Part; 8] = pole(1);
const POLE_MK3_MODEL: [Part; 8] = pole(2);
const POLE_MK4_MODEL: [Part; 8] = pole(3);

/// Four legs braced by two rings of bars round a mast, a wide crossarm on top (`pole::long_model`).
const LONG_MODEL: [Part; 17] = {
    let frame = tex::FRAME;
    [
        part([0.0, -0.47, 0.0], [0.86, 0.06, 0.86], frame),
        part([-0.34, 0.0, -0.34], [0.08, 0.92, 0.08], frame),
        part([0.0, -0.2, -0.34], [0.68, 0.06, 0.06], frame),
        part([-0.34, -0.2, 0.0], [0.06, 0.06, 0.68], frame),
        part([0.0, 0.14, -0.34], [0.68, 0.06, 0.06], frame),
        part([-0.34, 0.14, 0.0], [0.06, 0.06, 0.68], frame),
        part([0.34, 0.0, -0.34], [0.08, 0.92, 0.08], frame),
        part([-0.34, 0.0, 0.34], [0.08, 0.92, 0.08], frame),
        part([0.0, 0.0, 0.0], [0.12, 0.92, 0.12], frame),
        part([0.0, -0.2, 0.34], [0.68, 0.06, 0.06], frame),
        part([0.34, -0.2, 0.0], [0.06, 0.06, 0.68], frame),
        part([0.0, 0.14, 0.34], [0.68, 0.06, 0.06], frame),
        part([0.34, 0.14, 0.0], [0.06, 0.06, 0.68], frame),
        part([0.34, 0.0, 0.34], [0.08, 0.92, 0.08], frame),
        part([0.0, 0.38, 0.0], [0.96, 0.07, 0.12], frame),
        part([-0.42, 0.45, 0.0], [0.1, 0.1, 0.1], tex::FLASK_GLASS),
        part([0.42, 0.45, 0.0], [0.1, 0.1, 0.1], tex::FLASK_GLASS),
    ]
};

pub(super) fn parts(item: ItemId) -> &'static [Part] {
    match item {
        ITEM_POLE => &POLE_MK1_MODEL,
        POLE_MK2 => &POLE_MK2_MODEL,
        POLE_MK3 => &POLE_MK3_MODEL,
        POLE_MK4 => &POLE_MK4_MODEL,
        ITEM_LONG_POLE => &LONG_MODEL,
        _ => &[],
    }
}
