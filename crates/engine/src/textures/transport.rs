//! Trains (Milestone 9): the rail icon (two steel rails over wooden sleepers on gravel). Locomotive, wagon and station
//! looks join it as those steps land. Placeholder looks until the art pass (`docs/ART_HANDOVER.md`).

use crate::block::tex;

use super::{n, rgb};

pub fn pixel(layer: u16, x: i32, y: i32) -> [u8; 4] {
    match layer {
        tex::RAIL => rail(x, y),
        _ => [255, 0, 255, 255],
    }
}

/// Two rails run up the picture over sleepers, on grey gravel.
fn rail(x: i32, y: i32) -> [u8; 4] {
    if x == 4 || x == 11 {
        return rgb([190.0, 196.0, 206.0], 0.95 + 0.1 * n(420, x, y));
    }
    if (3..=12).contains(&x) && y % 5 < 2 {
        return rgb([116.0, 84.0, 52.0], 0.9 + 0.15 * n(421, x, y));
    }
    rgb([104.0, 102.0, 98.0], 0.8 + 0.4 * n(422, x, y))
}
