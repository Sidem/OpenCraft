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

/// Eleven blades of uneven height, the middle ones tallest, a few leaning.
fn tuft(p: &mut Pen) {
    for i in 0..11 {
        let f = f64::from(i) / 10.0;
        let x0 = 0.14 + 0.72 * f + (rand(1, i) - 0.5) * 0.06;
        let height = 0.2 + 0.4 * rand(2, i) * (1.0 - (f - 0.5).abs() * 1.2);
        let lean = (rand(3, i) - 0.5) * 0.3;
        p.blade(
            (x0, 0.0),
            (x0 + lean, height),
            (rand(4, i) - 0.5) * 0.14,
            0.024 + 0.014 * rand(5, i),
            green(GREEN, 6, i),
        );
    }
}

fn stem_and_leaves(p: &mut Pen, top: Pt, bow: f64) {
    p.blade((0.5, 0.0), top, bow, 0.016, STEM);
    p.ellipse((0.41, 0.17), 0.1, 0.024, 0.45, green(GREEN, 20, 0));
    p.ellipse((0.6, 0.27), 0.09, 0.022, -0.5, green(GREEN, 20, 1));
}

/// A white daisy: thirteen petals round a yellow heart on a stem with two leaves.
fn daisy(p: &mut Pen) {
    stem_and_leaves(p, (0.54, 0.62), 0.05);
    p.petals((0.54, 0.65), 13, 0.075, 0.062, 0.02, 0.2, [238.0, 236.0, 222.0]);
    p.disc((0.54, 0.65), 0.044, [226.0, 186.0, 70.0]);
    p.disc((0.54, 0.65), 0.022, [190.0, 140.0, 44.0]);
}

/// A poppy: four overlapping red petals round a dark heart.
fn poppy(p: &mut Pen) {
    stem_and_leaves(p, (0.52, 0.6), -0.05);
    for (i, (dx, dy)) in [(-0.05, 0.03), (0.05, 0.035), (-0.035, -0.04), (0.04, -0.035)].into_iter().enumerate() {
        p.disc((0.52 + dx, 0.64 + dy), 0.065, shade([196.0, 64.0, 56.0], 0.92 + 0.08 * i as f64));
    }
    p.disc((0.52, 0.64), 0.026, [52.0, 40.0, 40.0]);
}

/// Four small bells hanging from an arched stem.
fn bluebell(p: &mut Pen) {
    let (from, to, bow) = ((0.34, 0.0), (0.68, 0.66), -0.3);
    p.blade(from, to, bow, 0.016, STEM);
    p.ellipse((0.36, 0.1), 0.09, 0.022, 0.2, green(GREEN, 30, 0));
    for (i, s) in [0.42, 0.58, 0.74, 0.9].into_iter().enumerate() {
        let (x, y) = curve(from, to, bow, s);
        p.blade((x, y), (x + 0.03, y - 0.03), 0.0, 0.008, STEM);
        p.ellipse((x + 0.035, y - 0.075), 0.034, 0.052, 0.1, shade([108.0, 128.0, 206.0], 0.92 + 0.05 * i as f64));
        p.ellipse((x + 0.035, y - 0.118), 0.047, 0.016, 0.0, [146.0, 162.0, 228.0]);
    }
}

/// Two arching fronds: a midrib with pairs of leaflets that shorten towards the tip.
fn fern(p: &mut Pen) {
    for (f, (tip, bow)) in [((0.3, 0.9), 0.18), ((0.74, 0.84), -0.16), ((0.52, 0.97), 0.02)].into_iter().enumerate() {
        let from = (0.5, 0.0);
        p.blade(from, tip, bow, 0.014, [60.0, 104.0, 62.0]);
        for k in 1..=13 {
            let s = f64::from(k) / 14.0 * 0.95;
            let at = curve(from, tip, bow, s);
            let ahead = curve(from, tip, bow, s + 0.02);
            let along = (ahead.1 - at.1).atan2(ahead.0 - at.0);
            let len = 0.13 * (1.0 - s).powf(0.7) + 0.03;
            for side in [-1.0, 1.0] {
                let a = along + side * (PI / 3.0);
                let c = (at.0 + a.cos() * len * 0.9, at.1 + a.sin() * len * 0.9);
                p.ellipse(c, len, 0.017, a, shade([60.0, 104.0, 62.0], 0.9 + 0.2 * rand(40 + f as u32, k)));
            }
        }
    }
}

/// Tall thin stalks, the tallest crowned with a brown seed head, and two long leaves.
fn reed(p: &mut Pen) {
    const STALK: Rgb = [112.0, 132.0, 74.0];
    const HEAD: Rgb = [112.0, 80.0, 52.0];
    p.blade((0.4, 0.0), (0.1, 0.52), 0.14, 0.034, green(STALK, 50, 0));
    p.blade((0.6, 0.0), (0.9, 0.44), -0.12, 0.034, green(STALK, 50, 1));
    for (i, (x, top, lean)) in
        [(0.22, 0.62, -0.03), (0.4, 0.97, 0.02), (0.55, 0.74, 0.04), (0.72, 0.84, -0.02), (0.84, 0.5, 0.05)]
            .into_iter()
            .enumerate()
    {
        p.blade((x, 0.0), (x + lean, top), 0.03 * (i as f64 - 2.0) * 0.3, 0.014, green(STALK, 51, i as i32));
        if top > 0.7 {
            p.ellipse((x + lean, top - 0.06), 0.022, 0.075, 0.0, shade(HEAD, 0.9 + 0.2 * rand(52, i as i32)));
        }
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

/// Low flat patches of pale lichen, grey-green with warmer flecks.
fn lichen(p: &mut Pen) {
    const BASE: Rgb = [140.0, 156.0, 108.0];
    for i in 0..16 {
        let x = 0.08 + 0.84 * rand(70, i);
        let h = 0.04 + 0.14 * rand(71, i) * (1.0 - (x - 0.5).abs() * 1.4).max(0.2);
        p.ellipse(
            (x, h),
            0.07 + 0.08 * rand(72, i),
            0.035 + 0.04 * rand(73, i),
            0.0,
            shade(BASE, 0.85 + 0.25 * rand(74, i)),
        );
    }
    for i in 0..9 {
        p.disc((0.1 + 0.8 * rand(75, i), 0.03 + 0.1 * rand(76, i)), 0.012 + 0.01 * rand(77, i), [182.0, 150.0, 82.0]);
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

/// A rosette of leaves with three rust-orange blooms on short stems.
fn rust_bloom(p: &mut Pen) {
    const LEAF: Rgb = [88.0, 112.0, 64.0];
    for i in 0..9 {
        let a = FRAC_PI_2 + (f64::from(i) - 4.0) * 0.36;
        let len = 0.26 + 0.1 * rand(100, i);
        p.blade(
            (0.5, 0.0),
            (0.5 + a.cos() * len, a.sin() * len),
            (rand(101, i) - 0.5) * 0.1,
            0.05,
            green(LEAF, 102, i),
        );
    }
    for (x, y) in [(0.28, 0.4), (0.5, 0.62), (0.72, 0.4)] {
        p.blade((0.5, 0.05), (x, y - 0.02), 0.0, 0.012, STEM);
        p.petals((x, y), 7, 0.04, 0.036, 0.016, 0.1, [220.0, 118.0, 48.0]);
        p.disc((x, y), 0.026, [150.0, 66.0, 30.0]);
    }
}

/// A round bush of dark purple-green leaves with a few black berries.
fn coal_shrub(p: &mut Pen) {
    const LEAF: Rgb = [100.0, 108.0, 118.0];
    p.blade((0.5, 0.0), (0.5, 0.2), 0.0, 0.03, [58.0, 46.0, 40.0]);
    for i in 0..80 {
        let (a, r) = (rand(110, i) * PI * 2.0, rand(111, i).sqrt());
        let c = (0.5 + a.cos() * r * 0.4, 0.4 + a.sin() * r * 0.3);
        p.ellipse(c, 0.07, 0.04, rand(112, i) * PI, shade(LEAF, 0.62 + 0.42 * rand(113, i)));
    }
    for i in 0..10 {
        let (a, r) = (rand(114, i) * PI * 2.0, 0.3 + 0.7 * rand(115, i));
        p.disc((0.5 + a.cos() * r * 0.36, 0.4 + a.sin() * r * 0.27), 0.02, [34.0, 28.0, 40.0]);
    }
}

/// A tall spike with teal bells hanging up its length.
fn copper_bell(p: &mut Pen) {
    p.blade((0.5, 0.0), (0.5, 0.94), 0.0, 0.016, STEM);
    p.blade((0.5, 0.0), (0.25, 0.3), 0.1, 0.04, green(GREEN, 120, 0));
    p.blade((0.5, 0.0), (0.76, 0.26), -0.1, 0.04, green(GREEN, 120, 1));
    for i in 0..6 {
        let y = 0.24 + 0.115 * f64::from(i);
        let side = if i % 2 == 0 { -1.0 } else { 1.0 };
        p.blade((0.5, y), (0.5 + side * 0.07, y - 0.03), 0.0, 0.008, STEM);
        p.ellipse(
            (0.5 + side * 0.085, y - 0.065),
            0.034,
            0.048,
            0.0,
            shade([62.0, 172.0, 152.0], 0.9 + 0.2 * rand(121, i)),
        );
        p.ellipse((0.5 + side * 0.085, y - 0.105), 0.044, 0.014, 0.0, [110.0, 210.0, 190.0]);
    }
}

/// Thin stems each crowned with a small pale star.
fn pale_star(p: &mut Pen) {
    for (i, (x, top)) in [(0.24, 0.5), (0.5, 0.86), (0.76, 0.62)].into_iter().enumerate() {
        p.blade((x, 0.0), (x + 0.02, top), 0.03, 0.013, green(GREEN, 130, i as i32));
        p.ellipse((x - 0.06, 0.12 + 0.03 * i as f64), 0.07, 0.016, 0.4, green(GREEN, 131, i as i32));
        p.petals((x + 0.02, top + 0.03), 5, 0.048, 0.05, 0.014, FRAC_PI_2, [226.0, 220.0, 244.0]);
        p.disc((x + 0.02, top + 0.03), 0.022, [236.0, 214.0, 120.0]);
    }
}
