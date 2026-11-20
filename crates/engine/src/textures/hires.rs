//! The plants on crossed transparent quads, painted at the texture array's full 64 × 64 (`brush.rs`): ground cover
//! (`mesher/cover.rs`), the indicator plants over stained ground and the three saplings. Calm, slightly muted colours
//! so a meadow reads as a whole. Every texel is opaque or fully transparent (the cutout pass tests alpha at one half).
//! To add a plant: its layer in `block::tex`, an arm in `covers` and `pixel`, and a function that draws it on a `Pen`.

use super::brush::{curve, rand, shade, Pen, Pt, Rgb};
use crate::block::tex;
use std::f64::consts::{FRAC_PI_2, PI};

const GREEN: Rgb = [74.0, 112.0, 70.0];
const STEM: Rgb = [86.0, 122.0, 64.0];
const BARK: Rgb = [83.0, 71.0, 54.0];

/// Whether `layer` is painted here, at full resolution, instead of at 16 × 16 and repeated.
pub fn covers(layer: u16) -> bool {
    matches!(layer, tex::SAPLING | tex::GRASS_TUFT..=tex::LICHEN | tex::BIRCH_SAPLING..=tex::INDICATOR_PALE)
        || matches!(layer, tex::PALM_SAPLING | tex::CHERRY_SAPLING | tex::BAOBAB_SAPLING | tex::ACACIA_SAPLING)
        || matches!(layer, tex::REDWOOD_SAPLING | tex::EUCALYPTUS_SAPLING | tex::BAMBOO_SHOOT)
}

/// Texel (`x`, `y`) of a layer `covers` says is painted here.
pub fn pixel(layer: u16, x: i32, y: i32) -> [u8; 4] {
    let mut pen = Pen::new(x, y, GREEN);
    match layer {
        tex::GRASS_TUFT => tuft(&mut pen),
        tex::DAISY => daisy(&mut pen),
        tex::POPPY => poppy(&mut pen),
        tex::BLUEBELL => bluebell(&mut pen),
        tex::FERN => fern(&mut pen),
        tex::REED => reed(&mut pen),
        tex::DRY_BRUSH => dry_brush(&mut pen),
        tex::LICHEN => lichen(&mut pen),
        tex::SAPLING => needle_sapling(&mut pen, [62.0, 105.0, 79.0]),
        tex::PINE_SAPLING => needle_sapling(&mut pen, [36.0, 74.0, 62.0]),
        tex::BIRCH_SAPLING => birch_sapling(&mut pen),
        tex::REDWOOD_SAPLING => needle_sapling(&mut pen, [44.0, 84.0, 56.0]),
        tex::PALM_SAPLING | tex::BAMBOO_SHOOT => needle_sapling(&mut pen, [92.0, 140.0, 66.0]),
        tex::CHERRY_SAPLING | tex::BAOBAB_SAPLING | tex::ACACIA_SAPLING | tex::EUCALYPTUS_SAPLING => {
            birch_sapling(&mut pen)
        }
        tex::INDICATOR_IRON => rust_bloom(&mut pen),
        tex::INDICATOR_COAL => coal_shrub(&mut pen),
        tex::INDICATOR_COPPER => copper_bell(&mut pen),
        tex::INDICATOR_PALE => pale_star(&mut pen),
        _ => unreachable!("not a full-resolution plant"),
    }
    pen.rgba()
}

/// A green that varies from blade to blade.
fn green(base: Rgb, seed: u32, i: i32) -> Rgb {
    shade(base, 0.9 + 0.2 * rand(seed, i))
}

/// Eleven slender curving blades of uneven height, standing on y = 0.
fn tuft(p: &mut Pen) {
    for i in 0..13 {
        let f = f64::from(i) / 12.0;
        let x0 = 0.12 + 0.76 * f + (rand(1, i) - 0.5) * 0.05;
        let height = 0.36 + 0.52 * rand(2, i) * (1.0 - (f - 0.5).abs() * 0.7);
        let lean = (f - 0.5) * 0.32 + (rand(3, i) - 0.5) * 0.14;
        p.blade(
            (x0, 0.0),
            ((x0 + lean).clamp(0.05, 0.95), height),
            -lean * 0.45,
            0.016 + 0.008 * rand(5, i),
            green(GREEN, 6, i),
        );
    }
}

/// Bottom-left stem (rooted at (0.25, 0.0) to (0.25, 0.46)) and bottom-right basal leaf tuft (around (0.75, 0.0))
/// for the split 2x2 wildflower layout (`mesher/cover/shape.rs`).
fn quad_stem_and_basal(p: &mut Pen, stem: Rgb, leaf: Rgb, bow: f64) {
    p.blade((0.25, 0.0), (0.25, 0.47), bow, 0.022, stem);
    p.ellipse((0.17, 0.14), 0.075, 0.024, 0.45, green(leaf, 20, 0));
    p.ellipse((0.33, 0.24), 0.07, 0.022, -0.5, green(leaf, 20, 1));
    for i in 0..6 {
        let f = (f64::from(i) / 5.0) * 2.0 - 1.0;
        p.blade(
            (0.75 + f * 0.04, 0.0),
            (0.75 + f * 0.18, 0.20 + (1.0 - f.abs() * 0.45) * 0.16),
            -f * 0.04,
            0.022,
            green(leaf, 21, i),
        );
    }
}

/// Split 2x2 white daisy: two top-down blossom variants in the upper quadrants, stem and basal tuft below.
fn daisy(p: &mut Pen) {
    quad_stem_and_basal(p, STEM, GREEN, 0.02);
    for (cx, n, reach, len, core) in [(0.25, 13, 0.105, 0.082, 0.052), (0.75, 11, 0.095, 0.075, 0.046)] {
        p.petals((cx, 0.75), n, reach, len, 0.022, 0.2, [232.0, 230.0, 218.0]);
        p.disc((cx, 0.75), core, [224.0, 182.0, 68.0]);
        p.disc((cx, 0.75), core * 0.48, [186.0, 136.0, 40.0]);
    }
}

/// Split 2x2 poppy: two top-down crimson blossom cups above, stem and basal rosette below.
fn poppy(p: &mut Pen) {
    quad_stem_and_basal(p, STEM, GREEN, -0.02);
    for (cx, r) in [(0.25, 0.092), (0.75, 0.086)] {
        for (i, (dx, dy)) in [(-0.07, 0.04), (0.07, 0.045), (-0.05, -0.06), (0.055, -0.055)].into_iter().enumerate() {
            p.disc((cx + dx, 0.75 + dy), r, shade([204.0, 58.0, 50.0], 0.90 + 0.06 * i as f64));
        }
        p.disc((cx, 0.75), 0.038, [44.0, 32.0, 38.0]);
    }
}

/// Graceful arching stem rooted at x = 0.22 with five small blue bells hanging outward (`mesher/cover/shape.rs`).
fn bluebell(p: &mut Pen) {
    let (from, to, bow) = ((0.22, 0.0), (0.84, 0.74), -0.34);
    p.blade((0.22, 0.0), (0.08, 0.44), 0.14, 0.02, green(GREEN, 30, 0));
    p.blade((0.22, 0.0), (0.44, 0.34), -0.14, 0.02, green(GREEN, 30, 1));
    p.blade(from, to, bow, 0.014, STEM);
    for (i, s) in [0.44, 0.58, 0.71, 0.83, 0.93].into_iter().enumerate() {
        let (x, y) = curve(from, to, bow, s);
        p.blade((x, y), (x + 0.025, y - 0.03), 0.0, 0.007, STEM);
        p.ellipse((x + 0.03, y - 0.075), 0.034, 0.052, 0.12, shade([104.0, 126.0, 208.0], 0.92 + 0.04 * i as f64));
        p.ellipse((x + 0.03, y - 0.118), 0.046, 0.016, 0.0, [146.0, 166.0, 232.0]);
    }
}

/// One symmetrical woodland fern frond rooted at (0.5, 0.0), bent into a 5-frond 3D rosette by `cover/shape.rs`.
fn fern(p: &mut Pen) {
    let from = (0.5, 0.0);
    let tip = (0.5, 0.95);
    p.blade(from, tip, 0.01, 0.016, [60.0, 104.0, 62.0]);
    for k in 1..=14 {
        let s = f64::from(k) / 15.0 * 0.92;
        let y = s * 0.95;
        let len = 0.22 * (PI * s.powf(0.75)).sin().powf(0.75) + 0.025;
        let wide = 0.026 * (1.0 - 0.5 * s);
        for side in [-1.0, 1.0] {
            let a = FRAC_PI_2 - side * (1.2 - 0.35 * s);
            let c = (0.5 + a.cos() * len, y + a.sin() * len * 0.8);
            p.ellipse(c, len, wide, a, shade([62.0, 110.0, 64.0], 0.88 + 0.24 * s + 0.06 * side));
        }
    }
}

/// Slender riverside sedge blades and two clean stalks crowned with centred brown cattails.
fn reed(p: &mut Pen) {
    const STALK: Rgb = [112.0, 132.0, 74.0];
    const HEAD: Rgb = [108.0, 74.0, 46.0];
    for (i, (x0, lean, h)) in
        [(0.24, -0.14, 0.56), (0.38, 0.14, 0.66), (0.52, -0.08, 0.72), (0.68, 0.14, 0.60), (0.78, -0.12, 0.48)]
            .into_iter()
            .enumerate()
    {
        p.blade((x0, 0.0), (x0 + lean, h), -lean * 0.3, 0.018, green(STALK, 50, i as i32));
    }
    for (i, (x, top)) in [(0.38, 0.92), (0.62, 0.80)].into_iter().enumerate() {
        p.blade((x, 0.0), (x, top), 0.01, 0.012, green(STALK, 51, i as i32));
        p.ellipse((x, top - 0.12), 0.028, 0.088, 0.0, shade(HEAD, 0.9 + 0.15 * rand(52, i as i32)));
    }
}

/// Dry twigs fanning from the ground, forking, bare and tan.
fn dry_brush(p: &mut Pen) {
    for i in 0..5 {
        let a = FRAC_PI_2 + (f64::from(i) - 2.0) * 0.42 + (rand(60, i) - 0.5) * 0.15;
        twig(p, (0.5 + (f64::from(i) - 2.0) * 0.03, 0.0), a, 0.46 + 0.14 * rand(61, i), 0, i);
    }
}

fn twig(p: &mut Pen, from: Pt, angle: f64, len: f64, depth: i32, i: i32) {
    let to = (from.0 + angle.cos() * len, from.1 + angle.sin() * len);
    p.blade(from, to, (rand(62, i + depth * 7) - 0.5) * 0.06, 0.014 - 0.004 * f64::from(depth), [156.0, 128.0, 84.0]);
    if depth < 2 {
        let mid = curve(from, to, 0.0, 0.55);
        twig(p, mid, angle + 0.6 + 0.2 * rand(63, i), len * 0.5, depth + 1, i * 2 + 1);
        twig(p, to, angle - 0.5 - 0.2 * rand(64, i), len * 0.4, depth + 1, i * 2 + 2);
    }
}

/// Top-down organic patch of pale foliose rock lichen with tiny amber fruiting cups (`mesher/cover/shape.rs`).
fn lichen(p: &mut Pen) {
    const BASE: Rgb = [140.0, 156.0, 108.0];
    for (i, (cx, cy, rx, ry)) in [
        (0.48, 0.52, 0.26, 0.21),
        (0.34, 0.42, 0.19, 0.16),
        (0.64, 0.58, 0.19, 0.15),
        (0.56, 0.36, 0.16, 0.13),
        (0.38, 0.66, 0.15, 0.13),
    ]
    .into_iter()
    .enumerate()
    {
        p.ellipse((cx, cy), rx, ry, rand(72, i as i32), shade(BASE, 0.88 + 0.22 * rand(74, i as i32)));
    }
    for i in 0..10 {
        let (a, r) = (rand(75, i) * PI * 2.0, 0.05 + 0.22 * rand(76, i));
        p.disc((0.5 + a.cos() * r, 0.5 + a.sin() * r), 0.018, [186.0, 146.0, 72.0]);
    }
}

/// A young conifer: a thin trunk with short twigs, each ending in a spray of needles.
fn needle_sapling(p: &mut Pen, needles: Rgb) {
    p.blade((0.5, 0.0), (0.52, 0.8), 0.02, 0.022, BARK);
    let twigs = [(0.2, -1.0), (0.3, 1.0), (0.42, -1.0), (0.54, 1.0), (0.66, -1.0)];
    for (i, (y, side)) in twigs.into_iter().enumerate() {
        let from = (0.5 + 0.02 * y, y);
        let tip = (from.0 + side * (0.26 - 0.2 * y), y + 0.07);
        p.blade(from, tip, side * 0.02, 0.01, BARK);
        spray(p, tip, if side > 0.0 { 0.3 } else { PI - 0.3 }, needles, i as i32);
    }
    spray(p, (0.52, 0.8), FRAC_PI_2, needles, 9);
}

/// A fan of fine needles from `at`, centred on direction `heading`.
fn spray(p: &mut Pen, at: Pt, heading: f64, col: Rgb, i: i32) {
    for k in -4i32..=4 {
        let a = heading + f64::from(k) * 0.3;
        let len = 0.085 - 0.006 * f64::from(k.abs());
        p.blade(at, (at.0 + a.cos() * len, at.1 + a.sin() * len), 0.0, 0.009, green(col, 80, i * 9 + k));
    }
}

/// A pale thin trunk with small round leaves on short twigs.
fn birch_sapling(p: &mut Pen) {
    const LEAF: Rgb = [96.0, 138.0, 70.0];
    p.blade((0.5, 0.0), (0.5, 0.86), 0.04, 0.018, [214.0, 208.0, 194.0]);
    for i in 0..9 {
        let y = 0.16 + 0.07 * f64::from(i);
        let side = if i % 2 == 0 { -1.0 } else { 1.0 };
        let from = (0.5 + 0.04 * (y / 0.86).powi(2), y);
        let tip = (from.0 + side * (0.12 + 0.05 * rand(90, i)), y + 0.05);
        p.blade(from, tip, 0.0, 0.008, [196.0, 190.0, 176.0]);
        p.ellipse((tip.0 + side * 0.03, tip.1 + 0.01), 0.05, 0.034, side * 0.4, green(LEAF, 91, i));
    }
    p.ellipse((0.53, 0.9), 0.045, 0.06, FRAC_PI_2, green(LEAF, 91, 20));
}

/// Split 2x2 iron-indicator rust bloom: two top-down burnt-orange 6-petal flowers above, stem and basal tuft below.
fn rust_bloom(p: &mut Pen) {
    quad_stem_and_basal(p, STEM, [84.0, 110.0, 62.0], 0.02);
    for (cx, reach, len) in [(0.25, 0.10, 0.082), (0.75, 0.092, 0.076)] {
        p.petals((cx, 0.75), 6, reach, len, 0.036, 0.15, [224.0, 116.0, 44.0]);
        p.disc((cx, 0.75), 0.046, [226.0, 174.0, 62.0]);
        p.disc((cx, 0.75), 0.022, [138.0, 56.0, 26.0]);
    }
}

/// Split 2x2 coal-indicator heath shrub: dark woody stems below, two slate-charcoal foliage & black-berry sprigs above.
fn coal_shrub(p: &mut Pen) {
    const LEAF: Rgb = [72.0, 86.0, 102.0];
    for ox in [0.25, 0.75] {
        p.blade((ox, 0.0), (ox, 0.42), 0.01, 0.02, [64.0, 50.0, 42.0]);
        p.blade((ox, 0.16), (ox - 0.14, 0.28), 0.02, 0.011, [64.0, 50.0, 42.0]);
        p.blade((ox, 0.24), (ox + 0.14, 0.36), -0.02, 0.011, [64.0, 50.0, 42.0]);
    }
    for (q, cx) in [0.25, 0.75].into_iter().enumerate() {
        for i in 0..24 {
            let (a, r) = (rand(110 + q as u32, i) * PI * 2.0, rand(111 + q as u32, i).sqrt() * 0.15);
            let c = (cx + a.cos() * r, 0.75 + a.sin() * r);
            p.ellipse(c, 0.055, 0.024, a + 0.3, shade(LEAF, 0.62 + 0.50 * rand(113, i)));
        }
        for i in 0..6 {
            let (a, r) = (rand(114 + q as u32, i) * PI * 2.0, 0.04 + 0.10 * rand(115, i));
            p.disc((cx + a.cos() * r, 0.75 + a.sin() * r), 0.02, [28.0, 26.0, 38.0]);
        }
    }
}

/// Copper-indicator arching spire rooted at x = 0.24 with five hanging verdigris-teal bells.
fn copper_bell(p: &mut Pen) {
    let (from, to, bow) = ((0.24, 0.0), (0.78, 0.88), -0.24);
    p.blade((0.24, 0.0), (0.08, 0.44), 0.12, 0.022, green(GREEN, 120, 0));
    p.blade((0.24, 0.0), (0.46, 0.34), -0.12, 0.022, green(GREEN, 120, 1));
    p.blade(from, to, bow, 0.014, STEM);
    for (i, s) in [0.40, 0.53, 0.66, 0.79, 0.91].into_iter().enumerate() {
        let (x, y) = curve(from, to, bow, s);
        p.blade((x, y), (x + 0.032, y - 0.025), 0.0, 0.007, STEM);
        p.ellipse(
            (x + 0.038, y - 0.07),
            0.032,
            0.048,
            0.1,
            shade([62.0, 174.0, 156.0], 0.9 + 0.18 * rand(121, i as i32)),
        );
        p.ellipse((x + 0.038, y - 0.11), 0.042, 0.014, 0.0, [118.0, 214.0, 198.0]);
    }
}

/// Split 2x2 limestone/quartz-indicator pale star: two 5-pointed pale-lavender-white stars above, stem and tuft below.
fn pale_star(p: &mut Pen) {
    quad_stem_and_basal(p, [94.0, 126.0, 88.0], [88.0, 118.0, 84.0], -0.02);
    for (cx, reach, len) in [(0.25, 0.105, 0.088), (0.75, 0.098, 0.082)] {
        p.petals((cx, 0.75), 5, reach, len, 0.026, FRAC_PI_2, [224.0, 222.0, 240.0]);
        p.disc((cx, 0.75), 0.034, [232.0, 206.0, 114.0]);
    }
}
