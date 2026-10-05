//! Electronics (Milestone 7): the arc furnace (a graphite housing with a violet-white arc slit and
//! copper terminals, a roof with two electrode caps), silicon (a blue-grey crystalline ingot) and the
//! circuit (a green board with copper traces and a black chip) and the violet science pack (its flask fill). Placeholder looks until the art pass
//! (`docs/ART_HANDOVER.md`).

use crate::block::tex;

use super::{frame, n, rgb, smooth};

pub fn pixel(layer: u16, x: i32, y: i32) -> [u8; 4] {
    match layer {
        tex::ARC_SIDE => arc_side(x, y),
        tex::ARC_TOP => arc_top(x, y),
        tex::SILICON => silicon(x, y),
        tex::CIRCUIT => circuit(x, y),
        _ => violet_pack(x, y),
    }
}

/// Dark graphite plating with a glowing arc slit and copper terminal studs.
fn arc_side(x: i32, y: i32) -> [u8; 4] {
    let k = 0.9 + 0.1 * n(394, x, y);
    if (6..=8).contains(&y) && (2..14).contains(&x) {
        let core = y == 7;
        return rgb([196.0, 168.0, 255.0], if core { 1.2 } else { 0.85 });
    }
    if (y == 2 || y == 13) && x % 5 == 2 {
        return rgb([204.0, 128.0, 76.0], 1.05);
    }
    rgb([54.0, 56.0, 64.0], k + 0.08 * smooth(395, x, y, 4))
}

/// A steel roof with two electrode caps and a copper ring around each.
fn arc_top(x: i32, y: i32) -> [u8; 4] {
    for cx in [4.5, 11.5] {
        let r = ((x as f64 - cx).powi(2) + (y as f64 - 7.5).powi(2)).sqrt();
        if r < 1.8 {
            return rgb([40.0, 42.0, 50.0], 1.0);
        }
        if r < 2.9 {
            return rgb([204.0, 128.0, 76.0], 0.95);
        }
    }
    frame(x, y)
}

/// Blue-grey crystalline sheen with pale facets.
fn silicon(x: i32, y: i32) -> [u8; 4] {
    let facet = smooth(396, x, y, 4);
    let k = 0.75 + 0.5 * facet + 0.1 * n(397, x, y);
    rgb([98.0, 112.0, 148.0], k)
}

/// A green board: copper traces run across it, a black chip sits in the middle, solder dots at the edge.
fn circuit(x: i32, y: i32) -> [u8; 4] {
    if (5..11).contains(&x) && (5..11).contains(&y) {
        return if x == 5 && y == 5 { rgb([230.0, 230.0, 230.0], 1.0) } else { rgb([26.0, 26.0, 30.0], 1.0) };
    }
    let trace = y % 4 == 1 && x % 8 < 6 || x % 4 == 1 && y % 8 < 3;
    if trace {
        return rgb([214.0, 168.0, 84.0], 0.95);
    }
    if (x == 0 || x == 15) && y % 3 == 0 {
        return rgb([220.0, 220.0, 224.0], 1.0);
    }
    rgb([34.0, 118.0, 66.0], 0.9 + 0.1 * n(398, x, y))
}

/// The gold pack's flask fill: golden liquid under a pale meniscus, in a darker rim.
pub(super) fn gold_pack(x: i32, y: i32) -> [u8; 4] {
    let c = if y < 3 {
        [244.0, 232.0, 188.0]
    } else if x <= 1 || x >= 14 {
        [150.0, 108.0, 20.0]
    } else {
        [236.0, 184.0, 52.0]
    };
    rgb(c, 0.94 + 0.1 * n(399, x, y))
}

/// The pack's flask fill: violet liquid under a pale meniscus, in a darker rim (like the blue one in `steel.rs`).
fn violet_pack(x: i32, y: i32) -> [u8; 4] {
    let c = if y < 3 {
        [216.0, 204.0, 232.0]
    } else if x <= 1 || x >= 14 {
        [88.0, 44.0, 140.0]
    } else {
        [148.0, 96.0, 214.0]
    };
    rgb(c, 0.94 + 0.1 * n(399, x, y))
}
