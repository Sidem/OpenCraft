//! Worn gear icons (`equipment.rs`): the hauler packs (a satchel; Mk2 to Mk5 with side pouches and a flap in the tier's kit colour),
//! spring boots (boots on a coil), servo boots (boots with an orange joint), the exo frame (steel ribs round a
//! cyan core), the mining rig (a belt with a drill bit and a gauge) and the fabricator gloves (a dark glove with a
//! gold cuff and a cyan pad) and the Tool Belt (three heads on a belt). Placeholder looks until the art pass
//! (`docs/ART_HANDOVER.md`).

use crate::block::tex;

use super::{n, rgb};

pub fn pixel(layer: u16, x: i32, y: i32) -> [u8; 4] {
    match layer {
        tex::HAULER_PACK => pack(x, y, 0),
        tex::HAULER_PACK_MK2 => pack(x, y, 1),
        tex::HAULER_PACK_MK3 => pack(x, y, 2),
        tex::HAULER_PACK_MK4 => pack(x, y, 3),
        tex::HAULER_PACK_MK5 => pack(x, y, 4),
        tex::SPRING_BOOTS => boots(x, y, [40.0, 150.0, 150.0], true),
        tex::SERVO_BOOTS => boots(x, y, [60.0, 90.0, 170.0], false),
        tex::EXO_FRAME => exo_frame(x, y),
        tex::WORK_GLOVES => gloves(x, y, 0),
        tex::WORK_GLOVES_MK2 => gloves(x, y, 1),
        tex::WORK_GLOVES_MK3 => gloves(x, y, 2),
        tex::WORK_GLOVES_MK4 => gloves(x, y, 3),
        tex::CRAFT_GLOVES => gloves(x, y, 4),
        tex::XRAY_GOGGLES => goggles(x, y),
        tex::TOOL_BELT..=tex::TOOL_BELT_MK5 => tool_belt(x, y, usize::from(layer - tex::TOOL_BELT)),
        _ => mining_rig(x, y),
    }
}

const BACKDROP: f64 = 0.9;

fn dark() -> [u8; 4] {
    rgb([22.0, 24.0, 30.0], BACKDROP)
}

/// The flap colour of each pack tier above Mk1: the upgrade kit's (green, blue, violet, gold).
const FLAPS: [[f64; 3]; 4] = [[70.0, 170.0, 90.0], [60.0, 120.0, 210.0], [120.0, 70.0, 190.0], [225.0, 180.0, 50.0]];

/// A satchel with two straps and a buckle; from Mk2 on it is wider, with a pouch each side, a flap in the tier's kit
/// colour and one pip on the flap for every tier above Mk1. `tier` is 0 for Mk1.
fn pack(x: i32, y: i32, tier: usize) -> [u8; 4] {
    let (left, right) = if tier > 0 { (1, 14) } else { (3, 12) };
    if !(left..=right).contains(&x) || !(2..15).contains(&y) {
        return dark();
    }
    if tier > 0 && !(4..=11).contains(&x) && y > 6 {
        return rgb([92.0, 78.0, 56.0], 0.85 + 0.2 * n(420, x, y));
    }
    if y < 7 {
        if tier > 0 && y == 3 && x > 3 && x < 4 + 2 * tier as i32 && x % 2 == 0 {
            return rgb([250.0, 250.0, 250.0], 1.0);
        }
        let flap = if tier > 0 { FLAPS[tier - 1] } else { [140.0, 100.0, 60.0] };
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

/// A work glove (`tier` 0..=4): four fingers and a thumb, a cuff in leather (Mk1) or the tier's kit colour, and a cyan
/// servo pad on the top tier (`CRAFT_GLOVES`).
fn gloves(x: i32, y: i32, tier: usize) -> [u8; 4] {
    let finger = |i: i32| x >= 3 + 2 * i && x < 5 + 2 * i && y >= 2 + (i - 1).abs() && y < 8;
    let palm = (3..11).contains(&x) && (7..12).contains(&y);
    let thumb = (11..14).contains(&x) && (8..11).contains(&y);
    let cuff = (4..11).contains(&x) && (12..14).contains(&y);
    if !((0..4).any(finger) || palm || thumb || cuff) {
        return dark();
    }
    if y >= 12 {
        let tint = if tier > 0 { FLAPS[tier - 1] } else { [148.0, 108.0, 68.0] };
        return rgb(tint, 0.95 + 0.15 * n(427, x, y));
    }
    if tier == 4 && (6..9).contains(&x) && (8..11).contains(&y) {
        return rgb([90.0, 226.0, 255.0], 1.1);
    }
    rgb([54.0, 58.0, 72.0], 0.85 + 0.2 * n(428, x, y))
}

/// Two round lenses glowing cyan in brass rims, joined by a bridge, on a dark strap running off both sides.
fn goggles(x: i32, y: i32) -> [u8; 4] {
    let lens = |cx: f64| ((x as f64 - cx).powi(2) + (y as f64 - 8.0).powi(2)).sqrt();
    let (l, r) = (lens(4.5), lens(11.5));
    let nearest = l.min(r);
    if nearest < 2.6 {
        // A glint in the upper left of each lens, the glass brighter at its middle.
        let glint = (x as f64 - if l < r { 3.5 } else { 10.5 }).abs() < 1.0 && y == 7;
        return if glint {
            rgb([235.0, 255.0, 255.0], 1.2)
        } else {
            rgb([90.0, 226.0, 255.0], 1.15 - 0.3 * nearest / 2.6)
        };
    }
    if nearest < 3.8 {
        return rgb([184.0, 140.0, 60.0], 0.9 + 0.2 * n(429, x, y));
    }
    if (7..=8).contains(&x) && (7..=8).contains(&y) {
        return rgb([184.0, 140.0, 60.0], 0.9);
    }
    if (7..10).contains(&y) && !(1..=14).contains(&x) {
        return rgb([44.0, 48.0, 60.0], 1.0);
    }
    dark()
}

/// The Tool Belt (`tier` 0..=4): a belt with a brass buckle across the top, its band in leather (Mk1) or the tier's kit
/// colour, and three heads hanging from it: a pick, an axe and a spade, stone grey on Mk1 and steel above, the middle
/// one a drill bit on Mk4 and a cyan lens on Mk5.
fn tool_belt(x: i32, y: i32, tier: usize) -> [u8; 4] {
    if (2..5).contains(&y) {
        if (7..10).contains(&x) {
            return rgb([226.0, 200.0, 90.0], 1.1);
        }
        let band = if tier > 0 { FLAPS[tier - 1] } else { [120.0, 86.0, 54.0] };
        return rgb(band, 0.9 + 0.15 * n(430, x, y));
    }
    let head = if tier == 0 { [128.0, 128.0, 132.0] } else { [196.0, 200.0, 210.0] };
    let wood = [110.0, 80.0, 50.0];
    let shade = 0.85 + 0.2 * n(431, x, y);
    // The pick: a crossbar head on a handle, at the left.
    if y == 6 && (1..6).contains(&x) {
        return rgb(head, shade);
    }
    if x == 3 && (5..13).contains(&y) {
        return rgb(wood, 1.0);
    }
    // The middle head: an axe blade, a drill bit on Mk4, a lens on Mk5.
    if (6..11).contains(&x) && (5..13).contains(&y) {
        return match tier {
            3 if (x - 8).abs() <= (12 - y) / 3 => rgb(head, if (x + y) % 3 == 0 { 0.7 } else { 1.0 }),
            4 if (7..10).contains(&x) && (7..10).contains(&y) => rgb([90.0, 226.0, 255.0], 1.15),
            4 if x == 8 => rgb(head, shade),
            0..=2 if y < 9 && x < 9 => rgb(head, shade),
            0..=2 if x == 9 => rgb(wood, 1.0),
            _ => dark(),
        };
    }
    // The spade: a handle and a square blade, at the right.
    if x == 13 && (5..9).contains(&y) {
        return rgb(wood, 1.0);
    }
    if (11..16).contains(&x) && (9..14).contains(&y) {
        return rgb(head, shade);
    }
    dark()
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
