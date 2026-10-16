//! Ground cover (`mesher/cover.rs`): low things on crossed transparent quads that grow on top of the ground,
//! never placed or saved. Calm, slightly muted colours (turf's family) so a meadow reads as a whole; y grows
//! downward, the ground is at y = 15. Transparent texels keep the plant's own RGB so mips show no fringe.

use super::{n, rgb, smooth};
use crate::block::tex;

pub fn pixel(layer: u16, x: i32, y: i32) -> [u8; 4] {
    match layer {
        tex::GRASS_TUFT => tuft(x, y),
        tex::DAISY => flower(x, y, [236.0, 232.0, 214.0], Some([222.0, 184.0, 70.0])),
        tex::POPPY => flower(x, y, [196.0, 64.0, 56.0], Some([52.0, 40.0, 40.0])),
        tex::BLUEBELL => bluebell(x, y),
        tex::FERN => fern(x, y),
        tex::REED => reed(x, y),
        tex::DRY_BRUSH => dry_brush(x, y),
        tex::LICHEN => lichen(x, y),
        _ => unreachable!("not a ground cover texture"),
    }
}

const GREEN: [f64; 3] = [74.0, 112.0, 70.0];

fn clear(c: [f64; 3]) -> [u8; 4] {
    let mut p = rgb(c, 0.85);
    p[3] = 0;
    p
}

/// A green blade colour, lighter towards the tip (`rise` 0 at the root, 1 at the top).
fn blade_green(seed: u32, x: i32, y: i32, rise: f64) -> [u8; 4] {
    rgb(GREEN, 0.78 + 0.34 * rise + 0.12 * n(seed, x, y))
}

/// Short blades of uneven height, a few leaning.
fn tuft(x: i32, y: i32) -> [u8; 4] {
    let lean = i32::from(n(301, x, 0) > 0.7) - i32::from(n(301, x, 0) < 0.2);
    let col = x + lean * i32::from(y < 10);
    let height = 4 + (n(300, col, 0) * 6.0) as i32;
    if (2..=13).contains(&col) && n(302, col, 0) > 0.3 && y >= 16 - height {
        return blade_green(303, x, y, f64::from(15 - y) / f64::from(height));
    }
    clear(GREEN)
}

/// A stem with two leaves and a flower head of 3 × 3 (`petal`, with a `heart` when given).
fn flower(x: i32, y: i32, petal: [f64; 3], heart: Option<[f64; 3]>) -> [u8; 4] {
    let (dx, dy) = ((x - 8).abs(), (y - 5).abs());
    if dx <= 1 && dy <= 1 {
        return match heart {
            Some(h) if dx == 0 && dy == 0 => rgb(h, 1.0),
            _ if dx + dy == 2 => rgb(petal, 0.86),
            _ => rgb(petal, 0.96 + 0.08 * n(310, x, y)),
        };
    }
    let leaf = (y == 11 && (x == 6 || x == 7 || x == 9 || x == 10)) || (y == 10 && (x == 5 || x == 11));
    if (x == 8 && y >= 7) || leaf {
        return blade_green(311, x, y, f64::from(15 - y) / 9.0);
    }
    clear(GREEN)
}

/// Three small bells hanging from an arched stem.
fn bluebell(x: i32, y: i32) -> [u8; 4] {
    let stem = x == 8 && y >= 8 || x == 7 && (5..8).contains(&y) || x == 6 && y == 4;
    let bells = [(5, 5), (8, 7), (5, 8)];
    let bell = bells.iter().any(|&(bx, by)| (x - bx).abs() <= 1 && y == by || x == bx && y == by + 1);
    if bell {
        return rgb([108.0, 128.0, 206.0], 0.84 + 0.2 * n(312, x, y));
    }
    if stem {
        return blade_green(313, x, y, f64::from(15 - y) / 11.0);
    }
    clear(GREEN)
}

/// An upright frond: a midrib with leaflets in alternate rows that widen to the middle and taper.
fn fern(x: i32, y: i32) -> [u8; 4] {
    let (dx, rows) = ((x - 8).abs(), 14 - y);
    let width = if rows < 4 { 0 } else { (rows / 2).min(5 - (rows - 9).max(0) / 2) };
    if y <= 14 && (dx == 0 && y >= 2 || dx <= width && dx > 0 && (x + y) % 2 == 0 && y >= 3) {
        return rgb([60.0, 104.0, 62.0], 0.78 + 0.3 * f64::from(15 - y) / 14.0 + 0.1 * n(314, x, y));
    }
    clear(GREEN)
}

/// Tall thin stalks, the tallest crowned with a brown seed head.
fn reed(x: i32, y: i32) -> [u8; 4] {
    let stalks = [(4, 5), (7, 1), (10, 3), (12, 7)];
    for (sx, top) in stalks {
        if x == sx && y >= top {
            let head = sx == 7 && y <= top + 3 || sx == 10 && y <= top + 2;
            return if head {
                rgb([112.0, 80.0, 52.0], 0.9 + 0.2 * n(315, x, y))
            } else {
                rgb([112.0, 132.0, 74.0], 0.78 + 0.3 * f64::from(15 - y) / 15.0)
            };
        }
        if x == sx + 1 && y == top + 3 && sx != 7 {
            return rgb([112.0, 80.0, 52.0], 0.9);
        }
    }
    clear([112.0, 132.0, 74.0])
}

/// Dry twigs fanning out from the ground: a few thin diagonal sticks, bare and tan.
fn dry_brush(x: i32, y: i32) -> [u8; 4] {
    let dx = (x - 8).abs();
    let twig = |slope: f64, reach: i32| dx <= reach && (f64::from(15 - y) - slope * f64::from(dx)).abs() < 0.6;
    let up = x == 8 && y >= 6;
    if (up || twig(2.0, 4) || twig(1.0, 6) || twig(3.5, 2)) && n(316, x, y) > 0.12 {
        return rgb([156.0, 128.0, 84.0], 0.78 + 0.34 * n(317, x, y));
    }
    clear([156.0, 128.0, 84.0])
}

/// Low flat patches of pale lichen, grey-green with warmer flecks.
fn lichen(x: i32, y: i32) -> [u8; 4] {
    let rise = 1 + (smooth(318, x, 0, 4) * 4.0) as i32 + (smooth(319, x, 0, 8) * 3.0) as i32;
    if y >= 16 - rise && n(320, x, y) > 0.22 {
        let warm = n(321, x, y) > 0.82;
        let base = if warm { [182.0, 150.0, 82.0] } else { [140.0, 156.0, 108.0] };
        return rgb(base, 0.8 + 0.3 * n(322, x, y));
    }
    clear([140.0, 156.0, 108.0])
}
