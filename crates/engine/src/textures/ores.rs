//! Mineral inclusions in Alpine slate. Coal is dark and fractured, iron broad and pale, copper fine
//! and mid-value; tileable noise makes their boundaries irregular without obvious drawn motifs.

use super::{n, nature::stone, rgb, smooth};
use crate::block::tex;

pub(super) fn pixel(layer: u16, x: i32, y: i32) -> [u8; 4] {
    let rock = stone(x, y);
    match layer {
        tex::COAL_ORE => {
            let seam = smooth(211, x, y, 4) * 0.75 + smooth(212, x, y, 8) * 0.25;
            if seam < 0.33 {
                let graphite = if seam > 0.30 { [73.0, 82.0, 85.0] } else { [36.0, 44.0, 48.0] };
                rgb(graphite, 0.91 + 0.13 * n(213, x, y))
            } else {
                rock
            }
        }
        tex::IRON_ORE => {
            let bloom = smooth(214, x, y, 4) * 0.8 + smooth(215, x, y, 8) * 0.2;
            if bloom > 0.74 {
                rgb([215.0, 201.0, 178.0], 0.91 + 0.09 * n(216, x, y))
            } else if bloom > 0.68 {
                rgb([119.0, 109.0, 100.0], 0.96 + 0.07 * n(217, x, y))
            } else {
                rock
            }
        }
        tex::COPPER_ORE => {
            let field = smooth(218, x, y, 4) * 0.72 + smooth(219, x, y, 8) * 0.28;
            let ridge = (field - 0.53).abs();
            if ridge < 0.028 {
                rgb([187.0, 127.0, 85.0], 0.92 + 0.11 * n(220, x, y))
            } else if ridge < 0.055 {
                rgb([76.0, 106.0, 107.0], 0.92 + 0.08 * n(221, x, y))
            } else {
                rock
            }
        }
        _ => rock,
    }
}
