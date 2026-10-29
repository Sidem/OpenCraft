//! Item models of the space era: the rocket engine, the fuel tank, the four satellites and the drop pod. Add a const
//! and a `parts` row.

use super::shapes::{cylinder, faced, frustum, Build};
use super::{part, Part};
use crate::block::tex;
use crate::item::{
    ItemId, COMMS_SATELLITE, DROP_POD, FUEL_TANK, POWER_SATELLITE, ROCKET_ENGINE, SCIENCE_SATELLITE, SURVEY_SATELLITE,
};

const METAL: u16 = tex::MAT_METAL;
const GUNMETAL: u16 = tex::MAT_GUNMETAL;
const WHITE: u16 = tex::PLASTIC;

pub(super) fn parts(item: ItemId) -> &'static [Part] {
    match item {
        ROCKET_ENGINE => &ENGINE,
        FUEL_TANK => &TANK,
        SURVEY_SATELLITE => &SURVEY,
        COMMS_SATELLITE => &COMMS,
        POWER_SATELLITE => &POWER,
        SCIENCE_SATELLITE => &SCIENCE,
        DROP_POD => &POD,
        _ => &[],
    }
}

/// A rocket engine: a gunmetal bell widening downward in steps under the throat and the steel combustion chamber,
/// the injector plate, a brass turbopump and two feed pipes.
const ENGINE: [Part; 9] = Build::new()
    .with(cylinder([0.0, -0.33, 0.0], 0.6, 0.12, GUNMETAL))
    .with(cylinder([0.0, -0.2, 0.0], 0.48, 0.14, GUNMETAL))
    .with(cylinder([0.0, -0.07, 0.0], 0.34, 0.12, GUNMETAL))
    .with(cylinder([0.0, 0.03, 0.0], 0.2, 0.08, METAL))
    .with(cylinder([0.0, 0.15, 0.0], 0.3, 0.16, METAL))
    .with(cylinder([0.0, 0.25, 0.0], 0.36, 0.04, METAL))
    .with(part([-0.1, 0.33, -0.1], [0.14, 0.12, 0.12], tex::MAT_BRASS))
    .with(part([-0.16, 0.13, 0.0], [0.04, 0.24, 0.04], tex::MAT_BRASS))
    .with(part([0.16, 0.13, 0.0], [0.04, 0.24, 0.04], tex::MAT_BRASS))
    .done();

/// A rocket fuel tank: a tall white cylinder with domed ends, an ember band and a steel fitting on top.
const TANK: [Part; 6] = Build::new()
    .with(cylinder([0.0, -0.385, 0.0], 0.36, 0.04, WHITE))
    .with(cylinder([0.0, -0.1525, 0.0], 0.48, 0.425, WHITE))
    .with(cylinder([0.0, 0.1, 0.0], 0.49, 0.08, tex::MAT_EMBER))
    .with(cylinder([0.0, 0.2525, 0.0], 0.48, 0.225, WHITE))
    .with(cylinder([0.0, 0.385, 0.0], 0.36, 0.04, WHITE))
    .with(part([0.0, 0.42, 0.0], [0.08, 0.04, 0.08], METAL))
    .done();

/// A satellite: its bus (`bus`) between two solar wings on a steel strut, a thruster below, a beacon light (`light`)
/// on top, and what it carries (`payload`) last.
const fn satellite(bus: u16, light: u16, payload: [Part; 3]) -> [Part; 9] {
    Build::new()
        .with(faced([-0.31, 0.0, 0.0], [0.3, 0.02, 0.26], tex::SOLAR_TOP, METAL))
        .with(part([0.0, 0.0, 0.0], [0.36, 0.025, 0.025], METAL))
        .with(cylinder([0.0, -0.2, 0.0], 0.1, 0.06, GUNMETAL))
        .with(part([0.0, 0.0, 0.0], [0.24, 0.34, 0.24], bus))
        .with(part([0.08, 0.18, 0.08], [0.04, 0.02, 0.04], light))
        .with(faced([0.31, 0.0, 0.0], [0.3, 0.02, 0.26], tex::SOLAR_TOP, METAL))
        .all(payload)
        .done()
}

/// The survey satellite: gold foil, a camera with a dark lens and a sunshade on its front.
const SURVEY: [Part; 9] = satellite(
    tex::MAT_BRASS,
    tex::LAMP_YELLOW,
    [
        part([0.0, -0.02, 0.15], [0.12, 0.12, 0.08], GUNMETAL),
        part([0.0, -0.02, 0.195], [0.08, 0.08, 0.01], tex::FLASK_GLASS),
        part([0.0, 0.05, 0.16], [0.14, 0.02, 0.1], GUNMETAL),
    ],
);

/// The comms satellite: white, a shallow dish on a post on top, its feed horn in the middle.
const COMMS: [Part; 9] = satellite(
    WHITE,
    tex::LAMP_BLUE,
    [
        part([0.0, 0.2, 0.0], [0.03, 0.06, 0.03], METAL),
        frustum([0.0, 0.245, 0.0], 0.2, 0.03, 1.7, WHITE),
        cylinder([0.0, 0.275, 0.0], 0.03, 0.03, METAL),
    ],
);

/// The power satellite: ember, a panel on top and its emitter glowing red in front.
const POWER: [Part; 9] = satellite(
    tex::MAT_EMBER,
    tex::LAMP_RED,
    [
        faced([0.0, 0.19, 0.0], [0.2, 0.04, 0.2], tex::SOLAR_TOP, METAL),
        part([0.0, 0.0, 0.14], [0.16, 0.16, 0.04], GUNMETAL),
        part([0.0, 0.0, 0.165], [0.12, 0.12, 0.01], tex::LAMP_RED),
    ],
);

/// The science satellite: white, a glass dome on top and a sensor on a boom out of its front.
const SCIENCE: [Part; 9] = satellite(
    WHITE,
    tex::LAMP_GREEN,
    [
        part([0.0, 0.19, 0.0], [0.1, 0.04, 0.1], tex::FLASK_GLASS),
        part([0.0, 0.1, 0.26], [0.025, 0.025, 0.28], METAL),
        part([0.0, 0.1, 0.41], [0.06, 0.06, 0.06], GUNMETAL),
    ],
);

/// A drop pod: a white capsule narrowing in steps over its sooty heat shield, an ember band at its shoulder, a dark
/// window in front and the parachute housing on top.
const POD: [Part; 7] = Build::new()
    .with(cylinder([0.0, -0.41, 0.0], 0.7, 0.08, tex::SOOT))
    .with(cylinder([0.0, -0.23, 0.0], 0.6, 0.28, WHITE))
    .with(cylinder([0.0, -0.09, 0.0], 0.61, 0.03, tex::MAT_EMBER))
    .with(cylinder([0.0, 0.03, 0.0], 0.46, 0.24, WHITE))
    .with(cylinder([0.0, 0.225, 0.0], 0.3, 0.15, WHITE))
    .with(cylinder([0.0, 0.34, 0.0], 0.18, 0.08, GUNMETAL))
    .with(part([0.0, -0.22, 0.3], [0.14, 0.1, 0.012], tex::FLASK_GLASS))
    .done();
