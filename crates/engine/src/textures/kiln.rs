//! Procedural textures for the Glass Kiln (`tex::KILN_*`) and Paint Sprayer (`tex::PAINT_SPRAYER`) — non-cube model
//! and item layers past 512.

use super::paint::mottle;
use super::{n, rgb};
use crate::block::tex;

pub fn pixel(layer: u16, x: i32, y: i32) -> [u8; 4] {
    match layer {
        tex::KILN_SIDE => kiln_side(x, y),
        tex::KILN_TOP => kiln_top(x, y),
        tex::KILN_GLOW => kiln_glow(x, y),
        _ => paint_sprayer(x, y),
    }
}

/// Firebrick dome kiln shell bound with dark cast-iron bands and mortar courses.
fn kiln_side(x: i32, y: i32) -> [u8; 4] {
    if y == 2 || y == 13 {
        let rivet = x % 4 == 1;
        return rgb([68.0, 68.0, 74.0], if rivet { 1.25 } else { 0.85 + 0.08 * n(840, x, y) });
    }
    let row = y / 4;
    let shift = if row % 2 == 0 { 0 } else { 4 };
    if y % 4 == 3 || (x + shift).rem_euclid(8) == 7 {
        return rgb([128.0, 114.0, 100.0], 0.88 + 0.08 * n(841, x, y));
    }
    let own = 0.88 + 0.14 * n(842, (x + shift).rem_euclid(16) / 8 + row * 3, row);
    rgb([168.0, 88.0, 58.0], own + 0.06 * mottle(843, x, y, 0.25))
}

/// Refractory ceramic & dark iron chimney cap on top of the kiln.
fn kiln_top(x: i32, y: i32) -> [u8; 4] {
    let dx = (x - 8).abs();
    let dy = (y - 8).abs();
    if dx <= 3 && dy <= 3 {
        return rgb([52.0, 52.0, 58.0], 0.85 + 0.12 * n(844, x, y));
    }
    rgb([154.0, 84.0, 58.0], 0.88 + 0.16 * mottle(845, x, y, 0.25))
}

/// Warm molten-glass crucible window glow.
fn kiln_glow(x: i32, y: i32) -> [u8; 4] {
    let dist = ((x - 8) * (x - 8) + (y - 8) * (y - 8)) as f64;
    let hot = (1.0 - dist / 90.0).clamp(0.0, 1.0);
    let c = [235.0 + 20.0 * hot, 110.0 + 115.0 * hot, 26.0 + 70.0 * hot];
    rgb(c, 0.92 + 0.08 * n(846, x, y))
}

/// Brushed steel & brass paint-sprayer body texture (used on the handheld model).
fn paint_sprayer(x: i32, y: i32) -> [u8; 4] {
    if y % 5 == 0 {
        return rgb([190.0, 152.0, 74.0], 0.9 + 0.1 * n(847, x, y));
    }
    rgb([154.0, 162.0, 174.0], 0.88 + 0.14 * mottle(848, x, y, 0.25))
}
