//! Nuclear power (Milestone 10): the fuel cell (a steel-cased rod with a green glowing core), the centrifuge's tubes
//! and spun lid, and the reactor's round concrete shell with a hazard band and its lid of fuel channels. Placeholder looks
//! until the art pass (`docs/ART_HANDOVER.md`).

use crate::block::tex;

use super::paint::round_shade;
use super::{n, rgb, smooth};

pub fn pixel(layer: u16, x: i32, y: i32) -> [u8; 4] {
    match layer {
        tex::FUEL_CELL => fuel_cell(x, y),
        tex::CENTRIFUGE_SIDE => centrifuge_side(x, y),
        tex::CENTRIFUGE_TOP => centrifuge_top(x, y),
        tex::REACTOR_SIDE => reactor_side(x, y),
        _ => reactor_top(x, y),
    }
}

const GLOW: [f64; 3] = [150.0, 236.0, 70.0];
const HAZARD: [f64; 3] = [232.0, 190.0, 36.0];

/// A dark steel case with a bright green core down the middle and a yellow band at each end.
fn fuel_cell(x: i32, y: i32) -> [u8; 4] {
    if y <= 2 || y >= 13 {
        return rgb(HAZARD, 0.9 + 0.1 * n(551, x, y));
    }
    if (5..11).contains(&x) {
        return rgb(GLOW, 0.85 + 0.3 * smooth(552, x, y, 4));
    }
    rgb([62.0, 68.0, 78.0], 0.85 + 0.2 * n(553, x, y))
}

/// A slim spun tube of pale steel, shaded round (`round_shade`), with a dark ring near each end and a thin green
/// gauge line.
fn centrifuge_side(x: i32, y: i32) -> [u8; 4] {
    let shade = round_shade(x);
    if y == 1 || y == 14 {
        return rgb([70.0, 80.0, 92.0], shade);
    }
    if y == 8 && (6..10).contains(&x) {
        return rgb(GLOW, 0.9);
    }
    rgb([150.0, 160.0, 174.0], shade * (0.9 + 0.1 * n(555, x, y)))
}
/// A round lid with spiral grooves (the spin) round a dark hub.
fn centrifuge_top(x: i32, y: i32) -> [u8; 4] {
    let (dx, dy) = (x as f64 - 7.5, y as f64 - 7.5);
    let r = (dx * dx + dy * dy).sqrt();
    if r < 2.0 {
        return rgb([36.0, 40.0, 48.0], 1.0);
    }
    if r > 7.4 {
        return rgb([70.0, 80.0, 92.0], 0.9 + 0.1 * n(556, x, y));
    }
    let turn = (dy.atan2(dx) * 2.0 + r).rem_euclid(std::f64::consts::TAU);
    let groove = turn < 1.0;
    rgb([150.0, 160.0, 174.0], if groove { 0.62 } else { 0.95 + 0.1 * n(557, x, y) })
}

/// A round concrete shell (`round_shade`) with formwork lines and a yellow and black hazard band round its foot.
fn reactor_side(x: i32, y: i32) -> [u8; 4] {
    if y >= 13 {
        let black = (x + y) % 6 < 3;
        return if black { rgb([30.0, 30.0, 30.0], 1.0) } else { rgb(HAZARD, 0.95) };
    }
    let shade = round_shade(x);
    let line = y == 4 || y == 9;
    rgb(
        [150.0, 148.0, 140.0],
        shade * if line { 0.78 } else { 0.86 + 0.14 * n(559, x, y) + 0.08 * smooth(560, x, y, 4) },
    )
}
/// A concrete lid with a 3×3 grid of dark fuel channels, each with a green glow at the bottom.
fn reactor_top(x: i32, y: i32) -> [u8; 4] {
    let (cx, cy) = ((x - 1) % 5, (y - 1) % 5);
    if x > 0 && y > 0 && (1..4).contains(&cx) && (1..4).contains(&cy) {
        return if cx == 2 && cy == 2 { rgb(GLOW, 0.8) } else { rgb([34.0, 36.0, 38.0], 1.0) };
    }
    rgb([120.0, 120.0, 116.0], 0.85 + 0.2 * n(561, x, y))
}
