//! Solar power (Milestone 7): the solar panel (deep blue cells in a silver grid on the roof, a plain steel back)
//! and the accumulator (a grey roof with two copper terminals, a casing with a row of green charge cells).
//! Placeholder looks until the art pass (`docs/ART_HANDOVER.md`).

use crate::block::tex;

use super::{frame, n, rgb, smooth};

pub fn pixel(layer: u16, x: i32, y: i32) -> [u8; 4] {
    match layer {
        tex::SOLAR_TOP => cells(x, y),
        tex::SOLAR_SIDE => back(x, y),
        tex::ACCUMULATOR_TOP => terminals(x, y),
        _ => casing(x, y),
    }
}

/// Four by four blue cells with a sheen, split by thin silver bus lines, in a silver border.
fn cells(x: i32, y: i32) -> [u8; 4] {
    if x == 0 || y == 0 || x == 15 || y == 15 || x % 4 == 3 || y % 4 == 3 {
        return rgb([196.0, 202.0, 214.0], 0.9 + 0.1 * n(410, x, y));
    }
    let k = 0.8 + 0.4 * smooth(411, x, y, 4);
    rgb([28.0, 52.0, 122.0], k)
}

/// A brushed steel back with a row of vents.
fn back(x: i32, y: i32) -> [u8; 4] {
    if (5..=6).contains(&y) && x % 3 != 0 && (2..14).contains(&x) {
        return rgb([40.0, 44.0, 52.0], 1.0);
    }
    rgb([120.0, 128.0, 140.0], 0.88 + 0.12 * n(412, x, y / 2))
}

/// A steel roof with two copper terminal posts.
fn terminals(x: i32, y: i32) -> [u8; 4] {
    for cx in [4.5, 11.5] {
        let r = ((x as f64 - cx).powi(2) + (y as f64 - 7.5).powi(2)).sqrt();
        if r < 2.0 {
            return rgb([204.0, 128.0, 76.0], if r < 1.0 { 1.15 } else { 0.9 });
        }
    }
    frame(x, y)
}

/// A dark green casing with a row of lit charge cells and steel bands at top and bottom.
fn casing(x: i32, y: i32) -> [u8; 4] {
    if y <= 1 || y >= 14 {
        return rgb([140.0, 148.0, 160.0], 0.9 + 0.1 * n(413, x, y));
    }
    if (6..=9).contains(&y) && x % 4 != 0 && (1..15).contains(&x) {
        return rgb([120.0, 236.0, 120.0], 1.0);
    }
    rgb([30.0, 64.0, 48.0], 0.9 + 0.12 * n(414, x, y))
}
