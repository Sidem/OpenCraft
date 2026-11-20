//! Stained glass (GLASS, ids 239–254): sixteen translucent building blocks in the same sixteen-hue palette as the coloured
//! concretes (five primary hues, then `recipes/dye.rs` `HUES`), made in an assembler from glass and mineral colorants (not
//! from dyes). Solid cubes that let both sky/block light and laser beams through like plain glass (`light.rs`,
//! `factory/laser.rs`); adjacent blocks of the same colour hide their shared face in the mesher. To add a hue: append an
//! id, a `STAINED_GLASS_DEFS` row, a layer in `tex.rs`, a recipe in `recipes/glass.rs` and bump `STAINED_GLASS_COUNT`.

use super::make::{all, cube};
use super::{sound, tex, BlockDef, BlockId, Render, GLASS};

pub const STAINED_GLASS_FIRST: BlockId = 239;
pub const STAINED_GLASS_COUNT: usize = 16;

/// Whether `id` is one of the sixteen stained glass blocks.
#[inline]
pub const fn is_stained_glass(id: BlockId) -> bool {
    id >= STAINED_GLASS_FIRST && id < STAINED_GLASS_FIRST + STAINED_GLASS_COUNT as BlockId
}

/// Whether `id` is plain glass or any stained glass (lets sky light and laser beams straight through).
#[inline]
pub const fn is_glass(id: BlockId) -> bool {
    id == GLASS || is_stained_glass(id)
}

const fn pane(name: &'static str, i: u16) -> BlockDef {
    BlockDef {
        render: Render::Translucent,
        ..cube(name, 0.3, all(tex::stained_glass_layer(i)), STAINED_GLASS_FIRST + i, sound::STONE)
    }
}

/// The rows of `DEFS` from block 239 on, in id order: the five primary hues, then `recipes/dye.rs` `HUES`.
pub(super) const STAINED_GLASS_DEFS: [BlockDef; STAINED_GLASS_COUNT] = [
    pane("Red Stained Glass", 0),
    pane("Green Stained Glass", 1),
    pane("Black Stained Glass", 2),
    pane("White Stained Glass", 3),
    pane("Yellow Stained Glass", 4),
    pane("Blue Stained Glass", 5),
    pane("Orange Stained Glass", 6),
    pane("Purple Stained Glass", 7),
    pane("Cyan Stained Glass", 8),
    pane("Lime Stained Glass", 9),
    pane("Pink Stained Glass", 10),
    pane("Sky Stained Glass", 11),
    pane("Grey Stained Glass", 12),
    pane("Brown Stained Glass", 13),
    pane("Maroon Stained Glass", 14),
    pane("Navy Stained Glass", 15),
];
