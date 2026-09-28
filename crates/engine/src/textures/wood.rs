//! Processed wood: planks (four staggered boards with seams, grain and nail heads), the ladder (two
//! rails at the edges with three rungs; its top shows the rails' ends) and the stick item. Pale sawn
//! spruce, lighter than the bark. Transparent texels keep a nearby RGB to avoid dark fringes.

use super::{n, rgb, smooth};

const WOOD: [f64; 3] = [178.0, 140.0, 96.0];
const SEAM: [f64; 3] = [92.0, 68.0, 48.0];

/// Grain along x: long streaks with a little per-texel noise.
fn grain(seed: u32, x: i32, y: i32) -> f64 {
    0.84 + 0.1 * n(seed, x / 4, y) + 0.08 * smooth(seed + 1, x, y, 4)
}

pub fn planks(x: i32, y: i32) -> [u8; 4] {
    let board = y / 4;
    // Each board's end joint sits somewhere else, so the rows read as separate boards.
    let joint = (board * 7 + 3) % 16;
    let seam = y % 4 == 3 || x == joint;
    let nail = y % 4 == 1 && (x == (joint + 2) % 16 || x == (joint + 14) % 16);
    if seam {
        rgb(SEAM, 0.9 + 0.08 * n(300, x, y))
    } else if nail {
        rgb([70.0, 66.0, 62.0], 1.0)
    } else {
        rgb(WOOD, grain(301 + board as u32, x, y) - 0.03 * (board % 2) as f64)
    }
}

/// A ladder's side: rails on the two edge columns, rungs on rows 2–3, 7–8 and 12–13.
pub fn ladder(x: i32, y: i32) -> [u8; 4] {
    let rail = x <= 1 || x >= 14;
    let rung = matches!(y % 5, 2 | 3);
    if rail {
        rgb(WOOD, grain(310, y, x) - if x == 1 || x == 14 { 0.08 } else { 0.0 })
    } else if rung {
        rgb(WOOD, grain(311, x, y) - if y % 5 == 3 { 0.1 } else { 0.0 })
    } else {
        clear()
    }
}

/// A ladder's top and bottom: the four rails' ends in the corners.
pub fn ladder_top(x: i32, y: i32) -> [u8; 4] {
    if (x <= 1 || x >= 14) && (y <= 1 || y >= 14) {
        rgb(SEAM, 1.1 + 0.1 * n(312, x, y))
    } else {
        clear()
    }
}

pub fn stick(x: i32, y: i32) -> [u8; 4] {
    rgb(WOOD, grain(320, x, y) - if y % 8 == 0 { 0.12 } else { 0.0 })
}

fn clear() -> [u8; 4] {
    let mut c = rgb(WOOD, 0.9);
    c[3] = 0;
    c
}
