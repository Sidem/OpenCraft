//! What flies (`factory/process/rocket.rs`, `factory/capsule.rs`, `factory/plume.rs`): the rocket's brushed stainless hull
//! with its ring welds, its black heat tiles, the drop pod's white hull with a window and its charred heat shield, and
//! the exhaust's four glowing layers, white-hot core to cooling tail plus the pale shock diamonds. A round body wraps
//! a side texture once a quarter turn (mirrored), so these are banded top to bottom, never left to right.

use crate::block::tex;

use super::{n, rgb};

pub fn pixel(layer: u16, x: i32, y: i32) -> [u8; 4] {
    match layer {
        tex::ROCKET_STEEL => steel(x, y),
        tex::ROCKET_TILE => tile(x, y),
        tex::CAPSULE_HULL => capsule(x, y),
        tex::CAPSULE_SHIELD => rgb([52.0, 44.0, 40.0], 0.8 + 0.35 * n(905, x, y)),
        tex::PLUME_CORE => fade([255.0, 252.0, 236.0], [255.0, 228.0, 150.0], x, y),
        tex::PLUME_FLAME => fade([255.0, 214.0, 110.0], [250.0, 140.0, 44.0], x, y),
        tex::PLUME_TAIL => fade([246.0, 120.0, 40.0], [190.0, 64.0, 28.0], x, y),
        _ => fade([246.0, 250.0, 255.0], [214.0, 232.0, 255.0], x, y),
    }
}

/// Brushed stainless: fine vertical streaks, a darker ring weld every four rows and a faint seam down one edge.
fn steel(x: i32, y: i32) -> [u8; 4] {
    let streak = 0.95 + 0.07 * n(901, x, y / 8) + 0.03 * n(902, x, y);
    let weld = if y % 4 == 3 { 0.93 } else { 1.0 };
    let seam = if x == 0 { 0.95 } else { 1.0 };
    rgb([206.0, 211.0, 219.0], streak * weld * seam)
}

/// Black hexagonal heat tiles: staggered rows of cells with thin grey grout.
fn tile(x: i32, y: i32) -> [u8; 4] {
    let row = y / 3;
    let shift = if row % 2 == 0 { 0 } else { 2 };
    let grout = y % 3 == 2 || (x + shift) % 4 == 3;
    if grout {
        return rgb([58.0, 58.0, 64.0], 0.9 + 0.2 * n(903, x, y));
    }
    rgb([22.0, 22.0, 26.0], 0.85 + 0.3 * n(904, x / 4, row))
}

/// The capsule's white skin: soft panel lines, a dark window near the top, a scorched rim at the bottom.
fn capsule(x: i32, y: i32) -> [u8; 4] {
    if y >= 14 {
        return rgb([70.0, 62.0, 58.0], 0.85 + 0.3 * n(906, x, y));
    }
    let corner = (x == 6 || x == 9) && (y == 3 || y == 5);
    if (6..=9).contains(&x) && (3..=5).contains(&y) && !corner {
        return rgb([28.0, 40.0, 60.0], 0.9 + 0.15 * n(907, x, y));
    }
    let line = if y == 9 || x == 15 { 0.86 } else { 1.0 };
    rgb([236.0, 238.0, 242.0], line * (0.96 + 0.05 * n(908, x, y)))
}

/// A vertical ramp from `top` to `bottom` with a little flicker grain (the exhaust layers).
fn fade(top: [f64; 3], bottom: [f64; 3], x: i32, y: i32) -> [u8; 4] {
    let t = y as f64 / 15.0;
    let c = [0, 1, 2].map(|i| top[i] + (bottom[i] - top[i]) * t);
    rgb(c, 0.97 + 0.04 * n(909, x, y))
}
