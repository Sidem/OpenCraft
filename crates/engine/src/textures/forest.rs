//! The other trees' looks (`block/forest.rs`): pale birch bark, birch and pine leaves, cactus, grey dead wood and
//! the two new saplings. Same pixel-art language as `nature.rs`; the log tops share the oak's rings.

use super::nature::{grain, leaves_in};
use super::paint::{mottle, tone, Ramp};
use super::{n, rgb, smooth};
use crate::block::tex;

const BIRCH_LEAF: Ramp = [[58, 92, 52], [76, 114, 60], [98, 138, 68], [124, 158, 78], [150, 178, 92]];
const PINE_LEAF: Ramp = [[24, 48, 42], [32, 62, 52], [42, 78, 62], [56, 96, 72], [72, 114, 84]];
const CACTUS: Ramp = [[52, 86, 58], [64, 102, 66], [78, 120, 74], [94, 138, 84], [112, 156, 96]];

pub fn pixel(layer: u16, x: i32, y: i32) -> [u8; 4] {
    match layer {
        tex::BIRCH_SIDE => birch_bark(x, y),
        tex::BIRCH_LEAVES => leaves_in(&BIRCH_LEAF, 330, x, y, 4),
        tex::PINE_LEAVES => leaves_in(&PINE_LEAF, 340, x, y, 5),
        tex::CACTUS_SIDE => cactus(x, y, true),
        tex::CACTUS_TOP => cactus(x, y, false),
        tex::DEAD_SIDE => dead_wood(x, y),
        tex::BIRCH_SAPLING => birch_sapling(x, y),
        tex::PINE_SAPLING => super::plants::pine_sapling(x, y),
        _ => unreachable!("not a forest texture"),
    }
}

/// Near-white bark with short dark scars across it.
fn birch_bark(x: i32, y: i32) -> [u8; 4] {
    let mut c = grain([206, 202, 190], 350, x, y);
    let scar = n(354, x / 2, y) > 0.86 && n(355, x, y / 3) > 0.35;
    if scar {
        c = [58, 54, 50, 255];
    } else if smooth(356, x, y, 4) < 0.25 {
        c = [c[0].saturating_sub(16), c[1].saturating_sub(16), c[2].saturating_sub(14), 255];
    }
    c
}

/// Ribs and rows of pale spines on green; the top is a cross-section with a ring of ribs.
fn cactus(x: i32, y: i32, side: bool) -> [u8; 4] {
    let t = 0.3 + mottle(360, x, y, 0.15) * 0.4;
    if side {
        let rib = x % 4 == 0;
        let spine = x % 4 == 2 && y % 4 == 1 && n(361, x, y) > 0.3;
        if spine {
            [224, 220, 190, 255]
        } else {
            tone(&CACTUS, if rib { t - 0.25 } else { t })
        }
    } else {
        let (dx, dy) = (x as f64 - 7.5, y as f64 - 7.5);
        let r = (dx * dx + dy * dy).sqrt();
        tone(
            &CACTUS,
            if r > 6.5 {
                0.05
            } else if r > 4.5 {
                t - 0.1
            } else {
                t + 0.2
            },
        )
    }
}

/// Weathered grey wood with deep cracks.
fn dead_wood(x: i32, y: i32) -> [u8; 4] {
    let mut c = grain([112, 106, 98], 370, x, y);
    let crack = smooth(374, x, y, 4) * 0.6 + smooth(375, x, 0, 8) * 0.4;
    if crack < 0.34 {
        c = [c[0].saturating_sub(34), c[1].saturating_sub(34), c[2].saturating_sub(32), 255];
    } else if crack > 0.74 {
        c = [c[0].saturating_add(10), c[1].saturating_add(10), c[2].saturating_add(9), 255];
    }
    c
}

/// A pale thin stem with a few round leaves on short twigs.
fn birch_sapling(x: i32, y: i32) -> [u8; 4] {
    let leaf = [96.0, 138.0, 70.0];
    let stem = x == 8 && y >= 5 || x == 7 && (3..5).contains(&y);
    if stem {
        return rgb([214.0, 208.0, 194.0], 0.88 + 0.16 * n(380, x, y));
    }
    let tufts = [(6, 3), (10, 6), (5, 8), (11, 10), (6, 12)];
    for (cx, cy) in tufts {
        let (dx, dy) = ((x - cx).abs(), (y - cy).abs());
        if dx + dy <= 1 || (dx <= 1 && dy <= 1 && n(381, x, y) > 0.4) {
            return rgb(leaf, 0.84 + 0.26 * n(382, x, y));
        }
    }
    let mut c = rgb(leaf, 0.84);
    c[3] = 0;
    c
}
