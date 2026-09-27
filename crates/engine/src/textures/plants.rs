//! Plants and small things on crossed transparent quads: a young Alpine sapling (narrow stem, uneven
//! needle sprays) and a torch (a stick with a flame). Transparent texels keep a nearby RGB to avoid
//! dark fringes in the mip chain.

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

/// A stick two texels wide with a coal head and a flame above it.
pub fn torch(x: i32, y: i32) -> [u8; 4] {
    let stick = (7..=8).contains(&x);
    if stick && y >= 8 {
        return rgb([122.0, 88.0, 52.0], 0.82 + 0.2 * n(283, x, y));
    }
    if stick && (6..=7).contains(&y) {
        return rgb([48.0, 40.0, 36.0], 0.9 + 0.2 * n(284, x, y));
    }
    let (dx, dy) = (x as f64 - 7.5, y as f64 - 3.5);
    let r = dx * dx / 4.0 + dy * dy / 6.0;
    if r <= 1.0 {
        let hot = 1.0 - r;
        let c = [255.0, 150.0 + 100.0 * hot, 40.0 + 150.0 * hot * hot];
        return rgb(c, 0.95 + 0.1 * n(285, x, y));
    }
    let mut c = rgb([255.0, 170.0, 60.0], 0.8);
    c[3] = 0;
    c
}
