//! Constructors for the rows of `DEFS` (`mod.rs`): a block's faces (`all`, `pillar`) and its kinds (`cube`, `ore`,
//! `machine`, `frame`, `liquid`). To add a kind of block: a constructor here.

use super::{sound, tex, BlockDef, BlockId, Render, AIR};
pub(super) const fn all(t: u16) -> [u16; 6] {
    [t; 6]
}

pub(super) const fn pillar(side: u16, top: u16, bottom: u16) -> [u16; 6] {
    [side, side, top, bottom, side, side]
}

pub(super) const fn cube(name: &'static str, break_time: f32, faces: [u16; 6], drop: BlockId, sound: u8) -> BlockDef {
    BlockDef { name, render: Render::Opaque, solid: true, break_time, faces, drop, sound, placeable: true, light: 0 }
}

/// Ore stays in the ground: mining it by hand yields a handful of ore items that can't be placed.
pub(super) const fn ore(name: &'static str, faces: [u16; 6], drop: BlockId) -> BlockDef {
    BlockDef { placeable: false, ..cube(name, 1.6, faces, drop, sound::STONE) }
}

/// A machine drawn by the host as an instanced model rather than by the chunk mesher.
pub(super) const fn machine(
    name: &'static str,
    solid: bool,
    break_time: f32,
    faces: [u16; 6],
    id: BlockId,
) -> BlockDef {
    BlockDef { render: Render::None, solid, ..cube(name, break_time, faces, id, sound::METAL) }
}

/// A see-through climbing frame (the ladder, the hoist shaft): cutout, not solid.
pub(super) const fn frame(name: &'static str, break_time: f32, faces: [u16; 6], id: BlockId, sound: u8) -> BlockDef {
    BlockDef { render: Render::Cutout, solid: false, ..cube(name, break_time, faces, id, sound) }
}

/// Water, still or flowing: not solid, never targeted, dropping nothing.
pub(super) const fn liquid(name: &'static str) -> BlockDef {
    let base = cube(name, -1.0, all(tex::WATER), AIR, sound::SAND);
    BlockDef { render: Render::Liquid, solid: false, placeable: false, ..base }
}
