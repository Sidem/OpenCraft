//! Each wood's blocks (ids 182–218): the oak's tree log (what grown oaks are made of from generator 12, felled whole
//! like every tree: `sim/felling.rs`), and for each wood after the oak (`WOOD_NAMES` order: birch, pine, palm, cherry,
//! baobab, acacia, redwood, eucalyptus, bamboo) planks, a floor and a door (closed and open) in its colour. They build
//! and break like the oak's (`PLANKS`, `WOOD_FLOOR`, `WOOD_DOOR`), and recipes take any wood's as the oak's
//! (`crate::wood`). Layers after the trees': placeholders tinted from the oak's in `textures/woods.rs`, the art
//! pack's in `tools/art`. To add a wood: a name row here, its ids follow (append only: ids are saved).

use super::make::{all, cube, pillar};
use super::{sound, tex, BlockDef, BlockId, Render, LOG};

/// A grown oak's (and broadleaf's) wood: the oak's bark, never held.
pub const OAK_TREE_LOG: BlockId = 182;
/// The first wood's planks; each wood takes four ids from here (`planks_of` and on).
const FIRST: BlockId = 183;
/// Woods after the oak, each with planks, a floor and a door.
pub const OTHER_WOODS: usize = 9;
pub const WOODS_COUNT: usize = 1 + 4 * OTHER_WOODS;

/// The block names of each wood after the oak: planks, floor, door, open door.
const WOOD_NAMES: [[&str; 4]; OTHER_WOODS] = [
    ["Birch Planks", "Birch Floor", "Birch Door", "Open Birch Door"],
    ["Pine Planks", "Pine Floor", "Pine Door", "Open Pine Door"],
    ["Palm Planks", "Palm Floor", "Palm Door", "Open Palm Door"],
    ["Cherry Planks", "Cherry Floor", "Cherry Door", "Open Cherry Door"],
    ["Baobab Planks", "Baobab Floor", "Baobab Door", "Open Baobab Door"],
    ["Acacia Planks", "Acacia Floor", "Acacia Door", "Open Acacia Door"],
    ["Redwood Planks", "Redwood Floor", "Redwood Door", "Open Redwood Door"],
    ["Eucalyptus Planks", "Eucalyptus Floor", "Eucalyptus Door", "Open Eucalyptus Door"],
    ["Bamboo Planks", "Bamboo Floor", "Bamboo Door", "Open Bamboo Door"],
];

/// Wood `w`'s (0 = birch) planks, floor, door and open door.
pub const fn planks_of(w: usize) -> BlockId {
    FIRST + 4 * w as BlockId
}
pub const fn floor_of(w: usize) -> BlockId {
    planks_of(w) + 1
}
pub const fn door_of(w: usize) -> BlockId {
    planks_of(w) + 2
}
pub const fn open_door_of(w: usize) -> BlockId {
    planks_of(w) + 3
}

/// The wood (0 = birch) of one of these planks, floors or doors.
pub const fn wood_block(id: BlockId) -> Option<(usize, usize)> {
    if id >= FIRST && (id as usize) < FIRST as usize + 4 * OTHER_WOODS {
        let i = (id - FIRST) as usize;
        Some((i / 4, i % 4))
    } else {
        None
    }
}

/// Texture layers: four per wood after the trees' (`tex.rs` re-exports them).
pub mod layers {
    use super::OTHER_WOODS;
    use crate::block::tex::TREE_LAYERS_END;

    /// Wood `w`'s planks layer; its floor, door and open door follow.
    pub const fn planks_layer(w: usize) -> u16 {
        TREE_LAYERS_END + 4 * w as u16
    }
    /// One past the last: `tex::COUNT`.
    pub const WOOD_LAYERS_END: u16 = TREE_LAYERS_END + 4 * OTHER_WOODS as u16;
}

/// The rows of `DEFS` from block 182 on, in id order.
pub(super) const WOODS_DEFS: [BlockDef; WOODS_COUNT] = defs();

const fn defs() -> [BlockDef; WOODS_COUNT] {
    let oak = BlockDef {
        placeable: false,
        ..cube("Oak Log", 0.9, pillar(tex::LOG_SIDE, tex::LOG_TOP, tex::LOG_TOP), LOG, sound::WOOD)
    };
    let mut d = [oak; WOODS_COUNT];
    let mut w = 0;
    while w < OTHER_WOODS {
        let (n, l) = (&WOOD_NAMES[w], layers::planks_layer(w));
        let i = 1 + 4 * w;
        d[i] = cube(n[0], 0.7, all(l), planks_of(w), sound::WOOD);
        d[i + 1] = cube(n[1], 0.7, all(l + 1), floor_of(w), sound::WOOD);
        d[i + 2] = cube(n[2], 0.7, all(l + 2), door_of(w), sound::WOOD);
        d[i + 3] = BlockDef {
            render: Render::Cutout,
            solid: false,
            placeable: false,
            ..cube(n[3], 0.7, all(l + 3), door_of(w), sound::WOOD)
        };
        w += 1;
    }
    d
}
