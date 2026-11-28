//! Flat lookup tables over `DEFS`, indexed by raw block id, so hot loops (mesher, light, physics)
//! never bounds-check or branch on the registry. Derived at compile time; to add one, copy a table.

use super::{tex, Render, BLOCK_COUNT, DEFS, TABLE_LEN};
/// Flat lookup tables indexed by raw block id, so hot loops never bounds-check or branch on the registry.
pub const OPAQUE: [bool; TABLE_LEN] = {
    let mut t = [false; TABLE_LEN];
    let mut i = 0;
    while i < BLOCK_COUNT {
        t[i] = matches!(DEFS[i].render, Render::Opaque);
        i += 1;
    }
    t
};

pub const CUTOUT: [bool; TABLE_LEN] = {
    let mut t = [false; TABLE_LEN];
    let mut i = 0;
    while i < BLOCK_COUNT {
        t[i] = matches!(DEFS[i].render, Render::Cutout);
        i += 1;
    }
    t
};

/// Blocks the chunk mesher draws as cubes (opaque, cutout, liquid or translucent).
pub const MESHED: [bool; TABLE_LEN] = {
    let mut t = [false; TABLE_LEN];
    let mut i = 0;
    while i < BLOCK_COUNT {
        t[i] = matches!(DEFS[i].render, Render::Opaque | Render::Cutout | Render::Liquid | Render::Translucent);
        i += 1;
    }
    t
};

/// Translucent cubes (stained glass): drawn blended after opaque and boxes; adjacent equal blocks hide inner faces.
pub const TRANSLUCENT: [bool; TABLE_LEN] = {
    let mut t = [false; TABLE_LEN];
    let mut i = 0;
    while i < BLOCK_COUNT {
        t[i] = matches!(DEFS[i].render, Render::Translucent);
        i += 1;
    }
    t
};

/// Liquids: drawn translucent in their own mesh range, never against each other.
pub const LIQUID: [bool; TABLE_LEN] = {
    let mut t = [false; TABLE_LEN];
    let mut i = 0;
    while i < BLOCK_COUNT {
        t[i] = matches!(DEFS[i].render, Render::Liquid);
        i += 1;
    }
    t
};

/// Blocks the chunk mesher draws as crossed quads.
pub const PLANT: [bool; TABLE_LEN] = {
    let mut t = [false; TABLE_LEN];
    let mut i = 0;
    while i < BLOCK_COUNT {
        t[i] = matches!(DEFS[i].render, Render::Plant);
        i += 1;
    }
    t
};

pub const SOLID: [bool; TABLE_LEN] = {
    let mut t = [false; TABLE_LEN];
    let mut i = 0;
    while i < BLOCK_COUNT {
        t[i] = DEFS[i].solid;
        i += 1;
    }
    t
};

/// Blocks that give light (`BlockDef::light` names a `light::SOURCES` row).
pub const SHINES: [bool; TABLE_LEN] = {
    let mut t = [false; TABLE_LEN];
    let mut i = 0;
    while i < BLOCK_COUNT {
        t[i] = DEFS[i].light != 0;
        i += 1;
    }
    t
};

pub const FACE_TEX: [[u16; 6]; TABLE_LEN] = {
    let mut t = [[0u16; 6]; TABLE_LEN];
    let mut i = 0;
    while i < BLOCK_COUNT {
        t[i] = DEFS[i].faces;
        i += 1;
    }
    t
};

/// Per block and face, the first of the layer's alternates (`tex::alternates`), or 0.
pub const ALT_TEX: [[u16; 6]; TABLE_LEN] = {
    let mut t = [[0u16; 6]; TABLE_LEN];
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
