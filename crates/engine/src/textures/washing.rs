//! Ore washing (Milestone 10): washed iron, copper and bauxite (clean, rounded pebbles, brighter than the jagged
//! crushed ore they come from), tailings (a grey-brown wet gravel block) and the washer's plating and deck.
//! Placeholder looks until the art pass (`docs/ART_HANDOVER.md`).
//! To add a washed ore: a `tex` layer in the range `pixel` matches and its colour here.

use crate::block::tex;

use super::{frame, n, rgb, smooth};

pub fn pixel(layer: u16, x: i32, y: i32) -> [u8; 4] {
    match layer {
        tex::WASHED_IRON => washed(x, y, [190.0, 194.0, 208.0]),
        tex::WASHED_COPPER => washed(x, y, [226.0, 140.0, 96.0]),
        tex::WASHED_BAUXITE => washed(x, y, [226.0, 150.0, 120.0]),
        tex::TAILINGS => tailings(x, y),
        tex::WASHER_SIDE => washer_side(x, y),
        _ => washer_top(x, y),
    }
}

/// Smooth pebbles of `c` with a pale wet glint on some and dark gaps between them.
fn washed(x: i32, y: i32, c: [f64; 3]) -> [u8; 4] {
    let bit = smooth(521, x, y, 4);
    if bit < 0.3 {
        return rgb([c[0] * 0.45, c[1] * 0.45, c[2] * 0.45], 1.0);
    }
    let glint = if n(522, x, y) > 0.9 { 1.25 } else { 1.0 };
    rgb(c, (0.85 + 0.3 * bit) * glint)
}

/// Wet grey-brown gravel with darker silt patches.
fn tailings(x: i32, y: i32) -> [u8; 4] {
    let silt = smooth(523, x, y, 8);
    let base = if silt > 0.55 { [92.0, 84.0, 76.0] } else { [128.0, 120.0, 108.0] };
    rgb(base, 0.8 + 0.3 * n(524, x, y) + 0.1 * smooth(525, x, y, 4))
}

/// Teal plating with riveted seams and a wavy waterline in the lower third.
fn washer_side(x: i32, y: i32) -> [u8; 4] {
    if y == 0 || y == 15 {
        return rgb([54.0, 62.0, 70.0], 0.9 + 0.1 * n(526, x, y));
    }
    let wave = 10 + ((x as f64 * 0.8).sin() * 1.2).round() as i32;
    if y >= wave {
        return rgb([60.0, 120.0, 190.0], 0.9 + 0.2 * n(527, x, y));
    }
    let rivet = (y == 3 || y == 7) && x % 5 == 2;
    if rivet {
        return rgb([196.0, 200.0, 206.0], 1.0);
    }
    rgb([84.0, 138.0, 136.0], 0.88 + 0.14 * n(528, x, y) + 0.08 * smooth(529, x, y, 4))
}

/// A steel deck with a grated wash tank in the middle and water showing through.
fn washer_top(x: i32, y: i32) -> [u8; 4] {
    if (3..13).contains(&x) && (3..13).contains(&y) {
        if (x + y) % 3 == 0 {
            return rgb([34.0, 40.0, 46.0], 1.0);
        }
        return rgb([60.0, 120.0, 190.0], 0.85 + 0.25 * n(530, x, y));
    }
    frame(x, y)
}
