//! Item models of worn and handheld gear: the jetpack and hover pack, hauler packs, boots, the exo frame, the mining
//! rig, fabricator gloves, the planner, the telescope and the sickle. Packs face the viewer with their outer side (+z), harness straps behind.
//! Add a const and a `parts` row.

use std::f32::consts::FRAC_PI_4;

use super::shapes::{bar_x, cylinder, frustum, Build};
use super::{part, turned, Part};
use crate::block::tex;
use crate::item::{
    ItemId, CRAFT_GLOVES, EXO_FRAME, HAULER_PACK, HAULER_PACK_MK2, HAULER_PACK_MK3, HAULER_PACK_MK4, HAULER_PACK_MK5,
    HOVER_PACK, JETPACK, MINING_RIG, PLANNER, SERVO_BOOTS, SICKLE, SPRING_BOOTS, TELESCOPE,
};

const METAL: u16 = tex::MAT_METAL;
const GUNMETAL: u16 = tex::MAT_GUNMETAL;
const RUBBER: u16 = tex::MAT_RUBBER;
const BRASS: u16 = tex::MAT_BRASS;
const WEBBING: u16 = tex::MAT_FABRIC;
const EMBER: u16 = tex::MAT_EMBER;
const TEAL: u16 = tex::MAT_AETHER;

pub(super) fn parts(item: ItemId) -> &'static [Part] {
    match item {
        JETPACK => &JETPACK_MODEL,
        HOVER_PACK => &HOVER_PACK_MODEL,
        HAULER_PACK => &HAULER_1,
        HAULER_PACK_MK2 => &HAULER_2,
        HAULER_PACK_MK3 => &HAULER_3,
        HAULER_PACK_MK4 => &HAULER_4,
        HAULER_PACK_MK5 => &HAULER_5,
        SPRING_BOOTS => &SPRING_PAIR,
        SERVO_BOOTS => &SERVO_PAIR,
        EXO_FRAME => &EXO,
        MINING_RIG => &RIG,
        CRAFT_GLOVES => &GLOVES,
        PLANNER => &TABLET,
        TELESCOPE => &TELESCOPE_MODEL,
        SICKLE => &SICKLE_MODEL,
        _ => &[],
    }
}

/// Two harness straps behind a pack.
const STRAPS: [Part; 2] =
    [part([-0.14, 0.02, -0.19], [0.06, 0.5, 0.03], WEBBING), part([0.14, 0.02, -0.19], [0.06, 0.5, 0.03], WEBBING)];

/// A jetpack tank at `x`, bottom up (the icon has no depth test, so a band drawn after a taller body would paint
/// over it): the flared steel nozzle, the gunmetal tank with an ember band, a steel cap.
const fn tank(x: f32) -> [Part; 5] {
    Build::new()
        .with(frustum([x, -0.28, 0.0], 0.15, 0.1, 0.7, METAL))
        .with(cylinder([x, -0.08, 0.0], 0.2, 0.3, GUNMETAL))
        .with(cylinder([x, 0.1, 0.0], 0.21, 0.06, EMBER))
        .with(cylinder([x, 0.2, 0.0], 0.2, 0.14, GUNMETAL))
        .with(part([x, 0.285, 0.0], [0.1, 0.03, 0.1], METAL))
        .done()
}

/// The coal jetpack: two gunmetal tanks with ember bands and steel caps on a webbing backplate, nozzles below.
const JETPACK_MODEL: [Part; 13] = Build::new()
    .all(STRAPS)
    .with(part([0.0, 0.0, -0.14], [0.46, 0.6, 0.06], WEBBING))
    .all(tank(-0.12))
    .all(tank(0.12))
    .done();

/// A hover pack's lift fan at `x`, bottom up: gunmetal with a glowing band round its middle.
const fn fan(x: f32) -> [Part; 3] {
    Build::new()
        .with(cylinder([x, -0.165, -0.02], 0.2, 0.03, GUNMETAL))
        .with(cylinder([x, -0.14, -0.02], 0.21, 0.02, tex::LAMP_BLUE))
        .with(cylinder([x, -0.115, -0.02], 0.2, 0.03, GUNMETAL))
        .done()
}

/// The hover pack: a white shell on its harness, a strut through it to a lift fan at each side, and a light strip
/// down its back.
const HOVER_PACK_MODEL: [Part; 11] = Build::new()
    .all(STRAPS)
    .with(part([0.0, -0.14, -0.02], [0.56, 0.04, 0.06], METAL))
    .all(fan(-0.28))
    .with(part([0.0, 0.04, -0.06], [0.4, 0.5, 0.22], tex::PLASTIC))
    .all(fan(0.28))
    .with(part([0.0, 0.1, 0.053], [0.06, 0.3, 0.008], tex::LAMP_BLUE))
    .done();

/// A hauler pack: a webbing rucksack on a frame of wood (Mk1) or steel, banded in its tier's colour, with a flap, a
/// front pocket and two buckles. The bag is stacked bottom up round the band (see `tank`).
const fn hauler(tier: u8) -> [Part; 11] {
    let frame = if tier == 0 { tex::HANDLE } else { METAL };
    [
        part([-0.27, -0.03, -0.1], [0.04, 0.64, 0.04], frame),
        STRAPS[0],
        STRAPS[1],
        part([0.0, -0.165, 0.0], [0.48, 0.33, 0.32], WEBBING),
        part([0.0, 0.04, 0.0], [0.5, 0.07, 0.34], tex::stripe(tier)),
        part([0.0, 0.15, 0.0], [0.48, 0.15, 0.32], WEBBING),
        part([0.0, 0.2, 0.02], [0.5, 0.12, 0.34], WEBBING),
        part([0.0, -0.16, 0.18], [0.34, 0.22, 0.06], WEBBING),
        part([-0.1, 0.15, 0.195], [0.05, 0.06, 0.02], BRASS),
        part([0.1, 0.15, 0.195], [0.05, 0.06, 0.02], BRASS),
        part([0.27, -0.03, -0.1], [0.04, 0.64, 0.04], frame),
    ]
}
const HAULER_1: [Part; 11] = hauler(0);
const HAULER_2: [Part; 11] = hauler(1);
const HAULER_3: [Part; 11] = hauler(2);
const HAULER_4: [Part; 11] = hauler(3);
const HAULER_5: [Part; 11] = hauler(4);

/// One boot at `x`, raised by `lift`: a rubber sole, a gunmetal foot and shaft, a cuff in `accent`.
const fn boot(x: f32, lift: f32, accent: u16) -> [Part; 4] {
    [
        part([x, -0.31 + lift, 0.04], [0.16, 0.04, 0.38], RUBBER),
        part([x, -0.24 + lift, 0.07], [0.15, 0.1, 0.3], GUNMETAL),
        part([x, -0.06 + lift, -0.06], [0.16, 0.28, 0.17], GUNMETAL),
        part([x, 0.1 + lift, -0.06], [0.175, 0.05, 0.185], accent),
    ]
}

/// A steel coil under a spring boot's heel, on a rubber pad.
const fn spring(x: f32) -> [Part; 3] {
    [
        part([x, -0.32, 0.04], [0.14, 0.03, 0.3], RUBBER),
        turned([x, -0.285, -0.04], [0.11, 0.025, 0.11], 0.0, METAL),
        turned([x, -0.255, -0.04], [0.11, 0.025, 0.11], FRAC_PI_4, METAL),
    ]
}

/// Spring boots: a pair with ember cuffs, raised on steel coils.
const SPRING_PAIR: [Part; 14] =
    Build::new().all(spring(-0.12)).all(boot(-0.12, 0.08, EMBER)).all(spring(0.12)).all(boot(0.12, 0.08, EMBER)).done();

/// The outer side of a servo boot at `x` (`out` is -1 or 1): a strut up the shaft, the ankle servo and its light.
const fn ankle(x: f32, out: f32) -> [Part; 3] {
    [
        part([x + out * 0.09, -0.08, -0.06], [0.02, 0.26, 0.03], METAL),
        part([x + out * 0.095, -0.2, -0.02], [0.04, 0.1, 0.1], METAL),
        part([x + out * 0.118, -0.2, -0.02], [0.006, 0.03, 0.03], tex::LAMP_BLUE),
    ]
}

/// Servo boots: a pair with teal cuffs, each with an ankle servo on its outer side.
const SERVO_PAIR: [Part; 14] = Build::new()
    .all(ankle(-0.12, -1.0))
    .all(boot(-0.12, 0.0, TEAL))
    .all(boot(0.12, 0.0, TEAL))
    .all(ankle(0.12, 1.0))
    .done();

/// An exo frame: a gunmetal power pack and spine with a shoulder bar, a webbing hip belt with ember servos, steel
/// leg struts with teal knees and foot plates, and ember shoulder pads.
const EXO: [Part; 17] = [
    part([0.0, 0.14, -0.17], [0.22, 0.3, 0.1], GUNMETAL),
    part([0.0, 0.295, -0.17], [0.12, 0.01, 0.05], tex::LAMP_BLUE),
    part([0.0, 0.1, -0.1], [0.08, 0.62, 0.06], GUNMETAL),
    part([0.0, 0.37, -0.08], [0.5, 0.06, 0.08], GUNMETAL),
    part([0.0, -0.12, 0.0], [0.44, 0.08, 0.3], WEBBING),
    part([-0.15, -0.32, 0.0], [0.05, 0.3, 0.05], METAL),
    part([-0.15, -0.3, 0.0], [0.08, 0.08, 0.08], TEAL),
    part([-0.15, -0.46, 0.03], [0.1, 0.03, 0.16], GUNMETAL),
    part([0.15, -0.32, 0.0], [0.05, 0.3, 0.05], METAL),
    part([0.15, -0.3, 0.0], [0.08, 0.08, 0.08], TEAL),
    part([0.15, -0.46, 0.03], [0.1, 0.03, 0.16], GUNMETAL),
    part([-0.24, -0.12, 0.0], [0.07, 0.12, 0.12], EMBER),
    part([0.24, -0.12, 0.0], [0.07, 0.12, 0.12], EMBER),
    part([-0.22, 0.37, -0.04], [0.12, 0.07, 0.16], EMBER),
    part([0.22, 0.37, -0.04], [0.12, 0.07, 0.16], EMBER),
    part([-0.06, 0.37, -0.03], [0.03, 0.03, 0.01], METAL),
    part([0.06, 0.37, -0.03], [0.03, 0.03, 0.01], METAL),
];

/// A mining rig: a gunmetal back unit with a brass cap and its glowing core on a webbing harness, an arm out to its
/// right holding the ember drill motor, its steel bit stepping down to a point.
const RIG: [Part; 10] = Build::new()
    .all(STRAPS)
    .with(part([0.0, 0.1, -0.08], [0.36, 0.36, 0.16], GUNMETAL))
    .with(part([0.0, 0.3, -0.08], [0.38, 0.04, 0.18], BRASS))
    .with(part([0.0, 0.1, 0.005], [0.12, 0.12, 0.012], tex::LAMP_RED))
    .with(part([0.24, 0.12, 0.0], [0.08, 0.08, 0.24], METAL))
    .with(cylinder([0.26, -0.02, 0.08], 0.16, 0.18, EMBER))
    .with(turned([0.26, -0.15, 0.08], [0.12, 0.08, 0.12], 0.0, METAL))
    .with(turned([0.26, -0.22, 0.08], [0.08, 0.07, 0.08], 0.4, METAL))
    .with(turned([0.26, -0.28, 0.08], [0.04, 0.06, 0.04], 0.8, METAL))
    .done();

/// A fabricator glove at `x` lying palm down, its thumb toward `inner` (-1 or 1): an ember cuff, the back of the
/// hand with its glowing node, four fingers and the thumb.
const fn glove(x: f32, inner: f32) -> [Part; 8] {
    [
        part([x, -0.03, -0.16], [0.22, 0.08, 0.12], EMBER),
        part([x, -0.04, 0.0], [0.2, 0.06, 0.22], RUBBER),
        part([x, -0.005, 0.0], [0.08, 0.012, 0.08], tex::LAMP_BLUE),
        turned([x + inner * 0.12, -0.05, 0.03], [0.05, 0.04, 0.12], inner * 0.6, RUBBER),
        part([x - 0.075, -0.05, 0.16], [0.04, 0.04, 0.1], RUBBER),
        part([x - 0.025, -0.05, 0.175], [0.04, 0.04, 0.13], RUBBER),
        part([x + 0.025, -0.05, 0.175], [0.04, 0.04, 0.13], RUBBER),
        part([x + 0.075, -0.05, 0.165], [0.04, 0.04, 0.11], RUBBER),
    ]
}

/// Fabricator gloves: a pair, thumbs inward.
const GLOVES: [Part; 16] = Build::new().all(glove(-0.2, 1.0)).all(glove(0.2, -1.0)).done();

/// The telescope lying along x: a gunmetal eyepiece, a steel middle draw and a brass objective tube, glass at the far end.
const TELESCOPE_MODEL: [Part; 4] = Build::new()
    .with(bar_x([-0.41, 0.0, 0.0], 0.12, 0.14, GUNMETAL))
    .with(bar_x([-0.2, 0.0, 0.0], 0.3, 0.22, METAL))
    .with(bar_x([0.18, 0.0, 0.0], 0.5, 0.3, BRASS))
    .with(part([0.435, 0.0, 0.0], [0.02, 0.24, 0.24], tex::FLASK_GLASS))
    .done();

/// The sickle standing upright: a wooden grip with a steel ferrule, the blade rising from it and hooking round to the
/// right (a placeholder crescent made of small boxes).
const SICKLE_MODEL: [Part; 7] = [
    part([0.0, -0.15, 0.0], [0.07, 0.55, 0.07], tex::HANDLE),
    part([0.0, 0.14, 0.0], [0.1, 0.05, 0.1], METAL),
    part([0.0, 0.2, 0.0], [0.05, 0.1, 0.035], METAL),
    part([0.06, 0.29, 0.0], [0.11, 0.1, 0.03], METAL),
    part([0.17, 0.35, 0.0], [0.14, 0.08, 0.03], METAL),
    part([0.28, 0.31, 0.0], [0.09, 0.1, 0.03], METAL),
    part([0.32, 0.22, 0.0], [0.04, 0.1, 0.025], METAL),
];

/// The planner: a gunmetal tablet with a rubber grip, its blueprint screen in front and a stylus at its side.
const TABLET: [Part; 5] = [
    part([0.0, 0.0, 0.0], [0.5, 0.6, 0.06], GUNMETAL),
    part([0.0, -0.3, 0.0], [0.3, 0.1, 0.09], RUBBER),
    part([0.0, 0.03, 0.033], [0.42, 0.46, 0.008], tex::PLANNER),
    part([0.28, 0.0, 0.0], [0.025, 0.44, 0.025], METAL),
    part([0.28, 0.23, 0.0], [0.035, 0.04, 0.035], EMBER),
];
