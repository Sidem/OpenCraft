//! Subdued province rocks, quartz inclusions, surface-hint soils and cutout glass.
//! Opaque faces share the Alpine grain language; the terrain shader breaks tiling at world scale.

use super::{n, nature, rgb, smooth};
use crate::block::tex;

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

/// Chalky carbonate, warmer and softer than granite, with faint grey fossils.
pub fn limestone(x: i32, y: i32) -> [u8; 4] {
    let c = nature::grain([202, 199, 183], 230, x, y);
    let fossil = smooth(234, x, y, 4);
    if (0.46..0.52).contains(&fossil) && n(235, x, y) > 0.73 {
        tint(c, [-11, -10, -9])
    } else {
        c
    }
}

/// Pale quartz pockets follow a noisy fracture in the shared slate host.
pub fn quartz_ore(x: i32, y: i32) -> [u8; 4] {
    let host = nature::stone(x, y);
    let vein = smooth(240, x, y, 4) * 0.74 + smooth(241, x, y, 8) * 0.26;
    if vein > 0.69 {
        rgb([218.0, 215.0, 211.0], 0.94 + 0.1 * n(242, x, y))
    } else if vein > 0.63 {
        rgb([151.0, 155.0, 159.0], 0.95 + 0.08 * n(243, x, y))
    } else {
        host
    }
}

/// Soil hints retain the regular dirt grain; only a quiet local stain suggests the deposit below.
pub fn soil(x: i32, y: i32, hint: [f64; 3]) -> [u8; 4] {
    let dirt = nature::pixel(tex::DIRT, x, y);
    let strength = 0.13 + 0.1 * smooth(250, x, y, 4);
    let mix = |i: usize| dirt[i] as f64 * (1.0 - strength) + hint[i] * strength;
    rgb([mix(0), mix(1), mix(2)], 1.0)
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
