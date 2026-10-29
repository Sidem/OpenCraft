//! Item models of made components: upgrade kits, the motor, circuit, processor, servo and actuator, the drone cell
//! and guidance module, batteries and fuel cells, the AI accelerator and avionics. Built from the plain materials
//! (`tex::MAT_*`) plus a few printed faces (a board's traces, a chip's lid, a screen). Add a const and a `parts` row.

use super::shapes::{bar_x, cylinder, faced, frustum, Build};
use super::{part, Part};
use crate::block::tex;
use crate::item::{
    ItemId, ACTUATOR, AI_ACCELERATOR, AVIONICS, BATTERY, BLUE_KIT, CIRCUIT, DRONE_CELL, FUEL_CELL, GOLD_KIT, GREEN_KIT,
    GUIDANCE_MODULE, MOTOR, PROCESSOR, SERVO, VIOLET_KIT,
};

const METAL: u16 = tex::MAT_METAL;
const GUNMETAL: u16 = tex::MAT_GUNMETAL;
const RUBBER: u16 = tex::MAT_RUBBER;
const BRASS: u16 = tex::MAT_BRASS;
const TEAL: u16 = tex::MAT_AETHER;

pub(super) fn parts(item: ItemId) -> &'static [Part] {
    match item {
        GREEN_KIT => &KIT_2,
        BLUE_KIT => &KIT_3,
        VIOLET_KIT => &KIT_4,
        GOLD_KIT => &KIT_5,
        MOTOR => &MOTOR_MODEL,
        CIRCUIT => &BOARD,
        PROCESSOR => &CHIP,
        SERVO => &SERVO_MODEL,
        ACTUATOR => &ACTUATOR_MODEL,
        DRONE_CELL => &CELL_PACK,
        GUIDANCE_MODULE => &GUIDANCE,
        BATTERY => &BATTERY_MODEL,
        FUEL_CELL => &FUEL_ROD,
        AI_ACCELERATOR => &ACCELERATOR_CARD,
        AVIONICS => &AVIONICS_BOX,
        _ => &[],
    }
}

/// An upgrade kit: a gunmetal case under a lid in its tier's colour, a rubber grip on two posts, two latches.
const fn kit(tier: u8) -> [Part; 7] {
    [
        part([0.0, -0.08, 0.0], [0.74, 0.32, 0.44], GUNMETAL),
        part([0.0, 0.11, 0.0], [0.76, 0.06, 0.46], tex::stripe(tier)),
        part([-0.15, 0.17, 0.0], [0.04, 0.06, 0.05], METAL),
        part([0.15, 0.17, 0.0], [0.04, 0.06, 0.05], METAL),
        part([0.0, 0.21, 0.0], [0.36, 0.04, 0.07], RUBBER),
        part([-0.22, 0.05, 0.225], [0.07, 0.08, 0.02], METAL),
        part([0.22, 0.05, 0.225], [0.07, 0.08, 0.02], METAL),
    ]
}
const KIT_2: [Part; 7] = kit(1);
const KIT_3: [Part; 7] = kit(2);
const KIT_4: [Part; 7] = kit(3);
const KIT_5: [Part; 7] = kit(4);

/// An electric motor on its side: a teal body with two cooling ribs on a gunmetal foot, its end bell behind, a
/// terminal box on top and the shaft out of the front.
const MOTOR_MODEL: [Part; 7] = Build::new()
    .with(part([0.0, -0.22, 0.0], [0.4, 0.06, 0.3], GUNMETAL))
    .with(bar_x([-0.29, 0.0, 0.0], 0.06, 0.38, GUNMETAL))
    .with(bar_x([0.0, 0.0, 0.0], 0.52, 0.42, TEAL))
    .with(bar_x([-0.12, 0.0, 0.0], 0.04, 0.46, GUNMETAL))
    .with(bar_x([0.12, 0.0, 0.0], 0.04, 0.46, GUNMETAL))
    .with(part([0.0, 0.24, 0.0], [0.16, 0.08, 0.14], GUNMETAL))
    .with(part([0.33, 0.0, 0.0], [0.14, 0.07, 0.07], METAL))
    .done();

/// A circuit board: the printed board, a connector at its back, two capacitors, a black chip, two resistors and the
/// gold contacts along its front edge.
const BOARD: [Part; 8] = Build::new()
    .with(part([0.0, -0.035, 0.0], [0.68, 0.03, 0.54], tex::CIRCUIT))
    .with(part([0.08, -0.005, -0.22], [0.3, 0.03, 0.05], RUBBER))
    .with(cylinder([-0.22, 0.015, -0.08], 0.08, 0.07, METAL))
    .with(cylinder([-0.22, 0.015, 0.06], 0.08, 0.07, METAL))
    .with(part([0.06, -0.005, 0.0], [0.2, 0.03, 0.2], RUBBER))
    .with(part([0.24, -0.012, 0.08], [0.07, 0.016, 0.03], tex::MAT_EMBER))
    .with(part([0.24, -0.012, 0.15], [0.07, 0.016, 0.03], tex::MAT_EMBER))
    .with(part([0.0, -0.034, 0.268], [0.46, 0.031, 0.008], BRASS))
    .done();

/// A processor: a green substrate under the nickel lid with its markings, two tiny capacitors in front.
const CHIP: [Part; 4] = [
    part([0.0, -0.04, 0.0], [0.54, 0.04, 0.54], tex::CIRCUIT),
    faced([0.0, 0.0, 0.0], [0.36, 0.04, 0.36], tex::PROCESSOR, METAL),
    part([-0.2, -0.01, 0.21], [0.05, 0.02, 0.03], BRASS),
    part([0.2, -0.01, 0.21], [0.05, 0.02, 0.03], BRASS),
];

/// A servo: its lead out of the back, a gunmetal case with a teal label band on mounting tabs, the output spline
/// and its white horn on top.
const SERVO_MODEL: [Part; 7] = Build::new()
    .with(part([-0.27, -0.12, 0.0], [0.1, 0.04, 0.08], RUBBER))
    .with(part([0.0, -0.155, 0.0], [0.44, 0.09, 0.22], GUNMETAL))
    .with(part([0.0, -0.06, 0.0], [0.445, 0.1, 0.225], TEAL))
    .with(part([0.0, 0.045, 0.0], [0.44, 0.11, 0.22], GUNMETAL))
    .with(part([0.0, 0.04, 0.0], [0.58, 0.03, 0.2], GUNMETAL))
    .with(cylinder([0.1, 0.125, 0.0], 0.1, 0.05, METAL))
    .with(part([0.1, 0.16, 0.0], [0.07, 0.025, 0.32], tex::PLASTIC))
    .done();

/// A linear actuator: a teal cylinder between two clevises, the drive motor slung under its back, a collar, and
/// the polished rod out of the front.
const ACTUATOR_MODEL: [Part; 6] = Build::new()
    .with(part([-0.45, 0.02, 0.0], [0.06, 0.12, 0.1], GUNMETAL))
    .with(bar_x([-0.3, -0.1, 0.0], 0.2, 0.14, GUNMETAL))
    .with(bar_x([-0.13, 0.02, 0.0], 0.56, 0.2, TEAL))
    .with(bar_x([0.16, 0.02, 0.0], 0.04, 0.23, GUNMETAL))
    .with(bar_x([0.31, 0.02, 0.0], 0.26, 0.08, METAL))
    .with(part([0.46, 0.02, 0.0], [0.06, 0.12, 0.1], GUNMETAL))
    .done();

/// A drone's power cell: a white pack with a rubber grip band, brass terminals on top and three charge lights.
const CELL_PACK: [Part; 8] = [
    part([0.0, -0.23, 0.0], [0.42, 0.1, 0.28], tex::PLASTIC),
    part([0.0, -0.12, 0.0], [0.44, 0.12, 0.3], RUBBER),
    part([0.0, 0.06, 0.0], [0.42, 0.24, 0.28], tex::PLASTIC),
    part([-0.1, 0.2, 0.0], [0.07, 0.04, 0.07], BRASS),
    part([0.1, 0.2, 0.0], [0.07, 0.04, 0.07], BRASS),
    part([-0.1, 0.06, 0.145], [0.05, 0.03, 0.012], tex::LAMP_GREEN),
    part([0.0, 0.06, 0.145], [0.05, 0.03, 0.012], tex::LAMP_GREEN),
    part([0.1, 0.06, 0.145], [0.05, 0.03, 0.012], tex::LAMP_GREEN),
];

/// A guidance module: a flat gunmetal housing, an antenna at its back, the gyro under a glass dome with its light,
/// and a brass plug in front.
const GUIDANCE: [Part; 6] = Build::new()
    .with(part([0.0, -0.02, 0.0], [0.5, 0.06, 0.5], GUNMETAL))
    .with(part([0.17, 0.06, -0.17], [0.025, 0.14, 0.025], METAL))
    .with(cylinder([0.0, 0.02, 0.0], 0.26, 0.02, METAL))
    .with(cylinder([0.0, 0.05, 0.0], 0.18, 0.05, tex::FLASK_GLASS))
    .with(cylinder([0.0, 0.08, 0.0], 0.08, 0.02, tex::LAMP_BLUE))
    .with(part([0.0, -0.02, 0.26], [0.16, 0.04, 0.025], BRASS))
    .done();

/// A battery: a teal cell on a steel base ring, a gunmetal band, a steel top plate and the terminal. Round bodies
/// here are stacked bottom up: the icon has no depth test, so a band drawn after a taller body would paint over it.
const BATTERY_MODEL: [Part; 6] = Build::new()
    .with(cylinder([0.0, -0.27, 0.0], 0.35, 0.04, METAL))
    .with(cylinder([0.0, -0.15, 0.0], 0.34, 0.2, TEAL))
    .with(cylinder([0.0, 0.0, 0.0], 0.35, 0.1, GUNMETAL))
    .with(cylinder([0.0, 0.1275, 0.0], 0.34, 0.155, TEAL))
    .with(cylinder([0.0, 0.22, 0.0], 0.33, 0.03, METAL))
    .with(cylinder([0.0, 0.26, 0.0], 0.1, 0.05, METAL))
    .done();

/// A nuclear fuel cell: a round steel rod between chamfered gunmetal end caps, a glowing green window band round its
/// middle, a lifting eye on top. Stacked bottom up (the icon has no depth test).
const FUEL_ROD: [Part; 6] = Build::new()
    .with(frustum([0.0, -0.37, 0.0], 0.24, 0.06, 1.17, GUNMETAL))
    .with(cylinder([0.0, -0.225, 0.0], 0.24, 0.23, METAL))
    .with(cylinder([0.0, 0.0, 0.0], 0.25, 0.22, tex::LAMP_GREEN))
    .with(cylinder([0.0, 0.225, 0.0], 0.24, 0.23, METAL))
    .with(frustum([0.0, 0.37, 0.0], 0.28, 0.06, 0.86, GUNMETAL))
    .with(bar_x([0.0, 0.43, 0.0], 0.03, 0.08, METAL))
    .done();

/// An AI accelerator card: a board, a steel bracket at one end, a gunmetal shroud with two fans and an ember stripe,
/// and gold contacts under its front edge.
const ACCELERATOR_CARD: [Part; 9] = Build::new()
    .with(part([0.0, -0.045, 0.0], [0.74, 0.03, 0.48], tex::CIRCUIT))
    .with(part([-0.39, -0.01, 0.0], [0.02, 0.1, 0.48], METAL))
    .with(part([0.02, -0.005, 0.0], [0.66, 0.05, 0.42], GUNMETAL))
    .with(cylinder([-0.16, 0.025, 0.0], 0.24, 0.01, RUBBER))
    .with(cylinder([0.17, 0.025, 0.0], 0.24, 0.01, RUBBER))
    .with(part([-0.16, 0.032, 0.0], [0.06, 0.006, 0.06], METAL))
    .with(part([0.17, 0.032, 0.0], [0.06, 0.006, 0.06], METAL))
    .with(part([0.02, 0.0, 0.211], [0.6, 0.012, 0.004], tex::MAT_EMBER))
    .with(part([0.08, -0.05, 0.244], [0.36, 0.016, 0.01], BRASS))
    .done();

/// An avionics unit: a gunmetal box with cooling ribs and two plugs on top, a screen and two status lights in front.
const AVIONICS_BOX: [Part; 9] = [
    part([0.0, -0.01, 0.0], [0.56, 0.12, 0.46], GUNMETAL),
    part([-0.08, 0.06, -0.15], [0.32, 0.02, 0.03], METAL),
    part([-0.08, 0.06, -0.09], [0.32, 0.02, 0.03], METAL),
    part([-0.08, 0.06, -0.03], [0.32, 0.02, 0.03], METAL),
    part([0.18, 0.065, -0.1], [0.08, 0.03, 0.06], BRASS),
    part([0.18, 0.065, 0.0], [0.08, 0.03, 0.06], BRASS),
    part([-0.06, -0.01, 0.232], [0.28, 0.07, 0.006], tex::SCANNER),
    part([0.15, 0.0, 0.232], [0.04, 0.025, 0.006], tex::LAMP_GREEN),
    part([0.21, 0.0, 0.232], [0.04, 0.025, 0.006], tex::LAMP_YELLOW),
];
