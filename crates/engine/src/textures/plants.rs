//! Plant patterns (placeholders for the art agent): drawn on a transparent background for the
//! mesher's crossed quads (`Render::Plant`).

use super::{n, rgb};

/// A young tree: a thin stem and a small round crown.
pub fn sapling(x: i32, y: i32) -> [u8; 4] {
    let leaf = [70.0, 136.0, 48.0];
    let (dx, dy) = (x as f64 - 7.5, y as f64 - 6.0);
    if dx * dx + dy * dy * 1.3 < 22.0 && n(60, x, y) > 0.15 {
        rgb(leaf, 0.7 + 0.4 * n(61, x, y))
    } else if (7..=8).contains(&x) && y >= 8 {
        rgb([112.0, 82.0, 50.0], 0.85 + 0.2 * n(62, x, y))
    } else {
        // Transparent, but keep a leaf colour so mipmaps don't bleed dark fringes.
        let mut c = rgb(leaf, 0.8);
        c[3] = 0;
        c
    }
}
