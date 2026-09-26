//! Alpine natural materials: small palettes and periodic mineral/plant shapes at 16×16.
//! No frame borders on terrain. Leaf holes keep green RGB for mipmaps. Add natural layers in
//! `pixel`, keeping ore inclusions and manufactured materials in their existing modules.

use super::smooth;
use crate::block::tex;

const SLATE: [[u8; 3]; 4] = [[96, 113, 124], [111, 129, 139], [127, 143, 151], [146, 159, 163]];
const TURF: [[u8; 3]; 4] = [[67, 105, 85], [74, 113, 92], [81, 122, 99], [103, 139, 111]];
const EARTH: [[u8; 3]; 4] = [[88, 76, 67], [101, 88, 76], [111, 97, 83], [123, 110, 95]];
const FOLIAGE: [[u8; 3]; 4] = [[48, 78, 67], [56, 91, 76], [67, 105, 85], [84, 121, 98]];
const BARK: [[u8; 3]; 4] = [[61, 62, 58], [78, 76, 66], [95, 89, 75], [115, 106, 88]];

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
            } else if y == depth {
                colour(TURF[0])
            } else {
                dirt(x, y)
            }
        }
        tex::SAND => {
            let field = smooth(110, x, y, 2) * 0.75 + smooth(111, x, y, 4) * 0.25;
            let c = ramp([[180, 176, 152], [192, 187, 161], [203, 198, 173], [214, 208, 182]], field);
            if (y + (smooth(124, x, 0, 2) * 3.0) as i32).rem_euclid(5) == 0 {
                shade(c, -5)
            } else {
                c
            }
        }
        tex::LOG_SIDE => bark(x, y),
        tex::LOG_TOP => rings(x, y),
        tex::LEAVES => leaves(x, y),
        tex::BEDROCK => ramp([[41, 51, 62], [52, 63, 73], [65, 75, 84], [81, 91, 99]], facets(x, y)),
        tex::SPENT_ROCK => {
            let pit = smooth(142, x, y, 4);
            if pit > 0.77 {
                colour([91, 101, 106])
            } else if pit < 0.23 {
                colour([148, 126, 108])
            } else {
                ramp([[137, 146, 148], [151, 158, 157], [164, 169, 165], [179, 181, 174]], facets(x, y))
            }
        }
        _ => unreachable!("non-natural texture"),
    }
}

pub(super) fn stone(x: i32, y: i32) -> [u8; 4] {
    let c = shade(ramp(SLATE, facets(x, y)), (smooth(125, x, y, 8) * 10.0 - 5.0) as i8);
    // Short cleavage scratches sit within plates instead of making a noisy outline.
    if (x - y * 2).rem_euclid(7) == 0 && smooth(126, x, y, 4) > 0.53 {
        shade(c, 7)
    } else {
        c
    }
}

fn colour(c: [u8; 3]) -> [u8; 4] {
    [c[0], c[1], c[2], 255]
}

fn shade(c: [u8; 4], delta: i8) -> [u8; 4] {
    [c[0].saturating_add_signed(delta), c[1].saturating_add_signed(delta), c[2].saturating_add_signed(delta), c[3]]
}

fn ramp(palette: [[u8; 3]; 4], value: f64) -> [u8; 4] {
    colour(palette[((value * 4.0) as usize).min(3)])
}

// Nearest angular plates on a torus: each shard continues through the tile edge.
fn facets(x: i32, y: i32) -> f64 {
    let mut nearest = 100;
    let mut tone = 0.4;
    for (i, (cx, cy)) in [(2, 3), (11, 5), (6, 12), (14, 14)].into_iter().enumerate() {
        let dx = (x - cx + 8).rem_euclid(16) - 8;
        let dy = (y - cy + 8).rem_euclid(16) - 8;
        let distance = dx.abs() * 2 + dy.abs() * 3;
        if distance < nearest {
            nearest = distance;
            tone = if i == 0 || i == 3 { 0.4 } else { 0.65 };
            if i == 1 && dx < -1 && dy < 1 {
                tone = 0.85;
            }
        }
    }
    tone
}

fn dirt(x: i32, y: i32) -> [u8; 4] {
    let c = shade(
        ramp(EARTH, 0.7 * smooth(103, x, y, 4) + 0.3 * smooth(104, x, y, 8)),
        (smooth(127, x, y, 8) * 10.0 - 5.0) as i8,
    );
    if (x + y * 3).rem_euclid(9) < 2 && smooth(128, x, y, 4) > 0.58 {
        shade(c, 9)
    } else {
        c
    }
}

fn grass(x: i32, y: i32) -> [u8; 4] {
    // Three asymmetric tufts, wrapped at the edge; most of the tile stays quiet at a distance.
    for (cx, cy) in [(2, 3), (10, 7), (6, 14)] {
        let (dx, dy) = ((x - cx).rem_euclid(16), (y - cy).rem_euclid(16));
        if (dx == 0 && dy <= 1) || (dx == 1 && dy == 2) {
            return colour(TURF[3]);
        }
    }
    let field = smooth(106, x, y, 2);
    shade(
        colour(
            TURF[if field < 0.34 {
                0
            } else if field < 0.66 {
                1
            } else {
                2
            }],
        ),
        (smooth(129, x, y, 8) * 8.0 - 4.0) as i8,
    )
}

fn bark(x: i32, y: i32) -> [u8; 4] {
    let bend = (smooth(112, 0, y, 2) * 3.0) as i32;
    let groove = (x + bend).rem_euclid(8);
    let tone = if groove == 0 {
        0
    } else if groove == 1 {
        3
    } else if smooth(113, x, y, 4) < 0.5 {
        1
    } else {
        2
    };
    colour(BARK[tone])
}

fn rings(x: i32, y: i32) -> [u8; 4] {
    let (dx, dy) = (x as f64 - 7.5, y as f64 - 7.5);
    let radius = (dx * dx + dy * dy).sqrt();
    if radius > 7.0 {
        return colour(BARK[1]);
    }
    let ring = ((radius + smooth(116, x, y, 2) * 0.6) * 1.25) as usize;
    colour([[180, 165, 127], [159, 143, 109], [197, 179, 137]][ring % 3])
}

fn leaves(x: i32, y: i32) -> [u8; 4] {
    // Correlated holes leave leafy clusters instead of a regular mesh or random pinpricks.
    let field = smooth(118, x, y, 4);
    let mut c = ramp(FOLIAGE, 0.25 + field * 0.64);
    let gaps = smooth(119, x, y, 4);
    if gaps < 0.22 {
        c[3] = 0;
    } else if gaps < 0.30 {
        c = shade(c, -10);
    }
    c
}
