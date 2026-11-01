//! The models of the recipe machines and heavy processors as data: each spec's `parts` (`specs.rs`) are boxes
//! with a `Look` (`model.rs`). To give a processor a model: a `*_PARTS` const here.

use crate::block::tex;

use super::model::{leaned, part, round, round_x, tapered, Look, Part};
const BRICK: [u16; 3] = [tex::SMELTER_TOP, tex::SMELTER_SIDE, tex::FRAME];
const IVORY: [u16; 3] = [tex::CONSTRUCTOR_TOP, tex::CONSTRUCTOR_SIDE, tex::FRAME];
const FRAME: Look = Look::Tex([tex::FRAME; 3]);
const SOOT: Look = Look::Tex([tex::SOOT; 3]);

/// The product display of a one-cell machine (`display.rs`): a slot this size, standing on a roof.
pub(super) const SMALL_DISPLAY: [f32; 3] = [0.36, 0.44, 0.36];
/// The product display of a bigger machine.
pub(super) const DISPLAY: [f32; 3] = [0.6, 0.6, 0.6];

/// A brick furnace: a dark upper ore mouth over a glowing crucible, a short chimney that smokes while it works (lower
/// than the display beside it, which shows over it), a status lamp, the product display on its roof.
pub(super) const SMELTER_PARTS: [Part; 11] = [
    part([0.0, -0.42, 0.0], [0.94, 0.16, 0.94], Look::Band(tex::FRAME)),
    part([0.0, -0.06, -0.1], [0.82, 0.67, 0.72], Look::Tex(BRICK)),
    part([0.0, 0.31, -0.1], [0.9, 0.1, 0.8], FRAME),
    part([0.0, 0.14, 0.32], [0.46, 0.23, 0.13], SOOT),
    part([0.0, -0.24, 0.34], [0.55, 0.2, 0.15], Look::Fire(tex::SOOT)),
    part([0.0, -0.31, 0.46], [0.65, 0.07, 0.08], FRAME), // above the Mk studs
    part([-0.23, 0.375, -0.23], [0.27, 0.27, 0.27], Look::Tex(BRICK)),
    part([-0.23, 0.535, -0.23], [0.33, 0.07, 0.33], SOOT),
    part([-0.23, 0.6, -0.23], [0.5, 0.0, 0.0], Look::Smoke),
    part([0.33, 0.34, 0.32], [0.12, 0.08, 0.12], Look::Lamp),
    part([0.12, 0.58, -0.05], SMALL_DISPLAY, Look::Product),
];

/// An open press: tray, rear gantry, a ram and head that pump while working, a status lamp, the product display on
/// the gantry.
pub(super) const CONSTRUCTOR_PARTS: [Part; 10] = [
    part([0.0, -0.4, 0.0], [0.94, 0.18, 0.94], Look::Band(tex::CONSTRUCTOR_TOP)),
    part([0.0, -0.25, 0.14], [0.67, 0.08, 0.57], FRAME),
    part([-0.35, 0.0, -0.32], [0.13, 0.67, 0.16], Look::Tex(IVORY)),
    part([0.35, 0.0, -0.32], [0.13, 0.67, 0.16], Look::Tex(IVORY)),
    part([0.0, 0.35, -0.32], [0.85, 0.13, 0.22], Look::Tex(IVORY)),
    part([0.0, 0.23, -0.1], [0.18, 0.34, 0.18], Look::Press(0.5, [tex::DRILL; 3])),
    part([0.0, 0.04, -0.1], [0.48, 0.08, 0.48], Look::Press(1.0, [tex::IRON_PLATE; 3])),
    part([0.0, -0.32, 0.49], [0.64, 0.04, 0.06], Look::Tex([tex::COPPER_INGOT; 3])), // above the Mk studs
    part([0.39, 0.37, -0.31], [0.12, 0.08, 0.12], Look::Lamp),
    part([0.0, 0.635, -0.32], SMALL_DISPLAY, Look::Product),
];

const TEAL: [u16; 3] = [tex::ASSEMBLER_TOP, tex::ASSEMBLER_SIDE, tex::FRAME];
const CABINET: Look = Look::Tex([tex::FRAME, tex::ASSEMBLER_SIDE, tex::FRAME]);
const STEEL: Look = Look::Tex([tex::STEEL; 3]);
const JOINT: Look = Look::Tex([tex::MAT_GUNMETAL; 3]);

/// A robot cell: a cabinet bed on a banded plinth (the port hatches on it come from the footprint) under a work table
/// with a conveyor from the back inlet to the front outlet and a fixture at its middle. The two orange arms that carry
/// each batch's inputs to the fixture, and the hammer that hangs from the gantry, move (`Look::Work`); a guarded frame
/// carries the gantry, whose carriage holds the product display at the top centre; a drive cabinet at the back and a
/// stack light on a roof corner.
pub(super) const ASSEMBLER_PARTS: [Part; 22] = [
    part([0.0, -0.9, 0.0], [1.96, 0.2, 1.96], Look::Band(tex::FRAME)),
    part([0.0, -0.47, 0.0], [1.8, 0.66, 1.8], CABINET),
    part([0.0, -0.1, 0.0], [1.88, 0.08, 1.88], Look::Tex([tex::ASSEMBLER_TOP, tex::FRAME, tex::FRAME])),
    part([0.0, -0.03, 0.0], [0.42, 0.06, 1.92], Look::Tex([tex::MAT_RUBBER, tex::FRAME, tex::FRAME])),
    part([0.0, 0.04, 0.0], [0.3, 0.08, 0.3], STEEL),
    // The frame: corner posts, a roof ring, a guard rail round the back and sides (the front stays open).
    part([-0.86, 0.43, -0.86], [0.1, 0.98, 0.1], FRAME),
    part([0.86, 0.43, -0.86], [0.1, 0.98, 0.1], FRAME),
    part([-0.86, 0.43, 0.86], [0.1, 0.98, 0.1], FRAME),
    part([0.86, 0.43, 0.86], [0.1, 0.98, 0.1], FRAME),
    part([0.0, 0.96, -0.86], [1.82, 0.08, 0.1], FRAME),
    part([0.0, 0.96, 0.86], [1.82, 0.08, 0.1], FRAME),
    part([-0.86, 0.96, 0.0], [0.1, 0.08, 1.62], FRAME),
    part([0.86, 0.96, 0.0], [0.1, 0.08, 1.62], FRAME),
    part([0.0, 0.5, -0.86], [1.62, 0.05, 0.04], Look::Tex([tex::MAT_AETHER; 3])),
    part([-0.86, 0.5, 0.0], [0.04, 0.05, 1.62], Look::Tex([tex::MAT_AETHER; 3])),
    part([0.86, 0.5, 0.0], [0.04, 0.05, 1.62], Look::Tex([tex::MAT_AETHER; 3])),
    // The arms, the inputs they carry, the work cloud, the gantry's hammer and the product (`assembly_view.rs`).
    part([0.0, 0.08, 0.0], [0.0; 3], Look::Work),
    // The overhead gantry: its beam and the carriage.
    part([0.0, 0.96, 0.0], [1.62, 0.1, 0.16], STEEL),
    part([0.0, 0.97, 0.0], [0.42, 0.22, 0.36], Look::Tex(TEAL)),
    part([0.6, 0.2, -0.66], [0.3, 0.52, 0.24], Look::Tex(TEAL)),
    part([0.86, 1.06, 0.86], [0.12, 0.12, 0.12], Look::Lamp),
    part([0.0, 1.38, 0.0], DISPLAY, Look::Product),
];

const FIREBRICK: [u16; 3] = [tex::BLAST_TOP, tex::BLAST_SIDE, tex::FRAME];

/// A blast furnace: on a cast-house floor over a banded plinth, a round hearth girt by the bustle main, tuyere stocks
/// dropping from it, a tapering stack up to a deck with the charging bell (the product display on it, at the top
/// centre) and two uptakes whose bleeders smoke while it works. A skip bridge climbs to the deck from the front, a
/// hot-blast stove with a dome stands at the back, the tap hole and its runner glow while it works, a status lamp.
pub(super) const BLAST_PARTS: [Part; 23] = [
    part([0.0, -1.4, 0.0], [1.96, 0.2, 1.96], Look::Band(tex::FRAME)),
    part([0.0, -1.05, 0.0], [1.8, 0.5, 1.8], Look::Tex([tex::SOOT, tex::BLAST_SIDE, tex::FRAME])),
    round([0.0, -0.55, 0.0], [1.5, 0.5, 1.5], Look::Tex(FIREBRICK)),
    round([0.0, -0.22, 0.0], [1.64, 0.16, 1.64], JOINT),
    part([-0.55, -0.43, -0.55], [0.07, 0.3, 0.07], STEEL),
    part([0.55, -0.43, -0.55], [0.07, 0.3, 0.07], STEEL),
    part([-0.55, -0.43, 0.55], [0.07, 0.3, 0.07], STEEL),
    part([0.55, -0.43, 0.55], [0.07, 0.3, 0.07], STEEL),
    tapered([0.0, 0.43, 0.0], [1.46, 1.14, 1.46], Look::Tex(FIREBRICK), 0.62),
    part([0.0, 1.05, 0.0], [1.24, 0.1, 1.24], FRAME),
    round([0.0, 1.21, 0.0], [0.46, 0.22, 0.46], Look::Tex([tex::SOOT, tex::STEEL, tex::FRAME])),
    round([-0.42, 1.3, -0.42], [0.14, 0.4, 0.14], STEEL),
    round([0.42, 1.3, -0.42], [0.14, 0.4, 0.14], STEEL),
    part([-0.42, 1.53, -0.42], [0.2, 0.06, 0.2], SOOT),
    part([0.42, 1.53, -0.42], [0.2, 0.06, 0.2], SOOT),
    part([0.42, 1.6, -0.42], [0.5, 0.0, 0.0], Look::Smoke),
    leaned([-0.55, 0.15, 0.565], [0.22, 2.0, 0.08], FRAME, 0.0, -0.32),
    part([0.0, -0.62, 0.75], [0.3, 0.16, 0.08], Look::Fire(tex::SOOT)),
    part([0.0, -0.77, 0.88], [0.14, 0.06, 0.24], Look::Fire(tex::SOOT)),
    round([-0.75, 0.005, -0.75], [0.4, 1.59, 0.4], Look::Tex(FIREBRICK)),
    tapered([-0.75, 0.94, -0.75], [0.4, 0.28, 0.4], JOINT, 0.3),
    part([0.84, -0.74, 0.84], [0.14, 0.12, 0.14], Look::Lamp),
    part([0.0, 1.62, 0.0], DISPLAY, Look::Product),
];

const BLUE_TANK: [u16; 3] = [tex::BOILER_TOP, tex::BOILER_SIDE, tex::FRAME];

/// A banded plinth, a low brick firebox whose door glows while it burns, the riveted drum lying on it (two crossed
/// parts; the water gauge on its front and the pipe fittings are `steam_view.rs`'s), a steam dome, a smoking stack
/// and a status lamp.
pub(super) const BOILER_PARTS: [Part; 8] = [
    part([0.0, -0.9, 0.0], [1.96, 0.2, 1.96], Look::Band(tex::FRAME)),
    part([0.0, -0.45, 0.0], [1.8, 0.7, 1.8], Look::Tex(FIREBRICK)),
    part([0.0, -0.45, 0.91], [0.8, 0.4, 0.06], Look::Fire(tex::SOOT)),
    round_x([0.0, 0.35, -0.1], [1.7, 1.3, 1.3], Look::Tex(BLUE_TANK)),
    part([0.3, 1.05, 0.0], [0.4, 0.2, 0.4], STEEL),
    part([-0.55, 1.25, -0.2], [0.26, 0.6, 0.26], SOOT),
    part([-0.55, 1.58, -0.2], [0.8, 0.0, 0.0], Look::Smoke),
    part([0.85, -0.72, 0.92], [0.14, 0.1, 0.14], Look::Lamp),
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

/// A jaw crusher: a hazard-marked frame on a base, a flared hopper over a dark mouth with a fixed jaw and one that
/// pounds while crushing, an eccentric shaft through a bearing housing at the back with a flywheel on each end, a
/// motor at the front foot belted up to one flywheel, a discharge chute, a lamp and the product display on the
/// bearing housing.
pub(super) const CRUSHER_PARTS: [Part; 18] = [
    part([0.0, -0.42, 0.0], [0.94, 0.16, 0.94], Look::Band(tex::FRAME)),
    part([0.0, -0.05, 0.0], [0.56, 0.58, 0.8], Look::Tex(HAZARD)),
    part([0.0, 0.245, 0.16], [0.42, 0.01, 0.3], SOOT),
    leaned([0.0, 0.37, 0.35], [0.5, 0.28, 0.04], STEEL, 0.0, 0.37),
    leaned([0.0, 0.37, -0.01], [0.5, 0.28, 0.04], STEEL, 0.0, -0.2),
    leaned([-0.24, 0.37, 0.16], [0.36, 0.28, 0.04], STEEL, 1.0, 0.227),
    leaned([0.24, 0.37, 0.16], [0.36, 0.28, 0.04], STEEL, 1.0, -0.227),
    part([0.0, 0.33, 0.25], [0.36, 0.16, 0.04], Look::Tex([tex::MAT_GUNMETAL; 3])),
    part([0.0, 0.36, 0.08], [0.36, 0.2, 0.05], Look::Press(1.0, [tex::MAT_GUNMETAL; 3])),
    round_x([0.0, 0.28, -0.16], [0.8, 0.07, 0.07], STEEL),
    round_x([-0.36, 0.28, -0.16], [0.07, 0.54, 0.54], FLYWHEEL),
    round_x([0.36, 0.28, -0.16], [0.07, 0.54, 0.54], FLYWHEEL),
    part([0.0, 0.28, -0.16], [0.3, 0.24, 0.26], Look::Tex(HAZARD)),
    part([0.37, -0.22, 0.27], [0.2, 0.2, 0.24], Look::Tex([tex::FRAME, tex::MOTOR, tex::FRAME])),
    leaned([0.43, 0.03, 0.06], [0.03, 0.7, 0.06], Look::Tex([tex::MAT_RUBBER; 3]), 0.0, -0.721),
    part([0.0, -0.26, 0.45], [0.32, 0.08, 0.1], STEEL),
    part([-0.17, 0.0, 0.415], [0.1, 0.08, 0.03], Look::Lamp),
    part([0.0, 0.62, -0.26], SMALL_DISPLAY, Look::Product),
];

/// A flywheel's faces show its spokes; its rim is dark steel.
const FLYWHEEL: Look = Look::Tex([tex::WHEEL, tex::MAT_GUNMETAL, tex::WHEEL]);

const COOLED: [u16; 3] = [tex::ARC_TOP, tex::ARC_SIDE, tex::FRAME];
const ROOF: [u16; 3] = [tex::ARC_TOP; 3];
const BUS: Look = Look::Press(0.6, [tex::COPPER_INGOT; 3]);
const CLAMP: Look = Look::Press(0.6, [tex::MAT_METAL; 3]);
const ELECTRODE: Look = Look::Press(0.6, [tex::MAT_GUNMETAL; 3]);

/// An electric arc furnace: a round water-cooled vessel on a tilting platform over a banded plinth, a rim and a domed
/// roof whose centre glows while it arcs, three graphite electrodes through the roof held by copper arms off a mast at
/// the back (arms, clamps and electrodes dip together while it works), a slag door and a tap spout glowing at the
/// front, a status lamp and the product display on the mast's head.
pub(super) const ARC_PARTS: [Part; 21] = [
    part([0.0, -0.9, 0.0], [1.96, 0.2, 1.96], Look::Band(tex::FRAME)),
    part([0.0, -0.58, 0.0], [1.8, 0.44, 1.8], Look::Tex([tex::FRAME, tex::STEEL, tex::FRAME])),
    round([0.0, -0.08, 0.0], [1.5, 0.56, 1.5], Look::Tex(COOLED)),
    round([0.0, 0.24, 0.0], [1.58, 0.08, 1.58], JOINT),
    tapered([0.0, 0.43, 0.0], [1.5, 0.3, 1.5], Look::Tex(ROOF), 0.55),
    round([0.0, 0.6, 0.0], [0.62, 0.04, 0.62], Look::Fire(tex::SOOT)),
    part([0.0, -0.1, 0.72], [0.34, 0.22, 0.1], Look::Fire(tex::SOOT)),
    part([0.0, -0.3, 0.86], [0.16, 0.06, 0.2], Look::Fire(tex::SOOT)),
    // The mast and its head; the arms off it, a clamp on each electrode.
    part([0.0, 0.48, -0.82], [0.4, 1.66, 0.2], STEEL),
    part([0.0, 1.35, -0.74], [0.62, 0.08, 0.5], FRAME),
    part([-0.19, 1.17, -0.305], [0.1, 0.08, 0.83], BUS),
    part([0.19, 1.17, -0.305], [0.1, 0.08, 0.83], BUS),
    part([0.0, 1.17, -0.47], [0.1, 0.08, 0.5], BUS),
    part([-0.19, 1.17, 0.11], [0.18, 0.14, 0.18], CLAMP),
    part([0.19, 1.17, 0.11], [0.18, 0.14, 0.18], CLAMP),
    part([0.0, 1.17, -0.22], [0.18, 0.14, 0.18], CLAMP),
    round([-0.19, 0.87, 0.11], [0.14, 0.9, 0.14], ELECTRODE),
    round([0.19, 0.87, 0.11], [0.14, 0.9, 0.14], ELECTRODE),
    round([0.0, 0.87, -0.22], [0.14, 0.9, 0.14], ELECTRODE),
    part([0.84, -0.3, 0.84], [0.14, 0.12, 0.14], Look::Lamp),
    part([0.0, 1.69, -0.7], DISPLAY, Look::Product),
];

const HOOD: Look = Look::Tex([tex::FRAME, tex::CELL_TOP, tex::FRAME]);
const ANODE: Look = Look::Press(0.3, [tex::MAT_METAL; 3]);

/// A reduction pot: a long steel shell on a banded plinth girt by stiffener ribs (between the port hatches), the bath
/// on it glowing at the seams while it works, sloped hood covers along both sides up to a ridge, a copper anode beam
/// on two end posts with six anode rods hanging into the pot (they settle while it works), a status lamp on one post
/// and the product display on the beam at the top centre.
pub(super) const CELL_PARTS: [Part; 20] = [
    part([0.0, -0.9, 0.0], [2.96, 0.2, 1.96], Look::Band(tex::FRAME)),
    part([0.0, -0.43, 0.0], [2.8, 0.74, 1.8], Look::Tex([tex::FRAME, tex::CELL_SIDE, tex::FRAME])),
    part([-0.5, -0.41, 0.0], [0.1, 0.76, 1.92], STEEL),
    part([0.5, -0.41, 0.0], [0.1, 0.76, 1.92], STEEL),
    part([0.0, -0.4, 0.0], [2.92, 0.76, 0.1], STEEL),
    part([0.0, -0.03, 0.0], [2.6, 0.06, 1.6], Look::Fire(tex::SOOT)),
    leaned([0.0, 0.2, 0.575], [2.6, 0.62, 0.05], HOOD, 0.0, -0.848),
    leaned([0.0, 0.2, -0.575], [2.6, 0.62, 0.05], HOOD, 0.0, 0.848),
    part([0.0, 0.42, 0.0], [2.64, 0.06, 0.74], FRAME),
    part([-1.32, 0.47, 0.0], [0.12, 0.94, 0.3], STEEL),
    part([1.32, 0.47, 0.0], [0.12, 0.94, 0.3], STEEL),
    part([0.0, 0.8, 0.0], [2.72, 0.16, 0.36], Look::Tex([tex::COPPER_INGOT; 3])),
    part([-1.0, 0.605, 0.0], [0.08, 0.31, 0.08], ANODE),
    part([-0.6, 0.605, 0.0], [0.08, 0.31, 0.08], ANODE),
    part([-0.2, 0.605, 0.0], [0.08, 0.31, 0.08], ANODE),
    part([0.2, 0.605, 0.0], [0.08, 0.31, 0.08], ANODE),
    part([0.6, 0.605, 0.0], [0.08, 0.31, 0.08], ANODE),
    part([1.0, 0.605, 0.0], [0.08, 0.31, 0.08], ANODE),
    part([1.32, 1.0, 0.0], [0.12, 0.12, 0.12], Look::Lamp),
    part([0.0, 1.18, 0.0], DISPLAY, Look::Product),
];

const CONCRETE_RINGS: [u16; 3] = [tex::SILO_TOP, tex::SILO_SIDE, tex::FRAME];

/// A tall round concrete drum (two crossed parts) on a banded plinth with two steel rings, a low round roof with a
/// hatch, and a status lamp on its front.
pub(super) const SILO_PARTS: [Part; 6] = [
    part([0.0, -1.4, 0.0], [1.96, 0.2, 1.96], Look::Band(tex::FRAME)),
    round([0.0, -0.05, 0.0], [1.8, 2.6, 1.8], Look::Tex(CONCRETE_RINGS)),
    round([0.0, 0.5, 0.0], [1.9, 0.1, 1.9], FRAME),
    round([0.0, -0.6, 0.0], [1.9, 0.1, 1.9], FRAME),
    round([0.0, 1.32, 0.0], [1.3, 0.14, 1.3], Look::Tex([tex::SILO_TOP, tex::FRAME, tex::FRAME])),
    part([0.0, -1.1, 0.92], [0.14, 0.1, 0.06], Look::Lamp),
];
