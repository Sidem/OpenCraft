//! Recycling: the coin a recycler pays out (a gold disc with a raised rim and a triangle of arrows) and the recycler's
//! plating (green-grey panels with a gold ring round a dark chute) and lid (a grate over a pit). Placeholder looks until
//! the art pass (`docs/ART_HANDOVER.md`).

use crate::block::tex;

use super::{n, rgb, smooth};

pub fn pixel(layer: u16, x: i32, y: i32) -> [u8; 4] {
    match layer {
        tex::COIN => coin(x, y),
        tex::RECYCLER_SIDE => recycler_side(x, y),
        _ => recycler_top(x, y),
    }
}

const GOLD: [f64; 3] = [226.0, 182.0, 58.0];
const GREEN: [f64; 3] = [74.0, 122.0, 96.0];

/// A gold disc: a darker rim, a dark triangle in the middle (the recycling arrows' shape).
fn coin(x: i32, y: i32) -> [u8; 4] {
    let (dx, dy) = (x as f64 - 7.5, y as f64 - 7.5);
    let r = (dx * dx + dy * dy).sqrt();
    if r > 7.6 {
        return rgb([120.0, 90.0, 24.0], 0.9);
    }
    if r > 6.2 {
        return rgb(GOLD, 0.7 + 0.1 * n(601, x, y));
    }
    let up = dy > -3.5 && dy < 3.0 && dx.abs() < (dy + 3.5) * 0.6 + 0.4;
    let hollow = dy > -1.5 && dy < 1.5 && dx.abs() < (dy + 1.5) * 0.3;
    if up && !hollow {
        return rgb([130.0, 92.0, 20.0], 1.0);
    }
    rgb(GOLD, 0.9 + 0.2 * smooth(602, x, y, 4))
}

/// Green-grey plating with seams and a dark round chute ringed in gold.
fn recycler_side(x: i32, y: i32) -> [u8; 4] {
    if y == 0 || y == 15 || x == 0 || x == 15 {
        return rgb([40.0, 62.0, 52.0], 0.9 + 0.1 * n(603, x, y));
    }
    let (dx, dy) = (x as f64 - 7.5, y as f64 - 7.5);
    let r = (dx * dx + dy * dy).sqrt();
    if r < 3.0 {
        return rgb([22.0, 26.0, 24.0], 1.0);
    }
    if r < 4.6 {
        return rgb(GOLD, 0.85 + 0.15 * n(604, x, y));
    }
    rgb(GREEN, 0.85 + 0.2 * n(605, x, y) + 0.1 * smooth(606, x, y, 4))
}

/// A grate of dark slots over a deep pit, in a green frame.
fn recycler_top(x: i32, y: i32) -> [u8; 4] {
    if !(2..=13).contains(&x) || !(2..=13).contains(&y) {
        return rgb(GREEN, 0.8 + 0.2 * n(607, x, y));
    }
    if x % 3 == 0 {
        return rgb([110.0, 120.0, 116.0], 0.9);
    }
    rgb([18.0, 22.0, 20.0], 1.0 + 0.3 * n(608, x, y))
}
