//! Saplings (and the bamboo shoot) as loose items and in the hand: the plant as it stands in the world, two crossed
//! pictures (`mesher.rs` draws `Render::Plant` the same way), instead of a cube with the picture on every face. The
//! boxes are paper thin and cut out where the picture is clear (the box shader discards see-through texels), so the
//! picture's own outline is the model's. The inventory icon stays the flat picture (`icon_frame.rs`).
//! To add a plant: its block in `MODELS`.

use super::{turned, Part};
use crate::block::{self, FACE_SIDE};
use crate::item::ItemId;
use std::f32::consts::FRAC_PI_4;

/// How wide and tall a sapling's pictures are, as a share of the item cube.
const SIZE: f32 = 0.9;
/// How thick the pictures are: a sheet, not a box.
const THIN: f32 = 0.01;

/// The pictures of one plant: two sheets standing on the cube's floor, crossed at right angles.
const fn crossed(layer: u16) -> [Part; 2] {
    let at = [0.0, (SIZE - 1.0) * 0.5, 0.0];
    let size = [SIZE, SIZE, THIN];
    [turned(at, size, FRAC_PI_4, layer), turned(at, size, -FRAC_PI_4, layer)]
}

/// The model of the plant that is block `b`, from its picture (the layer on its sides).
const fn model(b: block::BlockId) -> (ItemId, [Part; 2]) {
    (ItemId::block(b), crossed(block::DEFS[b as usize].faces[FACE_SIDE]))
}

static MODELS: [(ItemId, [Part; 2]); 10] = [
    model(block::SAPLING),
    model(block::BIRCH_SAPLING),
    model(block::PINE_SAPLING),
    model(block::PALM_SAPLING),
    model(block::CHERRY_SAPLING),
    model(block::BAOBAB_SAPLING),
    model(block::ACACIA_SAPLING),
    model(block::REDWOOD_SAPLING),
    model(block::EUCALYPTUS_SAPLING),
    model(block::BAMBOO_SHOOT),
];

pub(super) fn parts(item: ItemId) -> &'static [Part] {
    MODELS.iter().find(|(id, _)| *id == item).map_or(&[], |(_, parts)| parts)
}
