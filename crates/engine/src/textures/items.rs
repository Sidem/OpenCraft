//! Small manufactured-part surfaces, science liquid and tool materials (a wooden handle, forged
//! steel), shaped by `item_models` geometry. Keep these restrained so the model silhouette remains
//! readable at hotbar size.

use super::{n, rgb, smooth};
use crate::block::tex;

pub(super) fn pixel(layer: u16, x: i32, y: i32) -> [u8; 4] {
    match layer {
        tex::IRON_ROD => {
            let groove = (y + x / 4).rem_euclid(4) == 0;
            rgb(if groove { [100.0, 111.0, 120.0] } else { [185.0, 196.0, 204.0] }, 0.94 + 0.08 * n(231, x, y))
        }
        tex::SCREW => {
            let thread = (x + y * 2).rem_euclid(5) < 2;
            rgb(if thread { [90.0, 101.0, 109.0] } else { [191.0, 201.0, 207.0] }, 0.94 + 0.08 * n(232, x, y))
        }
        tex::FLASK_GLASS => {
            let glint = x == 2 || x == 3 || (y == 2 && x < 9);
            rgb(if glint { [224.0, 241.0, 244.0] } else { [137.0, 177.0, 184.0] }, 0.96 + 0.06 * n(233, x, y))
        }
        tex::RED_PACK | tex::GREEN_PACK => {
            let red = layer == tex::RED_PACK;
            let meniscus = y < 3;
            let edge = x <= 1 || x >= 14;
            let c = if meniscus {
                [188.0, 215.0, 218.0]
            } else if edge {
                if red {
                    [125.0, 45.0, 50.0]
                } else {
                    [36.0, 118.0, 71.0]
                }
            } else if red {
                [193.0, 68.0, 65.0]
            } else {
                [62.0, 179.0, 105.0]
            };
            rgb(c, 0.94 + 0.1 * n(if red { 234 } else { 235 }, x, y))
        }
        tex::HANDLE => {
            // Planed ash: long grain lines that wander a little, lengthwise (along y).
            let grain = smooth(236, x, y, 2) * 3.0 + x as f64;
            let line = (grain as i32).rem_euclid(4) == 0;
            rgb(if line { [118.0, 84.0, 54.0] } else { [158.0, 118.0, 78.0] }, 0.95 + 0.08 * n(237, x, y))
        }
        tex::STEEL => {
            // Forged steel: blue-grey with hammer marks and a bright brushed streak.
            let mark = n(238, x, y) > 0.9;
            let streak = (y + (smooth(239, x, 0, 4) * 2.0) as i32).rem_euclid(8) == 2;
            let c = if streak {
                [196.0, 204.0, 212.0]
            } else if mark {
                [104.0, 112.0, 122.0]
            } else {
                [148.0, 156.0, 166.0]
            };
            rgb(c, 0.95 + 0.07 * n(240, x, y))
        }
        _ => [255, 0, 255, 255],
    }
}
