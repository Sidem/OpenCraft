//! Machine and item textures (belts, machines, routers, power, ingots and parts): one pixel function
//! per pattern, called from `textures::pixel`. To add one: a function here and its arm there.

use super::{n, rgb, smooth};

/// Rubber belt with chevrons pointing towards texture row 0, which the belt model maps to its
/// travel direction; the pattern repeats every 8 rows so scrolling the texture loops seamlessly.
pub(super) fn belt_top(x: i32, y: i32) -> [u8; 4] {
    if x == 0 || x == 15 {
        return rgb([26.0, 26.0, 30.0], 1.0);
    }
    let dx = (x as f64 - 7.5).abs();
    // Tips towards row 0, which the renderer maps to the belt's front edge.
    let chevron = ((y as f64 - dx * 0.75).floor() as i32).rem_euclid(8) < 2 && dx < 6.0;
    let k = 0.9 + 0.12 * n(46, x, y);
    if chevron {
        rgb([176.0, 150.0, 92.0], k)
    } else if y % 4 == 3 {
        rgb([34.0, 34.0, 38.0], k)
    } else {
        rgb([52.0, 52.0, 58.0], k)
    }
}

/// Dark machine housing with a band of hazard stripes.
pub(super) fn miner_side(x: i32, y: i32) -> [u8; 4] {
    let edge = x == 0 || y == 0 || x == 15 || y == 15;
    let k = 0.88 + 0.12 * n(47, x, y);
    if edge {
        rgb([38.0, 40.0, 46.0], k)
    } else if (10..=12).contains(&y) {
        if (x + y).rem_euclid(4) < 2 {
            rgb([232.0, 146.0, 60.0], k)
        } else {
            rgb([40.0, 38.0, 36.0], k)
        }
    } else if (x == 2 || x == 13) && (y == 2 || y == 7) {
        rgb([150.0, 156.0, 166.0], 1.0)
    } else {
        rgb([70.0, 76.0, 86.0], k)
    }
}

/// Machine top: a vent grille.
pub(super) fn miner_top(x: i32, y: i32) -> [u8; 4] {
    let edge = x == 0 || y == 0 || x == 15 || y == 15;
    let slot = (3..=12).contains(&x) && (3..=12).contains(&y) && y % 3 == 1;
    let k = 0.88 + 0.12 * n(48, x, y);
    if edge {
        rgb([38.0, 40.0, 46.0], k)
    } else if slot {
        rgb([20.0, 22.0, 26.0], 1.0)
    } else {
        rgb([74.0, 80.0, 90.0], k)
    }
}

/// Polished drill steel with a diagonal thread.
pub(super) fn drill(x: i32, y: i32) -> [u8; 4] {
    let k = 0.92 + 0.08 * n(49, x, y);
    if (x + y).rem_euclid(4) == 0 {
        rgb([104.0, 108.0, 116.0], k)
    } else {
        rgb([180.0, 184.0, 192.0], k)
    }
}

/// Wooden crate: planks inside a darker frame with iron corner brackets.
pub(super) fn crate_wood(x: i32, y: i32, vertical: bool) -> [u8; 4] {
    let (along, across) = if vertical { (y, x) } else { (x, y) };
    let bracket = (x <= 2 || x >= 13) && (y <= 2 || y >= 13);
    let frame = x <= 1 || y <= 1 || x >= 14 || y >= 14;
    let k = 0.85 + 0.12 * n(50, x, y) + 0.06 * n(51, along / 5, across);
    if bracket {
        rgb([104.0, 108.0, 116.0], 0.9 + 0.1 * n(52, x, y))
    } else if frame {
        rgb([112.0, 80.0, 48.0], k)
    } else if across % 4 == 1 {
        rgb([92.0, 64.0, 38.0], k)
    } else {
        rgb([168.0, 128.0, 80.0], k)
    }
}

/// Status lamp: a flat colour with a highlight, bright enough to read as lit.
pub(super) fn lamp(x: i32, y: i32, c: [f64; 3]) -> [u8; 4] {
    let (dx, dy) = (x as f64 - 6.0, y as f64 - 6.0);
    let glow = 1.25 - (dx * dx + dy * dy).sqrt() * 0.04;
    rgb(c, glow)
}

/// Cast metal: a bright bevelled rim around a slightly mottled face.
pub(super) fn ingot(x: i32, y: i32, c: [f64; 3]) -> [u8; 4] {
    let k = 0.9 + 0.08 * n(53, x, y) + 0.06 * smooth(54, x, y, 4);
    if x == 0 || y == 0 {
        rgb(c, 1.2)
    } else if x == 15 || y == 15 {
        rgb(c, 0.7)
    } else {
        rgb(c, k)
    }
}

/// Fire bricks in running bond (rows of 4 pixels), with a glowing grate low on the sides.
pub(super) fn smelter(x: i32, y: i32, side: bool) -> [u8; 4] {
    if side && (10..=13).contains(&y) && (5..=10).contains(&x) {
        let glow = if x % 2 == 0 { 0.7 } else { 1.0 + 0.2 * n(57, x, y) };
        return rgb([236.0, 124.0, 44.0], glow);
    }
    let row = y / 4;
    let shift = if row % 2 == 0 { 0 } else { 4 };
    let mortar = y % 4 == 3 || (x + shift) % 8 == 7;
    let k = 0.82 + 0.14 * n(55, x, y) + 0.1 * n(56, (x + shift) / 8, row);
    if mortar {
        rgb([96.0, 90.0, 84.0], 0.9 + 0.1 * n(58, x, y))
    } else {
        rgb([150.0, 82.0, 62.0], k)
    }
}

/// Rolled steel: brushed streaks, a bevelled edge and a punched hole in each corner.
pub(super) fn plate(x: i32, y: i32) -> [u8; 4] {
    let hole = (x == 2 || x == 13) && (y == 2 || y == 13);
    let k = 0.9 + 0.1 * n(59, x / 6, y) + 0.04 * n(60, x, y);
    if hole {
        rgb([52.0, 54.0, 60.0], 1.0)
    } else if x == 0 || y == 0 || x == 15 || y == 15 {
        rgb([150.0, 156.0, 166.0], 0.8)
    } else {
        rgb([168.0, 174.0, 184.0], k)
    }
}

/// Coiled copper wire: bright turns with dark gaps between them.
pub(super) fn wire(x: i32, y: i32) -> [u8; 4] {
    let turn = (y + x / 8).rem_euclid(3);
    let k = 0.9 + 0.12 * n(61, x, y);
    match turn {
        0 => rgb([120.0, 62.0, 34.0], k),
        1 => rgb([230.0, 146.0, 86.0], k),
        _ => rgb([200.0, 112.0, 60.0], k),
    }
}

/// Teal machine panel with a dark inspection window (sides) or a press plate (top).
pub(super) fn constructor(x: i32, y: i32, side: bool) -> [u8; 4] {
    let edge = x == 0 || y == 0 || x == 15 || y == 15;
    let k = 0.88 + 0.12 * n(62, x, y);
    if edge {
        rgb([34.0, 52.0, 56.0], k)
    } else if side && (4..=11).contains(&x) && (3..=8).contains(&y) {
        let glint = x - y == 1 || x - y == 2;
        rgb(if glint { [120.0, 150.0, 156.0] } else { [26.0, 34.0, 38.0] }, 1.0)
    } else if !side && (3..=12).contains(&x) && (3..=12).contains(&y) {
        rgb([150.0, 156.0, 166.0], 0.85 + 0.15 * n(63, x, y / 3))
    } else {
        rgb([58.0, 132.0, 138.0], k)
    }
}

/// Router top: dark steel with arrows pointing towards row 0 (the router's front, like the belt) and,
/// for the splitter, out to both sides; the filter's front arrow is amber.
pub(super) fn router_top(x: i32, y: i32, filter: bool) -> [u8; 4] {
    let (fx, fy) = (x as f64 - 7.5, y as f64 - 7.5);
    // Arrowhead: a triangle with its tip at row 0.
    let front = (-7.0..-1.0).contains(&fy) && fx.abs() <= (fy + 7.0) * 0.7;
    let side = fy.abs() < 1.2 && fx.abs() > 2.0 && fx.abs() < 7.0;
    let stem = fx.abs() < 1.0 && (0.0..6.0).contains(&fy);
    let k = 0.9 + 0.1 * n(64, x, y);
    if x == 0 || y == 0 || x == 15 || y == 15 {
        rgb([34.0, 36.0, 42.0], k)
    } else if front {
        rgb(if filter { [236.0, 170.0, 60.0] } else { [200.0, 204.0, 212.0] }, k)
    } else if side || stem {
        rgb([150.0, 156.0, 166.0], k)
    } else {
        rgb([62.0, 66.0, 76.0], k)
    }
}

/// Icon sides of the belts that climb and cross: a steel panel with the belt's path in rubber
/// (`kind`: 0 ramp up, 1 ramp down, 2 lift, 3 underpass entry, 4 underpass exit). An underpass has an
/// amber hood over the end where items go under or come back up.
pub(super) fn belt_side(x: i32, y: i32, kind: u8) -> [u8; 4] {
    let (fx, fy) = (x as f64, 15.0 - y as f64);
    let path = match kind {
        0 => (fy - fx).abs() < 2.5,
        1 => (fy + fx - 15.0).abs() < 2.5,
        2 => (fx - 7.5).abs() < 3.0,
        _ => (1.0..5.0).contains(&fy),
    };
    let stripe = (y as f64 + (fx - 7.5).abs() * 0.8) as i32 % 5 == 0;
    let (hx, hy) = (fx - if kind == 3 { 11.5 } else { 3.5 }, fy - 2.0);
    let hood = kind >= 3 && hy >= 0.0 && (4.5..7.0).contains(&(hx * hx + hy * hy).sqrt());
    let k = 0.9 + 0.1 * n(65, x, y);
    if x == 0 || y == 0 || x == 15 || y == 15 {
        rgb([34.0, 36.0, 42.0], k)
    } else if hood {
        rgb([236.0, 170.0, 60.0], k)
    } else if path && kind == 2 && stripe {
        rgb([176.0, 150.0, 92.0], k)
    } else if path {
        rgb([40.0, 40.0, 46.0], k)
    } else {
        rgb([120.0, 126.0, 134.0], k)
    }
}

/// Generator: dark steel with vents and a hazard stripe on the side; a round fan grille on top.
pub(super) fn generator(x: i32, y: i32, top: bool) -> [u8; 4] {
    let k = 0.9 + 0.1 * n(66, x, y);
    let edge = x == 0 || y == 0 || x == 15 || y == 15;
    if edge {
        return rgb([34.0, 36.0, 42.0], k);
    }
    if top {
        let r = ((x as f64 - 7.5).powi(2) + (y as f64 - 7.5).powi(2)).sqrt();
        return match r {
            r if r < 1.5 => rgb([236.0, 190.0, 64.0], k),
            r if r < 6.0 && (x + y) % 3 == 0 => rgb([40.0, 42.0, 48.0], k),
            r if r < 6.5 => rgb([88.0, 92.0, 100.0], k),
            _ => rgb([70.0, 74.0, 82.0], k),
        };
    }
    if (11..=13).contains(&y) {
        let stripe = (x + y) % 6 < 3;
        rgb(if stripe { [236.0, 190.0, 64.0] } else { [30.0, 30.0, 34.0] }, k)
    } else if (3..=8).contains(&y) && y % 2 == 1 && (2..=13).contains(&x) {
        rgb([40.0, 42.0, 48.0], k)
    } else {
        rgb([88.0, 92.0, 100.0], k)
    }
}

/// Power pole icon: a steel post with a crossarm and copper insulators, on a clear background.
pub(super) fn pole(x: i32, y: i32) -> [u8; 4] {
    let k = 0.9 + 0.1 * n(67, x, y);
    let post = (7..=8).contains(&x) && y >= 2;
    let arm = (3..=4).contains(&y) && (2..=13).contains(&x);
    let insulator = (1..=2).contains(&y) && (x == 3 || x == 12);
    if insulator {
        rgb([206.0, 118.0, 70.0], k)
    } else if post || arm {
        rgb([120.0, 126.0, 134.0], k)
    } else {
        [0, 0, 0, 0]
    }
}
