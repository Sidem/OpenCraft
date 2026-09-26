//! Alpine natural materials drawn as pixel art: a few hand-picked tones per material (`paint` ramps),
//! slate plates with fractures, grass blades with their shadows, a ragged grass edge over pebbled
//! soil, rippled sand and clustered leaves (four looks). All patterns wrap at the tile edge.
//!
//! Surface hints (`HINTS`) are the same grass, edge, soil and sand a shade off, with a few mineral
//! specks: noticeable when you look for them, never loud. To add a hint: a `HINTS` entry plus its
//! grass top, grass side, soil and sand layers in `block::tex`, in the same order.

use super::paint::{blob, cells, mix, mottle, shifted, tone, Ramp};
use super::{n, smooth};
use crate::block::tex;

const SLATE: Ramp = [[78, 91, 101], [96, 110, 120], [111, 126, 135], [127, 142, 150], [148, 161, 167]];
const TURF: Ramp = [[44, 74, 56], [56, 90, 66], [68, 106, 76], [84, 122, 86], [106, 140, 96]];
const EARTH: Ramp = [[72, 58, 48], [88, 72, 60], [104, 87, 73], [120, 102, 86], [140, 122, 104]];
const SAND: Ramp = [[168, 158, 134], [182, 173, 150], [195, 187, 164], [207, 200, 178], [220, 214, 194]];
const LEAF: Ramp = [[34, 60, 46], [46, 78, 57], [58, 95, 68], [75, 112, 79], [96, 132, 90]];
const PEBBLE: Ramp = [[82, 84, 84], [102, 102, 100], [122, 121, 117], [144, 142, 137], [168, 166, 160]];

/// A surface hint: how far its grass and soil tones shift, and the mineral specks strewn in it.
struct Hint {
    grass: [i16; 3],
    soil: [i16; 3],
    speck: [u8; 3],
    specks: f64,
}

/// Rusty (iron), dark (coal), verdigris (copper), pale (limestone, quartz): the order of the hint
/// layers in `block::tex`.
const HINTS: [Hint; 4] = [
    Hint { grass: [11, 4, -7], soil: [16, 4, -8], speck: [178, 94, 48], specks: 0.07 },
    Hint { grass: [-9, -9, -6], soil: [-18, -18, -15], speck: [30, 30, 33], specks: 0.08 },
    Hint { grass: [-5, 0, 8], soil: [-8, 4, 6], speck: [70, 152, 134], specks: 0.06 },
    Hint { grass: [11, 9, 3], soil: [18, 17, 14], speck: [228, 224, 210], specks: 0.07 },
];

/// Layer `layer` (look `alt` of it, for leaves) at texel (x, y).
pub(super) fn pixel(layer: u16, alt: u32, x: i32, y: i32) -> [u8; 4] {
    let (x, y) = (x.rem_euclid(16), y.rem_euclid(16));
    let hint = |first: u16| Some(&HINTS[(layer - first) as usize]);
    match layer {
        tex::STONE => stone(x, y),
        tex::DIRT => dirt(x, y, None),
        tex::GRASS_TOP => grass(x, y, None),
        tex::GRASS_SIDE => grass_side(x, y, None),
        tex::SAND => sand(x, y, None),
        tex::RUSTY_SOIL..=tex::PALE_SOIL => dirt(x, y, hint(tex::RUSTY_SOIL)),
        tex::RUSTY_GRASS_TOP..=tex::PALE_GRASS_TOP => grass(x, y, hint(tex::RUSTY_GRASS_TOP)),
        tex::RUSTY_GRASS_SIDE..=tex::PALE_GRASS_SIDE => grass_side(x, y, hint(tex::RUSTY_GRASS_SIDE)),
        tex::RUSTY_SAND..=tex::PALE_SAND => sand(x, y, hint(tex::RUSTY_SAND)),
        tex::LOG_SIDE => bark(x, y),
        tex::LOG_TOP => rings(x, y),
        tex::LEAVES => leaves(x, y, alt),
        tex::BEDROCK => {
            let c = grain([55, 67, 76], 140, x, y);
            if smooth(143, x, y, 4) < 0.25 {
                shade(c, -8)
            } else {
                c
            }
        }
        tex::SPENT_ROCK => {
            let base = if smooth(144, x, y, 2) < 0.38 { [152, 147, 139] } else { [157, 161, 158] };
            grain(base, 145, x, y)
        }
        _ => unreachable!("non-natural texture"),
    }
}

/// Shared tileable grain for the plainer materials: broad variation, flecks and a quiet grit.
pub(super) fn grain(base: [u8; 3], seed: u32, x: i32, y: i32) -> [u8; 4] {
    let broad = smooth(seed, x, y, 2) * 0.55 + smooth(seed + 1, x, y, 4) * 0.3 + smooth(seed + 2, x, y, 8) * 0.15;
    let grit = n(seed + 3, x, y);
    let delta = ((broad - 0.5) * 33.0 + (grit - 0.5) * 10.0) as i8;
    shade([base[0], base[1], base[2], 255], delta)
}

/// Slate, the host rock of every ore: broad plates of slightly different tone split by fine dark
/// fractures, each with a lit lip above it.
pub(super) fn stone(x: i32, y: i32) -> [u8; 4] {
    let crack = |x: i32, y: i32| {
        let (f1, f2, _) = cells(101, x, y, 2);
        f2 - f1 < 0.5 && n(103, x, y) > 0.2
    };
    let (_, _, plate) = cells(101, x, y, 2);
    let t = 0.3 + plate * 0.25 + (mottle(102, x, y, 0.3) - 0.5) * 0.35;
    if crack(x, y) {
        tone(&SLATE, 0.0)
    } else if crack(x, y + 1) {
        tone(&SLATE, t + 0.22)
    } else {
        tone(&SLATE, t)
    }
}

/// Earth with small stones lit from the upper left (each casting a shadow texel), grit, and for a
/// hint its mineral specks.
fn dirt(x: i32, y: i32, hint: Option<&Hint>) -> [u8; 4] {
    let ramp = hint.map_or(EARTH, |h| shifted(&EARTH, h.soil));
    let t = 0.28 + mottle(120, x, y, 0.3) * 0.44;
    let pebble = |x: i32, y: i32| blob(121, 0, 5, (0.7, 1.4), 1.0, x, y);
    let mut c = match pebble(x, y) {
        Some(s) => tone(&PEBBLE, 0.5 - (s.dx + s.dy) * 0.35 + s.k as f64 * 0.04),
        None if pebble(x - 1, y - 1).is_some() => tone(&ramp, t - 0.25),
        None if n(122, x, y) > 0.95 => tone(&ramp, t + 0.25),
        None if n(123, x, y) < 0.05 => tone(&ramp, t - 0.25),
        None => tone(&ramp, t),
    };
    if let Some(h) = hint {
        if n(124, x, y) < h.specks {
            c = mix(c, h.speck, 0.8);
        }
    }
    c
}

/// Tone of the turf at (x, y): clumps, and blades as a lit tip with its shadow a texel below.
fn turf(x: i32, y: i32) -> f64 {
    let t = 0.3 + mottle(125, x, y, 0.2) * 0.45;
    let blade = |y: i32| n(130, x, y) > 0.8;
    if blade(y) {
        t + 0.22
    } else if blade(y - 1) {
        t - 0.2
    } else {
        t
    }
}

fn grass(x: i32, y: i32, hint: Option<&Hint>) -> [u8; 4] {
    match hint {
        None => tone(&TURF, turf(x, y)),
        Some(h) => {
            let c = tone(&shifted(&TURF, h.grass), turf(x, y));
            if n(131, x, y) < h.specks * 0.5 {
                mix(c, h.speck, 0.6)
            } else {
                c
            }
        }
    }
}

/// Turf hanging 2..5 texels over the soil in a ragged fringe, with a shadow row under it.
fn grass_side(x: i32, y: i32, hint: Option<&Hint>) -> [u8; 4] {
    let depth = 2 + (smooth(108, x, 0, 8) * 2.6) as i32 + i32::from(n(109, x, 0) > 0.8);
    if y < depth {
        let ramp = hint.map_or(TURF, |h| shifted(&TURF, h.grass));
        tone(&ramp, turf(x, y) - 0.08)
    } else if y == depth {
        let soil = hint.map_or(EARTH, |h| shifted(&EARTH, h.soil));
        if n(110, x, y) > 0.7 {
            tone(&TURF, 0.0)
        } else {
            tone(&soil, 0.0)
        }
    } else {
        dirt(x, y, hint)
    }
}

/// Wind ripples (a shaded trough, a lit crest) with dark and light grains.
fn sand(x: i32, y: i32, hint: Option<&Hint>) -> [u8; 4] {
    // Sand is pale, so the same shift shows twice as much: halve it.
    let ramp = hint.map_or(SAND, |h| shifted(&SAND, h.soil.map(|v| v / 2)));
    let t = 0.35 + mottle(131, x, y, 0.2) * 0.35;
    let ripple = (y + (smooth(135, x, y, 4) * 3.0) as i32).rem_euclid(4);
    let t = match ripple {
        0 => t - 0.16,
        1 => t + 0.1,
        _ => t,
    };
    let mut c = if n(136, x, y) > 0.95 {
        tone(&ramp, 0.0)
    } else if n(137, x, y) < 0.04 {
        tone(&ramp, 0.99)
    } else {
        tone(&ramp, t)
    };
    if let Some(h) = hint {
        if n(138, x, y) < h.specks {
            c = mix(c, h.speck, 0.7);
        }
    }
    c
}

/// Leaf clusters, brighter towards their middles and outlined darker; gaps open along the outlines. Transparent texels keep a leaf colour so mipmaps don't fringe.
fn leaves(x: i32, y: i32, alt: u32) -> [u8; 4] {
    let seed = 165 + alt * 11;
    let (f1, f2, id) = cells(seed, x, y, 4);
    let edge = f2 - f1;
    if edge < 0.9 && n(seed + 2, x, y) > 0.5 {
        let mut c = tone(&LEAF, 0.3);
        c[3] = 0;
        return c;
    }
    let t = 0.2 + id * 0.35 + (1.0 - f1 / 3.0).max(0.0) * 0.35;
    tone(&LEAF, if edge < 0.6 { t - 0.25 } else { t })
}

fn shade(c: [u8; 4], delta: i8) -> [u8; 4] {
    [c[0].saturating_add_signed(delta), c[1].saturating_add_signed(delta), c[2].saturating_add_signed(delta), c[3]]
}

fn bark(x: i32, y: i32) -> [u8; 4] {
    let c = grain([93, 83, 69], 150, x, y);
    // Long anisotropic fissures bend with the grain, rather than repeating at fixed columns.
    let groove = smooth(154, x, y, 4) * 0.65 + smooth(155, x, 0, 8) * 0.35;
    if groove < 0.36 {
        shade(c, -16)
    } else if groove > 0.72 {
        shade(c, 8)
    } else {
        c
    }
}

fn rings(x: i32, y: i32) -> [u8; 4] {
    let warp = smooth(158, x, y, 2) - 0.5;
    let (dx, dy) = (x as f64 - 7.5 + warp, y as f64 - 7.5 - warp);
    let radius = (dx * dx + dy * dy).sqrt();
    if radius > 7.0 {
        return bark(x, y);
    }
    let c = grain([166, 148, 116], 159, x, y);
    if ((radius + warp * 0.7) * 1.6) as i32 % 2 == 0 {
        shade(c, 9)
    } else {
        shade(c, -10)
    }
}
