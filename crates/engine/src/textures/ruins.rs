//! The ruins' looks (`block/ruins.rs`): rusted plating (orange-brown panels with bolts and flaking), a machine husk's
//! side (a hatch and a dead gauge on rust), a girder (cross-braced), the scrap metal item (a heap of bent plate) and the
//! logbook (green cloth cover, pale pages) and a vault's door (iron with three brass rings).
//! Placeholder looks until the art pass (`docs/ART_HANDOVER.md`).

use super::paint::{mottle, tone, Ramp};
use super::{n, rgb};
use crate::block::tex;

const RUST: Ramp = [[58, 34, 24], [88, 48, 30], [122, 64, 34], [150, 82, 42], [178, 104, 56]];
const IRON: Ramp = [[40, 42, 46], [62, 64, 68], [84, 86, 90], [108, 110, 114], [132, 134, 138]];

pub fn pixel(layer: u16, x: i32, y: i32) -> [u8; 4] {
    match layer {
        tex::RUST_PLATE => plate(x, y),
        tex::RUST_MACHINE => machine(x, y),
        tex::RUST_GIRDER => girder(x, y),
        tex::LOGBOOK_SIDE => logbook(x, y, true),
        tex::LOGBOOK_TOP => logbook(x, y, false),
        tex::VAULT_DOOR => vault(x, y),
        _ => scrap(x, y),
    }
}

/// A heavy iron door: a riveted frame around three brass rings, one inside the other, on dark steel.
fn vault(x: i32, y: i32) -> [u8; 4] {
    if x == 0 || x == 15 || y == 0 || y == 15 {
        return rgb([30.0, 32.0, 36.0], 0.9 + 0.2 * n(713, x, y));
    }
    if matches!(x, 2 | 13) && matches!(y, 2 | 13) {
        return rgb([150.0, 150.0, 156.0], 0.9);
    }
    let (dx, dy) = (x as f64 - 7.5, y as f64 - 7.5);
    let r = (dx * dx + dy * dy).sqrt();
    for ring in [5.6, 3.8, 2.0] {
        if (r - ring).abs() < 0.6 {
            return rgb([186.0, 140.0, 58.0], 0.8 + 0.3 * n(714, x, y));
        }
    }
    tone(&IRON, mottle(715, x, y, 0.3) * 0.7)
}

/// A weathered green cloth cover with a pale page block and a brass clasp (the side), or the pages seen from above.
fn logbook(x: i32, y: i32, side: bool) -> [u8; 4] {
    if !side {
        let ruled = y % 3 == 1;
        return rgb([206.0, 196.0, 160.0], if ruled { 0.82 } else { 0.95 } + 0.08 * n(710, x, y));
    }
    if !(2..=13).contains(&y) {
        return rgb([206.0, 196.0, 160.0], 0.85 + 0.1 * n(711, x, y));
    }
    if (7..=8).contains(&x) && (5..=10).contains(&y) {
        return rgb([196.0, 150.0, 60.0], 0.9);
    }
    rgb([54.0, 84.0, 62.0], 0.8 + 0.3 * mottle(712, x, y, 0.4))
}

/// Rust over grey metal: bare iron shows through flaked patches.
fn flaked(seed: u32, x: i32, y: i32) -> [u8; 4] {
    let t = mottle(seed, x, y, 0.3);
    if mottle(seed + 7, x, y, 0.2) < 0.2 {
        tone(&IRON, t)
    } else {
        tone(&RUST, t)
    }
}

/// Four panels split by dark seams, a bolt in every corner.
fn plate(x: i32, y: i32) -> [u8; 4] {
    if x % 8 == 0 || y % 8 == 0 {
        return rgb([36.0, 22.0, 16.0], 0.9 + 0.2 * n(701, x, y));
    }
    if matches!(x % 8, 1 | 6) && matches!(y % 8, 1 | 6) {
        return rgb([168.0, 110.0, 70.0], 0.9);
    }
    flaked(702, x, y)
}

/// A machine side: a dark hatch with a rusted handle, and a dead gauge above it.
fn machine(x: i32, y: i32) -> [u8; 4] {
    if x == 0 || x == 15 || y == 0 || y == 15 {
        return rgb([40.0, 26.0, 20.0], 0.9 + 0.2 * n(703, x, y));
    }
    let hatch = (3..=12).contains(&x) && (7..=13).contains(&y);
    if hatch {
        let edge = x == 3 || x == 12 || y == 7 || y == 13;
        return if edge { rgb([54.0, 36.0, 28.0], 1.0) } else { rgb([30.0, 22.0, 20.0], 0.9 + 0.2 * n(704, x, y)) };
    }
    let (dx, dy) = (x as f64 - 7.5, y as f64 - 3.5);
    let r = (dx * dx + dy * dy).sqrt();
    if r < 2.6 {
        return if r < 1.6 { rgb([58.0, 62.0, 58.0], 1.0) } else { rgb([150.0, 142.0, 120.0], 0.8) };
    }
    flaked(705, x, y)
}

/// Iron I-beam seen end-on, rusted: flanges top and bottom, a web between, diagonal braces drawn as notches.
fn girder(x: i32, y: i32) -> [u8; 4] {
    let flange = !(3..=12).contains(&y);
    let web = (6..=9).contains(&x);
    let brace = (x + y) % 8 == 0 || (x - y).rem_euclid(8) == 0;
    if flange || web || brace {
        flaked(706, x, y)
    } else {
        rgb([22.0, 16.0, 14.0], 0.8 + 0.4 * n(707, x, y))
    }
}

/// A heap of bent plates on a dark ground (items are opaque tiles), rust on the top ones.
fn scrap(x: i32, y: i32) -> [u8; 4] {
    let heap = y >= 12 - (7.5 - (x as f64 - 7.5).abs()).max(0.0) as i32 && y <= 14 && (1..=14).contains(&x);
    if !heap {
        return rgb([30.0, 28.0, 28.0], 0.9 + 0.2 * n(709, x, y));
    }
    let layer = (x + y * 2) / 3 % 3;
    let t = mottle(708 + layer as u32, x, y, 0.4);
    if layer == 1 {
        tone(&IRON, t)
    } else {
        tone(&RUST, t)
    }
}
