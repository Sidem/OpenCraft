//! Compute (Milestone 11): the pure water canister (the canister drawing with a clear band), the wafer (a bluish disc
//! on a dark ground with a die grid), the AI accelerator (a dark board with a gold-pinned chip) and the chip fab's
//! clean-room panels and roof fan grille. Placeholder looks until the art pass (`docs/ART_HANDOVER.md`).

use crate::block::tex;

use super::chemistry::canister;
use super::{frame, n, rgb, smooth};

const PURE_BAND: [f64; 3] = [190.0, 236.0, 250.0];

pub fn pixel(layer: u16, x: i32, y: i32) -> [u8; 4] {
    match layer {
        tex::PURE_WATER_CANISTER => canister(x, y, PURE_BAND),
        tex::WAFER => wafer(x, y),
        tex::ACCELERATOR => accelerator(x, y),
        tex::FAB_SIDE => fab_side(x, y),
        _ => fab_top(x, y),
    }
}

/// A silicon disc, blue-grey, cut into a grid of dies, on a dark ground.
fn wafer(x: i32, y: i32) -> [u8; 4] {
    let (dx, dy) = (x as f64 - 7.5, y as f64 - 7.5);
    let r = (dx * dx + dy * dy).sqrt();
    if r > 7.4 {
        return rgb([30.0, 32.0, 40.0], 0.9 + 0.2 * n(611, x, y));
    }
    if r > 6.6 {
        return rgb([176.0, 184.0, 200.0], 0.9);
    }
    let seam = x % 4 == 0 || y % 4 == 0;
    let k = 0.92 + 0.14 * smooth(612, x, y, 4);
    if seam {
        rgb([70.0, 84.0, 124.0], k)
    } else {
        rgb([112.0, 132.0, 190.0], k)
    }
}

/// A dark green board with traces, and a square chip with gold pins in the middle.
fn accelerator(x: i32, y: i32) -> [u8; 4] {
    let chip = (4..=11).contains(&x) && (4..=11).contains(&y);
    let pins = chip && (x == 4 || x == 11 || y == 4 || y == 11) && (x + y) % 2 == 0;
    if pins {
        return rgb([226.0, 182.0, 58.0], 1.0);
    }
    if chip {
        return rgb([36.0, 40.0, 52.0], 0.9 + 0.2 * n(613, x, y));
    }
    let trace = (x % 4 == 1 || y % 4 == 1) && n(614, x / 4, y / 4) > 0.4;
    if trace {
        rgb([190.0, 150.0, 60.0], 0.8)
    } else {
        rgb([30.0, 84.0, 64.0], 0.85 + 0.2 * n(615, x, y))
    }
}

/// Pale clean-room panels with dark seams and a small pass-through window.
fn fab_side(x: i32, y: i32) -> [u8; 4] {
    if y == 0 || y == 15 || x == 0 || x == 15 || x == 8 {
        return rgb([70.0, 76.0, 96.0], 0.9 + 0.1 * n(616, x, y));
    }
    let window = (2..=5).contains(&x) && (5..=9).contains(&y);
    if window {
        return rgb([60.0, 150.0, 190.0], 0.9 + 0.2 * n(617, x, y));
    }
    rgb([210.0, 216.0, 228.0], 0.9 + 0.1 * n(618, x, y) + 0.06 * smooth(619, x, y, 4))
}

/// A steel deck with a round fan grille in the middle.
fn fab_top(x: i32, y: i32) -> [u8; 4] {
    let (dx, dy) = (x as f64 - 7.5, y as f64 - 7.5);
    let d = (dx * dx + dy * dy).sqrt();
    if d < 5.5 {
        return if (x + y) % 3 == 0 { rgb([150.0, 156.0, 168.0], 1.0) } else { rgb([22.0, 24.0, 30.0], 1.0) };
    }
    frame(x, y)
}
