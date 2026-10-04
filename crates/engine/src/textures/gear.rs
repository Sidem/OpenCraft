//! Worn gear icons (`equipment.rs`): the hauler packs (a satchel, the Mk2 with side pouches and a violet flap),
//! spring boots (boots on a coil), servo boots (boots with an orange joint), the exo frame (steel ribs round a
//! cyan core) and the mining rig (a belt with a drill bit and a gauge). Placeholder looks until the art pass
//! (`docs/ART_HANDOVER.md`).

use crate::block::tex;

use super::{n, rgb};

pub fn pixel(layer: u16, x: i32, y: i32) -> [u8; 4] {
    match layer {
        tex::HAULER_PACK => pack(x, y, false),
        tex::HAULER_PACK_MK2 => pack(x, y, true),
        tex::SPRING_BOOTS => boots(x, y, [40.0, 150.0, 150.0], true),
        tex::SERVO_BOOTS => boots(x, y, [60.0, 90.0, 170.0], false),
        tex::EXO_FRAME => exo_frame(x, y),
        _ => mining_rig(x, y),
    }
}

const BACKDROP: f64 = 0.9;

fn dark() -> [u8; 4] {
    rgb([22.0, 24.0, 30.0], BACKDROP)
}

/// A satchel with two straps and a buckle; the Mk2 is wider, with a pouch each side and a violet flap.
fn pack(x: i32, y: i32, mk2: bool) -> [u8; 4] {
    let (left, right) = if mk2 { (1, 14) } else { (3, 12) };
    if !(left..=right).contains(&x) || !(2..15).contains(&y) {
        return dark();
    }
    if mk2 && !(4..=11).contains(&x) && y > 6 {
        return rgb([92.0, 78.0, 56.0], 0.85 + 0.2 * n(420, x, y));
    }
    if y < 7 {
        let flap = if mk2 { [120.0, 70.0, 190.0] } else { [140.0, 100.0, 60.0] };
        return rgb(flap, 0.9 + 0.15 * n(421, x, y));
    }
    if (x == 6 || x == 9) && y > 7 {
        return rgb([52.0, 44.0, 36.0], 1.0);
    }
    if (7..9).contains(&x) && (6..9).contains(&y) {
        return rgb([226.0, 200.0, 90.0], 1.1);
    }
    rgb([120.0, 92.0, 58.0], 0.85 + 0.2 * n(422, x, y))
}

/// Two boots side by side, `tint` on the shaft; spring boots stand on a zig-zag coil, servo boots have an orange joint.
fn boots(x: i32, y: i32, tint: [f64; 3], spring: bool) -> [u8; 4] {
    let foot = |cx: i32| (cx - 3..=cx + 2).contains(&x);
    let (l, r) = (foot(4), foot(11));
    if !(l || r) {
        return dark();
    }
    let cx = if l { 4 } else { 11 };
    if y < 8 {
        return if x - cx == -3 || x - cx == 2 { rgb(tint, 0.7) } else { rgb(tint, 1.0 - 0.1 * (y % 2) as f64) };
    }
    if y < 11 {
        // The foot, with the toe running out to the right.
        return rgb([60.0, 62.0, 72.0], 0.9 + 0.1 * n(423, x, y));
    }
    if spring {
        let zig = (y + x) % 2 == 0;
        return if zig { rgb([230.0, 232.0, 240.0], 1.0) } else { dark() };
    }
    if y == 11 && (x - cx).abs() < 2 {
        return rgb([255.0, 140.0, 40.0], 1.1);
    }
    rgb([30.0, 32.0, 40.0], 1.0)
}

/// A steel breastplate: two ribs either side of a cyan core, a shoulder bar across the top.
fn exo_frame(x: i32, y: i32) -> [u8; 4] {
    if y < 3 && (1..15).contains(&x) {
        return rgb([188.0, 192.0, 202.0], 1.0);
    }
    if !(3..13).contains(&x) || !(3..15).contains(&y) {
        return dark();
    }
    let r = ((x as f64 - 7.5).powi(2) + (y as f64 - 8.0).powi(2)).sqrt();
    if r < 2.2 {
        return rgb([90.0, 226.0, 255.0], 1.15 - 0.3 * r / 2.2);
    }
    if r < 3.2 {
        return rgb([226.0, 230.0, 238.0], 1.0);
    }
    if x == 3 || x == 12 || y % 4 == 0 {
        return rgb([150.0, 158.0, 176.0], 1.0);
    }
    rgb([84.0, 92.0, 112.0], 0.85 + 0.15 * n(424, x, y))
}

/// A leather belt with a drill bit hanging from it and an orange gauge.
fn mining_rig(x: i32, y: i32) -> [u8; 4] {
    if (3..6).contains(&y) {
        return if (6..10).contains(&x) {
            rgb([226.0, 200.0, 90.0], 1.1)
        } else {
            rgb([104.0, 76.0, 50.0], 0.9 + 0.15 * n(425, x, y))
        };
    }
    if (2..7).contains(&x) && (6..14).contains(&y) {
        // The drill bit: a spiral of steel narrowing to a point.
        let width = 3 - (y - 6) / 3;
        if (x - 4).abs() < width {
            return rgb([188.0, 192.0, 202.0], if (y + x) % 3 == 0 { 0.7 } else { 1.0 });
        }
    }
    let r = ((x as f64 - 11.0).powi(2) + (y as f64 - 10.0).powi(2)).sqrt();
    if r < 3.0 {
        return if r > 2.0 {
            rgb([188.0, 192.0, 202.0], 1.0)
        } else {
            rgb([255.0, 140.0, 40.0], 1.0 + 0.1 * n(426, x, y))
        };
    }
    dark()
}
