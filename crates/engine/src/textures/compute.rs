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
        tex::NODE_SIDE => node_side(x, y),
        tex::NODE_TOP => node_top(x, y),
        tex::FIBRE => fibre(x, y),
        tex::DC_SIDE => dc_side(x, y),
        tex::DC_TOP => dc_top(x, y),
        tex::TOWER_SIDE => tower_side(x, y),
        tex::TOWER_TOP => tower_top(x, y),
        tex::AI_SIDE => ai_side(x, y),
        tex::AI_TOP => ai_top(x, y),
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

/// A dark blue-grey post with bright cyan bands.
fn node_side(x: i32, y: i32) -> [u8; 4] {
    if y % 5 == 2 {
        return rgb([90.0, 220.0, 235.0], 0.95 + 0.1 * n(621, x, y));
    }
    rgb([44.0, 52.0, 70.0], 0.9 + 0.2 * n(622, x, y) + 0.1 * smooth(623, x, y, 4))
}

/// A steel cap with a glowing cyan lens.
fn node_top(x: i32, y: i32) -> [u8; 4] {
    let (dx, dy) = (x as f64 - 7.5, y as f64 - 7.5);
    if (dx * dx + dy * dy).sqrt() < 4.5 {
        return rgb([110.0, 235.0, 245.0], 1.0 - 0.015 * (dx * dx + dy * dy));
    }
    frame(x, y)
}

/// The light a fibre line carries: pale cyan with a faint grain.
fn fibre(x: i32, y: i32) -> [u8; 4] {
    rgb([150.0, 240.0, 250.0], 0.92 + 0.14 * n(624, x, y))
}

/// Rows of dark server racks with a lit green indicator strip on every other row.
fn dc_side(x: i32, y: i32) -> [u8; 4] {
    if x == 0 || x == 15 || y == 0 || y == 15 {
        return rgb([70.0, 76.0, 96.0], 0.9 + 0.1 * n(625, x, y));
    }
    if y % 4 == 2 && x % 3 != 0 {
        let lit = n(626, x, y) > 0.45;
        return if lit { rgb([90.0, 235.0, 130.0], 1.0) } else { rgb([30.0, 80.0, 50.0], 1.0) };
    }
    rgb([34.0, 38.0, 50.0], 0.9 + 0.2 * n(627, x, y) + 0.08 * smooth(628, x, y, 4))
}

/// A steel roof with a square vent grille.
fn dc_top(x: i32, y: i32) -> [u8; 4] {
    if (3..=12).contains(&x) && (3..=12).contains(&y) {
        return if (x + y) % 2 == 0 { rgb([150.0, 156.0, 168.0], 1.0) } else { rgb([24.0, 26.0, 32.0], 1.0) };
    }
    frame(x, y)
}

/// Pale concrete louvres: slanted slats with dark gaps and a damp streak down the middle.
fn tower_side(x: i32, y: i32) -> [u8; 4] {
    if y % 4 == 3 {
        return rgb([36.0, 44.0, 52.0], 1.0);
    }
    let damp = if (6..=9).contains(&x) { 0.82 } else { 1.0 };
    rgb([150.0, 158.0, 166.0], damp * (0.9 + 0.15 * n(631, x, y) + 0.04 * (y % 4) as f64))
}

/// A dark water basin with a steel fan hub and four blades.
fn tower_top(x: i32, y: i32) -> [u8; 4] {
    let (dx, dy) = (x as f64 - 7.5, y as f64 - 7.5);
    let r2 = dx * dx + dy * dy;
    if r2 < 5.0 {
        return rgb([170.0, 176.0, 188.0], 1.0);
    }
    if (dx.abs() < 1.3 || dy.abs() < 1.3) && r2 < 42.0 {
        return rgb([84.0, 92.0, 104.0], 0.9 + 0.2 * n(632, x, y));
    }
    if r2 < 56.0 {
        return rgb([30.0, 62.0, 96.0], 0.9 + 0.2 * n(633, x, y));
    }
    frame(x, y)
}

/// A dark panel with a lattice of cyan nodes joined by faint lines: the neural net.
fn ai_side(x: i32, y: i32) -> [u8; 4] {
    if x == 0 || x == 15 || y == 0 || y == 15 {
        return frame(x, y);
    }
    let (col, row) = (x % 5 == 2, y % 5 == 2);
    if col && row {
        return rgb([110.0, 240.0, 255.0], 1.0);
    }
    if col || row {
        return rgb([30.0, 110.0, 130.0], 0.8 + 0.4 * n(641, x, y));
    }
    rgb([18.0, 24.0, 40.0], 0.9 + 0.2 * n(642, x, y))
}

/// A dark crown with a glowing cyan ring.
fn ai_top(x: i32, y: i32) -> [u8; 4] {
    let (dx, dy) = (x as f64 - 7.5, y as f64 - 7.5);
    let r2 = dx * dx + dy * dy;
    if (16.0..30.0).contains(&r2) {
        return rgb([90.0, 230.0, 250.0], 1.0);
    }
    if r2 < 5.0 {
        return rgb([200.0, 250.0, 255.0], 1.0);
    }
    rgb([20.0, 26.0, 44.0], 0.9 + 0.2 * n(643, x, y))
}
