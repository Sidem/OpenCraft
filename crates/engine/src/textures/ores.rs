//! Ores and ore-bearing rock, each in four looks (`alt` 0..=3: the layer and its three alternates,
//! which the mesher picks per block), so a seam never shows the same tile twice in a row.
//!
//! Inclusions are drawn shapes (`paint::blob`) set in the slate host (`nature::stone`) and kept inside
//! the tile, so faces still tile. Ores read by value as well as hue: coal near black and angular,
//! iron mid-value rusty nodules with a warm stain around them, copper banded green crusts with bright
//! glints, quartz near-white crystals, limestone pale overall with shell fossils.

use super::nature::stone;
use super::paint::{blob, mix, mottle, tone, Ramp};
use super::{n, smooth};

const COAL: Ramp = [[20, 22, 26], [30, 33, 38], [42, 46, 52], [60, 65, 72], [98, 104, 114]];
const IRON: Ramp = [[72, 38, 30], [98, 52, 38], [124, 70, 48], [150, 92, 62], [180, 126, 92]];
const MALACHITE: Ramp = [[28, 78, 60], [40, 104, 76], [56, 130, 92], [84, 158, 112], [130, 190, 146]];
const QUARTZ: Ramp = [[146, 152, 160], [182, 187, 194], [207, 211, 217], [227, 229, 233], [246, 247, 250]];
const LIME: Ramp = [[160, 156, 140], [180, 176, 160], [197, 194, 178], [211, 208, 193], [226, 224, 211]];

/// Black angular lumps made of flat facets (one catching the light, with a sheen texel), a sooty
/// rim, and loose grit in the rock.
pub fn coal(x: i32, y: i32, alt: u32) -> [u8; 4] {
    match blob(300, alt, 5, (1.2, 2.4), 1.25, x, y) {
        Some(s) if s.d > 1.0 => mix(stone(x, y), [40, 44, 50], 0.35),
        Some(s) if s.d > 0.8 => tone(&COAL, 0.05),
        Some(s) => {
            let lit = if s.dx + s.dy * 0.6 < -0.15 {
                0.55
            } else if s.dx > 0.25 {
                0.1
            } else {
                0.3
            };
            if lit > 0.5 && s.d < 0.6 && n(301 + alt, x, y) > 0.85 {
                tone(&COAL, 0.95)
            } else {
                tone(&COAL, lit + (n(302, x, y) - 0.5) * 0.15)
            }
        }
        None if n(303 + alt, x, y) > 0.96 => tone(&COAL, 0.25),
        None => stone(x, y),
    }
}

/// Rusty nodules shaded like little spheres lit from the upper left, with a warm stain soaking into
/// the rock around each.
pub fn iron(x: i32, y: i32, alt: u32) -> [u8; 4] {
    match blob(310, alt, 6, (0.9, 2.0), 1.6, x, y) {
        Some(s) if s.d > 1.0 => mix(stone(x, y), [146, 96, 70], 0.4 * (1.6 - s.d) / 0.6),
        Some(s) => {
            if s.dx < -0.15 && s.dy < -0.15 && s.d < 0.7 && n(311 + alt, x, y) > 0.4 {
                tone(&IRON, 0.95)
            } else {
                tone(&IRON, 0.55 - (s.dx + s.dy) * 0.3 - s.d * 0.2)
            }
        }
        None => stone(x, y),
    }
}

/// Malachite crusts with concentric growth bands, thin green veinlets, and bright glints of native
/// copper in both and now and then in the rock.
pub fn copper(x: i32, y: i32, alt: u32) -> [u8; 4] {
    let glint = |x: i32, y: i32| if n(324, x, y) > 0.5 { [246, 190, 120, 255] } else { [206, 124, 66, 255] };
    if let Some(s) = blob(320, alt, 4, (1.3, 2.3), 1.0, x, y) {
        if n(321 + alt, x, y) > 0.95 {
            return glint(x, y);
        }
        let band = if (s.d * 3.0) as i32 % 2 == 0 { 0.72 } else { 0.35 };
        return tone(&MALACHITE, band - s.dy * 0.12);
    }
    let vein = (smooth(322 + alt * 3, x, y, 4) - 0.5).abs() < 0.035;
    if vein && n(323 + alt, x, y) > 0.9 {
        glint(x, y)
    } else if vein {
        tone(&MALACHITE, 0.15)
    } else if n(325 + alt, x, y) > 0.992 {
        glint(x, y)
    } else {
        stone(x, y)
    }
}

/// Milky quartz crystals: a lit and a shaded facet split along a slanted edge, a dark grey rim.
pub fn quartz(x: i32, y: i32, alt: u32) -> [u8; 4] {
    match blob(330, alt, 4, (1.2, 2.4), 1.0, x, y) {
        Some(s) if s.d > 0.85 => tone(&QUARTZ, 0.05),
        Some(s) => {
            let lit = if s.dx - s.dy * 0.5 < 0.0 { 0.85 } else { 0.45 };
            tone(&QUARTZ, lit + (n(331, x, y) - 0.5) * 0.15)
        }
        None => stone(x, y),
    }
}

/// Pale carbonate with faint bent bedding, and ribbed shell fossils (a half disc outlined above,
/// ribs inside) placed differently in every look.
pub fn limestone(x: i32, y: i32, alt: u32) -> [u8; 4] {
    let t = 0.35 + mottle(230, x, y, 0.2) * 0.4;
    let bedding = (y + (smooth(231, x, 0, 4) * 2.0) as i32).rem_euclid(8) == 0;
    let base = tone(&LIME, if bedding { t - 0.18 } else { t });
    match blob(232, alt, 2, (1.8, 2.6), 1.0, x, y) {
        Some(s) if s.dy > 0.35 => base,
        Some(s) if s.d > 0.78 => tone(&LIME, 0.0),
        Some(s) if ((s.dx * 3.0).round() as i32).rem_euclid(2) == 0 => tone(&LIME, 0.2),
        Some(_) => tone(&LIME, 0.8),
        None => base,
    }
}
