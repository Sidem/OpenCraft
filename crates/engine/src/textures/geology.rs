//! Rock and mineral patterns of Milestone 4 (placeholders for the art agent): the province rocks
//! (granite, sandstone, basalt), limestone, and glass (cutout: mostly transparent).

use super::{n, rgb, smooth};

/// Coarse grains: pale, pink and dark specks.
pub fn granite(x: i32, y: i32) -> [u8; 4] {
    let s = n(70, x, y);
    let c = if s < 0.18 {
        [70.0, 66.0, 66.0]
    } else if s < 0.45 {
        [196.0, 150.0, 140.0]
    } else {
        [176.0, 170.0, 166.0]
    };
    rgb(c, 0.85 + 0.15 * smooth(71, x, y, 4))
}

/// Sand-coloured layers.
pub fn sandstone(x: i32, y: i32) -> [u8; 4] {
    let band = if (y / 4) % 2 == 0 { 1.0 } else { 0.9 };
    rgb([214.0, 186.0, 128.0], band * (0.88 + 0.1 * n(72, x, y) + 0.05 * smooth(73, x, y, 2)))
}

/// Dark, fine-grained, with a few pores.
pub fn basalt(x: i32, y: i32) -> [u8; 4] {
    let pore = if n(74, x, y) < 0.06 { 0.55 } else { 1.0 };
    rgb([62.0, 62.0, 68.0], pore * (0.85 + 0.2 * n(75, x, y) + 0.1 * smooth(76, x, y, 4)))
}

/// Pale cream with faint shell flecks.
pub fn limestone(x: i32, y: i32) -> [u8; 4] {
    let fleck = if n(77, x, y) < 0.08 { 0.8 } else { 1.0 };
    rgb([218.0, 212.0, 188.0], fleck * (0.9 + 0.08 * n(78, x, y) + 0.06 * smooth(79, x, y, 4)))
}

/// Dirt stained towards `tint` in blotches (surface hints over deposits).
pub fn soil(x: i32, y: i32, tint: [f64; 3]) -> [u8; 4] {
    let d = super::dirt(x, y);
    let k = 0.35 + 0.5 * smooth(81, x, y, 4);
    let mix = |i: usize| d[i] as f64 * (1.0 - k) + tint[i] * k;
    rgb([mix(0), mix(1), mix(2)], 1.0)
}

/// A light frame and a couple of glints; the rest is see-through.
pub fn glass(x: i32, y: i32) -> [u8; 4] {
    let edge = x == 0 || y == 0 || x == 15 || y == 15;
    let glint = (x - y == 4 || x - y == 6) && (3..9).contains(&x);
    let mut c = rgb([200.0, 226.0, 232.0], if edge { 0.9 } else { 1.0 });
    if !edge && !glint {
        c[3] = 0;
    }
    c
}
