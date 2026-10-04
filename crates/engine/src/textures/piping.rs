//! Pipe colours (`factory/pipes.rs`): water pipes are steel with a blue band, steam pipes pale lagging with a red
//! band, so a network's contents show at a glance. Placeholder looks until the art pass (`docs/ART_HANDOVER.md`).

use crate::block::tex;

use super::{n, rgb};

pub fn pixel(layer: u16, x: i32, y: i32) -> [u8; 4] {
    let band = (5..11).contains(&y);
    if layer == tex::PIPE_WATER {
        if band {
            return rgb([52.0, 120.0, 200.0], 0.9 + 0.1 * n(410, x, y));
        }
        return rgb([148.0, 156.0, 170.0], 0.85 + 0.15 * n(411, x, y));
    }
    if band {
        return rgb([196.0, 62.0, 48.0], 0.9 + 0.1 * n(412, x, y));
    }
    rgb([226.0, 222.0, 210.0], 0.9 + 0.1 * n(413, x, y))
}
