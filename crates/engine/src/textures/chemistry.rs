//! Chemistry (Milestone 10): the canister icons and the pumpjack's faces. Every fluid but water travels in a
//! canister, so one drawing (`canister`) serves them all: a steel drum with a coloured band, the band's colour
//! telling the fluid; empty is plain. Placeholder looks until the art pass (`docs/ART_HANDOVER.md`).
//! To add a canister: a band colour here and a `tex` layer in the range `pixel` matches.

use crate::block::tex;

use super::paint::round_shade;
use super::{frame, n, rgb, smooth};

const EMPTY_BAND: [f64; 3] = [150.0, 156.0, 166.0];
const CRUDE_BAND: [f64; 3] = [34.0, 28.0, 22.0];
const NAPHTHA_BAND: [f64; 3] = [236.0, 226.0, 120.0];
const DIESEL_BAND: [f64; 3] = [222.0, 130.0, 34.0];
const HEAVY_BAND: [f64; 3] = [110.0, 44.0, 34.0];
const ACID_BAND: [f64; 3] = [150.0, 220.0, 60.0];
const LUBRICANT_BAND: [f64; 3] = [226.0, 188.0, 70.0];
const HYDROGEN_BAND: [f64; 3] = [120.0, 190.0, 240.0];
const OXYGEN_BAND: [f64; 3] = [236.0, 240.0, 246.0];

/// A glossy white-blue sheet with a bright edge and a faint grain.
fn plastic(x: i32, y: i32) -> [u8; 4] {
    let edge = x == 0 || y == 0 || x == 15 || y == 15;
    let k = 0.94 + 0.06 * n(497, x, y) + if edge { 0.08 } else { 0.0 };
    rgb([226.0, 236.0, 246.0], k)
}

pub fn pixel(layer: u16, x: i32, y: i32) -> [u8; 4] {
    match layer {
        tex::EMPTY_CANISTER => canister(x, y, EMPTY_BAND),
        tex::CRUDE_CANISTER => canister(x, y, CRUDE_BAND),
        tex::NAPHTHA_CANISTER => canister(x, y, NAPHTHA_BAND),
        tex::DIESEL_CANISTER => canister(x, y, DIESEL_BAND),
        tex::HEAVY_OIL_CANISTER => canister(x, y, HEAVY_BAND),
        tex::SULFUR => sulfur(x, y),
        tex::PLASTIC => plastic(x, y),
        tex::ACID_CANISTER => canister(x, y, ACID_BAND),
        tex::LUBRICANT_CANISTER => canister(x, y, LUBRICANT_BAND),
        tex::CHEM_SIDE => plant_side(x, y, [96.0, 138.0, 124.0]),
        tex::CHEM_TOP => plant_top(x, y, 2),
        tex::HYDROGEN_CANISTER => canister(x, y, HYDROGEN_BAND),
        tex::OXYGEN_CANISTER => canister(x, y, OXYGEN_BAND),
        tex::DIESEL_SIDE => plant_side(x, y, [176.0, 108.0, 40.0]),
        tex::DIESEL_TOP => plant_top(x, y, 5),
        tex::ELECTROLYSER_SIDE => plant_side(x, y, [70.0, 120.0, 150.0]),
        tex::ELECTROLYSER_TOP => plant_top(x, y, 1),
        tex::PUMPJACK_SIDE => pumpjack_side(x, y),
        tex::PUMPJACK_TOP => pumpjack_top(x, y),
        tex::REFINERY_SIDE => plant_side(x, y, [118.0, 128.0, 142.0]),
        tex::REFINERY_TOP => plant_top(x, y, 4),
        tex::CRACKER_SIDE => plant_side(x, y, [150.0, 112.0, 96.0]),
        _ => plant_top(x, y, 3),
    }
}

/// Bright yellow crystals in a dull yellow powder, with dark gaps.
fn sulfur(x: i32, y: i32) -> [u8; 4] {
    if n(491, x, y) > 0.88 {
        return rgb([120.0, 104.0, 30.0], 1.0);
    }
    let crystal = smooth(492, x, y, 6) > 0.58;
    let base = if crystal { [244.0, 222.0, 40.0] } else { [196.0, 176.0, 52.0] };
    rgb(base, 0.88 + 0.2 * n(493, x, y))
}

/// The painted steel of a vessel (a tank, column or engine casing) in `paint`: shaded round across (`round_shade`),
/// two weld rings each lit along its upper edge, and a dark foot. No rows of rivets: stretched up a tall part those
/// read as the lit windows of a building.
fn plant_side(x: i32, y: i32, paint: [f64; 3]) -> [u8; 4] {
    let shade = round_shade(x);
    if y == 15 {
        return rgb([54.0, 58.0, 66.0], 0.9 + 0.1 * n(494, x, y));
    }
    if y == 4 || y == 11 {
        return rgb([paint[0] * 0.55, paint[1] * 0.55, paint[2] * 0.6], shade);
    }
    let lit = if y == 3 || y == 10 { 1.12 } else { 0.92 + 0.08 * n(495, x, y) + 0.06 * smooth(496, x, y, 4) };
    rgb(paint, shade * lit)
}

/// A steel deck with a grille of `bars` dark slots.
fn plant_top(x: i32, y: i32, bars: i32) -> [u8; 4] {
    let step = 16 / (bars + 1);
    let slot = (3..13).contains(&x) && (1..=bars).any(|k| (y - k * step).abs() == 0);
    if slot {
        rgb([20.0, 22.0, 26.0], 1.0)
    } else {
        frame(x, y)
    }
}

/// A drum seen from the front on a dark ground: steel body with rim lines, a cap and a band of `band` across it.
pub(super) fn canister(x: i32, y: i32, band: [f64; 3]) -> [u8; 4] {
    let body = (4..=11).contains(&x) && (3..=14).contains(&y);
    if (6..=9).contains(&x) && (1..=2).contains(&y) {
        return rgb([196.0, 200.0, 208.0], 0.9 + 0.1 * n(480, x, y));
    }
    if !body {
        return rgb([58.0, 62.0, 70.0], 0.85 + 0.3 * n(481, x, y));
    }
    let rim = y == 3 || y == 14 || x == 4;
    if rim {
        rgb([96.0, 102.0, 112.0], 0.9 + 0.1 * n(482, x, y))
    } else if (7..=10).contains(&y) {
        rgb(band, 0.9 + 0.2 * n(483, x, y))
    } else {
        rgb([176.0, 182.0, 192.0], 0.92 + 0.12 * smooth(484, x, y, 4))
    }
}

/// Olive-green plating with riveted seams and a dark oil stain creeping up from the foot.
fn pumpjack_side(x: i32, y: i32) -> [u8; 4] {
    if y == 0 || y == 15 || y == 7 {
        return rgb([60.0, 64.0, 58.0], 0.9 + 0.1 * n(485, x, y));
    }
    let stain = (y as f64) > 10.0 + 4.0 * smooth(486, x, 0, 4);
    let base = if stain { [40.0, 34.0, 28.0] } else { [112.0, 122.0, 82.0] };
    let rivet = (x == 2 || x == 13) && (y == 3 || y == 11);
    if rivet {
        rgb([170.0, 176.0, 150.0], 1.0)
    } else {
        rgb(base, 0.88 + 0.16 * n(487, x, y) + 0.1 * smooth(488, x, y, 4))
    }
}

/// A steel deck with a round black well-head in the middle.
fn pumpjack_top(x: i32, y: i32) -> [u8; 4] {
    let (dx, dy) = (x as f64 - 7.5, y as f64 - 7.5);
    let d = dx * dx + dy * dy;
    if d < 6.0 {
        rgb([22.0, 20.0, 18.0], 0.9 + 0.2 * n(489, x, y))
    } else if d < 12.0 {
        rgb([200.0, 150.0, 44.0], 0.95 + 0.1 * n(490, x, y))
    } else {
        frame(x, y)
    }
}
