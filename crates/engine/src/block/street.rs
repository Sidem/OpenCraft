//! The street lamp (ids 223–227): a post four cells tall (`street/`). The top cell, `STREET`, is the lamp head and the item
//! (it gives the brightest LED light: `light::LED_LIGHTS`); the three cells under it are `STREET_POST` plus the way the
//! arm leans (0..3, a `DIRS` index), so the lamp keeps its facing without any state of its own. All five are drawn as one
//! instanced model by `street/view.rs` (render none), solid, and drop the item. Only the head is placed by a player.
//! To add a block after this one: a set file of its own like this, joined in `defs.rs` and counted in `BLOCK_COUNT`.

use super::make::{all, machine};
use super::{tex, BlockDef, BlockId};

pub const STREET: BlockId = 223;
/// The post cells, one id a facing: `STREET_POST + dir`.
pub const STREET_POST: BlockId = 224;

pub const STREET_COUNT: usize = 5;

/// Whether `b` is any part of a street lamp.
pub const fn is_street(b: BlockId) -> bool {
    b >= STREET && b < STREET + STREET_COUNT as BlockId
}

/// The post block of a lamp whose arm leans toward `DIRS[dir]`.
pub const fn street_post(dir: u8) -> BlockId {
    STREET_POST + (dir % 4) as BlockId
}

/// The way a post cell's arm leans (a `DIRS` index), if `b` is a post.
pub const fn post_dir(b: BlockId) -> Option<u8> {
    if b >= STREET_POST && b < STREET_POST + 4 {
        Some((b - STREET_POST) as u8)
    } else {
        None
    }
}

const fn part(name: &'static str, light: u8, placeable: bool) -> BlockDef {
    BlockDef { light, placeable, ..machine(name, true, 0.5, all(tex::FRAME), STREET) }
}

/// The rows of `DEFS` from block 223 on, in id order.
pub(super) const STREET_DEFS: [BlockDef; STREET_COUNT] = [
    part("Street Lamp", crate::light::LED_LIGHTS[2], true),
    part("Street Lamp (post)", 0, false),
    part("Street Lamp (post)", 0, false),
    part("Street Lamp (post)", 0, false),
    part("Street Lamp (post)", 0, false),
];
