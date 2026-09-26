//! Alpine natural materials: subdued mineral grain, irregular strata and fine foliage at 16×16.
//! All fields wrap at the texture edge; the terrain shader adds independent world-scale detail.

use super::{n, smooth};
use crate::block::tex;

const SLATE: [u8; 3] = [112, 128, 137];
const TURF: [u8; 3] = [75, 113, 91];
const EARTH: [u8; 3] = [108, 92, 78];
const SAND: [u8; 3] = [199, 191, 168];
const BARK: [u8; 3] = [93, 83, 69];
const LEAF: [u8; 3] = [65, 101, 79];

pub(super) fn pixel(layer: u16, x: i32, y: i32) -> [u8; 4] {
    let (x, y) = (x.rem_euclid(16), y.rem_euclid(16));
    match layer {
        tex::STONE => stone(x, y),
        tex::DIRT => dirt(x, y),
        tex::GRASS_TOP => grass(x, y),
        tex::GRASS_SIDE => {
            let depth = 3 + (smooth(108, x, 0, 4) * 3.0) as i32;
            if y < depth {
                grass(x, y)
            } else if y <= depth + 1 {
                grain([82, 104, 78], 109, x, y)
            } else {
                dirt(x, y)
            }
        }
        tex::SAND => sand(x, y),
        tex::LOG_SIDE => bark(x, y),
        tex::LOG_TOP => rings(x, y),
        tex::LEAVES => leaves(x, y),
        tex::BEDROCK => {
            let c = grain([55, 67, 76], 140, x, y);
            if smooth(143, x, y, 4) < 0.25 {
                shade(c, -8)
            } else {
                c
            }
        }
        tex::SPENT_ROCK => {
            let base = if smooth(144, x, y, 2) < 0.38 { [152, 147, 139] } else { [157, 161, 158] };
            grain(base, 145, x, y)
        }
        _ => unreachable!("non-natural texture"),
    }
}

fn colour(c: [u8; 3]) -> [u8; 4] {
    [c[0], c[1], c[2], 255]
}

fn shade(c: [u8; 4], delta: i8) -> [u8; 4] {
    [c[0].saturating_add_signed(delta), c[1].saturating_add_signed(delta), c[2].saturating_add_signed(delta), c[3]]
}

/// Shared tileable grain: broad material variation, smaller flecks and a very quiet texel grit.
fn grain(base: [u8; 3], seed: u32, x: i32, y: i32) -> [u8; 4] {
    let broad = smooth(seed, x, y, 2) * 0.55 + smooth(seed + 1, x, y, 4) * 0.3 + smooth(seed + 2, x, y, 8) * 0.15;
    let grit = n(seed + 3, x, y);
    let delta = ((broad - 0.5) * 33.0 + (grit - 0.5) * 10.0) as i8;
    shade(colour(base), delta)
}

pub(super) fn stone(x: i32, y: i32) -> [u8; 4] {
    let c = grain(SLATE, 101, x, y);
    let fracture = smooth(105, x, y, 4);
    if (0.32..0.37).contains(&fracture) && n(106, x, y) > 0.18 {
        shade(c, -15)
    } else if n(107, x, y) > 0.97 {
        shade(c, 12)
    } else {
        c
    }
}

fn dirt(x: i32, y: i32) -> [u8; 4] {
    let c = grain(EARTH, 120, x, y);
    if n(124, x, y) > 0.97 {
        shade(c, 10)
    } else {
        c
    }
}

fn grass(x: i32, y: i32) -> [u8; 4] {
    let c = grain(TURF, 125, x, y);
    let blade = smooth(129, x, y, 8) > 0.76 && n(130, x, y) > 0.65;
    if blade {
        shade(c, 12)
    } else {
        c
    }
}

fn sand(x: i32, y: i32) -> [u8; 4] {
    let c = grain(SAND, 131, x, y);
    let ripple = smooth(135, x, y, 4);
    if ripple > 0.72 {
        shade(c, -6)
    } else {
        c
    }
}

fn bark(x: i32, y: i32) -> [u8; 4] {
    let c = grain(BARK, 150, x, y);
    // Long anisotropic fissures bend with the grain, rather than repeating at fixed columns.
    let groove = smooth(154, x, y, 4) * 0.65 + smooth(155, x, 0, 8) * 0.35;
    if groove < 0.36 {
        shade(c, -16)
    } else if groove > 0.72 {
        shade(c, 8)
    } else {
        c
    }
}

fn rings(x: i32, y: i32) -> [u8; 4] {
    let warp = smooth(158, x, y, 2) - 0.5;
    let (dx, dy) = (x as f64 - 7.5 + warp, y as f64 - 7.5 - warp);
    let radius = (dx * dx + dy * dy).sqrt();
    if radius > 7.0 {
        return bark(x, y);
    }
    let c = grain([166, 148, 116], 159, x, y);
    if ((radius + warp * 0.7) * 1.6) as i32 % 2 == 0 {
        shade(c, 9)
    } else {
        shade(c, -10)
    }
}

fn leaves(x: i32, y: i32) -> [u8; 4] {
    let mut c = grain(LEAF, 165, x, y);
    let gaps = smooth(169, x, y, 8) * 0.7 + smooth(170, x, y, 4) * 0.3;
    if gaps < 0.55 && n(171, x, y) > 0.75 {
        c[3] = 0;
    } else if gaps < 0.61 {
        c = shade(c, -10);
    }
    c
}
