//! Block registry. Block ids double as item ids until the item/recipe layer arrives.

pub type BlockId = u8;

pub const AIR: BlockId = 0;
pub const STONE: BlockId = 1;
pub const DIRT: BlockId = 2;
pub const GRASS: BlockId = 3;
pub const SAND: BlockId = 4;
pub const LOG: BlockId = 5;
pub const LEAVES: BlockId = 6;
pub const COAL_ORE: BlockId = 7;
pub const IRON_ORE: BlockId = 8;
pub const COPPER_ORE: BlockId = 9;
pub const BEDROCK: BlockId = 10;
pub const BLOCK_COUNT: usize = 11;

/// Texture array layers. Order must match `textures::pixel`.
pub mod tex {
    pub const STONE: u16 = 0;
    pub const DIRT: u16 = 1;
    pub const GRASS_TOP: u16 = 2;
    pub const GRASS_SIDE: u16 = 3;
    pub const SAND: u16 = 4;
    pub const LOG_SIDE: u16 = 5;
    pub const LOG_TOP: u16 = 6;
    pub const LEAVES: u16 = 7;
    pub const COAL_ORE: u16 = 8;
    pub const IRON_ORE: u16 = 9;
    pub const COPPER_ORE: u16 = 10;
    pub const BEDROCK: u16 = 11;
    pub const COUNT: usize = 12;
}

/// Face order used everywhere (mesher, shaders, textures): +X, -X, +Y, -Y, +Z, -Z.
pub const FACE_TOP: usize = 2;
pub const FACE_BOTTOM: usize = 3;
pub const FACE_SIDE: usize = 0;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Render {
    /// Not drawn at all.
    None,
    /// Full cube that hides the faces of its neighbours.
    Opaque,
    /// Alpha-tested cube (leaves). Drawn in a separate pass so the opaque shader never uses `discard`.
    Cutout,
}

pub struct BlockDef {
    pub name: &'static str,
    pub render: Render,
    pub solid: bool,
    /// Seconds to break by hand; negative means unbreakable.
    pub break_time: f32,
    /// Texture layer per face (+X, -X, +Y, -Y, +Z, -Z).
    pub faces: [u16; 6],
    /// Item dropped when broken.
    pub drop: BlockId,
    /// Sound material used for digging, breaking, placing and footsteps (see [`sound`]).
    pub sound: u8,
}

/// Sound materials. Must match `MATERIALS` in `web/src/audio.ts`.
pub mod sound {
    pub const STONE: u8 = 0;
    pub const DIRT: u8 = 1;
    pub const GRASS: u8 = 2;
    pub const SAND: u8 = 3;
    pub const WOOD: u8 = 4;
    pub const LEAVES: u8 = 5;
}

const fn all(t: u16) -> [u16; 6] {
    [t; 6]
}

const fn pillar(side: u16, top: u16, bottom: u16) -> [u16; 6] {
    [side, side, top, bottom, side, side]
}

const fn cube(name: &'static str, break_time: f32, faces: [u16; 6], drop: BlockId, sound: u8) -> BlockDef {
    BlockDef { name, render: Render::Opaque, solid: true, break_time, faces, drop, sound }
}

const DEFS: [BlockDef; BLOCK_COUNT] = [
    BlockDef {
        name: "Air",
        render: Render::None,
        solid: false,
        break_time: 0.0,
        faces: all(0),
        drop: AIR,
        sound: sound::STONE,
    },
    cube("Stone", 1.1, all(tex::STONE), STONE, sound::STONE),
    cube("Dirt", 0.45, all(tex::DIRT), DIRT, sound::DIRT),
    cube("Grass", 0.5, pillar(tex::GRASS_SIDE, tex::GRASS_TOP, tex::DIRT), DIRT, sound::GRASS),
    cube("Sand", 0.45, all(tex::SAND), SAND, sound::SAND),
    cube("Log", 0.9, pillar(tex::LOG_SIDE, tex::LOG_TOP, tex::LOG_TOP), LOG, sound::WOOD),
    BlockDef {
        name: "Leaves",
        render: Render::Cutout,
        solid: true,
        break_time: 0.2,
        faces: all(tex::LEAVES),
        drop: LEAVES,
        sound: sound::LEAVES,
    },
    cube("Coal Ore", 1.4, all(tex::COAL_ORE), COAL_ORE, sound::STONE),
    cube("Iron Ore", 1.6, all(tex::IRON_ORE), IRON_ORE, sound::STONE),
    cube("Copper Ore", 1.6, all(tex::COPPER_ORE), COPPER_ORE, sound::STONE),
    cube("Bedrock", -1.0, all(tex::BEDROCK), BEDROCK, sound::STONE),
];

pub static BLOCK_DEFS: [BlockDef; BLOCK_COUNT] = DEFS;

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

#[inline]
pub fn def(id: BlockId) -> &'static BlockDef {
    BLOCK_DEFS.get(id as usize).unwrap_or(&BLOCK_DEFS[AIR as usize])
}

#[inline]
pub fn is_placeable(id: BlockId) -> bool {
    id != AIR && (id as usize) < BLOCK_COUNT
}
