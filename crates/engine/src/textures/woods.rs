//! Placeholder looks for the woods after the oak (`block/woods.rs`): each wood's planks, floor, door and open door are
//! the oak's (`wood.rs`, `house.rs`) recoloured by lightness onto that wood's tone, keeping the oak's pattern, its
//! grey nails and hinges, its brass handle and the open door's cut-out. The art pack replaces them in the browser
//! (`tools/art/woods.py`). To add a wood: a row in `TONES` (same order as `block/woods.rs`'s names).

use super::{house, n, smooth, wood};
use crate::block::tex;

/// Each wood's mid tone (birch, pine, palm, cherry, baobab, acacia, redwood, eucalyptus, bamboo).
const TONES: [[f64; 3]; 9] = [
    [206.0, 184.0, 138.0],
    [208.0, 166.0, 98.0],
    [190.0, 172.0, 138.0],
    [150.0, 78.0, 56.0],
    [172.0, 158.0, 138.0],
    [190.0, 112.0, 56.0],
    [128.0, 56.0, 42.0],
    [204.0, 148.0, 120.0],
    [200.0, 176.0, 96.0],
];
/// The lightness of the oak's own mid tone: texels at this lightness take the wood's tone as it is.
const OAK_LIGHT: f64 = 122.0;
const PALM: usize = 2;
const EUCALYPTUS: usize = 7;

/// Texel (`x`, `y`) of layer `layer`, one of `tex::planks_layer(0)..tex::COUNT`.
pub fn pixel(layer: u16, x: i32, y: i32) -> [u8; 4] {
    let i = layer - tex::planks_layer(0);
    let w = usize::from(i / 4);
    let oak = match i % 4 {
        0 => wood::planks(x, y),
        1 => house::pixel(tex::WOOD_FLOOR, x, y),
        2 => house::pixel(tex::WOOD_DOOR, x, y),
        _ => house::pixel(tex::WOOD_DOOR_OPEN, x, y),
    };
    let brass = i % 4 >= 2 && matches!((x, y), (12 | 13, 7 | 8));
    let grey = oak[..3].iter().max().unwrap() - oak[..3].iter().min().unwrap() <= 16;
    if brass || grey {
        return oak;
    }
    recolour(w, oak, x, y)
}

fn recolour(w: usize, oak: [u8; 4], x: i32, y: i32) -> [u8; 4] {
    let mut tone = TONES[w];
    if w == EUCALYPTUS {
        // Patches of pinkish tan and light brown.
        let t = smooth(610, x, y, 4);
        tone = [204.0 - 16.0 * t, 148.0 - 8.0 * t, 120.0 - 22.0 * t];
    }
    let light = 0.299 * f64::from(oak[0]) + 0.587 * f64::from(oak[1]) + 0.114 * f64::from(oak[2]);
    let mut k = light / OAK_LIGHT;
    if w == PALM {
        // Fibres along the board.
        k *= 1.0 + 0.3 * (n(600, x / 4, y) - 0.5);
    }
    let ch = |t: f64| {
        let v = if k <= 1.0 { t * k } else { t + (255.0 - t) * ((k - 1.0) * 0.8).min(1.0) };
        v.round().clamp(0.0, 255.0) as u8
    };
    [ch(tone[0]), ch(tone[1]), ch(tone[2]), oak[3]]
}
