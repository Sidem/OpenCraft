//! The mixed-hue concretes of CHEM-DYE (ids 228–238, one per `recipes/dye.rs` `HUES` row, in that order): blue, orange,
//! purple, cyan, lime, pink, sky, grey, brown, maroon and navy. Plain concrete blocks like the decor set's five
//! (`decor.rs`): machine-made only (an assembler, 4 concrete and a pigment), built to look at, drop themselves. Their faces
//! are `tex::tint_layer(i)` (painted from the palette, `textures/decor.rs`). To add a hue: a row in `HUES`, an id here
//! (append), a `TINT_DEFS` row, a layer, an item, a plant row and a tint row in `recipes/dye.rs`, and `TINT_COUNT` bumped.

use super::make::{all, cube};
use super::{sound, tex, BlockDef, BlockId};

pub const BLUE_CONCRETE: BlockId = 228;

pub const TINT_COUNT: usize = 11;

const fn concrete(name: &'static str, i: u16) -> BlockDef {
    cube(name, 1.4, all(tex::tint_layer(i)), BLUE_CONCRETE + i, sound::STONE)
}

/// The rows of `DEFS` from block 228 on, in id order.
pub(super) const TINT_DEFS: [BlockDef; TINT_COUNT] = [
    concrete("Blue Concrete", 0),
    concrete("Orange Concrete", 1),
    concrete("Purple Concrete", 2),
    concrete("Cyan Concrete", 3),
    concrete("Lime Concrete", 4),
    concrete("Pink Concrete", 5),
    concrete("Sky Concrete", 6),
    concrete("Grey Concrete", 7),
    concrete("Brown Concrete", 8),
    concrete("Maroon Concrete", 9),
    concrete("Navy Concrete", 10),
];
