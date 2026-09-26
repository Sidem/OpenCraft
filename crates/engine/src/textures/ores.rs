//! Alpine ore faces: coal fractures, pale iron lenses and branching copper veins on the same slate.
//! Each mark wraps on the 16-texel torus so a greedy terrain quad can repeat without a border.

use super::{n, nature::stone, rgb};
use crate::block::tex;

fn wrapped_delta(a: i32, b: i32) -> i32 {
    (a - b + 8).rem_euclid(16) - 8
}

pub(super) fn pixel(layer: u16, x: i32, y: i32) -> [u8; 4] {
    let rock = stone(x, y);
    match layer {
        tex::COAL_ORE => {
            // Long, broken carbon seams with a graphite cleavage highlight.
            let line = (x + y * 2 + if y < 8 { 0 } else { 3 }).rem_euclid(16);
            let cross = (x * 2 - y + 5).rem_euclid(16);
            let cleft =
                (2..=5).contains(&line) && !(6..=7).contains(&y) || (9..=11).contains(&cross) && !(5..=10).contains(&x);
            if cleft {
                if line == 2 || cross == 9 {
                    rgb([73.0, 86.0, 91.0], 0.92 + 0.1 * n(211, x, y))
                } else {
                    rgb([27.0, 34.0, 39.0], 0.9 + 0.18 * n(212, x, y))
                }
            } else {
                rock
            }
        }
        tex::IRON_ORE => {
            // Heavy, rounded blooms with a rust-coloured socket and broad ivory glint.
            for (cx, cy, radius) in [(3, 4, 3), (12, 10, 3), (8, 15, 2)] {
                let dx = wrapped_delta(x, cx);
                let dy = wrapped_delta(y, cy);
                let r2 = dx * dx + dy * dy;
                if r2 <= (radius + 1) * (radius + 1) {
                    if r2 > radius * radius {
                        return rgb([91.0, 83.0, 78.0], 0.95);
                    }
                    return if dx + dy < 0 {
                        rgb([224.0, 211.0, 186.0], 0.91 + 0.09 * n(215, x, y))
                    } else {
                        rgb([157.0, 142.0, 128.0], 0.92 + 0.08 * n(216, x, y))
                    };
                }
            }
            rock
        }
        tex::COPPER_ORE => {
            // Thin angular forks, with dark oxidation beside a bright exposed edge.
            let vein = (x - 2 * y + if x < 8 { 0 } else { 3 }).rem_euclid(16);
            let branch = (x + y + 2).rem_euclid(16);
            if vein == 3 || vein == 4 || ((10..=12).contains(&branch) && y > 5) {
                if vein == 3 || branch == 10 {
                    rgb([215.0, 156.0, 104.0], 0.92 + 0.08 * n(219, x, y))
                } else {
                    rgb([159.0, 99.0, 69.0], 0.94 + 0.08 * n(220, x, y))
                }
            } else if vein == 2 || branch == 9 && y > 5 {
                rgb([65.0, 99.0, 103.0], 0.9 + 0.1 * n(221, x, y))
            } else {
                rock
            }
        }
        _ => rock,
    }
}
