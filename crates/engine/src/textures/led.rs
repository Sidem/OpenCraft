//! The LED lamp's look (`block/led.rs`): a frosted white diffuser inside a dark frame with a lit rim. White on purpose, so
//! the halo and the tint box of a coloured lamp (`led/view.rs`) read on it. Placeholder until the art pass
//! (`docs/ART_HANDOVER.md`).

use super::{n, rgb};

pub fn panel(x: i32, y: i32) -> [u8; 4] {
    let edge = x == 0 || y == 0 || x == 15 || y == 15;
    let rim = x == 1 || y == 1 || x == 14 || y == 14;
    if edge {
        rgb([58.0, 62.0, 70.0], 1.0)
    } else if rim {
        rgb([236.0, 238.0, 240.0], 1.0)
    } else {
        rgb([214.0, 218.0, 222.0], 0.96 + 0.04 * n(910, x, y))
    }
}
