//! Item models of raw and semi-finished stock: the stick, heaps of crushed and washed ore, sulfur and quicklime, the
//! silicon boule, aluminium, plastic sheets, coins, the wafer and scrap. Mostly one material each; the shape says
//! what the thing is. Add a model as a const and a row in `parts`.

use std::f32::consts::FRAC_PI_4;

use super::shapes::{bar_x, cylinder, faced, pile, Build};
use super::{part, turned, Part};
use crate::block::tex;
use crate::item::{
    ItemId, ALUMINIUM_INGOT, ALUMINIUM_PLATE, COIN, CRUSHED_BAUXITE, CRUSHED_COPPER, CRUSHED_IRON, PLASTIC, QUICKLIME,
    SCRAP_METAL, SILICON, STICK, SULFUR, WAFER, WASHED_BAUXITE, WASHED_COPPER, WASHED_IRON,
};

pub(super) fn parts(item: ItemId) -> &'static [Part] {
    match item {
        STICK => &STICK_MODEL,
        CRUSHED_IRON => &CRUSHED_IRON_HEAP,
        CRUSHED_COPPER => &CRUSHED_COPPER_HEAP,
        CRUSHED_BAUXITE => &CRUSHED_BAUXITE_HEAP,
        WASHED_IRON => &WASHED_IRON_HEAP,
        WASHED_COPPER => &WASHED_COPPER_HEAP,
        WASHED_BAUXITE => &WASHED_BAUXITE_HEAP,
        SULFUR => &SULFUR_HEAP,
        QUICKLIME => &QUICKLIME_HEAP,
        SILICON => &BOULE,
        ALUMINIUM_INGOT => &ALUMINIUM_BAR,
        ALUMINIUM_PLATE => &ALUMINIUM_SHEET,
        PLASTIC => &PLASTIC_SHEETS,
        COIN => &COINS,
        WAFER => &WAFER_DISC,
        SCRAP_METAL => &SCRAP_HEAP,
        _ => &[],
    }
}

/// A crooked stick: two lengths meeting at a slight bend, and a twig.
const STICK_MODEL: [Part; 3] = [
    turned([-0.21, 0.0, 0.01], [0.52, 0.09, 0.09], 0.07, tex::STICK),
    turned([0.22, 0.0, -0.01], [0.46, 0.08, 0.08], -0.05, tex::STICK),
    turned([0.08, -0.005, 0.08], [0.18, 0.05, 0.05], -0.7, tex::STICK),
];

const CRUSHED_IRON_HEAP: [Part; 8] = pile(tex::CRUSHED_IRON);
const CRUSHED_COPPER_HEAP: [Part; 8] = pile(tex::CRUSHED_COPPER);
const CRUSHED_BAUXITE_HEAP: [Part; 8] = pile(tex::CRUSHED_BAUXITE);
const WASHED_IRON_HEAP: [Part; 8] = pile(tex::WASHED_IRON);
const WASHED_COPPER_HEAP: [Part; 8] = pile(tex::WASHED_COPPER);
const WASHED_BAUXITE_HEAP: [Part; 8] = pile(tex::WASHED_BAUXITE);
const SULFUR_HEAP: [Part; 8] = pile(tex::SULFUR);
const QUICKLIME_HEAP: [Part; 8] = pile(tex::QUICKLIME);

/// A silicon boule lying on its side: a round crystal with tapered ends, the seed neck at the front.
const BOULE: [Part; 12] = Build::new()
    .all(bar_x([-0.36, 0.0, 0.0], 0.08, 0.26, tex::SILICON))
    .all(bar_x([0.0, 0.0, 0.0], 0.64, 0.36, tex::SILICON))
    .all(bar_x([0.36, 0.0, 0.0], 0.08, 0.26, tex::SILICON))
    .all(bar_x([0.42, 0.0, 0.0], 0.05, 0.12, tex::SILICON))
    .done();

/// A bar over a plate, in aluminium (as the iron and steel bars).
const ALUMINIUM_BAR: [Part; 2] = [
    part([0.0, -0.07, 0.0], [0.88, 0.29, 0.53], tex::ALUMINIUM_INGOT),
    part([0.0, 0.105, 0.0], [0.69, 0.07, 0.4], tex::ALUMINIUM_PLATE),
];
const ALUMINIUM_SHEET: [Part; 2] = [
    part([0.0, -0.07, 0.0], [0.84, 0.1, 0.84], tex::ALUMINIUM_PLATE),
    part([0.0, 0.005, 0.0], [0.66, 0.05, 0.66], tex::ALUMINIUM_PLATE),
];

/// A stack of three white plastic sheets, each turned a little off the one below.
const PLASTIC_SHEETS: [Part; 3] = [
    turned([0.0, -0.04, 0.0], [0.78, 0.035, 0.52], 0.0, tex::PLASTIC),
    turned([0.01, -0.005, 0.01], [0.78, 0.035, 0.52], 0.07, tex::PLASTIC),
    turned([-0.01, 0.03, 0.0], [0.76, 0.035, 0.5], -0.05, tex::PLASTIC),
];

/// A brass coin centred at `c`: a 12-sided disc with a raised face.
const fn coin(c: [f32; 3]) -> [Part; 5] {
    Build::new()
        .all(cylinder::<3>(c, 0.46, 0.05, tex::MAT_BRASS))
        .all(cylinder::<2>([c[0], c[1] + 0.006, c[2]], 0.3, 0.05, tex::MAT_BRASS))
        .done()
}

/// Two coins, the top one slid aside onto the first.
const COINS: [Part; 10] = Build::new().all(coin([-0.05, -0.035, -0.03])).all(coin([0.06, 0.025, 0.05])).done();

/// A polished silicon wafer: a thin round disc with its grid of dies printed in the middle.
const WAFER_DISC: [Part; 4] = Build::new()
    .all(cylinder::<3>([0.0, -0.01, 0.0], 0.66, 0.02, tex::MAT_GUNMETAL))
    .with(faced([0.0, 0.002, 0.0], [0.44, 0.006, 0.44], tex::WAFER, tex::MAT_GUNMETAL))
    .done();

/// A jumble of scrap: bent rusty plates, a length of girder, a broken housing, a pipe and a bolt.
const SCRAP_HEAP: [Part; 7] = [
    turned([-0.12, -0.11, -0.1], [0.4, 0.05, 0.3], 0.4, tex::RUST_PLATE),
    turned([0.1, -0.1, -0.12], [0.36, 0.08, 0.1], 1.2, tex::RUST_GIRDER),
    turned([-0.16, -0.06, 0.1], [0.3, 0.05, 0.22], -0.5, tex::RUST_PLATE),
    turned([0.14, -0.08, 0.12], [0.2, 0.14, 0.18], 0.3, tex::RUST_MACHINE),
    turned([-0.02, -0.01, 0.0], [0.32, 0.06, 0.06], 0.9, tex::MAT_GUNMETAL),
    turned([0.04, 0.04, -0.06], [0.22, 0.04, 0.16], -0.2, tex::RUST_PLATE),
    turned([0.21, -0.12, -0.03], [0.07, 0.06, 0.07], FRAC_PI_4, tex::MAT_METAL),
];
