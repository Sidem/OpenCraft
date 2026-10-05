//! The hoist (Milestone 10): the shaft's cutout frame (steel rails with a yellow cross brace, see-through between them,
//! like the ladder's), its ends, and the winch's plating. Placeholder looks until the art pass (`docs/ART_HANDOVER.md`).

use crate::block::tex;

use super::{n, rgb};

pub fn pixel(layer: u16, x: i32, y: i32) -> [u8; 4] {
    match layer {
        tex::HOIST => shaft(x, y),
        tex::HOIST_TOP => shaft_end(x, y),
        _ => winch_side(x, y),
    }
}

const CLEAR: [u8; 4] = [0, 0, 0, 0];
const STEEL: [f64; 3] = [118.0, 128.0, 142.0];
const BRACE: [f64; 3] = [226.0, 180.0, 40.0];

/// Two steel rails with a diagonal yellow brace between them.
fn shaft(x: i32, y: i32) -> [u8; 4] {
    if x <= 1 || x >= 14 {
        return rgb(STEEL, 0.85 + 0.2 * n(540, x, y) - if x == 1 || x == 14 { 0.1 } else { 0.0 });
    }
    if (x + y) % 8 <= 1 {
        return rgb(BRACE, 0.9 + 0.1 * n(541, x, y));
    }
    CLEAR
}

/// The rails' ends in the corners.
fn shaft_end(x: i32, y: i32) -> [u8; 4] {
    if (x <= 1 || x >= 14) && (y <= 1 || y >= 14) {
        rgb(STEEL, 1.0 + 0.1 * n(542, x, y))
    } else {
        CLEAR
    }
}

/// Dark steel plating with a yellow stripe and a drum of cable in the middle.
fn winch_side(x: i32, y: i32) -> [u8; 4] {
    if y == 0 || y == 15 {
        return rgb([54.0, 62.0, 70.0], 0.9 + 0.1 * n(543, x, y));
    }
    if (6..10).contains(&y) {
        let ring = (x + y) % 3 == 0;
        return rgb([92.0, 96.0, 104.0], if ring { 0.7 } else { 1.0 });
    }
    if y == 3 || y == 12 {
        return rgb(BRACE, 0.95);
    }
    rgb([78.0, 88.0, 100.0], 0.88 + 0.14 * n(544, x, y))
}
