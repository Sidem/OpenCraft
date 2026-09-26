//! Placeholder tool pictures, to be restyled (docs/ART_HANDOVER.md): a wooden handle on the diagonal and a
//! head in the tier's colour, shaped by kind (0 pickaxe, 1 axe, 2 shovel), on a dark card (item boxes are
//! opaque). The prospecting devices: a scanner (a handset with a green sweep on its screen) and a core
//! drill (a motor housing over a long bit).

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

const CARD: [f64; 3] = [44.0, 48.0, 58.0];

pub fn scanner(x: i32, y: i32) -> [u8; 4] {
    let k = 0.9 + 0.1 * n(80, x, y);
    let body = (3..13).contains(&x) && (2..15).contains(&y);
    let screen = (5..11).contains(&x) && (4..10).contains(&y);
    if screen {
        // Rings of a sweep around the screen's lower centre.
        let r = ((x as f64 - 7.5).powi(2) + (y as f64 - 9.0).powi(2)).sqrt();
        let ring = (r % 2.5) < 0.9;
        rgb(if ring { [120.0, 230.0, 140.0] } else { [24.0, 60.0, 36.0] }, 1.0)
    } else if body {
        let button = (y == 12) && (x == 6 || x == 9);
        rgb(if button { [220.0, 180.0, 60.0] } else { [196.0, 120.0, 52.0] }, k)
    } else if (6..10).contains(&x) && y == 1 {
        rgb([90.0, 90.0, 96.0], k) // antenna
    } else {
        rgb(CARD, 1.0)
    }
}

pub fn core_drill(x: i32, y: i32) -> [u8; 4] {
    let k = 0.88 + 0.12 * n(81, x, y);
    let housing = (3..13).contains(&x) && (1..7).contains(&y);
    let bit = (7..9).contains(&x) && (7..15).contains(&y);
    if housing {
        let vent = y == 3 && x % 2 == 0;
        rgb(if vent { [60.0, 60.0, 66.0] } else { [210.0, 170.0, 60.0] }, k)
    } else if bit {
        let flute = (x + y) % 3 == 0;
        rgb(if flute { [140.0, 144.0, 152.0] } else { [204.0, 208.0, 216.0] }, k)
    } else {
        rgb(CARD, 1.0)
    }
}
