//! Steelmaking (Steelmaking tech): the blast furnace's housing (firebrick bound in steel bands, a top
//! with a charging hole), the byproduct hatch (a violet framed square with a chevron pointing down, so
//! it reads apart from the in and out hatches by shape as well as colour), slag (dark glassy rock with
//! bright flecks), the steel ingot, plate and beam and the blue-grey forged head steel tools wear.
//! Placeholder looks until the art pass (`docs/ART_HANDOVER.md`).

use crate::block::tex;

use super::{frame, n, rgb, smooth};

pub fn pixel(layer: u16, x: i32, y: i32) -> [u8; 4] {
    match layer {
        tex::BLAST_SIDE => blast_side(x, y),
        tex::BLAST_TOP => blast_top(x, y),
        tex::PORT_SIDE => port_side(x, y),
        tex::SLAG => slag(x, y),
        tex::STEEL_INGOT => bar(x, y),
        tex::STEEL_PLATE => plate(x, y),
        tex::STEEL_BEAM => beam(x, y),
        _ => head(x, y),
    }
}

/// Firebrick courses (offset every other row) between two steel bands.
fn blast_side(x: i32, y: i32) -> [u8; 4] {
    let k = 0.9 + 0.1 * n(360, x, y);
    if y == 6 || y == 7 || y == 14 || y == 15 {
        return rgb([116, 124, 138].map(f64::from), if y % 2 == 0 { 0.85 } else { 1.0 } * k);
    }
    let course = y / 3;
    let joint = (x + course * 4) % 8 == 0 || y % 3 == 2;
    if joint {
        rgb([46.0, 34.0, 32.0], 1.0)
    } else {
        rgb([132.0, 62.0, 44.0], k + 0.06 * smooth(361, x, y, 4))
    }
}

/// A steel deck with a dark charging hole in a glowing rim.
fn blast_top(x: i32, y: i32) -> [u8; 4] {
    let r = ((x as f64 - 7.5).powi(2) + (y as f64 - 7.5).powi(2)).sqrt();
    if r < 3.2 {
        rgb([24.0, 20.0, 20.0], 1.0)
    } else if r < 4.4 {
        rgb([236.0, 122.0, 40.0], 0.9 + 0.1 * n(362, x, y))
    } else {
        frame(x, y)
    }
}

/// A framed hatch: a slag-violet rim, a chevron pointing down.
fn port_side(x: i32, y: i32) -> [u8; 4] {
    let colour = [172.0, 128.0, 204.0];
    let rim = x <= 1 || x >= 14 || y <= 1 || y >= 14;
    let chevron = (11 - y == (x - 7).abs() || 10 - y == (x - 7).abs()) && (4..12).contains(&y);
    if rim {
        rgb(colour, 0.85 + 0.1 * n(363, x, y))
    } else if chevron {
        rgb(colour, 1.0)
    } else {
        rgb([34.0, 38.0, 44.0], 0.9 + 0.1 * n(364, x, y))
    }
}

/// Dark, glassy rock with pale flecks.
fn slag(x: i32, y: i32) -> [u8; 4] {
    let k = 0.86 + 0.14 * smooth(365, x, y, 4) + 0.06 * n(366, x, y);
    if n(367, x, y) > 0.94 {
        rgb([176.0, 168.0, 196.0], 1.0)
    } else {
        rgb([64.0, 58.0, 76.0], k)
    }
}

/// A steel bar: cool blue-grey, lit along its top edge.
fn bar(x: i32, y: i32) -> [u8; 4] {
    let k = 0.9 + 0.08 * n(368, x, y) + if y < 4 { 0.1 } else { 0.0 };
    rgb([148.0, 164.0, 188.0], k)
}

/// A plate with a rivet at each corner.
fn plate(x: i32, y: i32) -> [u8; 4] {
    let rivet = (x == 2 || x == 13) && (y == 2 || y == 13);
    let k = 0.92 + 0.08 * n(369, x, y);
    if rivet {
        rgb([88.0, 100.0, 120.0], 1.0)
    } else {
        rgb([170.0, 182.0, 200.0], k)
    }
}

/// An I-beam seen from the side: bright flanges and a darker web.
fn beam(x: i32, y: i32) -> [u8; 4] {
    let k = 0.92 + 0.08 * n(370, x, y);
    if !(3..13).contains(&y) {
        rgb([182.0, 194.0, 212.0], k)
    } else if y == 3 || y == 12 {
        rgb([98.0, 110.0, 130.0], 1.0)
    } else {
        rgb([130.0, 144.0, 168.0], k)
    }
}

/// Forged blue-grey steel for tool heads.
fn head(x: i32, y: i32) -> [u8; 4] {
    rgb([98.0, 116.0, 146.0], 0.88 + 0.12 * n(371, x, y) + if x < 3 { 0.1 } else { 0.0 })
}
