//! The tier colours (Mk1 to Mk5) in two palettes, and which one this player has chosen. The standard one is
//! `factory::upgrades::TIER_COLOURS` (the science packs' colours: red, green, blue, violet, gold); the colour-blind one
//! keeps all five apart under protan, deutan and tritan sight and runs dark to light, so lightness says the Mk as well as
//! hue (`web/test/tier-palette.test.mjs` checks both with CIEDE2000).
//!
//! Presentation only: `Game::tier_palette` is a local setting the host sends (`set_tier_palette`), never saved and never in
//! the state hash. Engine-side colours (the item chip, the upgrade outline, the planned-line outlines) read
//! [`Game::tier_colour`]; the baked texture layers are recoloured by the host (`web/src/render/tier-paint.ts`) from the
//! two palettes `tier_palette_colours` hands it. To add a palette: a table here and an arm in [`colours`].

use crate::block::tex;
use crate::factory::upgrades::TIER_COLOURS;
use crate::Game;

/// Palette numbers as the host sends them.
pub const STANDARD: u8 = 0;
pub const COLOUR_BLIND: u8 = 1;

/// Mk1 to Mk5 as 0xRRGGBB, dark to light: blue, vermillion, violet, teal, gold. The worst pair of the five is 20 apart
/// (CIEDE2000) for normal, protan, deutan and tritan sight; the standard palette's blue and violet are under 1 apart for deutan.
pub const TIER_COLOURS_CB: [u32; 5] = [0x054fa9, 0xc54a29, 0x8a7cea, 0x41b597, 0xf0b036];

/// The five colours of palette `palette` (an unknown number is the standard one).
pub fn colours(palette: u8) -> [u32; 5] {
    if palette == COLOUR_BLIND {
        TIER_COLOURS_CB
    } else {
        TIER_COLOURS
    }
}

/// The layers that wear a tier colour as (layer, tier, kind) triples: kind 0 a stripe (`tex::stripe`), 1 a stud's paint
/// (`tex::paint`), 2 a kit's case (`tex::kit_paint`, Mk2 to Mk5 only).
pub fn layers() -> Vec<u16> {
    let mut out = Vec::new();
    for tier in 0..5u8 {
        out.extend([tex::stripe(tier), u16::from(tier), 0, tex::paint(tier), u16::from(tier), 1]);
        if tier > 0 {
            out.extend([tex::kit_paint(tier), u16::from(tier), 2]);
        }
    }
    out
}

impl Game {
    /// The colour of tier colour `tier` (0 is Mk1) in the palette this player chose, 0xRRGGBB.
    pub(crate) fn tier_colour(&self, tier: u8) -> u32 {
        colours(self.tier_palette).get(usize::from(tier)).copied().unwrap_or(0)
    }
}

#[cfg(test)]
mod tests;
