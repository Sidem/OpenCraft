//! The ore washer (Milestone 10): a processor that takes only *crushed* ore (`Pick::ByInput`, `Category::Washing`), so
//! the crusher stays the first step of the chain and raw ore does not fit. Three crushed ore and a unit of water
//! (a blue inlet on the left, `steam.rs`) make four washed ore at the front and a tailings block at the right-hand
//! hatch, which belts must take away (a full hatch stops it, the status naming the tailings). Washed ore smelts one
//! for one, so raw ore gives 1.0 ingots, crushed 1.5 and washed 2.0 (`recipes/machine.rs`); the crusher grinds
//! tailings to sand. Tailings are also a plain fill block for the Planner.
//!
//! To wash another ore: a washed item, a `Washing` recipe row, and the smelter's (or machine's) row for it.

use crate::block::{tex, WASHER};

use super::super::footprint::{Footprint, Port, Role, Side, Which};
use super::model::{part, Look, Part};
use super::{Energy, Pick, ProcessSpec, ProcessTier};
use crate::recipes::Category;

#[cfg(test)]
mod tests;

pub const WASHER_SPEC: ProcessSpec = ProcessSpec {
    block: WASHER,
    categories: &[Category::Washing],
    pick: Pick::ByInput,
    buffers: [1, 0, 1],
    side: 1,
    tiers: &[ProcessTier { energy: Energy::Electric, speed: 1000, fuel: 0, power: 60 }],
    footprint: Footprint {
        size: [2, 2, 2],
        ports: &[
            Port { side: Side::Back, role: Role::In, cell: Which::All },
            Port { side: Side::Front, role: Role::Out, cell: Which::All },
            Port { side: Side::Right, role: Role::Side, cell: Which::All },
            Port { side: Side::Left, role: Role::Water, cell: Which::Nth(0) },
        ],
    },
    verb: "Washing",
    products: "washed ore",
    waiting: "Waiting for crushed iron, copper or bauxite",
    map_colour: 0x4f9a98,
    parts: &WASHER_PARTS,
};

const BODY: [u16; 3] = [tex::WASHER_TOP, tex::WASHER_SIDE, tex::FRAME];
const FRAME: Look = Look::Tex([tex::FRAME; 3]);

/// Across and deep 2, 2 high (±1): a banded plinth, a long wash trough across the back with a grated top, a
/// motor house in front of it, a hopper chute on the back wall and a status lamp.
const WASHER_PARTS: [Part; 6] = [
    part([0.0, -0.9, 0.0], [1.96, 0.2, 1.96], Look::Band(tex::FRAME)),
    part([0.0, -0.3, -0.35], [1.9, 0.9, 1.2], Look::Tex(BODY)),
    part([0.0, 0.2, -0.35], [1.7, 0.1, 1.0], Look::Tex([tex::WASHER_TOP; 3])),
    part([0.0, -0.45, 0.55], [1.5, 0.6, 0.7], Look::Tex(BODY)),
    part([0.0, 0.55, -0.8], [0.9, 0.5, 0.3], FRAME),
    part([0.78, -0.1, 0.92], [0.14, 0.1, 0.14], Look::Lamp),
];
