//! Placeholder tool pictures, to be restyled (docs/ART_HANDOVER.md): a wooden handle on the diagonal and a
//! head in the tier's colour, shaped by kind (0 pickaxe, 1 axe, 2 shovel), on a dark card (item boxes are
//! opaque).

use super::{n, rgb};

pub fn tool(x: i32, y: i32, kind: u16, iron: bool) -> [u8; 4] {
    // Along and across the handle, which runs from the bottom left to the head at the top right.
    let (rx, ry) = (x as f64 - 11.5, y as f64 - 3.5);
    let along = (rx - ry) * std::f64::consts::FRAC_1_SQRT_2;
    let across = (rx + ry) * std::f64::consts::FRAC_1_SQRT_2;
    let head = match kind {
        0 => along.abs() < 1.3 && across.abs() < 5.5,
        1 => along.abs() < 1.8 && (0.0..4.0).contains(&across),
        _ => (-1.5..3.0).contains(&along) && across.abs() < 2.3,
    };
    let k = 0.88 + 0.12 * n(70 + kind as u32, x, y);
    if head {
        rgb(if iron { [204.0, 208.0, 216.0] } else { [124.0, 124.0, 130.0] }, k)
    } else if across.abs() < 0.8 && (-12.0..0.0).contains(&along) {
        rgb([140.0, 98.0, 58.0], k)
    } else {
        rgb([44.0, 48.0, 58.0], 1.0)
    }
}
