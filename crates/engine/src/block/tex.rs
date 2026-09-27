//! Texture array layers, one per `textures::pixel` pattern. Append new layers at the end, never
//! renumber (the art branch appends too; the second to merge renumbers its own).
//!
//! Alternates: the layers in `WITH_ALTERNATES` have three more looks each, stored in that order from
//! `FIRST_ALT`; the mesher gives each block one of the four, chosen by its position, so ore seams and
//! canopies don't repeat. `textures` draws an alternate with its base's pattern (`look`). To add one:
//! append it to `WITH_ALTERNATES` and give it three layers (only possible before the next group).

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
pub const AVATAR_SUIT: u16 = 47;
pub const AVATAR_SKIN: u16 = 48;
pub const AVATAR_HELMET: u16 = 49;
pub const AVATAR_VISOR: u16 = 50;
pub const STONE_PICKAXE: u16 = 51;
pub const STONE_AXE: u16 = 52;
pub const STONE_SHOVEL: u16 = 53;
pub const IRON_PICKAXE: u16 = 54;
pub const IRON_AXE: u16 = 55;
pub const IRON_SHOVEL: u16 = 56;
pub const SAPLING: u16 = 57;
pub const GRANITE: u16 = 58;
pub const SANDSTONE: u16 = 59;
pub const BASALT: u16 = 60;
pub const LIMESTONE: u16 = 61;
pub const QUARTZ_ORE: u16 = 62;
pub const GLASS: u16 = 63;
/// The stained dirt under each surface hint (its bottom face).
pub const RUSTY_SOIL: u16 = 64;
pub const DARK_SOIL: u16 = 65;
pub const GREEN_SOIL: u16 = 66;
pub const PALE_SOIL: u16 = 67;
pub const SCANNER: u16 = 68;
pub const CORE_DRILL: u16 = 69;
pub const IRON_ROD: u16 = 70;
pub const SCREW: u16 = 71;
pub const FLASK_GLASS: u16 = 72;
/// Layers 73..91: three alternates for each of `WITH_ALTERNATES` (see the module header).
pub const FIRST_ALT: u16 = 73;
/// Surface hints over grass: a grass top and grass edge a shade off the plain ones.
pub const RUSTY_GRASS_TOP: u16 = 91;
pub const DARK_GRASS_TOP: u16 = 92;
pub const GREEN_GRASS_TOP: u16 = 93;
pub const PALE_GRASS_TOP: u16 = 94;
pub const RUSTY_GRASS_SIDE: u16 = 95;
pub const DARK_GRASS_SIDE: u16 = 96;
pub const GREEN_GRASS_SIDE: u16 = 97;
pub const PALE_GRASS_SIDE: u16 = 98;
/// Surface hints over sand.
pub const RUSTY_SAND: u16 = 99;
pub const DARK_SAND: u16 = 100;
pub const GREEN_SAND: u16 = 101;
pub const PALE_SAND: u16 = 102;
/// Tool and device model materials: a planed wooden handle and forged steel.
pub const HANDLE: u16 = 103;
pub const STEEL: u16 = 104;
/// The lamp block (the `LAMP_*` layers above are machine status lights).
pub const LAMP: u16 = 105;
/// Water (`block::WATER`).
pub const WATER: u16 = 106;
/// The quarry's status light while it waits for its pit to be pumped dry.
pub const LAMP_BLUE: u16 = 107;
pub const COUNT: usize = 108;

/// How many alternates a layer with any has.
pub const ALTERNATES: u16 = 3;
/// The layers with alternates, in the order their alternates are stored.
pub const WITH_ALTERNATES: [u16; 6] = [COAL_ORE, IRON_ORE, COPPER_ORE, LIMESTONE, QUARTZ_ORE, LEAVES];

/// The first of `layer`'s alternates, or 0 when it has none.
pub const fn alternates(layer: u16) -> u16 {
    let mut i = 0;
    while i < WITH_ALTERNATES.len() {
        if WITH_ALTERNATES[i] == layer {
            return FIRST_ALT + i as u16 * ALTERNATES;
        }
        i += 1;
    }
    0
}

/// The layer whose pattern `layer` draws and which look of it this is (0 = the layer itself).
pub const fn look(layer: u16) -> (u16, u16) {
    let end = FIRST_ALT + WITH_ALTERNATES.len() as u16 * ALTERNATES;
    if layer < FIRST_ALT || layer >= end {
        return (layer, 0);
    }
    let i = layer - FIRST_ALT;
    (WITH_ALTERNATES[(i / ALTERNATES) as usize], i % ALTERNATES + 1)
}
