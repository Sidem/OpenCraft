//! Which texture layers the terrain shader may turn per block, so a realistic tile doesn't show the block grid on
//! a large floor or wall. Presentation only: the mesher and the core never see it.
//!
//! The shader (`web/src/render/shaders.ts`, `turnUv`) picks each block face's orientation from a hash of its
//! position, among those its layer allows: `TURNS` any of eight (four turns, each maybe mirrored), `LAYERED` a mirror
//! and an upside-down flip (strata and ripples stay level), `UPRIGHT` only a mirror (a grass edge stays on top).
//! Greedy quads stay merged: the choice is made per texel from the cell it falls in. Alternates turn with their base.
//! To extend: add a natural layer whose look has no direction (no arrows, seams or panels) to the list that fits it.

use super::tex;

/// Layers that turn any of eight ways: rock and soil with no grain.
pub const TURNS: [u16; 21] = [
    tex::STONE,
    tex::DIRT,
    tex::GRASS_TOP,
    tex::BEDROCK,
    tex::SPENT_ROCK,
    tex::COAL_ORE,
    tex::IRON_ORE,
    tex::COPPER_ORE,
    tex::GRANITE,
    tex::BASALT,
    tex::QUARTZ_ORE,
    tex::RUSTY_SOIL,
    tex::DARK_SOIL,
    tex::GREEN_SOIL,
    tex::PALE_SOIL,
    tex::RUSTY_GRASS_TOP,
    tex::DARK_GRASS_TOP,
    tex::GREEN_GRASS_TOP,
    tex::PALE_GRASS_TOP,
    tex::BAUXITE_ORE,
    tex::URANIUM_ORE,
];

/// Layers with level strata or ripples: they mirror and flip upside down, never turn on their side.
pub const LAYERED: [u16; 8] = [
    tex::SAND,
    tex::SANDSTONE,
    tex::LIMESTONE,
    tex::RUSTY_SAND,
    tex::DARK_SAND,
    tex::GREEN_SAND,
    tex::PALE_SAND,
    tex::OIL_SAND,
];

/// Side faces with a top (a grass edge): they only mirror left to right.
pub const UPRIGHT: [u16; 5] =
    [tex::GRASS_SIDE, tex::RUSTY_GRASS_SIDE, tex::DARK_GRASS_SIDE, tex::GREEN_GRASS_SIDE, tex::PALE_GRASS_SIDE];

/// Words in `MASK`: two bits for each of the 512 layers a vertex can name (9 bits, `mesher/quad.rs`).
pub const MASK_WORDS: usize = 32;

/// Bits `2 * (layer % 16)` of word `layer / 16`: 0 still, 1 upright, 2 layered, 3 turns (what the host uploads:
/// `Game::texture_turns`).
pub const MASK: [u32; MASK_WORDS] = mask();

#[cfg(test)]
mod tests;

const fn mask() -> [u32; MASK_WORDS] {
    let mut out = [0u32; MASK_WORDS];
    let mut layer = 0;
    while layer < tex::COUNT as u16 {
        let base = tex::look(layer).0;
        let mode = if listed(&TURNS, base) {
            3
        } else if listed(&LAYERED, base) {
            2
        } else if listed(&UPRIGHT, base) {
            1
        } else {
            0
        };
        out[(layer / 16) as usize] |= mode << (2 * (layer % 16));
        layer += 1;
    }
    out
}

const fn listed(list: &[u16], layer: u16) -> bool {
    let mut i = 0;
    while i < list.len() {
        if list[i] == layer {
            return true;
        }
        i += 1;
    }
    false
}
