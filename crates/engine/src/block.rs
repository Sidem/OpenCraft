//! Block registry: ids, one `BlockDef` row per block in `DEFS`, texture layers (`tex`), sound
//! materials (`sound`), and flat lookup tables (`OPAQUE`, `SOLID`, `FACE_TEX`…) for hot loops.
//! Every block is also the item with the same id (`item.rs`); blocks that should never be put back
//! into the world (ore, bedrock) are simply not placeable.
//!
//! To add a block: append an id constant (never renumber; ids will be saved) and bump
//! `BLOCK_COUNT`, add its `DEFS` row (`cube`, `ore` or `machine`), and any new texture layers in
//! `tex` with their patterns in `textures::pixel`. A machine also needs factory code (docs/CODEMAP.md).

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
/// What an ore block turns into once a miner has drawn its share out of the deposit.
pub const SPENT_ROCK: BlockId = 11;
pub const BELT: BlockId = 12;
pub const MINER: BlockId = 13;
pub const STORAGE: BlockId = 14;
pub const SMELTER: BlockId = 15;
pub const CONSTRUCTOR: BlockId = 16;
pub const SPLITTER: BlockId = 17;
pub const FILTER: BlockId = 18;
pub const RAMP_UP: BlockId = 19;
pub const RAMP_DOWN: BlockId = 20;
pub const LIFT: BlockId = 21;
pub const UNDERPASS_IN: BlockId = 22;
pub const UNDERPASS_OUT: BlockId = 23;
pub const GENERATOR: BlockId = 24;
pub const POLE: BlockId = 25;
pub const LAB: BlockId = 26;
pub const MINER_MK2: BlockId = 27;
pub const FAST_BELT: BlockId = 28;
pub const BLOCK_COUNT: usize = 29;

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
    pub const SPENT_ROCK: u16 = 12;
    pub const BELT_TOP: u16 = 13;
    pub const FRAME: u16 = 14;
    pub const MINER_SIDE: u16 = 15;
    pub const MINER_TOP: u16 = 16;
    pub const DRILL: u16 = 17;
    pub const BOX_SIDE: u16 = 18;
    pub const BOX_TOP: u16 = 19;
    pub const LAMP_GREEN: u16 = 20;
    pub const LAMP_YELLOW: u16 = 21;
    pub const LAMP_RED: u16 = 22;
    pub const IRON_INGOT: u16 = 23;
    pub const COPPER_INGOT: u16 = 24;
    pub const SMELTER_SIDE: u16 = 25;
    pub const SMELTER_TOP: u16 = 26;
    pub const IRON_PLATE: u16 = 27;
    pub const COPPER_WIRE: u16 = 28;
    pub const CONSTRUCTOR_SIDE: u16 = 29;
    pub const CONSTRUCTOR_TOP: u16 = 30;
    pub const SPLITTER_TOP: u16 = 31;
    pub const FILTER_TOP: u16 = 32;
    pub const RAMP_UP_SIDE: u16 = 33;
    pub const RAMP_DOWN_SIDE: u16 = 34;
    pub const LIFT_SIDE: u16 = 35;
    pub const UNDERPASS_IN_SIDE: u16 = 36;
    pub const UNDERPASS_OUT_SIDE: u16 = 37;
    pub const GENERATOR_SIDE: u16 = 38;
    pub const GENERATOR_TOP: u16 = 39;
    pub const POLE_SIDE: u16 = 40;
    pub const LAB_SIDE: u16 = 41;
    pub const LAB_TOP: u16 = 42;
    pub const RED_PACK: u16 = 43;
    pub const GREEN_PACK: u16 = 44;
    pub const MINER_MK2_SIDE: u16 = 45;
    pub const FAST_BELT_TOP: u16 = 46;
    pub const COUNT: usize = 47;
}

/// Face order used everywhere (mesher, shaders, textures): +X, -X, +Y, -Y, +Z, -Z.
pub const FACE_TOP: usize = 2;
pub const FACE_BOTTOM: usize = 3;
pub const FACE_SIDE: usize = 0;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Render {
    /// Not drawn by the chunk mesher (air, or machines the host draws as instanced models).
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
    /// Item dropped when broken (the block item with this id).
    pub drop: BlockId,
    /// Sound material used for digging, breaking, placing and footsteps (see [`sound`]).
    pub sound: u8,
    /// Whether the item can be put back into the world.
    pub placeable: bool,
}

/// Sound materials. Must match `MATERIALS` in `web/src/audio/settings.ts`.
pub mod sound {
    pub const STONE: u8 = 0;
    pub const DIRT: u8 = 1;
    pub const GRASS: u8 = 2;
    pub const SAND: u8 = 3;
    pub const WOOD: u8 = 4;
    pub const LEAVES: u8 = 5;
    pub const METAL: u8 = 6;
}

const fn all(t: u16) -> [u16; 6] {
    [t; 6]
}

const fn pillar(side: u16, top: u16, bottom: u16) -> [u16; 6] {
    [side, side, top, bottom, side, side]
}

const fn cube(name: &'static str, break_time: f32, faces: [u16; 6], drop: BlockId, sound: u8) -> BlockDef {
    BlockDef { name, render: Render::Opaque, solid: true, break_time, faces, drop, sound, placeable: true }
}

/// Ore stays in the ground: mining it by hand yields a handful of ore items that can't be placed.
const fn ore(name: &'static str, faces: [u16; 6], drop: BlockId) -> BlockDef {
    BlockDef { placeable: false, ..cube(name, 1.6, faces, drop, sound::STONE) }
}

/// A machine drawn by the host as an instanced model rather than by the chunk mesher.
const fn machine(name: &'static str, solid: bool, break_time: f32, faces: [u16; 6], id: BlockId) -> BlockDef {
    BlockDef { render: Render::None, solid, ..cube(name, break_time, faces, id, sound::METAL) }
}

pub(crate) const DEFS: [BlockDef; BLOCK_COUNT] = [
    BlockDef {
        name: "Air",
        render: Render::None,
        solid: false,
        break_time: 0.0,
        faces: all(0),
        drop: AIR,
        sound: sound::STONE,
        placeable: false,
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
        placeable: true,
    },
    ore("Coal Ore", all(tex::COAL_ORE), COAL_ORE),
    ore("Iron Ore", all(tex::IRON_ORE), IRON_ORE),
    ore("Copper Ore", all(tex::COPPER_ORE), COPPER_ORE),
    BlockDef { placeable: false, ..cube("Bedrock", -1.0, all(tex::BEDROCK), BEDROCK, sound::STONE) },
    cube("Spent Rock", 0.7, all(tex::SPENT_ROCK), SPENT_ROCK, sound::STONE),
    machine("Conveyor Belt", false, 0.3, pillar(tex::FRAME, tex::BELT_TOP, tex::FRAME), BELT),
    machine("Miner Mk1", true, 0.8, pillar(tex::MINER_SIDE, tex::MINER_TOP, tex::FRAME), MINER),
    cube("Storage Box", 0.7, pillar(tex::BOX_SIDE, tex::BOX_TOP, tex::BOX_TOP), STORAGE, sound::WOOD),
    BlockDef {
        sound: sound::STONE,
        ..machine("Smelter", true, 1.0, pillar(tex::SMELTER_SIDE, tex::SMELTER_TOP, tex::SMELTER_TOP), SMELTER)
    },
    machine("Constructor", true, 0.8, pillar(tex::CONSTRUCTOR_SIDE, tex::CONSTRUCTOR_TOP, tex::FRAME), CONSTRUCTOR),
    machine("Splitter", false, 0.4, pillar(tex::FRAME, tex::SPLITTER_TOP, tex::FRAME), SPLITTER),
    machine("Filter", false, 0.4, pillar(tex::FRAME, tex::FILTER_TOP, tex::FRAME), FILTER),
    machine("Belt Ramp Up", false, 0.3, pillar(tex::RAMP_UP_SIDE, tex::BELT_TOP, tex::FRAME), RAMP_UP),
    machine("Belt Ramp Down", false, 0.3, pillar(tex::RAMP_DOWN_SIDE, tex::BELT_TOP, tex::FRAME), RAMP_DOWN),
    machine("Belt Lift", false, 0.4, pillar(tex::LIFT_SIDE, tex::FRAME, tex::FRAME), LIFT),
    machine("Underpass Entry", false, 0.4, pillar(tex::UNDERPASS_IN_SIDE, tex::BELT_TOP, tex::FRAME), UNDERPASS_IN),
    machine("Underpass Exit", false, 0.4, pillar(tex::UNDERPASS_OUT_SIDE, tex::BELT_TOP, tex::FRAME), UNDERPASS_OUT),
    machine("Coal Generator", true, 0.8, pillar(tex::GENERATOR_SIDE, tex::GENERATOR_TOP, tex::FRAME), GENERATOR),
    machine("Power Pole", false, 0.3, pillar(tex::POLE_SIDE, tex::FRAME, tex::FRAME), POLE),
    machine("Research Lab", true, 0.8, pillar(tex::LAB_SIDE, tex::LAB_TOP, tex::FRAME), LAB),
    machine("Miner Mk2", true, 0.8, pillar(tex::MINER_MK2_SIDE, tex::MINER_TOP, tex::FRAME), MINER_MK2),
    machine("Fast Belt", false, 0.3, pillar(tex::FRAME, tex::FAST_BELT_TOP, tex::FRAME), FAST_BELT),
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

/// Blocks the chunk mesher draws (opaque or cutout).
pub const MESHED: [bool; 256] = {
    let mut t = [false; 256];
    let mut i = 0;
    while i < BLOCK_COUNT {
        t[i] = !matches!(DEFS[i].render, Render::None);
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
pub fn is_ore(id: BlockId) -> bool {
    matches!(id, COAL_ORE | IRON_ORE | COPPER_ORE)
}

/// Short resource name used in deposit names ("Iron vein").
pub fn ore_label(id: BlockId) -> &'static str {
    match id {
        COAL_ORE => "Coal",
        IRON_ORE => "Iron",
        COPPER_ORE => "Copper",
        _ => "Ore",
    }
}
