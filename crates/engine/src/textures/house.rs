//! The house set's looks (`block/house.rs`): wooden and iron doors (shut: a whole face; open: a leaf along one edge, the
//! rest see-through, the same on every face because a door has no facing), three roof tiles, thatch, a timber frame,
//! a wood floor, a window, chimney bricks and plaster. Placeholder looks until the art pass (`docs/ART_HANDOVER.md`).

use super::decor::bricks;
use super::paint::mottle;
use super::{n, rgb};
use crate::block::tex;

const OAK: [f64; 3] = [158.0, 114.0, 66.0];
const IRON: [f64; 3] = [96.0, 100.0, 108.0];
const BRASS: [f64; 3] = [214.0, 170.0, 70.0];
const PLASTER: [f64; 3] = [226.0, 218.0, 196.0];

pub fn pixel(layer: u16, x: i32, y: i32) -> [u8; 4] {
    match layer {
        tex::WOOD_DOOR => wood_door(x, y),
        tex::WOOD_DOOR_OPEN => leaf(x, wood_door(x, y)),
        tex::IRON_DOOR => iron_door(x, y),
        tex::IRON_DOOR_OPEN => leaf(x, iron_door(x, y)),
        tex::RED_ROOF => roof(x, y, [176.0, 78.0, 56.0], 820),
        tex::SLATE_ROOF => roof(x, y, [74.0, 82.0, 96.0], 830),
        tex::GREEN_ROOF => roof(x, y, [70.0, 128.0, 104.0], 840),
        tex::THATCH => thatch(x, y),
        tex::TIMBER_FRAME => timber_frame(x, y),
        tex::WOOD_FLOOR => wood_floor(x, y),
        tex::WINDOW => window(x, y),
        tex::CHIMNEY_BRICKS => bricks(x, y, [120.0, 74.0, 62.0], [44.0, 38.0, 38.0], 850),
        _ => plaster(x, y),
    }
}

/// An open door seen edge-on from any side: its leaf is the two columns at the edge, the rest is open.
fn leaf(x: i32, shut: [u8; 4]) -> [u8; 4] {
    if x <= 1 {
        shut
    } else {
        [shut[0], shut[1], shut[2], 0]
    }
}

/// Upright boards with two cross braces, a brass handle and dark hinges.
fn wood_door(x: i32, y: i32) -> [u8; 4] {
    if x == 0 || x == 15 || y == 0 || y == 15 {
        return rgb(OAK, 0.5);
    }
    if matches!((x, y), (12, 7) | (12, 8) | (13, 7) | (13, 8)) {
        return rgb(BRASS, 1.0 + 0.1 * n(860, x, y));
    }
    if x == 1 && matches!(y, 2..=3 | 12..=13) {
        return rgb([52.0, 50.0, 54.0], 1.0);
    }
    let brace = matches!(y, 3..=4 | 11..=12);
    let seam = x % 4 == 0;
    let k = if seam {
        0.62
    } else if brace {
        0.82
    } else {
        0.95
    };
    rgb(OAK, k + 0.1 * n(861, x / 4, y) + 0.05 * n(862, x, y))
}

/// Riveted steel plate in two panels with a dark handle.
fn iron_door(x: i32, y: i32) -> [u8; 4] {
    if x == 0 || x == 15 || y == 0 || y == 15 || y == 7 || y == 8 {
        return rgb(IRON, 0.62 + 0.06 * n(870, x, y));
    }
    if matches!((x, y), (12, 7) | (12, 8) | (13, 7) | (13, 8)) {
        return rgb([36.0, 38.0, 42.0], 1.0);
    }
    if matches!(x, 2 | 13) && matches!(y, 2 | 5 | 10 | 13) {
        return rgb(IRON, 1.45);
    }
    rgb(IRON, 0.9 + 0.08 * n(871, x, y / 5) + 0.05 * mottle(872, x, y, 0.2))
}

/// Tiles in overlapping rows, each row shifted half a tile, the lower edge of every tile in shadow.
fn roof(x: i32, y: i32, colour: [f64; 3], seed: u32) -> [u8; 4] {
    let (row, shift) = (y / 4, if (y / 4) % 2 == 0 { 0 } else { 4 });
    let col = (x + shift).rem_euclid(16) / 8;
    let own = 0.9 + 0.16 * n(seed, col + row * 2, row);
    let k = match y % 4 {
        3 => 0.55,
        0 => 1.12,
        _ => 1.0,
    };
    let joint = (x + shift).rem_euclid(8) == 7;
    rgb(colour, own * k * if joint { 0.78 } else { 1.0 } + 0.04 * n(seed + 1, x, y))
}

/// Straw laid in horizontal courses, streaked along its length.
fn thatch(x: i32, y: i32) -> [u8; 4] {
    let course = if y % 5 == 4 { 0.68 } else { 1.0 };
    let streak = 0.82 + 0.3 * n(880, x * 3 + y / 5, y % 5 / 2) + 0.08 * n(881, x, y);
    rgb([200.0, 170.0, 92.0], streak * course)
}

/// Plaster between dark oak beams: a frame all round, a cross brace and a post.
fn timber_frame(x: i32, y: i32) -> [u8; 4] {
    let beam = x <= 1 || x >= 14 || y <= 1 || y >= 14 || (x - y).abs() <= 1 || x == 7 || x == 8;
    if beam {
        let grain = 0.8 + 0.2 * n(890, x / 2, y);
        return rgb([92.0, 62.0, 38.0], grain * if (x + y) % 5 == 0 { 0.85 } else { 1.0 });
    }
    rgb(PLASTER, 0.9 + 0.14 * mottle(891, x, y, 0.2))
}

/// Boards laid lengthwise, a different shade each, with staggered butt joints.
fn wood_floor(x: i32, y: i32) -> [u8; 4] {
    let board = y / 4;
    if y % 4 == 3 {
        return rgb(OAK, 0.5);
    }
    let end = (x + board * 5).rem_euclid(16) == 0;
    let own = 0.86 + 0.2 * n(900, board, 0);
    rgb([176.0, 128.0, 74.0], if end { 0.55 } else { own } + 0.07 * n(901, x, y))
}

/// A wooden frame round four small panes: open between, with a few glints so it reads as glass.
fn window(x: i32, y: i32) -> [u8; 4] {
    let frame = x <= 1 || x >= 14 || y <= 1 || y >= 14 || matches!(x, 7 | 8) || matches!(y, 7 | 8);
    if frame {
        return rgb(OAK, if x == 0 || x == 15 || y == 0 || y == 15 { 0.5 } else { 0.85 + 0.1 * n(910, x, y) });
    }
    let mut c = rgb([186, 214, 222].map(f64::from), 0.95);
    c[3] = if n(911, x, y) > 0.96 { 255 } else { 0 };
    c
}

/// Smooth lime plaster with faint trowel marks.
fn plaster(x: i32, y: i32) -> [u8; 4] {
    rgb(PLASTER, 0.93 + 0.12 * mottle(920, x, y, 0.2) + 0.03 * n(921, x, y))
}
