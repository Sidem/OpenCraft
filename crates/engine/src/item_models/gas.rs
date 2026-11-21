//! Icons of the natural gas machines (CHEM-GAS): gas well Christmas tree, wellhead separator, gas plant, gas turbine
//! and steam reformer. Built from real cylinders, frustums and boxes after each machine's placed silhouette, parts back
//! to front (viewer at +x, +z, above), floor at -0.5, all within [-0.5, 0.5].

use super::shapes::{bar_x, cylinder, frustum};
use super::{part, Part};
use crate::block::{self, tex, BlockId};

pub(super) fn parts(block: BlockId) -> &'static [Part] {
    match block {
        block::GAS_WELL => &GAS_WELL,
        block::WELLHEAD_SEPARATOR => &WELLHEAD_SEPARATOR,
        block::GAS_PLANT => &GAS_PLANT,
        block::GAS_TURBINE => &GAS_TURBINE,
        block::STEAM_REFORMER => &STEAM_REFORMER,
        _ => &[],
    }
}

/// A gas wellhead ("Christmas tree"): concrete pad, steel flange stack, valve cross with handwheels and pressure cap.
const GAS_WELL: [Part; 9] = [
    part([0.0, -0.46, 0.0], [0.84, 0.08, 0.84], tex::CONCRETE),
    cylinder([0.0, -0.34, 0.0], 0.42, 0.16, tex::MAT_GUNMETAL),
    cylinder([0.0, -0.02, 0.0], 0.22, 0.52, tex::GAS_WELL_SIDE),
    cylinder([0.0, -0.14, 0.0], 0.34, 0.06, tex::STEEL),
    cylinder([0.0, 0.12, 0.0], 0.34, 0.06, tex::STEEL),
    bar_x([0.0, 0.02, 0.0], 0.68, 0.14, tex::GAS_WELL_SIDE),
    bar_x([-0.37, 0.02, 0.0], 0.06, 0.26, tex::MAT_EMBER),
    bar_x([0.37, 0.02, 0.0], 0.06, 0.26, tex::MAT_EMBER),
    frustum([0.0, 0.34, 0.0], 0.24, 0.18, 0.4, tex::MAT_BRASS),
];

/// A wellhead separator: steel skid, vertical well riser at back, horizontal three-phase pressure drum with end caps.
const WELLHEAD_SEPARATOR: [Part; 8] = [
    part([0.0, -0.46, 0.0], [0.9, 0.08, 0.72], tex::CONCRETE),
    cylinder([-0.24, -0.02, -0.18], 0.2, 0.8, tex::SEPARATOR_SIDE),
    cylinder([-0.24, 0.4, -0.18], 0.26, 0.06, tex::MAT_BRASS),
    part([-0.22, -0.3, 0.12], [0.1, 0.24, 0.3], tex::FRAME),
    part([0.22, -0.3, 0.12], [0.1, 0.24, 0.3], tex::FRAME),
    bar_x([0.04, -0.08, 0.12], 0.72, 0.36, tex::SEPARATOR_SIDE),
    bar_x([-0.34, -0.08, 0.12], 0.08, 0.32, tex::STEEL),
    bar_x([0.42, -0.08, 0.12], 0.08, 0.32, tex::STEEL),
];

/// A gas processing plant: sweetening amine tower, taller demethanizer column, horizontal receiver drum and blue flare.
const GAS_PLANT: [Part; 10] = [
    part([0.0, -0.46, 0.0], [0.92, 0.08, 0.92], tex::CONCRETE),
    cylinder([0.3, -0.02, -0.32], 0.06, 0.8, tex::STEEL),
    frustum([0.3, 0.42, -0.32], 0.12, 0.14, 0.2, tex::LAMP_BLUE),
    cylinder([-0.14, -0.02, -0.12], 0.26, 0.8, tex::GAS_PLANT_SIDE),
    cylinder([-0.14, 0.18, -0.12], 0.32, 0.03, tex::FRAME),
    cylinder([0.22, -0.12, 0.02], 0.22, 0.6, tex::GAS_PLANT_SIDE),
    cylinder([0.22, 0.06, 0.02], 0.28, 0.03, tex::FRAME),
    bar_x([0.04, 0.26, -0.05], 0.38, 0.06, tex::STEEL),
    bar_x([-0.04, -0.28, 0.26], 0.64, 0.24, tex::GAS_PLANT_SIDE),
    part([0.32, -0.34, 0.3], [0.12, 0.16, 0.12], tex::LAMP_GREEN),
];

/// A gas turbine: air intake plenum, tapered compressor/combustor drum, generator housing and angled exhaust stack.
const GAS_TURBINE: [Part; 8] = [
    part([0.0, -0.46, 0.0], [0.94, 0.08, 0.62], tex::FRAME),
    part([-0.32, -0.14, 0.0], [0.24, 0.56, 0.48], tex::GAS_TURBINE_SIDE),
    bar_x([0.02, -0.14, 0.0], 0.46, 0.38, tex::GAS_TURBINE_SIDE),
    bar_x([0.02, -0.14, 0.0], 0.06, 0.42, tex::STEEL),
    bar_x([0.32, -0.16, 0.0], 0.24, 0.3, tex::MAT_GUNMETAL),
    part([0.32, 0.02, 0.0], [0.12, 0.08, 0.12], tex::COPPER_INGOT),
    cylinder([-0.32, 0.28, 0.0], 0.22, 0.3, tex::STEEL),
    cylinder([-0.32, 0.44, 0.0], 0.26, 0.05, tex::MAT_GUNMETAL),
];

/// A steam reformer: furnace firebox, vertical catalyst tube bank, steam drum overhead and flue stack.
const STEAM_REFORMER: [Part; 9] = [
    part([0.0, -0.46, 0.0], [0.88, 0.08, 0.88], tex::FRAME),
    part([0.0, -0.24, 0.06], [0.76, 0.36, 0.68], tex::REFORMER_SIDE),
    cylinder([0.28, 0.16, -0.24], 0.14, 0.54, tex::SOOT),
    cylinder([-0.22, 0.14, -0.06], 0.12, 0.42, tex::STEEL),
    cylinder([0.0, 0.14, -0.06], 0.12, 0.42, tex::STEEL),
    cylinder([-0.22, 0.14, 0.18], 0.12, 0.42, tex::STEEL),
    cylinder([0.0, 0.14, 0.18], 0.12, 0.42, tex::STEEL),
    bar_x([-0.1, 0.38, 0.06], 0.54, 0.18, tex::PIPE_STEAM),
    part([0.28, -0.22, 0.42], [0.12, 0.1, 0.04], tex::LAMP_GREEN),
];
