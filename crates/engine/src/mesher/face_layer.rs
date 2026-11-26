//! Which texture layer a cube face draws (presentation only). A block's own layer per face (`block::FACE_TEX`), one of
//! four looks for layers with alternates, and turf (`block::is_turf`): grass and the stained soils draw their green top
//! on every face, the top included, that daylight reaches and grass has grown over, and their soil elsewhere: in shade,
//! and on the walls of a fresh hole until the grass creeps back (bare faces: `sim/sod.rs`). Whether a block is grass and
//! which faces are bare is core state; only the look is decided here, from the light the mesher already has, so nothing
//! remeshes at night (sky light is a level, the shader dims it by the time of day).
//!
//! Invariants: the result depends only on the block, its face, its chunk-local cell, the light in front of the face and
//! the block's own chunk, so neighbouring chunks agree. To tune: `GREEN_SKY`.

use crate::block::{is_turf, BlockId, ALT_TEX, FACE_TEX};
use crate::chunk::index;

/// Sky light (0..=15) in front of a face from which turf draws green: open sky is 15, and it falls by one a block into
/// shade, so a forest floor and the first blocks of an overhang or a cave mouth stay green.
pub(super) const GREEN_SKY: u8 = 8;
/// Face indices of the top and the underside (`mesher::FACES`, `block::pillar`).
const TOP: usize = 2;
const BOTTOM: usize = 3;

/// The texture layer for face `face` of block `b` at padded cell `c`, with sky light `sky` in front of it; `bares`
/// is its chunk's bare turf faces (`Chunk::bares`).
#[inline]
pub(super) fn face_layer(b: BlockId, face: usize, c: [usize; 3], sky: u8, bares: &[(u16, u8)]) -> u16 {
    let face = if is_turf(b) {
        if sky >= GREEN_SKY && !is_bare(bares, c, face) {
            TOP
        } else {
            BOTTOM
        }
    } else {
        face
    };
    pick_layer(b, face, c)
}

/// Whether face `face` of padded cell `c` is bare in its chunk's sorted table (no search when it is empty).
#[inline]
fn is_bare(bares: &[(u16, u8)], c: [usize; 3], face: usize) -> bool {
    if bares.is_empty() {
        return false;
    }
    let cell = index(c[0] - 1, c[1] - 1, c[2] - 1) as u16;
    bares.binary_search_by_key(&cell, |&(k, _)| k).is_ok_and(|at| bares[at].1 >> face & 1 != 0)
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
