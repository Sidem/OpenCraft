//! Shared painting helpers for natural textures: palette ramps (a few hand-picked tones per material,
//! so textures read as pixel art rather than noise), a wrapping cell pattern (plates, leaves,
//! grains), and blobs placed inside the tile (ore inclusions, pebbles) that differ per alternate.
//!
//! Everything wraps at the 16-texel edge or stays inside it, so tiles repeat without seams.

use super::{n, smooth};
use crate::math::{hash3, unit};

/// Five tones from dark to light.
pub type Ramp = [[u8; 3]; 5];

/// The tone of `ramp` at `t` (0 dark .. 1 light, clamped).
pub fn tone(ramp: &Ramp, t: f64) -> [u8; 4] {
    let i = (t.clamp(0.0, 0.999) * 5.0) as usize;
    let c = ramp[i];
    [c[0], c[1], c[2], 255]
}

/// `ramp` with every tone moved by `d` (a hint's shift in colour).
pub fn shifted(ramp: &Ramp, d: [i16; 3]) -> Ramp {
    ramp.map(|c| [0, 1, 2].map(|i| (c[i] as i16 + d[i]).clamp(0, 255) as u8))
}

/// `a` mixed towards `b` by `k` (0..1).
pub fn mix(a: [u8; 4], b: [u8; 3], k: f64) -> [u8; 4] {
    let m = |i: usize| (a[i] as f64 + (b[i] as f64 - a[i] as f64) * k).round() as u8;
    [m(0), m(1), m(2), a[3]]
}

/// Broad mottling in 0..1 from two octaves of wrapping value noise plus a little texel grain.
pub fn mottle(seed: u32, x: i32, y: i32, grain: f64) -> f64 {
    smooth(seed, x, y, 4) * 0.6 + smooth(seed + 1, x, y, 8) * 0.4 + (n(seed + 2, x, y) - 0.5) * grain
}

/// Cells of about `16 / cells` texels on a wrapping grid: distances to the nearest and second
/// nearest feature point, and a 0..1 value per nearest cell.
pub fn cells(seed: u32, x: i32, y: i32, cells: i32) -> (f64, f64, f64) {
    let size = 16.0 / cells as f64;
    let (px, py) = (x as f64 + 0.5, y as f64 + 0.5);
    let (gx, gy) = ((px / size).floor() as i32, (py / size).floor() as i32);
    let (mut f1, mut f2, mut id) = (f64::MAX, f64::MAX, 0.0);
    for oy in -1..=1 {
        for ox in -1..=1 {
            let (cx, cy) = ((gx + ox).rem_euclid(cells), (gy + oy).rem_euclid(cells));
            let fx = (cx as f64 + unit(hash3(seed, cx, cy, 7))) * size;
            let fy = (cy as f64 + unit(hash3(seed, cx, cy, 8))) * size;
            let wrap = |d: f64| d - (d / 16.0).round() * 16.0;
            let (dx, dy) = (wrap(fx - px), wrap(fy - py));
            let d = (dx * dx + dy * dy).sqrt();
            if d < f1 {
                f2 = f1;
                f1 = d;
                id = unit(hash3(seed, cx, cy, 9));
            } else if d < f2 {
                f2 = d;
            }
        }
    }
    (f1, f2, id)
}

/// Where a texel lies relative to the nearest [`blob`].
#[derive(Clone, Copy)]
pub struct Spot {
    /// Which blob (0..count), for per-blob variation.
    pub k: u32,
    /// Normalised distance: 0 at the centre, 1 at the (ragged) rim, up to `reach` outside it.
    pub d: f64,
    /// Offset from the centre in radii (x right, y down), for lighting and facets.
    pub dx: f64,
    pub dy: f64,
}

/// The nearest of `count` blobs with radii in `r`, placed so that each blob stays inside the tile
/// (a faint halo out to `reach` radii may cross its edge), different for every `alt`, if `(x, y)`
/// lies within `reach` of one. Rims are ragged by a fixed per-texel jitter.
pub fn blob(seed: u32, alt: u32, count: u32, r: (f64, f64), reach: f64, x: i32, y: i32) -> Option<Spot> {
    let mut best: Option<Spot> = None;
    for k in 0..count {
        let h = |i: i32| unit(hash3(seed, alt as i32, k as i32, i));
        let radius = r.0 + (r.1 - r.0) * h(0);
        let m = (radius * 1.15).ceil() + 0.5;
        let cx = m + (16.0 - 2.0 * m) * h(1) - 0.5;
        let cy = m + (16.0 - 2.0 * m) * h(2) - 0.5;
        let (dx, dy) = (x as f64 - cx, y as f64 - cy);
        let ragged = radius * (0.85 + 0.3 * unit(hash3(seed + k, x, y, alt as i32)));
        let d = (dx * dx + dy * dy).sqrt() / ragged;
        if d <= reach && best.is_none_or(|b| d < b.d) {
            best = Some(Spot { k, d, dx: dx / radius, dy: dy / radius });
        }
    }
    best
}
