//! Placeholder looks for generator 12's species (`block/trees.rs`): bark for each log and a leaf tile for each leaf
//! block, in the 16 × 16 pixel language of `nature.rs` (the art pack replaces them in the browser: `tools/art`). The
//! saplings are painted at full size in `hires.rs`. Log tops share the oak's rings.

use super::nature::{grain, leaves_in};
use super::paint::Ramp;
use super::{n, smooth};
use crate::block::tex;

const PALM: Ramp = [[46, 86, 44], [60, 106, 50], [78, 126, 58], [100, 146, 66], [124, 166, 80]];
const CHERRY: Ramp = [[196, 112, 146], [214, 134, 166], [230, 160, 186], [242, 186, 204], [250, 214, 226]];
const BAOBAB: Ramp = [[64, 96, 50], [80, 114, 58], [98, 132, 66], [118, 150, 76], [140, 168, 90]];
const ACACIA: Ramp = [[62, 90, 46], [78, 106, 52], [96, 124, 60], [116, 142, 70], [138, 160, 84]];
const REDWOOD: Ramp = [[26, 56, 38], [34, 70, 46], [44, 86, 54], [58, 102, 64], [74, 118, 76]];
const EUCALYPTUS: Ramp = [[70, 98, 88], [86, 116, 102], [104, 134, 116], [124, 152, 132], [146, 170, 150]];
const BAMBOO: Ramp = [[66, 110, 50], [82, 128, 56], [100, 146, 64], [122, 164, 74], [146, 182, 92]];

/// Texel (`x`, `y`) of one of the species' bark or leaf layers.
pub fn pixel(layer: u16, x: i32, y: i32) -> [u8; 4] {
    match layer {
        tex::PINE_SIDE => plates([118, 74, 52], [70, 48, 38], 400, x, y),
        tex::PALM_SIDE => rings([150, 128, 98], [104, 86, 64], 410, x, y),
        tex::CHERRY_SIDE => lenticels([92, 58, 52], 420, x, y),
        tex::BAOBAB_SIDE => grain([164, 148, 132], 430, x, y),
        tex::ACACIA_SIDE => plates([96, 76, 60], [58, 46, 38], 440, x, y),
        tex::REDWOOD_SIDE => fibres([150, 78, 54], 450, x, y),
        tex::EUCALYPTUS_SIDE => patches(460, x, y),
        tex::BAMBOO_SIDE => culm(470, x, y),
        tex::PALM_FRONDS => leaves_in(&PALM, 480, x, y, 5),
        tex::CHERRY_BLOSSOM => leaves_in(&CHERRY, 490, x, y, 4),
        tex::BAOBAB_LEAVES => leaves_in(&BAOBAB, 500, x, y, 4),
        tex::ACACIA_LEAVES => leaves_in(&ACACIA, 510, x, y, 3),
        tex::REDWOOD_NEEDLES => leaves_in(&REDWOOD, 520, x, y, 5),
        tex::EUCALYPTUS_LEAVES => leaves_in(&EUCALYPTUS, 530, x, y, 5),
        tex::BAMBOO_LEAVES => leaves_in(&BAMBOO, 540, x, y, 5),
        _ => unreachable!("not a tree texture"),
    }
}

/// Bark of plates split by dark furrows running up the trunk (pine, acacia).
fn plates(base: [u8; 3], furrow: [u8; 3], seed: u32, x: i32, y: i32) -> [u8; 4] {
    if smooth(seed, x * 3, y / 2, 4) < 0.3 {
        [furrow[0], furrow[1], furrow[2], 255]
    } else {
        grain(base, seed + 5, x, y)
    }
}

/// A palm's ringed trunk: a scar across it every few texels.
fn rings(base: [u8; 3], scar: [u8; 3], seed: u32, x: i32, y: i32) -> [u8; 4] {
    if y % 4 == 0 && n(seed, x, y) > 0.2 {
        [scar[0], scar[1], scar[2], 255]
    } else {
        grain(base, seed + 5, x, y)
    }
}

/// A cherry's smooth dark bark with pale bands of lenticels across it.
fn lenticels(base: [u8; 3], seed: u32, x: i32, y: i32) -> [u8; 4] {
    if y % 5 == 2 && n(seed, x / 2, y) > 0.35 {
        [168, 132, 110, 255]
    } else {
        grain(base, seed + 5, x, y)
    }
}

/// A redwood's soft red bark in long fibres up the trunk.
fn fibres(base: [u8; 3], seed: u32, x: i32, y: i32) -> [u8; 4] {
    let k = 0.75 + 0.35 * smooth(seed, x * 4, y / 4, 4);
    [(f64::from(base[0]) * k) as u8, (f64::from(base[1]) * k) as u8, (f64::from(base[2]) * k) as u8, 255]
}

/// A eucalyptus' smooth bark peeling in pale, grey and ochre patches.
fn patches(seed: u32, x: i32, y: i32) -> [u8; 4] {
    let p = smooth(seed, x, y, 4);
    let base = if p < 0.35 {
        [206, 200, 186]
    } else if p < 0.6 {
        [168, 170, 160]
    } else {
        [190, 160, 112]
    };
    grain(base, seed + 5, x, y)
}

/// Bamboo: green culm with a pale node ring.
fn culm(seed: u32, x: i32, y: i32) -> [u8; 4] {
    if y % 8 == 7 {
        [168, 176, 104, 255]
    } else {
        grain([92, 138, 62], seed, x, y)
    }
}
