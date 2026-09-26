//! Young Alpine sapling on crossed transparent quads; narrow stem and uneven needle sprays.
//! Transparent texels retain green RGB to avoid dark fringes in the mip chain.

use super::{n, rgb};

pub fn sapling(x: i32, y: i32) -> [u8; 4] {
    let needles = [62.0, 105.0, 79.0];
    let stem = x == 7 && y >= 4 || x == 8 && y >= 9;
    let twig = y == 7 && (4..=7).contains(&x) || y == 9 && (8..=11).contains(&x) || y == 11 && (5..=7).contains(&x);
    if stem || twig {
        return rgb([83.0, 71.0, 54.0], 0.88 + 0.16 * n(280, x, y));
    }
    let sprays = [(6, 4, 3, 2), (10, 5, 3, 2), (4, 8, 3, 2), (11, 9, 3, 2), (5, 11, 2, 1)];
    for (cx, cy, rx, ry) in sprays {
        let (dx, dy) = ((x - cx).abs(), (y - cy).abs());
        if dx * ry + dy * rx <= rx * ry && n(281, x, y) > 0.28 {
            return rgb(needles, 0.82 + 0.26 * n(282, x, y));
        }
    }
    let mut c = rgb(needles, 0.82);
    c[3] = 0;
    c
}
