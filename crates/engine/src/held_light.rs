//! Light a hand gives: a torch or lamp (any block that shines, `BlockDef::light`) in either hand of a player lights the
//! ground, machines and other players around them as they walk. Presentation only: the chunk meshes keep their baked
//! light, and the host adds this on top in the fragment shader (`render/held-light.ts`), so it moves with no remeshing.
//! `Game::write_avatars` fills `Avatars::lights` each frame, [`LIGHT_FLOATS`] a player: the light's position relative to
//! the camera, then its reach in blocks. A headlamp or X-ray goggles worn on the head light the same way. To make
//! another item shine: teach [`reach`] about it.

use crate::block;
use crate::inventory::Inventory;
use crate::item::ItemId;

/// Floats per light: x, y, z (camera-relative) and the reach in blocks.
pub const LIGHT_FLOATS: usize = 4;
/// Dynamic point lights the shader draws at once: up to 4 lit players plus up to 12 fire lights (flamethrower plume,
/// burning trees, ground embers). Matches `HELD_LIGHT_SLOTS` in `web/src/render/shaders.ts`.
pub const MAX_LIGHTS: usize = 16;
/// How far a held light reaches, in blocks: a little less than a torch standing in the ground (`light.rs`).
const REACH: f32 = 9.0;
/// Reach of an active flame (flamethrower plume or burning tree), matching a placed torch (~11 blocks).
pub const FLAME_REACH: f32 = 11.0;

/// How far `item` lights its surroundings when held; 0 for anything that doesn't shine.
pub fn reach(item: ItemId) -> f32 {
    if item.places().is_some_and(|b| block::def(b).light != 0) {
        REACH
    } else {
        0.0
    }
}

/// The reach of the brightest of a player's two hands and a worn head light (`Inventory::head_light`).
pub fn reach_of(inv: &Inventory) -> f32 {
    let head = if inv.head_light() { REACH } else { 0.0 };
    reach(inv.selected_stack().item).max(reach(inv.offhand.item)).max(head)
}

#[cfg(test)]
mod tests;
