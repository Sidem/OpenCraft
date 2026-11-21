//! Seismic Thumper procedural textures (F-SEIS): brushed slate-steel geophysical housing with recessed louvres and
//! amber telemetry stripe, concentric machined transducer deck, and heavy forged chevron-banded hydraulic drop-weight.
//! Placeholder looks until the art pass (`docs/ART_HANDOVER.md`).
//! To add a seismic texture: append a layer in `block/tex.rs` and an arm in `pixel`.

use crate::block::tex;

use super::{n, rgb};

fn shade(base: [f64; 3], delta: f64) -> [u8; 4] {
    rgb([base[0] + delta, base[1] + delta, base[2] + delta], 1.0)
}

pub fn pixel(layer: u16, x: i32, y: i32) -> [u8; 4] {
    match layer {
        tex::THUMPER_SIDE => side(x, y),
        tex::THUMPER_TOP => top(x, y),
        _ => piston(x, y),
    }
}

/// Brushed gunmetal-slate geophysical housing with beveled steel seams, precision hex bolts, amber telemetry bar and
/// recessed acoustic cooling louvres.
fn side(x: i32, y: i32) -> [u8; 4] {
    let grain = (n(710, x, y) - 0.5) * 10.0;
    // Beveled outer steel frame (bright top/left highlight, dark bottom/right shadow).
    if x == 0 || y == 0 {
        return shade([104.0, 114.0, 128.0], grain * 0.5);
    }
    if x == 15 || y == 15 {
        return shade([38.0, 44.0, 54.0], grain * 0.5);
    }
    // Corner hex bolts.
    if (x == 2 || x == 13) && (y == 2 || y == 13) {
        return shade([182.0, 192.0, 204.0], grain * 0.3);
    }
    // Crisp amber technical telemetry band with fine calibration ticks.
    if y == 4 || y == 5 {
        let tick = x % 4 == 0 && y == 4;
        return if tick { shade([255.0, 206.0, 118.0], grain * 0.3) } else { shade([228.0, 134.0, 46.0], grain * 0.5) };
    }
    // Dark recessed louvre grille panel.
    if (3..=12).contains(&x) && (8..=12).contains(&y) {
        return if y % 2 == 0 {
            shade([24.0, 30.0, 38.0], grain * 0.3)
        } else {
            shade([74.0, 88.0, 102.0], grain * 0.4)
        };
    }
    // Subtle vertical brushed-metal sheen across the steel plate.
    let brush = (n(713, x, 0) - 0.5) * 8.0;
    shade([64.0, 76.0, 90.0], grain + brush)
}

/// Machined concentric transducer deck with bolted corner pads, brushed steel field, brass inner collar and cyan
/// geophone pickup ring.
fn top(x: i32, y: i32) -> [u8; 4] {
    let dx = x as f64 - 7.5;
    let dy = y as f64 - 7.5;
    let r = (dx * dx + dy * dy).sqrt();
    let grain = (n(711, x, y) - 0.5) * 10.0;
    if r < 2.2 {
        return shade([28.0, 36.0, 46.0], grain * 0.4);
    }
    if r < 3.3 {
        return shade([216.0, 174.0, 84.0], grain * 0.4);
    }
    if r > 5.0 && r < 5.9 {
        return shade([78.0, 174.0, 214.0], grain * 0.4);
    }
    if x == 0 || y == 0 {
        return shade([112.0, 122.0, 134.0], grain * 0.4);
    }
    if x == 15 || y == 15 {
        return shade([44.0, 52.0, 62.0], grain * 0.4);
    }
    if (x == 2 || x == 13) && (y == 2 || y == 13) {
        return shade([178.0, 186.0, 198.0], grain * 0.3);
    }
    let radial = ((dx.abs() - dy.abs()).abs() * 1.4).min(8.0);
    shade([82.0 + radial * 0.5, 94.0 + radial * 0.5, 108.0 + radial * 0.5], grain)
}

/// Heavy forged hydraulic drop-weight with machined steel retention rims, vertical guide flutes and industrial
/// charcoal/amber hazard chevrons.
fn piston(x: i32, y: i32) -> [u8; 4] {
    let grain = (n(712, x, y) - 0.5) * 9.0;
    // Top & bottom machined steel retention flanges.
    if y <= 2 || y >= 13 {
        let rim = if y == 0 || y == 13 {
            20.0
        } else if y == 2 || y == 15 {
            -22.0
        } else {
            0.0
        };
        return shade([126.0 + rim, 134.0 + rim, 146.0 + rim], grain * 0.5);
    }
    // Thin dark oil-seal grooves separating the flanges from the chevron body.
    if y == 3 || y == 12 {
        return shade([28.0, 32.0, 40.0], grain * 0.3);
    }
    // Crisp high-contrast industrial diagonal hazard bands.
    let stripe = (x + y).rem_euclid(6) < 3;
    if stripe {
        shade([238.0, 148.0, 48.0], grain * 0.5)
    } else {
        shade([36.0, 42.0, 50.0], grain * 0.4)
    }
}
