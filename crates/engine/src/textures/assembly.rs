//! Assembly (Assembly tech): the assembler's housing (teal panels with a service grille, a riveted
//! top with a hatch), the port hatches (a framed square: a cyan inlet, an amber outlet, each with a
//! chevron), concrete (grey with fine aggregate and form lines) and the motor item (a copper-wound
//! drum with a steel end). Placeholder looks until the art pass (`docs/ART_HANDOVER.md`).

use crate::block::tex;

use super::{frame, n, rgb, smooth};

pub fn pixel(layer: u16, x: i32, y: i32) -> [u8; 4] {
    match layer {
        tex::ASSEMBLER_SIDE => assembler_side(x, y),
        tex::ASSEMBLER_TOP => assembler_top(x, y),
        tex::PORT_IN => port(x, y, [70.0, 190.0, 225.0]),
        tex::PORT_OUT => port(x, y, [240.0, 170.0, 50.0]),
        tex::CONCRETE => concrete(x, y),
        _ => motor(x, y),
    }
}

fn assembler_side(x: i32, y: i32) -> [u8; 4] {
    let k = 0.9 + 0.08 * n(340, x, y / 3);
    if x == 0 || x == 15 || y == 0 || y == 15 {
        rgb([46.0, 60.0, 64.0], k)
    } else if (4..12).contains(&x) && (9..13).contains(&y) {
        // A service grille.
        if y % 2 == 0 {
            rgb([30.0, 36.0, 40.0], 1.0)
        } else {
            rgb([90.0, 100.0, 104.0], k)
        }
    } else {
        rgb([52.0, 138.0, 128.0], k + if y < 3 { 0.08 } else { 0.0 })
    }
}

fn assembler_top(x: i32, y: i32) -> [u8; 4] {
    let hatch = (5..11).contains(&x) && (5..11).contains(&y);
    if hatch && (x == 5 || x == 10 || y == 5 || y == 10) {
        rgb([40.0, 44.0, 50.0], 1.0)
    } else if hatch {
        rgb([96.0, 104.0, 112.0], 0.9 + 0.1 * n(341, x, y))
    } else {
        frame(x, y)
    }
}

/// A framed hatch in `colour` with a chevron pointing up.
fn port(x: i32, y: i32, colour: [f64; 3]) -> [u8; 4] {
    let rim = x <= 1 || x >= 14 || y <= 1 || y >= 14;
    let chevron = (y - 4 == (x - 7).abs() || y - 5 == (x - 7).abs()) && (4..11).contains(&y);
    if rim {
        rgb(colour, 0.85 + 0.1 * n(342, x, y))
    } else if chevron {
        rgb(colour, 1.0)
    } else {
        rgb([34.0, 38.0, 44.0], 0.9 + 0.1 * n(343, x, y))
    }
}

fn concrete(x: i32, y: i32) -> [u8; 4] {
    let grain = 0.92 + 0.06 * n(344, x, y) + 0.05 * smooth(345, x, y, 4);
    let form = if y == 7 { 0.9 } else { 1.0 };
    let stone = if n(346, x, y) > 0.93 { 0.85 } else { 1.0 };
    rgb([168.0, 166.0, 160.0], grain * form * stone)
}

fn motor(x: i32, y: i32) -> [u8; 4] {
    if x < 4 {
        return rgb([150.0, 156.0, 164.0], 0.9 + 0.1 * n(347, x, y));
    }
    let winding = if y % 2 == 0 { 1.0 } else { 0.82 };
    rgb([196.0, 112.0, 60.0], winding * (0.9 + 0.08 * n(348, x, y)))
}
