//! Plain materials for the item models (`item_models`): brushed steel, gunmetal, rubber, brass, canvas webbing and
//! the kit's two paint colours. Even surfaces with a little grain, no frames or rivets, so they read on small parts.
//! The art pack paints the 128 px versions (`tools/art/materials.py`).

use super::paint::mottle;
use super::{n, rgb};
use crate::block::tex;

pub fn pixel(layer: u16, x: i32, y: i32) -> [u8; 4] {
    match layer {
        tex::MAT_METAL => brushed(x, y, [176.0, 182.0, 188.0], 900),
        tex::MAT_GUNMETAL => brushed(x, y, [74.0, 80.0, 90.0], 910),
        tex::MAT_RUBBER => rgb([46.0, 46.0, 50.0], 0.94 + 0.08 * mottle(920, x, y, 0.3)),
        tex::MAT_BRASS => brushed(x, y, [196.0, 160.0, 82.0], 930),
        tex::MAT_FABRIC => fabric(x, y),
        tex::MAT_EMBER => paint(x, y, [214.0, 104.0, 44.0], 950),
        _ => paint(x, y, [52.0, 160.0, 160.0], 960),
    }
}

/// Metal brushed along x: fine streaks that run the width of the tile.
fn brushed(x: i32, y: i32, colour: [f64; 3], seed: u32) -> [u8; 4] {
    rgb(colour, 0.92 + 0.08 * n(seed, 0, y) + 0.03 * n(seed + 1, x / 4, y))
}

/// Woven webbing: an over-under weave of olive-grey threads.
fn fabric(x: i32, y: i32) -> [u8; 4] {
    let over = (x + y) % 2 == 0;
    rgb([96.0, 98.0, 82.0], if over { 1.0 } else { 0.86 } + 0.06 * n(940, x, y))
}

/// Smooth painted metal with faint mottling.
fn paint(x: i32, y: i32, colour: [f64; 3], seed: u32) -> [u8; 4] {
    rgb(colour, 0.95 + 0.06 * mottle(seed, x, y, 0.25))
}
