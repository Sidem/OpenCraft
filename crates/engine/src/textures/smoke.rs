//! Smoke puffs for chimneys, exhausts and the locomotive (`factory/smoke.rs`): a soft grey ball, cut out (texels
//! with alpha below one half are not drawn), full in `SMOKE` and smaller and paler in `SMOKE_THIN`, the look of a
//! puff thinning as it rises. Every face of a puff's box shows the whole ball, so it reads round from any side.
//! The cut-away texels keep the smoke's colour, so mipmaps don't ring the ball in black. Also `SOOT`, the plain
//! blackened iron of the chimneys the smoke comes out of.

use crate::block::tex;

use super::{n, rgb, smooth};

const GREY: [f64; 3] = [200.0, 202.0, 206.0];

pub fn pixel(layer: u16, x: i32, y: i32) -> [u8; 4] {
    if layer == tex::SOOT {
        return soot(x, y);
    }
    let (dx, dy) = (x as f64 - 7.5, y as f64 - 7.5);
    // A ragged edge: the radius wobbles with low-frequency noise.
    let r = (dx * dx + dy * dy).sqrt() / 7.5 + 0.25 * (smooth(611, x, y, 4) - 0.5);
    let thin = layer == tex::SMOKE_THIN;
    if r >= if thin { 0.75 } else { 0.95 } {
        let [r, g, b, _] = rgb(GREY, 0.9);
        return [r, g, b, 0];
    }
    let k = if thin { 0.95 } else { 0.82 };
    rgb(GREY, k + 0.16 * smooth(613, x, y, 4) - 0.1 * r)
}

/// Soot-blackened iron: near black, faintly mottled, no markings.
fn soot(x: i32, y: i32) -> [u8; 4] {
    rgb([44.0, 42.0, 42.0], 0.85 + 0.15 * n(614, x, y) + 0.12 * smooth(615, x, y, 4))
}
