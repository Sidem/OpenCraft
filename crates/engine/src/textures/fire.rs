//! Procedural textures for FIRE items: charred wood charcoal, the flamethrower's tan vented shroud, dark olive-green
//! fuel cylinder, and mirror-polished chrome torch bell.

use super::{n, rgb};
use crate::block::tex;

pub(super) fn pixel(layer: u16, x: i32, y: i32) -> [u8; 4] {
    match layer {
        tex::CHARCOAL => charcoal(x, y),
        tex::FLAMETHROWER => tan_shroud(x, y),
        tex::FLAMETHROWER_TANK => olive_tank(x, y),
        tex::FLAMETHROWER_CHROME => chrome_bell(x, y),
        _ => [255, 0, 255, 255],
    }
}

/// Charred wood grain: deep matte charcoal with lengthwise split lines and pale ash specks.
fn charcoal(x: i32, y: i32) -> [u8; 4] {
    let g = n(911, x, y);
    let split = if (x + (g > 0.5) as i32) & 3 == 0 { 0.8 } else { 1.0 };
    let ash = if g > 0.86 && ((x ^ y) & 1) == 0 { 0.35 } else { 0.0 };
    let v = (34.0 * split + g * 14.0 + ash * 68.0).clamp(18.0, 120.0);
    rgb([v, v * 0.96, v * 0.98], 1.0)
}

/// Tactical sand/khaki upper shroud with top picatinny rail notches and dark recessed triangular side vents.
fn tan_shroud(x: i32, y: i32) -> [u8; 4] {
    let k = 0.92 + n(912, x, y) * 0.14;
    // Top rail notches (dark gaps every 2 texels).
    if y <= 2 && (x & 1) == 0 {
        return rgb([128.0, 114.0, 86.0], k);
    }
    // Recessed dark triangular vent cutouts along the mid-band.
    if (6..=11).contains(&y) {
        let cell = x.rem_euclid(4);
        let row = y - 6;
        let in_tri = (x / 4) & 1 == 0 && cell >= row / 2 && cell <= 3 - row / 2
            || (x / 4) & 1 == 1 && cell >= (5 - row) / 2 && cell <= 3 - (5 - row) / 2;
        if in_tri && row > 0 && row < 5 {
            return rgb([36.0, 34.0, 32.0], k);
        }
    }
    // Warm desert tan / khaki polymer body.
    let bevel = if y == 3 || y == 13 {
        1.06
    } else if y == 14 || y == 15 {
        0.84
    } else {
        1.0
    };
    rgb([204.0 * bevel, 184.0 * bevel, 138.0 * bevel], k)
}

/// Matte dark forest-olive pressurized fuel bottle with lower seam band and subtle cylindrical sheen.
fn olive_tank(x: i32, y: i32) -> [u8; 4] {
    let k = 0.90 + n(913, x, y) * 0.16;
    let sheen = if (4..=6).contains(&x) {
        1.22
    } else if (11..=14).contains(&x) {
        0.82
    } else {
        1.0
    };
    if y == 13 {
        return rgb([24.0, 40.0, 30.0], k);
    }
    rgb([42.0 * sheen, 74.0 * sheen, 54.0 * sheen], k)
}

/// Polished mirror-chrome cylindrical nozzle bell with bright specular band and cool steel reflection.
fn chrome_bell(x: i32, y: i32) -> [u8; 4] {
    let k = 0.95 + n(914, x, y) * 0.08;
    // Cylindrical reflection bands along the barrel circumference.
    let v = match y {
        0 | 15 => 145.0,
        2..=4 => 246.0,
        5..=6 => 214.0,
        7..=9 => 126.0,
        10..=11 => 176.0,
        _ => 198.0,
    };
    let tint = if x == 0 || x == 15 { 0.86 } else { 1.0 };
    rgb([v * 0.96 * tint, v * 0.98 * tint, (v * 1.03).min(255.0) * tint], k)
}
