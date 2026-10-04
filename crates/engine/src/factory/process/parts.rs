//! The models of the recipe machines and heavy processors as data: each spec's `parts` (`specs.rs`) are boxes
//! with a `Look` (`model.rs`). To give a processor a model: a `*_PARTS` const here.

use crate::block::tex;

use super::model::{part, Look, Part};
const BRICK: [u16; 3] = [tex::SMELTER_TOP, tex::SMELTER_SIDE, tex::FRAME];
const IVORY: [u16; 3] = [tex::CONSTRUCTOR_TOP, tex::CONSTRUCTOR_SIDE, tex::FRAME];
const FRAME: Look = Look::Tex([tex::FRAME; 3]);
const SOOT: Look = Look::Tex([tex::GENERATOR_SIDE; 3]);

/// A brick furnace: a dark upper ore mouth over a glowing crucible, a tall chimney, a status lamp.
pub(super) const SMELTER_PARTS: [Part; 9] = [
    part([0.0, -0.42, 0.0], [0.94, 0.16, 0.94], Look::Band(tex::FRAME)),
    part([0.0, -0.06, -0.1], [0.82, 0.67, 0.72], Look::Tex(BRICK)),
    part([0.0, 0.31, -0.1], [0.9, 0.1, 0.8], FRAME),
    part([0.0, 0.14, 0.32], [0.46, 0.23, 0.13], SOOT),
    part([0.0, -0.24, 0.34], [0.55, 0.2, 0.15], Look::Fire(tex::GENERATOR_SIDE)),
    part([0.0, -0.36, 0.46], [0.65, 0.07, 0.08], FRAME),
    part([-0.23, 0.48, -0.23], [0.27, 0.48, 0.27], Look::Tex(BRICK)),
    part([-0.23, 0.75, -0.23], [0.33, 0.07, 0.33], SOOT),
    part([0.33, 0.34, 0.32], [0.12, 0.08, 0.12], Look::Lamp),
];

/// An open press: tray, rear gantry, a ram and head that pump while working, a status lamp.
pub(super) const CONSTRUCTOR_PARTS: [Part; 9] = [
    part([0.0, -0.4, 0.0], [0.94, 0.18, 0.94], Look::Band(tex::CONSTRUCTOR_TOP)),
    part([0.0, -0.25, 0.14], [0.67, 0.08, 0.57], FRAME),
    part([-0.35, 0.0, -0.32], [0.13, 0.67, 0.16], Look::Tex(IVORY)),
    part([0.35, 0.0, -0.32], [0.13, 0.67, 0.16], Look::Tex(IVORY)),
    part([0.0, 0.35, -0.32], [0.85, 0.13, 0.22], Look::Tex(IVORY)),
    part([0.0, 0.23, -0.1], [0.18, 0.34, 0.18], Look::Press(0.5, [tex::DRILL; 3])),
    part([0.0, 0.04, -0.1], [0.48, 0.08, 0.48], Look::Press(1.0, [tex::IRON_PLATE; 3])),
    part([0.0, -0.36, 0.49], [0.64, 0.09, 0.06], Look::Tex([tex::COPPER_INGOT; 3])),
    part([0.39, 0.37, -0.31], [0.12, 0.08, 0.12], Look::Lamp),
];

const TEAL: [u16; 3] = [tex::ASSEMBLER_TOP, tex::ASSEMBLER_SIDE, tex::FRAME];
const STEEL: Look = Look::Tex([tex::STEEL; 3]);

/// A teal housing on a banded plinth, a gantry on its roof whose head works while assembling, a motor
/// casing high on its flank (the port hatches below come from the footprint) and a status lamp.
pub(super) const ASSEMBLER_PARTS: [Part; 9] = [
    part([0.0, -0.9, 0.0], [1.96, 0.2, 1.96], Look::Band(tex::FRAME)),
    part([0.0, -0.05, 0.0], [1.8, 1.5, 1.8], Look::Tex(TEAL)),
    part([0.0, 0.74, 0.0], [1.9, 0.1, 1.9], FRAME),
    part([-0.6, 1.0, -0.05], [0.12, 0.42, 0.12], STEEL),
    part([0.6, 1.0, -0.05], [0.12, 0.42, 0.12], STEEL),
    part([0.0, 1.2, -0.05], [1.32, 0.1, 0.14], STEEL),
    part([0.0, 0.98, -0.05], [0.28, 0.24, 0.28], Look::Press(1.0, [tex::IRON_PLATE; 3])),
    part([0.9, 0.3, -0.3], [0.2, 0.5, 0.7], Look::Tex([tex::FRAME, tex::MOTOR, tex::FRAME])),
    part([0.78, 0.86, 0.78], [0.14, 0.12, 0.14], Look::Lamp),
];

const FIREBRICK: [u16; 3] = [tex::BLAST_TOP, tex::BLAST_SIDE, tex::FRAME];

/// A firebrick stack on a banded plinth: a wide hearth under a narrower shaft and chimney, a tap hole
/// on the front that glows while it works, a hot-blast pipe up one flank and a status lamp.
pub(super) const BLAST_PARTS: [Part; 9] = [
    part([0.0, -1.4, 0.0], [1.96, 0.2, 1.96], Look::Band(tex::FRAME)),
    part([0.0, -0.55, 0.0], [1.8, 1.5, 1.8], Look::Tex(FIREBRICK)),
    part([0.0, 0.26, 0.0], [1.92, 0.12, 1.92], FRAME),
    part([0.0, 0.8, 0.0], [1.3, 1.0, 1.3], Look::Tex(FIREBRICK)),
    part([0.0, 1.38, 0.0], [1.42, 0.12, 1.42], FRAME),
    part([0.0, 1.2, 0.0], [0.6, 0.5, 0.6], SOOT),
    part([0.0, -0.4, 0.92], [0.7, 0.5, 0.12], Look::Fire(tex::GENERATOR_SIDE)),
    part([-0.98, 0.0, -0.5], [0.16, 2.0, 0.16], STEEL),
    part([0.78, 0.36, 0.78], [0.14, 0.12, 0.14], Look::Lamp),
];

const BLUE_TANK: [u16; 3] = [tex::BOILER_TOP, tex::BOILER_SIDE, tex::FRAME];

/// A riveted tank on a banded plinth with a glowing firebox door high on the front (the steam outlets are
/// below it, drawn by `steam_view.rs`), a roof plate, a steam stack and a status lamp.
pub(super) const BOILER_PARTS: [Part; 6] = [
    part([0.0, -0.9, 0.0], [1.96, 0.2, 1.96], Look::Band(tex::FRAME)),
    part([0.0, 0.0, 0.0], [1.8, 1.6, 1.8], Look::Tex(BLUE_TANK)),
    part([0.0, 0.86, 0.0], [1.9, 0.1, 1.9], FRAME),
    part([0.0, 0.3, 0.92], [0.8, 0.5, 0.12], Look::Fire(tex::GENERATOR_SIDE)),
    part([-0.5, 1.15, -0.5], [0.3, 0.5, 0.3], STEEL),
    part([0.78, 0.86, 0.78], [0.14, 0.12, 0.14], Look::Lamp),
];

const CASING: [u16; 3] = [tex::TURBINE_TOP, tex::TURBINE_SIDE, tex::FRAME];

/// A steel casing on a banded plinth, a rotor drum on top, a generator block at one end and a status lamp.
pub(super) const TURBINE_PARTS: [Part; 5] = [
    part([0.0, -0.9, 0.0], [2.96, 0.2, 1.96], Look::Band(tex::FRAME)),
    part([0.0, -0.1, 0.0], [2.8, 1.4, 1.8], Look::Tex(CASING)),
    part([0.4, 0.75, 0.0], [1.5, 0.5, 1.5], Look::Tex(CASING)),
    part([-1.15, 0.75, 0.0], [0.7, 0.6, 0.9], Look::Tex([tex::FRAME, tex::MOTOR, tex::FRAME])),
    part([1.3, 0.86, 0.78], [0.14, 0.12, 0.14], Look::Lamp),
];

const HAZARD: [u16; 3] = [tex::CRUSHER_TOP, tex::CRUSHER_SIDE, tex::FRAME];

/// A hazard-striped body on a base, a wide hopper on top whose steel jaw works while crushing, a lamp.
pub(super) const CRUSHER_PARTS: [Part; 5] = [
    part([0.0, -0.4, 0.0], [0.94, 0.18, 0.94], Look::Band(tex::FRAME)),
    part([0.0, -0.05, 0.0], [0.86, 0.6, 0.86], Look::Tex(HAZARD)),
    part([0.0, 0.36, 0.0], [0.96, 0.26, 0.96], Look::Tex(HAZARD)),
    part([0.0, 0.22, 0.0], [0.5, 0.16, 0.5], Look::Press(1.0, [tex::STEEL; 3])),
    part([0.36, 0.32, 0.36], [0.12, 0.08, 0.12], Look::Lamp),
];

const GRAPHITE: [u16; 3] = [tex::ARC_TOP, tex::ARC_SIDE, tex::FRAME];

/// A graphite housing on a banded plinth with a glowing arc slit on the front, two electrode rods that
/// pump on the roof while working, a roof plate and a status lamp.
pub(super) const ARC_PARTS: [Part; 7] = [
    part([0.0, -0.9, 0.0], [1.96, 0.2, 1.96], Look::Band(tex::FRAME)),
    part([0.0, -0.05, 0.0], [1.8, 1.5, 1.8], Look::Tex(GRAPHITE)),
    part([0.0, 0.74, 0.0], [1.9, 0.1, 1.9], FRAME),
    part([0.0, -0.1, 0.92], [1.1, 0.3, 0.12], Look::Fire(tex::ARC_SIDE)),
    part([-0.45, 1.0, 0.0], [0.24, 0.4, 0.24], Look::Press(0.6, [tex::STEEL; 3])),
    part([0.45, 1.0, 0.0], [0.24, 0.4, 0.24], Look::Press(0.6, [tex::STEEL; 3])),
    part([0.78, 0.86, 0.78], [0.14, 0.12, 0.14], Look::Lamp),
];

const CONCRETE_RINGS: [u16; 3] = [tex::SILO_TOP, tex::SILO_SIDE, tex::FRAME];

/// A tall concrete drum on a banded plinth with two steel rings, a roof and hatch, a status lamp.
pub(super) const SILO_PARTS: [Part; 6] = [
    part([0.0, -1.4, 0.0], [1.96, 0.2, 1.96], Look::Band(tex::FRAME)),
    part([0.0, -0.05, 0.0], [1.8, 2.7, 1.8], Look::Tex(CONCRETE_RINGS)),
    part([0.0, 0.5, 0.0], [1.9, 0.1, 1.9], FRAME),
    part([0.0, -0.6, 0.0], [1.9, 0.1, 1.9], FRAME),
    part([0.0, 1.45, 0.0], [1.9, 0.12, 1.9], Look::Tex([tex::SILO_TOP, tex::FRAME, tex::FRAME])),
    part([0.78, 1.36, 0.78], [0.14, 0.12, 0.14], Look::Lamp),
];
