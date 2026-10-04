//! Aluminium (Milestone 9): crushed bauxite (red-brown chunks), the light silvery ingot and plate, the battery (a
//! green cell with a copper cap) and the electrolytic cell (a white-tiled tank with a molten bath and bus bars on
//! its roof). Placeholder looks until the art pass (`docs/ART_HANDOVER.md`).

use crate::block::tex;

use super::{frame, n, rgb, smooth};

pub fn pixel(layer: u16, x: i32, y: i32) -> [u8; 4] {
    match layer {
        tex::CRUSHED_BAUXITE => crushed_bauxite(x, y),
        tex::ALUMINIUM_INGOT => ingot(x, y),
        tex::ALUMINIUM_PLATE => plate(x, y),
        tex::BATTERY => battery(x, y),
        tex::CELL_SIDE => cell_side(x, y),
        _ => cell_top(x, y),
    }
}

/// Jagged red-brown chunks with dark gaps, like the other crushed ores but with a clay tint.
fn crushed_bauxite(x: i32, y: i32) -> [u8; 4] {
    let k = 0.8 + 0.5 * smooth(400, x, y, 8) + 0.1 * n(401, x, y);
    if n(402, x, y) > 0.86 {
        rgb([88.0, 44.0, 30.0], 1.0)
    } else {
        rgb([176.0, 92.0, 60.0], k)
    }
}

/// A light bar with a bright top edge: whiter and cooler than iron.
fn ingot(x: i32, y: i32) -> [u8; 4] {
    let k = 0.92 + 0.06 * n(403, x, y) + if y < 4 { 0.1 } else { 0.0 };
    rgb([206.0, 214.0, 222.0], k)
}

/// A pale plate with a dark rivet at each corner.
fn plate(x: i32, y: i32) -> [u8; 4] {
    let rivet = (x == 2 || x == 13) && (y == 2 || y == 13);
    let k = 0.94 + 0.06 * n(404, x, y);
    if rivet {
        rgb([120.0, 128.0, 140.0], 1.0)
    } else {
        rgb([218.0, 224.0, 232.0], k)
    }
}

/// A green cell body under a copper cap, with a dark plus sign.
fn battery(x: i32, y: i32) -> [u8; 4] {
    if y < 3 {
        return rgb([204.0, 128.0, 76.0], 0.9 + 0.1 * n(405, x, y));
    }
    let plus = (x == 7 || x == 8) && (6..12).contains(&y) || (y == 8 || y == 9) && (5..11).contains(&x);
    if plus {
        rgb([30.0, 36.0, 34.0], 1.0)
    } else {
        rgb([70.0, 170.0, 96.0], 0.88 + 0.12 * n(406, x, y))
    }
}

/// White tiles in a steel frame, with a dark glowing slot where the bath shows.
fn cell_side(x: i32, y: i32) -> [u8; 4] {
    if (6..=8).contains(&y) && (3..13).contains(&x) {
        return rgb([255.0, 168.0, 72.0], if y == 7 { 1.1 } else { 0.8 });
    }
    if x == 0 || x == 15 || y == 0 || y == 15 || y == 4 || y == 11 {
        return rgb([96.0, 102.0, 112.0], 0.9 + 0.1 * n(407, x, y));
    }
    rgb([220.0, 224.0, 228.0], 0.9 + 0.08 * n(408, x, y) + 0.06 * smooth(409, x, y, 4))
}

/// A steel roof with two copper bus bars across it.
fn cell_top(x: i32, y: i32) -> [u8; 4] {
    if y == 4 || y == 5 || y == 10 || y == 11 {
        return rgb([204.0, 128.0, 76.0], 0.95 + 0.1 * n(410, x, y));
    }
    frame(x, y)
}
