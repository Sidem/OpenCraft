//! Which texture layer a cube face draws (presentation only). A block's own layer per face (`block::FACE_TEX`), one of
//! four looks for layers with alternates, and turf: grass and the stained soils draw their green top on every side and
//! underside that daylight reaches, and their soil where it doesn't. Whether a block is grass at all is core state
//! (`sim/timers.rs`); only the per-face look is decided here, from the light the mesher already has, so it costs no
//! state and nothing remeshes at night (sky light is a level, the shader dims it by the time of day).
//!
//! Invariants: the result depends only on the block, its face, its chunk-local cell and the light in front of the face,
//! so neighbouring chunks agree. To tune: `GREEN_SKY`.

use crate::block::{BlockId, ALT_TEX, FACE_TEX, GRASS, PALE_SOIL, RUSTY_SOIL};

/// Sky light (0..=15) in front of a side or underside from which turf draws green: open sky is 15, and it falls by
/// one a block into shade, so the first four blocks of an overhang or a cave mouth stay green.
pub(super) const GREEN_SKY: u8 = 11;
/// Face indices of the top and the underside (`mesher::FACES`, `block::pillar`).
const TOP: usize = 2;
const BOTTOM: usize = 3;

/// The texture layer for face `face` of block `b` at padded cell `c`, with sky light `sky` in front of it.
#[inline]
pub(super) fn face_layer(b: BlockId, face: usize, c: [usize; 3], sky: u8) -> u16 {
    let face = if face != TOP && is_turf(b) {
        if sky >= GREEN_SKY {
            TOP
        } else {
            BOTTOM
        }
    } else {
        face
    };
    pick_layer(b, face, c)
}

/// Grass and the soils that stain it over ore (`worldgen/geology.rs`): a green top over soil.
#[inline]
fn is_turf(b: BlockId) -> bool {
    b == GRASS || (RUSTY_SOIL..=PALE_SOIL).contains(&b)
}

/// Its own layer, or for a layer with alternates one of the four, the same for every face of the block. Chosen from
/// the chunk-local position, so the pattern repeats only every chunk.
#[inline]
fn pick_layer(b: BlockId, face: usize, c: [usize; 3]) -> u16 {
    let base = FACE_TEX[b as usize][face];
    let alt = ALT_TEX[b as usize][face];
    if alt == 0 {
        return base;
    }
    let h = (c[0] as u32).wrapping_mul(0x9E37_79B1)
        ^ (c[1] as u32).wrapping_mul(0x85EB_CA77)
        ^ (c[2] as u32).wrapping_mul(0xC2B2_AE3D);
    match ((h ^ (h >> 15)).wrapping_mul(0x2C1B_3C6D) >> 30) as u16 {
        0 => base,
        k => alt + k - 1,
    }
}
