//! Tool pictures, shown where an item is drawn as a plain box (on belts): a wooden handle on the
//! diagonal and a head in the tier's colour, shaped by kind (0 pickaxe, 1 axe, 2 shovel), on a dark
//! card. Icons and loose items use the box models in `item_models` instead, for which the two
//! prospecting layers are materials: the scanner's screen and the burnt-orange device casing.

use super::{n, rgb};

/// Head colours by tier.
pub const STONE_HEAD: [f64; 3] = [124.0, 124.0, 130.0];
pub const IRON_HEAD: [f64; 3] = [204.0, 208.0, 216.0];
pub const STEEL_HEAD: [f64; 3] = [104.0, 124.0, 158.0];

pub fn tool(x: i32, y: i32, kind: u16, head_colour: [f64; 3]) -> [u8; 4] {
    // Along and across the handle, which runs from the bottom left to the head at the top right.
    let (rx, ry) = (x as f64 - 11.5, y as f64 - 3.5);
    let along = (rx - ry) * std::f64::consts::FRAC_1_SQRT_2;
    let across = (rx + ry) * std::f64::consts::FRAC_1_SQRT_2;
    let head = match kind {
        0 => along.abs() < 1.3 && across.abs() < 5.5,
        1 => along.abs() < 1.8 && (0.0..4.0).contains(&across),
        _ => (-1.5..3.0).contains(&along) && across.abs() < 2.3,
    };
    let k = 0.88 + 0.12 * n(70 + kind as u32, x, y);
    if head {
        rgb(head_colour, k)
    } else if across.abs() < 0.8 && (-12.0..0.0).contains(&along) {
        rgb([140.0, 98.0, 58.0], k)
    } else {
        rgb([44.0, 48.0, 58.0], 1.0)
    }
}

/// The scanner's screen: a dark bezel around a green sweep rising from its lower edge.
pub fn scanner(x: i32, y: i32) -> [u8; 4] {
    if x == 0 || y == 0 || x == 15 || y == 15 {
        return rgb([58.0, 60.0, 66.0], 1.0);
    }
    let r = ((x as f64 - 7.5).powi(2) + (y as f64 - 15.0).powi(2)).sqrt();
    let ring = r % 4.0 < 1.0;
    let blip = (x == 5 || x == 6) && (y == 6 || y == 7);
    rgb(
        if blip {
            [226.0, 246.0, 200.0]
        } else if ring {
            [118.0, 214.0, 140.0]
        } else {
            [22.0, 52.0, 36.0]
        },
        1.0,
    )
}

/// Device casing: burnt-orange shell with ivory seams and corner screws.
pub fn core_drill(x: i32, y: i32) -> [u8; 4] {
    let k = 0.92 + 0.08 * n(81, x, y);
    if (x == 2 || x == 13) && (y == 4 || y == 11) {
        rgb([64.0, 58.0, 54.0], 1.0)
    } else if y == 1 || y == 14 {
        rgb([226.0, 222.0, 208.0], k)
    } else {
        rgb([196.0, 108.0, 52.0], k)
    }
}
