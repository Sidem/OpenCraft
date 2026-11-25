//! Clay and kilns' looks (`block/clay.rs`): grey-brown river clay, white kaolin with quartz grit, red clay bricks in
//! pale lime mortar, terracotta roof tiles, white porcelain tiles and the two glazed tiles. Placeholder looks until the
//! art pass (`docs/ART_HANDOVER.md`). Colours from photos of the real materials: alluvial clay is grey-brown, not red
//! (it fires red from its iron); kaolin is chalk white; copper glazes fire turquoise, iron glazes honey amber.

use super::decor::bricks;
use super::house::roof;
use super::paint::mottle;
use super::{n, rgb};
use crate::block::tex;

pub fn pixel(layer: u16, x: i32, y: i32) -> [u8; 4] {
    match layer {
        tex::CLAY => clay(x, y),
        tex::KAOLIN => kaolin(x, y),
        tex::CLAY_BRICKS => bricks(x, y, [168.0, 82.0, 56.0], [196.0, 188.0, 170.0], 940),
        tex::TERRACOTTA => roof(x, y, [196.0, 106.0, 62.0], 950),
        tex::PORCELAIN => glazed(x, y, [232.0, 234.0, 230.0], 960),
        tex::TURQUOISE_TILES => glazed(x, y, [58.0, 166.0, 164.0], 970),
        _ => glazed(x, y, [194.0, 124.0, 42.0], 980),
    }
}

/// Smooth, dense wet clay: soft smears and a few darker silt streaks, no grain to speak of.
fn clay(x: i32, y: i32) -> [u8; 4] {
    let smear = 0.9 + 0.14 * mottle(930, x, y, 0.35);
    let streak = if n(931, x / 5, y) > 0.82 { 0.86 } else { 1.0 };
    rgb([146.0, 128.0, 108.0], smear * streak + 0.03 * n(932, x, y))
}

/// Chalky china clay, ochre-stained in places, with glinting quartz grains left from the granite.
fn kaolin(x: i32, y: i32) -> [u8; 4] {
    if n(935, x, y) > 0.93 {
        return rgb([196.0, 198.0, 204.0], 1.0 + 0.08 * n(936, x, y));
    }
    let stain = mottle(937, x, y, 0.25);
    let base = if stain > 0.62 { [224.0, 206.0, 168.0] } else { [230.0, 226.0, 214.0] };
    rgb(base, 0.93 + 0.08 * mottle(938, x, y, 0.4) + 0.03 * n(939, x, y))
}

/// Square glazed tiles in pale grout, each a shade of its own, with a highlight along the top-left of the glaze.
fn glazed(x: i32, y: i32, colour: [f64; 3], seed: u32) -> [u8; 4] {
    if x % 8 == 7 || y % 8 == 7 {
        return rgb([206.0, 202.0, 192.0], 0.9 + 0.08 * n(seed, x, y));
    }
    let own = 0.94 + 0.1 * n(seed + 1, x / 8, y / 8);
    let sheen = if x % 8 == 0 || y % 8 == 0 { 0.08 } else { 0.0 };
    let pool = 0.05 * mottle(seed + 2, x, y, 0.3);
    rgb(colour, own + sheen - pool)
}
