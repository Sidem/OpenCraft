//! The upgrader (research: Assembly): a 2×2×2 processor (`Pick::Chosen`, `Category::Refit`) that raises a Mk1 item to the tier
//! the player chooses. Belts bring the Mk1 item and the kits of every step up to that tier; the kits are used up and the
//! tier's item comes out of the front. The recipes are one row per tier (`recipes/refit.rs`), derived from `FAMILIES`,
//! so a new tier or family needs nothing here. Behaviour is the generic processor's; this file is only the spec row.
//!
//! To change its pace or price: [`STEP_KW`] and the recipe rows' seconds (`recipes/refit.rs`).

use crate::block::{tex, UPGRADER};
use crate::recipes::Category;

use super::super::footprint::{Footprint, Port, Role, Side, Which};
use super::model::{part, Look, Part};
use super::{Energy, Pick, ProcessSpec, ProcessTier};

#[cfg(test)]
mod tests;

/// What it draws while it works, in kW.
const STEP_KW: u32 = 30;

pub const UPGRADER_SPEC: ProcessSpec = ProcessSpec {
    block: UPGRADER,
    categories: &[Category::Refit],
    pick: Pick::Chosen,
    // The Mk1 item and up to four kinds of kit (green, blue, violet, gold).
    buffers: [5, 0, 1],
    side: 0,
    tiers: &[ProcessTier { energy: Energy::Electric, speed: 1000, fuel: 0, power: STEP_KW }],
    footprint: Footprint {
        size: [2, 2, 2],
        ports: &[
            Port { side: Side::Back, role: Role::In, cell: Which::All },
            Port { side: Side::Left, role: Role::In, cell: Which::All },
            Port { side: Side::Right, role: Role::In, cell: Which::All },
            Port { side: Side::Front, role: Role::Out, cell: Which::All },
        ],
    },
    verb: "Upgrading",
    products: "upgraded items",
    waiting: "",
    map_colour: 0x7a5bd6,
    compute: 0,
    parts: &PARTS,
};

const BODY: [u16; 3] = [tex::STEEL, tex::ASSEMBLER_SIDE, tex::FRAME];
const FRAME: Look = Look::Tex([tex::FRAME; 3]);
const STEEL: Look = Look::Tex([tex::STEEL; 3]);

/// Across and deep 2, 2 high (±1): a banded plinth, a low housing, two posts and a crossbar over it with a platen that
/// presses down while it works, a kit chute on the back, a status lamp and the product display on the housing's front.
const PARTS: [Part; 9] = [
    part([0.0, -0.9, 0.0], [1.96, 0.2, 1.96], Look::Band(tex::FRAME)),
    part([0.0, -0.4, 0.0], [1.8, 0.8, 1.8], Look::Tex(BODY)),
    part([-0.7, 0.55, 0.0], [0.14, 1.0, 0.14], STEEL),
    part([0.7, 0.55, 0.0], [0.14, 1.0, 0.14], STEEL),
    part([0.0, 1.0, 0.0], [1.5, 0.12, 0.2], FRAME),
    part([0.0, 0.62, 0.0], [0.7, 0.2, 0.7], Look::Press(1.0, [tex::IRON_PLATE; 3])),
    part([0.0, 0.2, -0.95], [0.9, 0.4, 0.3], FRAME),
    part([0.78, 0.1, 0.92], [0.14, 0.1, 0.14], Look::Lamp),
    part([0.0, 0.3, 0.6], super::parts::SIGN, Look::Product),
];
