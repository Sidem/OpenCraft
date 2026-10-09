//! The swarm hub's faces (Milestone 11): a dark body with cyan drone-lane stripes and a crown with a ring of small
//! lights. Placeholders until the art pass.

use crate::block::tex;

use super::{frame, n, rgb};

pub fn pixel(layer: u16, x: i32, y: i32) -> [u8; 4] {
    if layer == tex::HUB_TOP {
        return top(x, y);
    }
    side(x, y)
}

/// Dark panels with two cyan lanes and a row of dots between them.
fn side(x: i32, y: i32) -> [u8; 4] {
    if x == 0 || x == 15 || y == 0 || y == 15 {
        return frame(x, y);
    }
    if y == 4 || y == 11 {
        return rgb([80.0, 220.0, 255.0], 0.85 + 0.3 * n(671, x, y));
    }
    if y == 7 && x % 3 == 1 {
        return rgb([200.0, 245.0, 255.0], 1.0);
    }
    rgb([24.0, 30.0, 40.0], 0.9 + 0.2 * n(672, x, y))
}

/// A crown: a cyan ring round a dark dome.
fn top(x: i32, y: i32) -> [u8; 4] {
    let (dx, dy) = (x as f64 - 7.5, y as f64 - 7.5);
    let r2 = dx * dx + dy * dy;
    if (24.0..38.0).contains(&r2) {
        return rgb([90.0, 225.0, 255.0], 1.0);
    }
    if r2 < 5.0 {
        return rgb([220.0, 250.0, 255.0], 1.0);
    }
    rgb([22.0, 28.0, 38.0], 0.9 + 0.2 * n(673, x, y))
}
