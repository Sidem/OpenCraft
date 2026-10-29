//! Item models of vehicles and containers: the three quad-rotor drones, the locomotive and wagon (scaled from their
//! models on the rails, `factory/trains/model.rs`), the rail signal, the fluid canisters and the light frame.
//! Add a const and a `parts` row.

use std::f32::consts::FRAC_PI_4;

use super::shapes::{cylinder, Build, EMPTY};
use super::{part, turned, Axis, Form, Part};
use crate::block::tex;
use crate::factory::trains::model::{CarBox, LOCOMOTIVE_BOXES, WAGON_BOXES};
use crate::item::{
    ItemId, ACID_CANISTER, CARGO_DRONE, CRUDE_CANISTER, DIESEL_CANISTER, DRONE, EMPTY_CANISTER, HEAVY_OIL_CANISTER,
    HYDROGEN_CANISTER, LIGHT_FRAME, LOCOMOTIVE, LUBRICANT_CANISTER, NAPHTHA_CANISTER, OXYGEN_CANISTER, PERSONAL_DRONE,
    PURE_WATER_CANISTER, RAIL_SIGNAL, ROCKET_FUEL_CANISTER, WAGON,
};

const METAL: u16 = tex::MAT_METAL;
const GUNMETAL: u16 = tex::MAT_GUNMETAL;
const RUBBER: u16 = tex::MAT_RUBBER;

pub(super) fn parts(item: ItemId) -> &'static [Part] {
    match item {
        DRONE => &DRONE_MODEL,
        PERSONAL_DRONE => &PERSONAL_DRONE_MODEL,
        CARGO_DRONE => &CARGO_DRONE_MODEL,
        LOCOMOTIVE => &LOCO,
        WAGON => &WAGON_MODEL,
        RAIL_SIGNAL => &SIGNAL,
        EMPTY_CANISTER => &EMPTY_DRUM,
        CRUDE_CANISTER => &CRUDE_DRUM,
        NAPHTHA_CANISTER => &NAPHTHA_DRUM,
        DIESEL_CANISTER => &DIESEL_DRUM,
        HEAVY_OIL_CANISTER => &HEAVY_OIL_DRUM,
        ACID_CANISTER => &ACID_DRUM,
        LUBRICANT_CANISTER => &LUBRICANT_DRUM,
        HYDROGEN_CANISTER => &HYDROGEN_DRUM,
        OXYGEN_CANISTER => &OXYGEN_DRUM,
        PURE_WATER_CANISTER => &PURE_WATER_DRUM,
        ROCKET_FUEL_CANISTER => &ROCKET_FUEL_DRUM,
        LIGHT_FRAME => &FRAME,
        _ => &[],
    }
}

/// A quad-rotor `s` times the size of a drone: skids, two crossed gunmetal arms with a motor at each tip, a white
/// body under a cover in `cover`, `under` slung beneath it, and a two-blade rotor on each motor.
const fn quad(cover: u16, s: f32, under: Part) -> [Part; 15] {
    let t = 0.233 * s;
    const fn motor(x: f32, z: f32, s: f32) -> Part {
        turned([x, 0.0, z], [0.08 * s, 0.08 * s, 0.08 * s], FRAC_PI_4, GUNMETAL)
    }
    const fn rotor(x: f32, z: f32, yaw: f32, s: f32) -> Part {
        turned([x, 0.05 * s, z], [0.32 * s, 0.008, 0.04 * s], yaw, RUBBER)
    }
    [
        part([-0.1 * s, -0.12 * s, 0.0], [0.03 * s, 0.03 * s, 0.36 * s], GUNMETAL),
        part([0.1 * s, -0.12 * s, 0.0], [0.03 * s, 0.03 * s, 0.36 * s], GUNMETAL),
        turned([0.0, -0.02 * s, 0.0], [0.66 * s, 0.04 * s, 0.05 * s], FRAC_PI_4, GUNMETAL),
        turned([0.0, -0.02 * s, 0.0], [0.66 * s, 0.04 * s, 0.05 * s], -FRAC_PI_4, GUNMETAL),
        motor(-t, -t, s),
        motor(t, -t, s),
        motor(-t, t, s),
        under,
        part([0.0, -0.02 * s, 0.0], [0.22 * s, 0.1 * s, 0.26 * s], tex::PLASTIC),
        part([0.0, 0.04 * s, 0.0], [0.16 * s, 0.03 * s, 0.2 * s], cover),
        motor(t, t, s),
        rotor(-t, -t, 0.3, s),
        rotor(t, -t, 1.2, s),
        rotor(-t, t, 2.0, s),
        rotor(t, t, 0.7, s),
    ]
}

/// The construction drone: an ember cover and a camera under its nose.
const DRONE_MODEL: [Part; 15] = quad(tex::MAT_EMBER, 1.0, part([0.0, -0.08, 0.12], [0.07, 0.05, 0.07], GUNMETAL));
/// The personal drone: smaller, a teal cover and a lamp under its nose.
const PERSONAL_DRONE_MODEL: [Part; 15] =
    quad(tex::MAT_AETHER, 0.75, part([0.0, -0.06, 0.09], [0.05, 0.03, 0.05], tex::LAMP_BLUE));
/// The cargo drone: a crate slung under it.
const CARGO_DRONE_MODEL: [Part; 15] =
    quad(tex::MAT_EMBER, 1.0, part([0.0, -0.19, 0.0], [0.26, 0.16, 0.26], tex::BOX_SIDE));

/// A train car's boxes as an item: scaled by `s`, nose to +x (the car's nose is at -z), its middle height `mid` at the
/// centre, sorted back to front (by x + y + z) for the icon.
const fn car<const N: usize>(boxes: &[CarBox; N], s: f32, mid: f32) -> [Part; N] {
    let mut out = [EMPTY; N];
    let mut i = 0;
    while i < N {
        let (x, y, z, size, layer, shape) = boxes[i];
        let c = [-z as f32 * s, (y as f32 - mid) * s, x as f32 * s];
        out[i] = part(c, [size[2] * s, size[1] * s, size[0] * s], layer);
        // The car's x is the item's z and the other way round.
        out[i].form = match shape {
            Form::Round(Axis::X, t) => Form::Round(Axis::Z, t),
            Form::Round(Axis::Z, t) => Form::Round(Axis::X, t),
            s => s,
        };
        let mut j = i;
        while j > 0 && depth(&out[j - 1]) > depth(&out[j]) {
            let p = out[j - 1];
            out[j - 1] = out[j];
            out[j] = p;
            j -= 1;
        }
        i += 1;
    }
    out
}

const fn depth(p: &Part) -> f32 {
    p.center[0] + p.center[1] + p.center[2]
}

const LOCO: [Part; 17] = car(&LOCOMOTIVE_BOXES, 0.35, 0.667);
const WAGON_MODEL: [Part; 21] = car(&WAGON_BOXES, 0.36, 0.515);

/// A rail signal: a gunmetal footing, a steel post, the signal head with green over red lamps under their hoods.
const SIGNAL: [Part; 7] = [
    part([0.0, -0.37, 0.0], [0.2, 0.06, 0.2], GUNMETAL),
    part([0.0, -0.06, 0.0], [0.05, 0.56, 0.05], METAL),
    part([0.0, 0.24, 0.0], [0.14, 0.3, 0.1], GUNMETAL),
    part([0.0, 0.32, 0.06], [0.07, 0.07, 0.02], tex::LAMP_GREEN),
    part([0.0, 0.17, 0.06], [0.07, 0.07, 0.02], tex::LAMP_RED),
    part([0.0, 0.365, 0.08], [0.1, 0.015, 0.05], GUNMETAL),
    part([0.0, 0.215, 0.08], [0.1, 0.015, 0.05], GUNMETAL),
];

/// A canister: a round steel drum painted in its fluid's colour (`paint`), as an oil drum is made: a gunmetal chime at
/// its foot and its top rim, two painted rolling hoops standing out of it in thirds, the painted lid inside the top chime and a brass
/// bung and vent on it. Stacked bottom up (the icon has no depth test, so a hoop drawn after a taller body would paint
/// over it).
const fn drum(paint: u16) -> [Part; 10] {
    Build::new()
        .with(cylinder([0.0, -0.28, 0.0], 0.43, 0.04, GUNMETAL))
        .with(cylinder([0.0, -0.19, 0.0], 0.42, 0.14, paint))
        .with(cylinder([0.0, -0.11, 0.0], 0.44, 0.02, paint))
        .with(cylinder([0.0, -0.01, 0.0], 0.42, 0.18, paint))
        .with(cylinder([0.0, 0.09, 0.0], 0.44, 0.02, paint))
        .with(cylinder([0.0, 0.17, 0.0], 0.42, 0.14, paint))
        .with(cylinder([0.0, 0.26, 0.0], 0.43, 0.04, GUNMETAL))
        .with(cylinder([0.0, 0.2825, 0.0], 0.39, 0.005, paint))
        .with(cylinder([-0.11, 0.29, -0.07], 0.04, 0.012, tex::MAT_BRASS))
        .with(cylinder([0.1, 0.295, 0.08], 0.07, 0.02, tex::MAT_BRASS))
        .done()
}
const EMPTY_DRUM: [Part; 10] = drum(tex::EMPTY_CANISTER);
const CRUDE_DRUM: [Part; 10] = drum(tex::CRUDE_CANISTER);
const NAPHTHA_DRUM: [Part; 10] = drum(tex::NAPHTHA_CANISTER);
const DIESEL_DRUM: [Part; 10] = drum(tex::DIESEL_CANISTER);
const HEAVY_OIL_DRUM: [Part; 10] = drum(tex::HEAVY_OIL_CANISTER);
const ACID_DRUM: [Part; 10] = drum(tex::ACID_CANISTER);
const LUBRICANT_DRUM: [Part; 10] = drum(tex::LUBRICANT_CANISTER);
const HYDROGEN_DRUM: [Part; 10] = drum(tex::HYDROGEN_CANISTER);
const OXYGEN_DRUM: [Part; 10] = drum(tex::OXYGEN_CANISTER);
const PURE_WATER_DRUM: [Part; 10] = drum(tex::PURE_WATER_CANISTER);
const ROCKET_FUEL_DRUM: [Part; 10] = drum(tex::ROCKET_FUEL_CANISTER);

/// A light frame: an aluminium space frame of two square rings of bars on four corner posts, each ring crossed by
/// a diagonal brace.
const fn ring(y: f32) -> [Part; 4] {
    [
        part([0.0, y, -0.37], [0.78, 0.05, 0.05], METAL),
        part([-0.37, y, 0.0], [0.05, 0.05, 0.78], METAL),
        part([0.37, y, 0.0], [0.05, 0.05, 0.78], METAL),
        part([0.0, y, 0.37], [0.78, 0.05, 0.05], METAL),
    ]
}
const FRAME: [Part; 14] = Build::new()
    .all(ring(-0.12))
    .with(turned([0.0, -0.12, 0.0], [1.0, 0.04, 0.04], FRAC_PI_4, METAL))
    .with(part([-0.37, 0.0, -0.37], [0.05, 0.29, 0.05], METAL))
    .with(part([0.37, 0.0, -0.37], [0.05, 0.29, 0.05], METAL))
    .with(part([-0.37, 0.0, 0.37], [0.05, 0.29, 0.05], METAL))
    .with(part([0.37, 0.0, 0.37], [0.05, 0.29, 0.05], METAL))
    .all(ring(0.12))
    .with(turned([0.0, 0.12, 0.0], [1.0, 0.04, 0.04], -FRAC_PI_4, METAL))
    .done();
