//! Subdued province rocks (granite, sandstone, basalt) and cutout glass, in the shared Alpine grain.
//! Ore-bearing rock (limestone, quartz) lives in `ores.rs`, surface hints in `nature.rs`.

use super::{n, nature, rgb, smooth};

/// Warm feldspar, grey mica and tiny dark inclusions in a fine-grained matrix.
pub fn granite(x: i32, y: i32) -> [u8; 4] {
    let mut c = nature::grain([153, 148, 143], 200, x, y);
    let mineral = smooth(204, x, y, 4);
    if mineral > 0.67 {
        c = [c[0].saturating_add(9), c[1].saturating_add(2), c[2].saturating_sub(2), 255];
    } else if mineral < 0.28 {
        c = [c[0].saturating_sub(13), c[1].saturating_sub(12), c[2].saturating_sub(9), 255];
    }
    c
}

/// Fine ochre sediment with bent, low-contrast bedding rather than four-pixel stripes.
pub fn sandstone(x: i32, y: i32) -> [u8; 4] {
    let c = nature::grain([194, 177, 147], 210, x, y);
    let bend = (smooth(214, x, 0, 4) * 3.0) as i32;
    let lamina = (y + bend).rem_euclid(8);
    let d = if lamina == 0 {
        -7
    } else if lamina == 1 {
        -3
    } else {
        0
    };
    tint(c, [d, d, d])
}

/// Cool volcanic stone with occasional fine vesicles.
pub fn basalt(x: i32, y: i32) -> [u8; 4] {
    let c = nature::grain([68, 79, 86], 220, x, y);
    if n(224, x, y) > 0.976 && smooth(225, x, y, 4) > 0.4 {
        tint(c, [-13, -13, -12])
    } else {
        c
    }
}

/// Broken icy glints describe a clear panel without drawing a bright grid on every block.
pub fn glass(x: i32, y: i32) -> [u8; 4] {
    let edge = x == 0 || x == 15 || y == 0 || y == 15;
    let glint = n(261, x, y) > 0.974 && smooth(262, x, y, 4) > 0.53;
    let mut c = rgb([174.0, 207.0, 211.0], 0.92 + 0.08 * n(263, x, y));
    if !(edge && n(264, x, y) > 0.89 || glint) {
        c[3] = 0;
    }
    c
}

fn tint(c: [u8; 4], delta: [i8; 3]) -> [u8; 4] {
    [
        c[0].saturating_add_signed(delta[0]),
        c[1].saturating_add_signed(delta[1]),
        c[2].saturating_add_signed(delta[2]),
        c[3],
    ]
}
