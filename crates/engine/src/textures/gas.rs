//! Natural gas textures (CHEM-GAS): deep gas shale (dark blue-grey layered rock with pale blue mineral veins), the
//! five gas barrels (steel drums with distinct colour bands) and the five gas machines' vessel and casing paints.
//! Placeholder procedural looks until the art pass (`docs/ART_HANDOVER.md`).
//! To add a gas texture: add a layer in `block/tex.rs` and its arm in `pixel`.

use crate::block::tex;

use super::chemistry::{barrel, plant_side, plant_top};
use super::paint::{mottle, tone, Ramp};
use super::{n, smooth};

/// Dark blue-slate shale ramp with subtle pale-cyan mineral veins along horizontal bedding.
const SHALE: Ramp = [[32, 42, 54], [46, 58, 74], [62, 78, 98], [90, 126, 152], [140, 206, 236]];

/// Distinct band colours for the five gas barrels (chosen far from oil/acid/hydrogen/oxygen bands).
const RAW_GAS_BAND: [f64; 3] = [88.0, 146.0, 178.0];
const METHANE_BAND: [f64; 3] = [56.0, 198.0, 236.0];
const ETHANE_BAND: [f64; 3] = [74.0, 204.0, 154.0];
const LPG_BAND: [f64; 3] = [232.0, 134.0, 82.0];
const ETHYLENE_BAND: [f64; 3] = [186.0, 132.0, 232.0];

pub fn pixel(layer: u16, x: i32, y: i32) -> [u8; 4] {
    match layer {
        tex::GAS_SHALE => gas_shale(x, y),
        tex::RAW_GAS_BARREL => barrel(x, y, RAW_GAS_BAND),
        tex::METHANE_BARREL => barrel(x, y, METHANE_BAND),
        tex::ETHANE_BARREL => barrel(x, y, ETHANE_BAND),
        tex::LPG_BARREL => barrel(x, y, LPG_BAND),
        tex::ETHYLENE_BARREL => barrel(x, y, ETHYLENE_BAND),
        tex::GAS_WELL_SIDE => plant_side(x, y, [72.0, 124.0, 156.0]),
        tex::GAS_WELL_TOP => plant_top(x, y, 2),
        tex::SEPARATOR_SIDE => plant_side(x, y, [112.0, 132.0, 102.0]),
        tex::SEPARATOR_TOP => plant_top(x, y, 3),
        tex::GAS_PLANT_SIDE => plant_side(x, y, [88.0, 146.0, 166.0]),
        tex::GAS_PLANT_TOP => plant_top(x, y, 4),
        tex::GAS_TURBINE_SIDE => plant_side(x, y, [78.0, 136.0, 178.0]),
        tex::GAS_TURBINE_TOP => plant_top(x, y, 5),
        tex::REFORMER_SIDE => plant_side(x, y, [142.0, 118.0, 104.0]),
        _ => plant_top(x, y, 3),
    }
}

/// Dark blue-grey laminated shale with horizontal bedding seams and occasional pale blue-cyan gas pockets.
/// Tiles seamlessly on all four edges (tested by `natural_surfaces_have_no_tile_border`).
fn gas_shale(x: i32, y: i32) -> [u8; 4] {
    let shift = (smooth(610, x, y, 4) * 2.0) as i32;
    let bed = (y + shift).rem_euclid(4);
    let base = 0.25 + mottle(611, x, y, 0.25) * 0.35;
    if bed == 0 {
        let pocket = smooth(612, x, y, 4) > 0.62;
        if pocket {
            return tone(&SHALE, 0.85 + 0.14 * n(613, x, y));
        }
        return tone(&SHALE, (base - 0.16).max(0.02));
    }
    let glint = smooth(614, x, y, 4) > 0.72 && n(615, x, y) > 0.78;
    tone(&SHALE, if glint { 0.88 } else { base })
}
