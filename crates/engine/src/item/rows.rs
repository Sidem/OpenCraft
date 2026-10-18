//! Row builders for `EXTRA` in `item.rs`: one per kind of item (ingot, part, canister, machine tier, tool, worn
//! device) and the proportions some rows share. Add a builder when a new kind of item arrives.

use super::{ItemDef, MAX_STACK};
use crate::block::AIR;
use crate::tools::{Tier, DEVICE_TIER};

/// An ingot: a small bar of metal.
pub(super) const fn ingot(name: &'static str, layer: u16) -> ItemDef {
    ItemDef { name, stack: MAX_STACK, tex: [layer; 3], size: [0.9, 0.45, 0.55], places: AIR }
}

/// A part: `layer` on every face, proportions `size`.
pub(super) const fn part(name: &'static str, layer: u16, size: [f32; 3]) -> ItemDef {
    ItemDef { name, stack: MAX_STACK, tex: [layer; 3], size, places: AIR }
}

/// A canister: a small drum that holds a fluid, 16 to a slot.
pub(super) const fn canister(name: &'static str, layer: u16) -> ItemDef {
    ItemDef { name, stack: 16, tex: [layer; 3], size: [0.5, 0.6, 0.5], places: AIR }
}

/// A tier of a machine block: that block's look (the icon adds the tier chip).
pub(super) const fn machine(name: &'static str, tex: [u16; 3]) -> ItemDef {
    ItemDef { name, stack: MAX_STACK, tex, size: [1.0; 3], places: AIR }
}

/// A tool: a flat plate showing it, whose stack size is its uses.
pub(super) const fn tool(name: &'static str, layer: u16, tier: &Tier) -> ItemDef {
    ItemDef { name, stack: tier.uses, tex: [layer; 3], size: [0.7, 0.9, 0.12], places: AIR }
}

/// A worn or carried device: a tool's stack of uses, proportions `size`.
pub(super) const fn device(name: &'static str, layer: u16, size: [f32; 3]) -> ItemDef {
    ItemDef { name, stack: DEVICE_TIER.uses, tex: [layer; 3], size, places: AIR }
}

/// Proportions shared by an upgrade kit's case, a hauler pack and a pair of boots.
pub(super) const KIT: [f32; 3] = [0.76, 0.48, 0.46];
pub(super) const PACK: [f32; 3] = [0.6, 0.7, 0.45];
pub(super) const BOOTS: [f32; 3] = [0.5, 0.66, 0.5];
