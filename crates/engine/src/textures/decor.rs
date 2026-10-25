//! The decor set's looks (`block/decor.rs`): five coloured concretes, four kinds of rock bricks, framed glass, metal
//! panels, a catwalk grating, a railing and a panel light. Placeholder looks until the art pass (`docs/ART_HANDOVER.md`).

use super::paint::mottle;
use super::{n, rgb};
use crate::block::tex;

const RED: [f64; 3] = [172.0, 72.0, 58.0];
const GREEN: [f64; 3] = [78.0, 124.0, 82.0];
const BLACK: [f64; 3] = [52.0, 54.0, 60.0];
const WHITE: [f64; 3] = [222.0, 222.0, 214.0];
const YELLOW: [f64; 3] = [214.0, 182.0, 62.0];
const STEEL: [f64; 3] = [150.0, 156.0, 164.0];

pub fn pixel(layer: u16, x: i32, y: i32) -> [u8; 4] {
    match layer {
        tex::RED_CONCRETE => concrete(x, y, RED),
        tex::GREEN_CONCRETE => concrete(x, y, GREEN),
        tex::BLACK_CONCRETE => concrete(x, y, BLACK),
        tex::WHITE_CONCRETE => concrete(x, y, WHITE),
        tex::YELLOW_CONCRETE => concrete(x, y, YELLOW),
        tex::BASALT_TILES => tiles(x, y),
        tex::GRANITE_BRICKS => bricks(x, y, [166.0, 122.0, 112.0], [96.0, 74.0, 70.0], 740),
        tex::SANDSTONE_BRICKS => bricks(x, y, [218.0, 192.0, 142.0], [150.0, 128.0, 92.0], 750),
        tex::SLAG_BRICKS => bricks(x, y, [74.0, 64.0, 70.0], [34.0, 30.0, 34.0], 760),
        tex::FRAMED_GLASS => framed_glass(x, y),
        tex::METAL_PANEL => metal_panel(x, y),
        tex::CATWALK => catwalk(x, y),
        tex::RAILING => railing(x, y),
        _ => panel_light(x, y),
    }
}

/// Poured concrete: broad mottling, a seam every 8 texels like cast slabs and a few pits.
fn concrete(x: i32, y: i32, colour: [f64; 3]) -> [u8; 4] {
    if x % 8 == 0 || y % 8 == 0 {
        return rgb(colour, 0.78 + 0.06 * n(720, x, y));
    }
    let pit = n(721, x, y) > 0.96;
    rgb(colour, 0.9 + 0.2 * mottle(722, x, y, 0.2) - if pit { 0.12 } else { 0.0 })
}

/// Bricks in a running bond, each its own shade, recessed mortar between them.
pub(super) fn bricks(x: i32, y: i32, brick: [f64; 3], mortar: [f64; 3], seed: u32) -> [u8; 4] {
    let (row, shift) = (y / 4, if (y / 4) % 2 == 0 { 0 } else { 4 });
    let mortar_line = y % 4 == 3 || (x + shift).rem_euclid(8) == 7;
    if mortar_line {
        return rgb(mortar, 0.9 + 0.1 * n(seed, x, y));
    }
    let own = 0.88 + 0.14 * n(seed + 1, (x + shift).rem_euclid(16) / 8 + row * 3, row);
    let lit = if y % 4 == 0 { 0.07 } else { 0.0 };
    rgb(brick, own + lit + 0.05 * n(seed + 2, x, y))
}

/// Dark basalt cut into square tiles with pale grout.
fn tiles(x: i32, y: i32) -> [u8; 4] {
    if x % 8 == 7 || y % 8 == 7 {
        return rgb([128.0, 128.0, 130.0], 0.9 + 0.1 * n(730, x, y));
    }
    let own = 0.9 + 0.12 * n(731, x / 8, y / 8);
    rgb([70.0, 72.0, 80.0], own + 0.08 * mottle(732, x, y, 0.2) + if x % 8 == 0 || y % 8 == 0 { 0.08 } else { 0.0 })
}

/// A see-through pane in a steel frame with one cross bar.
fn framed_glass(x: i32, y: i32) -> [u8; 4] {
    let frame = x <= 1 || x >= 14 || y <= 1 || y >= 14 || x == 7 || x == 8 || y == 7 || y == 8;
    if frame {
        let edge = x == 0 || x == 15 || y == 0 || y == 15;
        return rgb(STEEL, if edge { 0.55 } else { 0.8 + 0.12 * n(770, x, y) });
    }
    let mut c = rgb([178.0, 210.0, 214.0], 0.9 + 0.08 * n(771, x, y));
    c[3] = if n(772, x, y) > 0.97 { 255 } else { 0 };
    c
}

/// Plain steel panels, bolted at each corner of a 8×8 plate.
fn metal_panel(x: i32, y: i32) -> [u8; 4] {
    if x % 8 == 0 || y % 8 == 0 {
        return rgb(STEEL, 0.5);
    }
    if matches!(x % 8, 1 | 6) && matches!(y % 8, 1 | 6) {
        return rgb(STEEL, 1.2);
    }
    rgb(STEEL, 0.82 + 0.08 * n(780, x, y / 4) + 0.04 * n(781, x, y))
}

/// A walkable grating: a lattice of flat bars with open squares between (full block, see-through).
fn catwalk(x: i32, y: i32) -> [u8; 4] {
    if x % 4 < 2 || y % 4 < 2 {
        return rgb(STEEL, if (x % 4 == 0) || (y % 4 == 0) { 0.7 } else { 0.95 } + 0.05 * n(790, x, y));
    }
    let mut c = rgb(STEEL, 0.4);
    c[3] = 0;
    c
}

/// A handrail: a top rail, a low rail and posts, open between.
fn railing(x: i32, y: i32) -> [u8; 4] {
    if (3..=4).contains(&y) || (10..=11).contains(&y) || matches!(x, 1 | 2 | 7 | 8 | 13 | 14) {
        return rgb(STEEL, 0.75 + 0.2 * n(800, x, y) + if y == 3 || x % 6 == 1 { 0.1 } else { 0.0 });
    }
    let mut c = rgb(STEEL, 0.4);
    c[3] = 0;
    c
}

/// A glowing white panel in a thin dark frame, brightest in the middle.
fn panel_light(x: i32, y: i32) -> [u8; 4] {
    if x == 0 || x == 15 || y == 0 || y == 15 {
        return rgb([84.0, 86.0, 92.0], 0.9 + 0.1 * n(810, x, y));
    }
    let (dx, dy) = (f64::from(x) - 7.5, f64::from(y) - 7.5);
    let glow = 1.0 - (dx * dx + dy * dy).sqrt() / 14.0;
    rgb([255.0, 246.0, 220.0], 0.84 + 0.16 * glow + 0.03 * n(811, x, y))
}
