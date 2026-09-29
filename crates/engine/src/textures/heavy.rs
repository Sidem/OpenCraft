//! Heavy industry (Steam Power, Ore Crushing, Bulk Storage): the boiler (riveted dark-blue tank with
//! a glowing firebox strip, a steel top with a fuel hatch), the steam turbine (steel casing with a vent
//! grille, a top with a spinning-blade disc), the crusher (a hazard-striped hopper wall, jaws on top),
//! the silo (concrete rings under a steel roof) and crushed iron and copper (jagged chunks).
//! Placeholder looks until the art pass (`docs/ART_HANDOVER.md`).

use crate::block::tex;

use super::{frame, n, rgb, smooth};

pub fn pixel(layer: u16, x: i32, y: i32) -> [u8; 4] {
    match layer {
        tex::BOILER_SIDE => boiler_side(x, y),
        tex::BOILER_TOP => boiler_top(x, y),
        tex::TURBINE_SIDE => turbine_side(x, y),
        tex::TURBINE_TOP => turbine_top(x, y),
        tex::CRUSHER_SIDE => crusher_side(x, y),
        tex::CRUSHER_TOP => crusher_top(x, y),
        tex::SILO_SIDE => silo_side(x, y),
        tex::SILO_TOP => silo_top(x, y),
        tex::CRUSHED_IRON => crushed(x, y, [150.0, 152.0, 164.0]),
        _ => crushed(x, y, [196.0, 112.0, 72.0]),
    }
}

/// A dark-blue tank plated in riveted bands, a glowing firebox slit low down.
fn boiler_side(x: i32, y: i32) -> [u8; 4] {
    let k = 0.9 + 0.1 * n(380, x, y);
    if (11..=13).contains(&y) && (3..13).contains(&x) {
        return rgb([246.0, 140.0, 44.0], if y == 12 { 1.0 } else { 0.7 });
    }
    if y % 8 == 0 || y % 8 == 7 {
        let rivet = x % 4 == 2 && y % 8 == 0;
        return rgb([148.0, 156.0, 170.0], if rivet { 1.15 } else { 0.85 * k });
    }
    rgb([52.0, 78.0, 118.0], k + 0.08 * smooth(381, x, y, 4))
}

/// A steel deck with a round fuel hatch.
fn boiler_top(x: i32, y: i32) -> [u8; 4] {
    let r = ((x as f64 - 7.5).powi(2) + (y as f64 - 7.5).powi(2)).sqrt();
    if r < 2.6 {
        rgb([30.0, 28.0, 30.0], 1.0)
    } else if r < 3.8 {
        rgb([182.0, 188.0, 198.0], 0.9 + 0.1 * n(382, x, y))
    } else {
        rgb([64.0, 92.0, 136.0], 0.9 + 0.1 * n(383, x, y))
    }
}

/// Grey steel casing with a slatted vent.
fn turbine_side(x: i32, y: i32) -> [u8; 4] {
    let k = 0.9 + 0.1 * n(384, x, y);
    if (4..12).contains(&y) && (3..13).contains(&x) {
        return if y % 2 == 0 { rgb([28.0, 32.0, 38.0], 1.0) } else { rgb([170.0, 178.0, 190.0], k) };
    }
    if x == 0 || x == 15 {
        return rgb([90.0, 98.0, 112.0], k);
    }
    rgb([196.0, 202.0, 212.0], k + 0.06 * smooth(385, x, y, 4))
}

/// A steel deck with a blade disc: spokes around a hub.
fn turbine_top(x: i32, y: i32) -> [u8; 4] {
    let (dx, dy) = (x as f64 - 7.5, y as f64 - 7.5);
    let r = (dx * dx + dy * dy).sqrt();
    if r < 1.8 {
        rgb([236.0, 190.0, 64.0], 1.0)
    } else if r < 6.4 {
        let spoke = dx.abs() < 0.9 || dy.abs() < 0.9 || (dx - dy).abs() < 1.1 || (dx + dy).abs() < 1.1;
        if spoke {
            rgb([210.0, 216.0, 226.0], 0.95)
        } else {
            rgb([40.0, 46.0, 56.0], 1.0)
        }
    } else {
        frame(x, y)
    }
}

/// Yellow-and-black hazard chevrons over a heavy grey plate.
fn crusher_side(x: i32, y: i32) -> [u8; 4] {
    let k = 0.9 + 0.1 * n(386, x, y);
    if (5..=10).contains(&y) {
        let band = (x + y).rem_euclid(6) < 3;
        if band {
            rgb([232.0, 184.0, 40.0], k)
        } else {
            rgb([34.0, 32.0, 30.0], 1.0)
        }
    } else {
        rgb([104.0, 110.0, 120.0], k + 0.05 * smooth(387, x, y, 4))
    }
}

/// A dark hopper mouth with two toothed jaws.
fn crusher_top(x: i32, y: i32) -> [u8; 4] {
    let in_mouth = (3..13).contains(&x) && (3..13).contains(&y);
    if !in_mouth {
        return frame(x, y);
    }
    let tooth = (x + (y / 4) * 2) % 4 < 2 && y % 4 < 2;
    if tooth {
        rgb([188.0, 194.0, 204.0], 1.0)
    } else {
        rgb([22.0, 22.0, 26.0], 1.0)
    }
}

/// Poured concrete in rings, two dark seams and a row of bolts.
fn silo_side(x: i32, y: i32) -> [u8; 4] {
    let k = 0.92 + 0.08 * n(388, x, y) + 0.05 * smooth(389, x, y, 4);
    if y == 7 || y == 8 {
        return rgb([84.0, 88.0, 94.0], 1.0);
    }
    if y == 3 && x % 4 == 1 {
        return rgb([186.0, 190.0, 198.0], 1.0);
    }
    rgb([168.0, 168.0, 162.0], k)
}

/// A steel roof with a central hatch.
fn silo_top(x: i32, y: i32) -> [u8; 4] {
    let hatch = (5..11).contains(&x) && (5..11).contains(&y);
    if hatch {
        rgb([64.0, 70.0, 82.0], 0.9 + 0.1 * n(390, x, y))
    } else {
        frame(x, y)
    }
}

/// Jagged ore chunks on a transparent-looking dark ground (items draw as a small box).
fn crushed(x: i32, y: i32, c: [f64; 3]) -> [u8; 4] {
    let bit = smooth(391, x, y, 8);
    let k = 0.8 + 0.5 * bit + 0.1 * n(392, x, y);
    if n(393, x, y) > 0.86 {
        rgb([c[0] * 0.5, c[1] * 0.5, c[2] * 0.5], 1.0)
    } else {
        rgb(c, k)
    }
}
