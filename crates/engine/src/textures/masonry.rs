//! Masonry (Masonry tech): stone bricks (grey dressed blocks in a running bond, recessed mortar, each
//! brick's face a little lighter at its top edge) and the quicklime item (white lumps with a chalky
//! grain). Placeholder looks until the art pass (`docs/ART_HANDOVER.md`).

use super::{n, rgb, smooth};

pub fn stone_bricks(x: i32, y: i32) -> [u8; 4] {
    let row = y / 4;
    let shift = if row % 2 == 0 { 0 } else { 4 };
    let bx = (x + shift).rem_euclid(16) / 8;
    let mortar = y % 4 == 3 || (x + shift).rem_euclid(8) == 7;
    if mortar {
        return rgb([88.0, 88.0, 86.0], 0.9 + 0.08 * n(320, x, y));
    }
    // Each brick its own shade; the top row of a brick catches the light.
    let own = 0.88 + 0.1 * n(321, bx + row * 3, row);
    let lit = if y % 4 == 0 { 0.07 } else { 0.0 };
    rgb([150.0, 150.0, 146.0], own + lit + 0.05 * n(322, x, y) - 0.04 * smooth(323, x, y, 4))
}

pub fn quicklime(x: i32, y: i32) -> [u8; 4] {
    let lump = smooth(330, x, y, 4);
    let k = 0.86 + 0.12 * lump + 0.05 * n(331, x, y);
    if lump < 0.3 {
        rgb([200.0, 198.0, 188.0], k)
    } else {
        rgb([238.0, 236.0, 228.0], k)
    }
}
