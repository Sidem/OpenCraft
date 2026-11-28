//! Subdued province rocks (granite, sandstone, basalt), cutout glass, the lamp (glass in an iron
//! frame) and water (a placeholder: see-through blue with soft ripples), in the shared Alpine grain.
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

/// A warm glowing pane behind an iron frame and a cross of glazing bars, brightest in the middle.
pub fn lamp(x: i32, y: i32) -> [u8; 4] {
    let frame = x <= 1 || x >= 14 || y <= 1 || y >= 14;
    if frame || x == 7 || x == 8 || y == 7 || y == 8 {
        let rivet = frame && (x == 1 || x == 14) && (y == 1 || y == 14);
        return rgb(if rivet { [120.0, 124.0, 130.0] } else { [74.0, 76.0, 82.0] }, 0.9 + 0.1 * n(271, x, y));
    }
    let (dx, dy) = (f64::from(x) - 7.5, f64::from(y) - 7.5);
    let glow = 1.0 - (dx * dx + dy * dy).sqrt() / 12.0;
    rgb([255.0, 214.0, 140.0], 0.82 + 0.18 * glow + 0.04 * n(272, x, y))
}

/// Placeholder water: deep blue-green, lighter ripple bands, mostly see-through.
pub fn water(x: i32, y: i32) -> [u8; 4] {
    let ripple = smooth(265, x, y, 4);
    let mut c = rgb([46.0, 104.0, 150.0], 0.92 + 0.16 * ripple);
    c[3] = 180 + (24.0 * ripple) as u8;
    c
}

/// Translucent frozen ice: pale glacial blue with soft frost mottling, subtle diagonal fracture gleams and a crisp rim.
pub fn ice(x: i32, y: i32) -> [u8; 4] {
    let frost = smooth(266, x, y, 4);
    let edge = x == 0 || x == 15 || y == 0 || y == 15;
    let streak = (x + y * 2).rem_euclid(16) == 5 && n(267, x, y) > 0.35;
    let glint = (x - y).rem_euclid(16) == 3 && n(268, x, y) > 0.65;
    let base = if glint || streak {
        [224.0, 242.0, 252.0]
    } else if edge {
        [182.0, 218.0, 238.0]
    } else {
        [152.0 + 48.0 * frost, 198.0 + 34.0 * frost, 224.0 + 24.0 * frost]
    };
    let mut c = rgb(base, 0.94 + 0.08 * n(269, x, y));
    c[3] = if glint || streak {
        232
    } else if edge {
        216
    } else {
        188 + (28.0 * frost) as u8
    };
    c
}

/// Wet dark mossy aquifer stone with bright turquoise-blue water seepage fissures (`block/spring.rs`).
pub fn spring(x: i32, y: i32) -> [u8; 4] {
    let mut c = nature::grain([76, 86, 92], 270, x, y);
    let moss = smooth(273, x, y, 4);
    let seep = smooth(274, x, y, 3);
    let crack = (x + (y / 2)).rem_euclid(8) == 3 && seep > 0.44 || ((5..=10).contains(&y) && seep > 0.66);
    if crack {
        let foam = n(275, x, y);
        c = rgb([58.0 + 62.0 * foam, 148.0 + 58.0 * foam, 198.0 + 44.0 * foam], 0.94 + 0.08 * seep);
    } else if moss > 0.62 {
        c = rgb([72.0, 106.0, 74.0], 0.88 + 0.16 * n(276, x, y));
    } else if seep > 0.56 {
        c = tint(c, [-12, 4, 14]);
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
