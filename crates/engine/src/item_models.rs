//! Presentation-only box assemblies shared by loose items and inventory icons.
//! Coordinates and dimensions are fractions of a full item cube (y up); parts are drawn in order, so
//! list them back to front (the icon looks from +x, +z, above). Add a static slice for a new model.
//! A part may turn about its own vertical axis (`yaw`) and is a box or a round body (`form`: `shapes::cylinder`,
//! `bar_x`); never fake a round thing from turned or crossed boxes (`tests.rs` checks).
//! Tools stand upright in the x-y plane, head up; their heads use their tier's material.
//! Invariant: every item that is not a block has a model, its bottom at `-size[1] / 2` of its `ItemDef` (where it
//! rests on the ground and on belts), built from materials (`tex::MAT_*`, metals, rubble), never from a picture of the
//! item. Later items live in submodules by kind (`stock`, `components`, `wear`, `craft`, `space`, `power`) with the shape
//! helpers in `shapes`; a new item gets a const there and a row in its module's `parts`.

use std::f32::consts::{FRAC_PI_2, FRAC_PI_6};

use crate::block::{tex, COAL_MINER, MINER, MINER_MK2};
pub use crate::factory::{Axis, Form};
use crate::item::{
    ItemId, BLUE_PACK, COPPER_INGOT, COPPER_WIRE, CORE_DRILL, GEAR, GOLD_PACK, GREEN_PACK, IRON_AXE, IRON_INGOT,
    IRON_PICKAXE, IRON_PLATE, IRON_ROD, IRON_SHOVEL, RED_PACK, SCANNER, SCANNER_MK2, SCREW, STEEL_AXE, STEEL_BEAM,
    STEEL_INGOT, STEEL_PICKAXE, STEEL_PLATE, STEEL_SHOVEL, STONE_AXE, STONE_PICKAXE, STONE_SHOVEL, VIOLET_PACK,
};

mod belts;
mod components;
mod craft;
mod icon_frame;
mod machines;
mod machines_plant;
mod placed;
mod power;
mod shapes;
mod space;
mod stock;
mod wear;

pub use icon_frame::icon_frame;

const ITEM_MINER_MK1: ItemId = ItemId::block(MINER);
const ITEM_MINER_MK2: ItemId = ItemId::block(MINER_MK2);
const ITEM_COAL_MINER: ItemId = ItemId::block(COAL_MINER);

#[derive(Clone, Copy)]
pub struct Part {
    pub center: [f32; 3],
    pub size: [f32; 3],
    /// Turn about the part's own vertical axis, radians (0 for most parts: the box stays axis-aligned).
    pub yaw: f32,
    pub tex: [u16; 3],
    /// A box, or a round body filling the box (`factory::push_round`: its size is across, along, across).
    pub form: Form,
}

const fn part(center: [f32; 3], size: [f32; 3], layer: u16) -> Part {
    turned(center, size, 0.0, layer)
}

const fn turned(center: [f32; 3], size: [f32; 3], yaw: f32, layer: u16) -> Part {
    Part { center, size, yaw, tex: [layer; 3], form: Form::Box }
}

const IRON_BAR: [Part; 2] = [
    part([0.0, -0.07, 0.0], [0.88, 0.29, 0.53], tex::IRON_INGOT),
    part([0.0, 0.105, 0.0], [0.69, 0.07, 0.4], tex::IRON_PLATE),
];
const COPPER_BAR: [Part; 2] = [
    part([0.0, -0.07, 0.0], [0.88, 0.29, 0.53], tex::COPPER_INGOT),
    part([0.0, 0.105, 0.0], [0.69, 0.07, 0.4], tex::COPPER_INGOT),
];
const PLATE: [Part; 2] = [
    part([0.0, -0.07, 0.0], [0.84, 0.1, 0.84], tex::IRON_PLATE),
    part([0.0, 0.005, 0.0], [0.66, 0.05, 0.66], tex::IRON_PLATE),
];
const ROD: [Part; 3] = [
    part([0.0, 0.0, 0.0], [0.9, 0.13, 0.13], tex::IRON_ROD),
    part([-0.39, 0.0, 0.0], [0.13, 0.19, 0.19], tex::IRON_PLATE),
    part([0.39, 0.0, 0.0], [0.13, 0.19, 0.19], tex::IRON_PLATE),
];
const SCREWS: [Part; 3] = [
    part([-0.09, -0.06, -0.1], [0.62, 0.1, 0.1], tex::SCREW),
    part([-0.36, -0.06, -0.1], [0.13, 0.24, 0.24], tex::IRON_PLATE),
    part([0.11, 0.09, 0.17], [0.43, 0.09, 0.09], tex::SCREW),
];
/// A round spool on its side: copper windings between two round wooden flanges, a dark axle hole, and a loose end
/// unrolled from under it along the ground.
const WIRE: [Part; 5] = [
    shapes::bar_x([-0.22, 0.0, 0.0], 0.06, 0.5, tex::HANDLE),
    shapes::bar_x([0.0, 0.0, 0.0], 0.38, 0.4, tex::COPPER_WIRE),
    Part {
        center: [0.08, -0.23, 0.18],
        size: [0.04, 0.04, 0.36],
        yaw: 0.0,
        tex: [tex::COPPER_WIRE; 3],
        form: Form::Round(Axis::Z, 1.0),
    },
    shapes::bar_x([0.22, 0.0, 0.0], 0.06, 0.5, tex::HANDLE),
    shapes::bar_x([0.25, 0.0, 0.0], 0.02, 0.12, tex::FRAME),
];
/// Where each of a cog's 12 teeth points: the cosine and sine of k turns of 30 degrees.
const TEETH: [(f32, f32); 12] = {
    const H: f32 = 0.866_025_4;
    [
        (1.0, 0.0),
        (H, 0.5),
        (0.5, H),
        (0.0, 1.0),
        (-0.5, H),
        (-H, 0.5),
        (-1.0, 0.0),
        (-H, -0.5),
        (-0.5, -H),
        (0.0, -1.0),
        (0.5, -H),
        (H, -0.5),
    ]
};

/// A cog lying flat: a round steel disc with 12 teeth standing out of its rim, a hub and a bore on top. The icon has
/// no depth test, so the teeth behind the disc (towards -x, -z) come first, then the disc, then the teeth in front.
/// The floor of a 0.2 high item is at -0.1.
const COG: [Part; 15] = cog();

const fn cog() -> [Part; 15] {
    let mut parts = [const { part([0.0; 3], [0.0; 3], tex::STEEL) }; 15];
    let mut n = 0;
    let mut pass = 0;
    while pass < 2 {
        let mut k = 0;
        while k < 12 {
            let (c, s) = TEETH[k];
            if (c + s < 0.0) == (pass == 0) {
                let yaw = k as f32 * FRAC_PI_6;
                parts[n] = turned([0.36 * c, -0.02, 0.36 * s], [0.14, 0.16, 0.09], yaw, tex::STEEL);
                n += 1;
            }
            k += 1;
        }
        if pass == 0 {
            parts[n] = shapes::cylinder([0.0, -0.02, 0.0], 0.62, 0.16, tex::STEEL);
            n += 1;
        }
        pass += 1;
    }
    parts[13] = shapes::cylinder([0.0, 0.085, 0.0], 0.24, 0.05, tex::STEEL);
    parts[14] = shapes::cylinder([0.0, 0.088, 0.0], 0.09, 0.05, tex::FRAME);
    parts
}
/// A miner for the icon and loose items, drill down like `factory/miner_model.rs` (the working rig, with its cage
/// and a turning auger): a plate and four posts round a screw auger under the tier's band and the motor housing,
/// with a fan and a lamp on top. Back to front for the icon: the far post, the auger, the near posts, the housing.
const fn miner(side: u16, tier: u8) -> [Part; 18] {
    let frame = tex::FRAME;
    let mut p = [const { part([0.0; 3], [0.0; 3], tex::FRAME) }; 18];
    p[0] = part([0.0, -0.46, 0.0], [0.9, 0.08, 0.9], frame);
    p[1] = part([-0.38, -0.21, -0.38], [0.1, 0.5, 0.1], frame);
    p[2] = part([0.0, -0.2, 0.0], [0.1, 0.5, 0.1], tex::DRILL);
    let mut k = 0;
    while k < 6 {
        p[3 + k] = turned([0.0, -0.38 + 0.07 * k as f32, 0.0], [0.36, 0.07, 0.08], 0.7 * k as f32, tex::DRILL);
        k += 1;
    }
    p[9] = part([0.38, -0.21, -0.38], [0.1, 0.5, 0.1], frame);
    p[10] = part([-0.38, -0.21, 0.38], [0.1, 0.5, 0.1], frame);
    p[11] = part([0.38, -0.21, 0.38], [0.1, 0.5, 0.1], frame);
    p[12] = Part {
        center: [0.0, 0.29, 0.0],
        size: [0.8, 0.38, 0.8],
        yaw: 0.0,
        tex: [tex::MINER_TOP, side, frame],
        form: Form::Box,
    };
    p[13] = Part {
        center: [0.0, 0.07, 0.0],
        size: [0.86, 0.06, 0.86],
        yaw: 0.0,
        tex: [frame, tex::stripe(tier), frame],
        form: Form::Box,
    };
    p[14] = part([0.0, 0.49, 0.0], [0.1, 0.04, 0.1], tex::DRILL);
    p[15] = turned([0.0, 0.492, 0.0], [0.36, 0.03, 0.06], 0.5, tex::DRILL);
    p[16] = turned([0.0, 0.496, 0.0], [0.36, 0.03, 0.06], 0.5 + FRAC_PI_2, tex::DRILL);
    p[17] = part([0.27, 0.49, 0.27], [0.12, 0.03, 0.12], tex::LAMP_GREEN);
    p
}
const MINER_BODY_1: [Part; 18] = miner(tex::MINER_SIDE, 0);
const MINER_BODY_2: [Part; 18] = miner(tex::MINER_MK2_SIDE, 1);
const MINER_BODY_3: [Part; 18] = miner(tex::MINER_MK3_SIDE, 2);
const MINER_BODY_4: [Part; 18] = miner(tex::MINER_MK4_SIDE, 3);
const MINER_BODY_5: [Part; 18] = miner(tex::MINER_MK5_SIDE, 4);
/// The coal miner: a smelter-sided housing under a soot band instead of a tier's stripe.
const COAL_MINER_BODY: [Part; 18] = {
    let mut p = miner(tex::SMELTER_SIDE, 0);
    p[13].tex = [tex::FRAME, tex::SOOT, tex::FRAME];
    p
};
const RED_FLASK: [Part; 4] = [
    part([0.0, -0.12, 0.0], [0.42, 0.42, 0.42], tex::RED_PACK),
    part([0.0, 0.11, 0.0], [0.32, 0.06, 0.32], tex::FLASK_GLASS),
    part([0.0, 0.23, 0.0], [0.17, 0.2, 0.17], tex::FLASK_GLASS),
    part([0.0, 0.35, 0.0], [0.22, 0.08, 0.22], tex::BOX_TOP),
];
const GREEN_FLASK: [Part; 4] = [
    part([0.0, -0.12, 0.0], [0.42, 0.42, 0.42], tex::GREEN_PACK),
    part([0.0, 0.11, 0.0], [0.32, 0.06, 0.32], tex::FLASK_GLASS),
    part([0.0, 0.23, 0.0], [0.17, 0.2, 0.17], tex::FLASK_GLASS),
    part([0.0, 0.35, 0.0], [0.22, 0.08, 0.22], tex::BOX_TOP),
];

const BLUE_FLASK: [Part; 4] = [
    part([0.0, -0.12, 0.0], [0.42, 0.42, 0.42], tex::BLUE_PACK),
    part([0.0, 0.11, 0.0], [0.32, 0.06, 0.32], tex::FLASK_GLASS),
    part([0.0, 0.23, 0.0], [0.17, 0.2, 0.17], tex::FLASK_GLASS),
    part([0.0, 0.35, 0.0], [0.22, 0.08, 0.22], tex::BOX_TOP),
];

const VIOLET_FLASK: [Part; 4] = [
    part([0.0, -0.12, 0.0], [0.42, 0.42, 0.42], tex::VIOLET_PACK),
    part([0.0, 0.11, 0.0], [0.32, 0.06, 0.32], tex::FLASK_GLASS),
    part([0.0, 0.23, 0.0], [0.17, 0.2, 0.17], tex::FLASK_GLASS),
    part([0.0, 0.35, 0.0], [0.22, 0.08, 0.22], tex::BOX_TOP),
];

const GOLD_FLASK: [Part; 4] = [
    part([0.0, -0.12, 0.0], [0.42, 0.42, 0.42], tex::GOLD_PACK),
    part([0.0, 0.11, 0.0], [0.32, 0.06, 0.32], tex::FLASK_GLASS),
    part([0.0, 0.23, 0.0], [0.17, 0.2, 0.17], tex::FLASK_GLASS),
    part([0.0, 0.35, 0.0], [0.22, 0.08, 0.22], tex::BOX_TOP),
];

const fn pickaxe(head: u16) -> [Part; 6] {
    [
        part([0.0, -0.08, 0.0], [0.08, 0.8, 0.08], tex::HANDLE),
        part([0.0, 0.3, 0.0], [0.14, 0.16, 0.14], head),
        part([0.0, 0.3, 0.0], [0.5, 0.1, 0.1], head),
        part([-0.3, 0.25, 0.0], [0.12, 0.1, 0.09], head),
        part([0.3, 0.25, 0.0], [0.12, 0.1, 0.09], head),
        part([0.39, 0.18, 0.0], [0.07, 0.08, 0.07], head),
    ]
}

const fn axe(head: u16) -> [Part; 4] {
    [
        part([0.0, -0.05, 0.0], [0.08, 0.84, 0.08], tex::HANDLE),
        part([-0.04, 0.27, 0.0], [0.14, 0.18, 0.12], head),
        part([0.14, 0.27, 0.0], [0.22, 0.2, 0.06], head),
        part([0.28, 0.27, 0.0], [0.08, 0.32, 0.05], head),
    ]
}

const fn shovel(head: u16) -> [Part; 5] {
    [
        part([0.0, 0.1, 0.0], [0.08, 0.66, 0.08], tex::HANDLE),
        part([0.0, 0.43, 0.0], [0.22, 0.06, 0.08], tex::HANDLE),
        part([0.0, -0.24, 0.0], [0.1, 0.08, 0.08], head),
        part([0.0, -0.36, 0.0], [0.3, 0.18, 0.05], head),
        part([0.0, -0.47, 0.0], [0.2, 0.05, 0.05], head),
    ]
}

/// A bar over a plate, in steel.
const STEEL_BAR: [Part; 2] = [
    part([0.0, -0.07, 0.0], [0.88, 0.29, 0.53], tex::STEEL_INGOT),
    part([0.0, 0.105, 0.0], [0.69, 0.07, 0.4], tex::STEEL_PLATE),
];
const STEEL_SHEET: [Part; 2] = [
    part([0.0, -0.07, 0.0], [0.84, 0.1, 0.84], tex::STEEL_PLATE),
    part([0.0, 0.005, 0.0], [0.66, 0.05, 0.66], tex::STEEL_PLATE),
];
/// An I-beam on its side: a web between two flanges.
const STEEL_GIRDER: [Part; 3] = [
    part([0.0, 0.0, 0.0], [0.9, 0.12, 0.12], tex::STEEL_BEAM),
    part([0.0, -0.09, 0.0], [0.9, 0.06, 0.34], tex::STEEL_BEAM),
    part([0.0, 0.09, 0.0], [0.9, 0.06, 0.34], tex::STEEL_BEAM),
];

const STONE_PICK: [Part; 6] = pickaxe(tex::STONE);
const IRON_PICK: [Part; 6] = pickaxe(tex::STEEL);
const STONE_HATCHET: [Part; 4] = axe(tex::STONE);
const IRON_HATCHET: [Part; 4] = axe(tex::STEEL);
const STONE_SPADE: [Part; 5] = shovel(tex::STONE);
const IRON_SPADE: [Part; 5] = shovel(tex::STEEL);
const STEEL_PICK: [Part; 6] = pickaxe(tex::STEEL_HEAD);
const STEEL_HATCHET: [Part; 4] = axe(tex::STEEL_HEAD);
const STEEL_SPADE: [Part; 5] = shovel(tex::STEEL_HEAD);

/// A handset: casing, a wooden grip, a steel antenna, and the screen on its front (+z).
const SCANNER_SET: [Part; 4] = [
    part([0.0, 0.0, 0.0], [0.44, 0.56, 0.16], tex::CORE_DRILL),
    part([0.0, -0.36, 0.0], [0.16, 0.18, 0.12], tex::HANDLE),
    part([0.14, 0.36, 0.0], [0.04, 0.18, 0.04], tex::STEEL),
    part([0.0, 0.07, 0.09], [0.34, 0.3, 0.02], tex::SCANNER),
];

/// The Mk2 handset: the same shape with a violet tier stripe across the casing and the Mk2 screen.
const SCANNER_MK2_SET: [Part; 5] = [
    part([0.0, 0.0, 0.0], [0.44, 0.56, 0.16], tex::CORE_DRILL),
    part([0.0, -0.36, 0.0], [0.16, 0.18, 0.12], tex::HANDLE),
    part([0.14, 0.36, 0.0], [0.04, 0.18, 0.04], tex::STEEL),
    part([0.0, 0.07, 0.09], [0.34, 0.3, 0.02], tex::SCANNER_MK2),
    part([0.0, -0.2, 0.0], [0.46, 0.06, 0.18], tex::stripe(3)),
];

/// A motor housing with a carrying bar, over a fluted bit with a steel point.
const DRILL_SET: [Part; 4] = [
    part([0.0, -0.16, 0.0], [0.09, 0.46, 0.09], tex::DRILL),
    part([0.0, -0.43, 0.0], [0.05, 0.08, 0.05], tex::STEEL),
    part([0.0, 0.2, 0.0], [0.4, 0.28, 0.3], tex::CORE_DRILL),
    part([0.0, 0.4, 0.0], [0.3, 0.06, 0.08], tex::HANDLE),
];

pub fn parts(item: ItemId) -> &'static [Part] {
    match item {
        IRON_INGOT => &IRON_BAR,
        COPPER_INGOT => &COPPER_BAR,
        IRON_PLATE => &PLATE,
        IRON_ROD => &ROD,
        SCREW => &SCREWS,
        COPPER_WIRE => &WIRE,
        GEAR => &COG,
        ITEM_MINER_MK1 => &MINER_BODY_1,
        ITEM_MINER_MK2 => &MINER_BODY_2,
        crate::item::MINER_MK3 => &MINER_BODY_3,
        crate::item::MINER_MK4 => &MINER_BODY_4,
        crate::item::MINER_MK5 => &MINER_BODY_5,
        ITEM_COAL_MINER => &COAL_MINER_BODY,
        RED_PACK => &RED_FLASK,
        GREEN_PACK => &GREEN_FLASK,
        BLUE_PACK => &BLUE_FLASK,
        VIOLET_PACK => &VIOLET_FLASK,
        GOLD_PACK => &GOLD_FLASK,
        STONE_PICKAXE => &STONE_PICK,
        IRON_PICKAXE => &IRON_PICK,
        STONE_AXE => &STONE_HATCHET,
        IRON_AXE => &IRON_HATCHET,
        STONE_SHOVEL => &STONE_SPADE,
        IRON_SHOVEL => &IRON_SPADE,
        STEEL_INGOT => &STEEL_BAR,
        STEEL_PLATE => &STEEL_SHEET,
        STEEL_BEAM => &STEEL_GIRDER,
        STEEL_PICKAXE => &STEEL_PICK,
        STEEL_AXE => &STEEL_HATCHET,
        STEEL_SHOVEL => &STEEL_SPADE,
        SCANNER => &SCANNER_SET,
        SCANNER_MK2 => &SCANNER_MK2_SET,
        CORE_DRILL => &DRILL_SET,
        _ => {
            for find in [
                stock::parts,
                components::parts,
                wear::parts,
                craft::parts,
                space::parts,
                power::parts,
                belts::parts,
                machines::parts,
                placed::parts,
            ] {
                let model = find(item);
                if !model.is_empty() {
                    return model;
                }
            }
            &[]
        }
    }
}

#[cfg(test)]
mod tests;
