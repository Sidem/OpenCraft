//! Procedural textures for the Telescoping Bucket-Wheel Site Grader (`factory/process/grader/`):
//! - `GRADER_YELLOW`: weathered industrial ochre-yellow heavy steel paneling with beveled seams, rivets & bottom stripe.
//! - `GRADER_LOUVRE`: heavy diesel engine housing with dark recessed cooling louvres and brushed steel fin highlights.
//! - `GRADER_TREAD`: oiled manganese-steel crawler track grouser cleats and central track-pin bolts.
//! - `GRADER_TRUSS`: gunmetal boom backing framed by riveted yellow chord flanges and diagonal steel X-bracing.
//! - `GRADER_BUCKET`: scuffed forged burnt-orange digging bucket steel with charcoal/ochre hazard trim.

use crate::block::tex;

use super::{n, rgb};

fn shade(base: [f64; 3], delta: f64) -> [u8; 4] {
    rgb([base[0] + delta, base[1] + delta, base[2] + delta], 1.0)
}

pub fn pixel(layer: u16, x: i32, y: i32) -> [u8; 4] {
    match layer {
        tex::GRADER_YELLOW => hull_yellow(x, y),
        tex::GRADER_LOUVRE => engine_louvre(x, y),
        tex::GRADER_TREAD => crawler_tread(x, y),
        tex::GRADER_TRUSS => boom_truss(x, y),
        _ => bucket_steel(x, y),
    }
}

/// Weathered heavy-equipment ochre-yellow steel plating: warm muted gold (never neon), beveled panel edges,
/// four corner rivets, subtle brushed metal grain, and a crisp charcoal lower wear skirt.
fn hull_yellow(x: i32, y: i32) -> [u8; 4] {
    let grain = (n(820, x, y) - 0.5) * 11.0;
    let brush = (n(821, 0, y) - 0.5) * 6.0;
    // Outer panel bevel (highlight top/left, recessed shadow bottom/right).
    if x == 0 || y == 0 {
        return shade([234.0, 186.0, 68.0], grain * 0.4);
    }
    if x == 15 || y == 15 {
        return shade([118.0, 82.0, 18.0], grain * 0.4);
    }
    // Dark charcoal lower chassis rub-rail with subtle hazard ticks.
    if y >= 13 {
        let hazard = y == 13 && (x % 4 < 2);
        return if hazard { shade([208.0, 152.0, 34.0], grain * 0.4) } else { shade([42.0, 46.0, 52.0], grain * 0.5) };
    }
    // Corner rivets.
    if (x == 2 || x == 13) && (y == 2 || y == 11) {
        return shade([144.0, 108.0, 36.0], grain * 0.3);
    }
    // Subtle vertical inspection seam down the center.
    if x == 8 && (2..=11).contains(&y) {
        return shade([168.0, 122.0, 26.0], grain * 0.4);
    }
    shade([208.0, 154.0, 36.0], grain + brush)
}

/// Diesel engine cowling: ochre-yellow riveted housing with dark recessed radiator louvres and brushed steel fins.
fn engine_louvre(x: i32, y: i32) -> [u8; 4] {
    let grain = (n(822, x, y) - 0.5) * 10.0;
    if x == 0 || y == 0 {
        return shade([230.0, 182.0, 64.0], grain * 0.4);
    }
    if x == 15 || y == 15 {
        return shade([108.0, 76.0, 18.0], grain * 0.4);
    }
    // Recessed dark steel radiator frame & horizontal cooling slats.
    if (2..=13).contains(&x) && (3..=12).contains(&y) {
        if x == 2 || x == 13 || y == 3 || y == 12 {
            return shade([48.0, 54.0, 62.0], grain * 0.4);
        }
        return if y % 2 == 0 {
            shade([24.0, 28.0, 34.0], grain * 0.3)
        } else {
            shade([78.0, 86.0, 96.0], grain * 0.4)
        };
    }
    // Corner rivets in the yellow frame.
    if (x == 1 || x == 14) && (y == 1 || y == 14) {
        return shade([148.0, 110.0, 34.0], grain * 0.3);
    }
    shade([202.0, 148.0, 34.0], grain)
}

/// Heavy forged manganese-steel crawler track shoes with raised horizontal grouser cleats and twin track-pin bolts.
fn crawler_tread(x: i32, y: i32) -> [u8; 4] {
    let grain = (n(823, x, y) - 0.5) * 10.0;
    let row = y.rem_euclid(4);
    // Outer steel side links of the track chain.
    if x <= 1 || x >= 14 {
        return if row == 0 { shade([30.0, 34.0, 40.0], grain * 0.4) } else { shade([62.0, 68.0, 76.0], grain * 0.5) };
    }
    // Deep shadow gap between track shoes.
    if row == 0 {
        return shade([22.0, 25.0, 30.0], grain * 0.3);
    }
    // Bright worn steel ridge on top of each grouser cleat.
    if row == 1 {
        return shade([98.0, 104.0, 114.0], grain * 0.5);
    }
    // Twin forged track-pin bolt heads along the shoe face.
    if row == 2 && (x == 5 || x == 10) {
        return shade([126.0, 132.0, 142.0], grain * 0.3);
    }
    // Oiled charcoal track shoe plate.
    shade([48.0, 52.0, 58.0], grain)
}

/// Heavy telescoping box-girder truss: ochre-yellow top/bottom chord flanges over brushed gunmetal X-bracing.
fn boom_truss(x: i32, y: i32) -> [u8; 4] {
    let grain = (n(824, x, y) - 0.5) * 9.0;
    // Heavy ochre-yellow outer chord flanges along left/right edges (which wrap along beam sides).
    if x <= 2 || x >= 13 {
        if x == 0 || x == 13 {
            return shade([232.0, 184.0, 62.0], grain * 0.4);
        }
        if x == 2 || x == 15 {
            return shade([124.0, 88.0, 20.0], grain * 0.4);
        }
        // Rivets along flange centerline.
        if y % 4 == 1 {
            return shade([138.0, 98.0, 26.0], grain * 0.3);
        }
        return shade([206.0, 152.0, 34.0], grain);
    }
    // Cross-ties and diagonal X-truss webs over dark recessed steel interior.
    let diag_a = ((x - 3) - (y.rem_euclid(10))).abs() <= 1;
    let diag_b = ((12 - x) - (y.rem_euclid(10))).abs() <= 1;
    let cross_tie = y % 8 == 0;
    if cross_tie || diag_a || diag_b {
        return shade([104.0, 112.0, 124.0], grain * 0.6);
    }
    shade([36.0, 42.0, 50.0], grain * 0.5)
}

/// Scuffed forged burnt-orange bucket-wheel steel with dark beveled weld seams and subtle industrial hazard trim.
fn bucket_steel(x: i32, y: i32) -> [u8; 4] {
    let grain = (n(825, x, y) - 0.5) * 12.0;
    // Worn steel cutting bevel along top & sides.
    if y == 0 {
        return shade([176.0, 182.0, 190.0], grain * 0.4);
    }
    if x == 0 || x == 15 || y == 15 {
        return shade([96.0, 42.0, 18.0], grain * 0.4);
    }
    // Crisp charcoal/amber industrial warning band near the base (great on counterweight trim & wheel buckets).
    if y >= 12 {
        let stripe = (x + y).rem_euclid(6) < 3;
        return if stripe { shade([224.0, 156.0, 38.0], grain * 0.4) } else { shade([38.0, 42.0, 48.0], grain * 0.4) };
    }
    // Corner rivets.
    if (x == 2 || x == 13) && (y == 2 || y == 10) {
        return shade([146.0, 62.0, 24.0], grain * 0.3);
    }
    shade([208.0, 88.0, 32.0], grain)
}
