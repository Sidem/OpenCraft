//! Tier stripes: a band of the tier colour (`factory::upgrades::TIER_COLOURS`) across brushed metal,
//! with one bright pip per Mk so colour is never the only cue. Belts wear them on their rails, miners as
//! a band, kits on every face.

use super::{n, rgb};
use crate::factory::upgrades::TIER_COLOURS;

pub(super) fn stripe(x: i32, y: i32, tier: u8) -> [u8; 4] {
    let c = TIER_COLOURS[tier as usize];
    let base = [(c >> 16) as f64, (c >> 8 & 0xff) as f64, (c & 0xff) as f64];
    let pips = tier as i32 + 1;
    // Pips 2 px square, 3 px apart, centred along x on the middle rows.
    let first = 8 - (pips * 3 - 1) / 2;
    let pip = (7..=8).contains(&y) && x >= first && x < first + pips * 3 - 1 && (x - first) % 3 < 2;
    if pip {
        rgb([245.0, 245.0, 240.0], 1.0)
    } else if !(4..=11).contains(&y) {
        rgb([122.0, 128.0, 136.0], 0.92 + 0.1 * n(249, x, y))
    } else if y == 4 || y == 11 {
        rgb(base, 0.6)
    } else {
        rgb(base, 0.93 + 0.08 * n(250 + u32::from(tier), x, y))
    }
}
