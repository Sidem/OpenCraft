//! The laser link blocks (Milestone 11): the emitter's dark housing with a red lens stripe and a ring crown, the
//! receiver's collector panels with a pale cell lattice, and the beam's light. Placeholders until the art pass.

use crate::block::tex;

use super::{frame, n, rgb};

pub fn pixel(layer: u16, x: i32, y: i32) -> [u8; 4] {
    match layer {
        tex::EMIT_SIDE => emit_side(x, y),
        tex::EMIT_TOP => emit_top(x, y),
        tex::RECV_SIDE => recv_side(x, y),
        tex::RECV_TOP => recv_top(x, y),
        _ => beam(x, y),
    }
}

/// A dark housing with a bright red stripe across the middle.
fn emit_side(x: i32, y: i32) -> [u8; 4] {
    if x == 0 || x == 15 || y == 0 || y == 15 {
        return frame(x, y);
    }
    if (6..=9).contains(&y) {
        return rgb([255.0, 70.0, 60.0], 0.85 + 0.3 * n(661, x, y));
    }
    rgb([30.0, 26.0, 34.0], 0.9 + 0.2 * n(662, x, y))
}

/// A dark cap with a red lens ring.
fn emit_top(x: i32, y: i32) -> [u8; 4] {
    let (dx, dy) = (x as f64 - 7.5, y as f64 - 7.5);
    let r2 = dx * dx + dy * dy;
    if (16.0..30.0).contains(&r2) {
        return rgb([255.0, 90.0, 70.0], 1.0);
    }
    if r2 < 6.0 {
        return rgb([255.0, 220.0, 210.0], 1.0);
    }
    rgb([30.0, 26.0, 34.0], 0.9 + 0.2 * n(663, x, y))
}

/// Dark blue collector panels in a lattice of pale cells.
fn recv_side(x: i32, y: i32) -> [u8; 4] {
    if x == 0 || x == 15 || y == 0 || y == 15 {
        return frame(x, y);
    }
    if x % 5 == 0 || y % 5 == 0 {
        return rgb([150.0, 170.0, 200.0], 0.9 + 0.2 * n(664, x, y));
    }
    rgb([22.0, 40.0, 82.0], 0.9 + 0.2 * n(665, x, y))
}

/// A wide collector dish: concentric blue rings with a pale centre.
fn recv_top(x: i32, y: i32) -> [u8; 4] {
    let (dx, dy) = (x as f64 - 7.5, y as f64 - 7.5);
    let r = (dx * dx + dy * dy).sqrt();
    if r < 1.8 {
        return rgb([230.0, 240.0, 255.0], 1.0);
    }
    if (r as i32) % 3 == 0 {
        return rgb([90.0, 130.0, 210.0], 1.0);
    }
    rgb([24.0, 42.0, 90.0], 0.9 + 0.2 * n(666, x, y))
}

/// The beam: a hot white-red core, fully lit.
fn beam(x: i32, y: i32) -> [u8; 4] {
    let d = ((x as f64 - 7.5).abs()).max((y as f64 - 7.5).abs());
    if d < 3.0 {
        rgb([255.0, 235.0, 225.0], 1.0)
    } else {
        rgb([255.0, 80.0, 70.0], 1.0)
    }
}
