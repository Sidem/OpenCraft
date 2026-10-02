//! Kestrel's quiet industrial palette: ivory ceramic, petrol fabric, graphite joints and amber trim.
//! Broad panels and a few seams read at third-person distance without noisy skin-like pixels.

use super::rgb;
use crate::block::tex;

pub(super) fn pixel(layer: u16, x: i32, y: i32) -> [u8; 4] {
    let edge = x == 0 || x == 15 || y == 0 || y == 15;
    let k = if edge { 0.78 } else { 0.94 + (15 - y) as f64 * 0.004 };
    let color = match layer {
        tex::AVATAR_SUIT => [48.0, 112.0, 117.0],
        tex::AVATAR_SKIN => [220.0, 225.0, 207.0],
        tex::AVATAR_HELMET => [207.0, 134.0, 66.0],
        tex::AVATAR_VISOR => [23.0, 43.0, 49.0],
        tex::AVATAR_JOINT => [43.0, 57.0, 63.0],
        tex::AVATAR_TRIM => return rgb([255.0, 197.0, 91.0], 1.0),
        _ if y == 4 || y == 11 => [35.0, 61.0, 67.0],
        _ => [65.0, 100.0, 104.0],
    };
    rgb(color, k)
}
