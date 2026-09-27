//! Flat lookup tables over `DEFS`, indexed by raw block id, so hot loops (mesher, light, physics)
//! never bounds-check or branch on the registry. Derived at compile time; to add one, copy a table.

use super::{tex, Render, BLOCK_COUNT, DEFS};
/// Flat lookup tables indexed by raw block id, so hot loops never bounds-check or branch on the registry.
pub const OPAQUE: [bool; 256] = {
    let mut t = [false; 256];
    let mut i = 0;
    while i < BLOCK_COUNT {
        t[i] = matches!(DEFS[i].render, Render::Opaque);
        i += 1;
    }
    t
};

pub const CUTOUT: [bool; 256] = {
    let mut t = [false; 256];
    let mut i = 0;
    while i < BLOCK_COUNT {
        t[i] = matches!(DEFS[i].render, Render::Cutout);
        i += 1;
    }
    t
};

/// Blocks the chunk mesher draws as cubes (opaque, cutout or liquid).
pub const MESHED: [bool; 256] = {
    let mut t = [false; 256];
    let mut i = 0;
    while i < BLOCK_COUNT {
        t[i] = matches!(DEFS[i].render, Render::Opaque | Render::Cutout | Render::Liquid);
        i += 1;
    }
    t
};

/// Liquids: drawn translucent in their own mesh range, never against each other.
pub const LIQUID: [bool; 256] = {
    let mut t = [false; 256];
    let mut i = 0;
    while i < BLOCK_COUNT {
        t[i] = matches!(DEFS[i].render, Render::Liquid);
        i += 1;
    }
    t
};

/// Blocks the chunk mesher draws as crossed quads.
pub const PLANT: [bool; 256] = {
    let mut t = [false; 256];
    let mut i = 0;
    while i < BLOCK_COUNT {
        t[i] = matches!(DEFS[i].render, Render::Plant);
        i += 1;
    }
    t
};

pub const SOLID: [bool; 256] = {
    let mut t = [false; 256];
    let mut i = 0;
    while i < BLOCK_COUNT {
        t[i] = DEFS[i].solid;
        i += 1;
    }
    t
};

pub const FACE_TEX: [[u16; 6]; 256] = {
    let mut t = [[0u16; 6]; 256];
    let mut i = 0;
    while i < BLOCK_COUNT {
        t[i] = DEFS[i].faces;
        i += 1;
    }
    t
};

/// Per block and face, the first of the layer's alternates (`tex::alternates`), or 0.
pub const ALT_TEX: [[u16; 6]; 256] = {
    let mut t = [[0u16; 6]; 256];
    let mut i = 0;
    while i < BLOCK_COUNT {
        let mut f = 0;
        while f < 6 {
            t[i][f] = tex::alternates(DEFS[i].faces[f]);
            f += 1;
        }
        i += 1;
    }
    t
};
