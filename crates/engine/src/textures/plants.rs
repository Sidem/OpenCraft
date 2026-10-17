//! The torch on crossed transparent quads (a stick with a flame; the plants are in `hires.rs`). Transparent
//! texels keep a nearby RGB to avoid dark fringes in the mip chain.

use super::{n, rgb};

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
