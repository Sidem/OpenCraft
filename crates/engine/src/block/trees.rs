//! The species of generator version 12 (`worldgen/grow/`), ids 160–181: a log, a leaf block and a sapling each for
//! pine bark, palm, cherry, baobab, acacia, redwood and eucalyptus, and bamboo (its culm is a log, its leaves leaves, a
//! shoot its sapling). Like the birch, their logs and leaves are generated, never held: logs drop the oak's log, so
//! every log saws into the same planks, and leaves the oak's leaves. Saplings are items like any block.
//! Their texture layers (`layers`) come after every other; placeholders in `textures/trees.rs`, the art pack's in
//! `tools/art`. To add a species: its blocks here, their groups in `forest.rs`, a row in `worldgen/grow/kinds.rs`,
//! a leaf style in `mesher/foliage/style.rs` and a thickness in `mesher/logs/node.rs`.

use super::make::{cube, leaves, pillar, sapling};
use super::{sound, tex, BlockDef, BlockId, LEAVES, LOG};

pub const PINE_LOG: BlockId = 160;
pub const PALM_LOG: BlockId = 161;
pub const PALM_FRONDS: BlockId = 162;
pub const PALM_SAPLING: BlockId = 163;
pub const CHERRY_LOG: BlockId = 164;
pub const CHERRY_BLOSSOM: BlockId = 165;
pub const CHERRY_SAPLING: BlockId = 166;
pub const BAOBAB_LOG: BlockId = 167;
pub const BAOBAB_LEAVES: BlockId = 168;
pub const BAOBAB_SAPLING: BlockId = 169;
pub const ACACIA_LOG: BlockId = 170;
pub const ACACIA_LEAVES: BlockId = 171;
pub const ACACIA_SAPLING: BlockId = 172;
pub const REDWOOD_LOG: BlockId = 173;
pub const REDWOOD_NEEDLES: BlockId = 174;
pub const REDWOOD_SAPLING: BlockId = 175;
pub const EUCALYPTUS_LOG: BlockId = 176;
pub const EUCALYPTUS_LEAVES: BlockId = 177;
pub const EUCALYPTUS_SAPLING: BlockId = 178;
pub const BAMBOO: BlockId = 179;
pub const BAMBOO_LEAVES: BlockId = 180;
pub const BAMBOO_SHOOT: BlockId = 181;

pub const TREES_COUNT: usize = 22;

/// The texture layers, after every other (`tex::COUNT`).
pub mod layers {
    use super::super::tex::SIGN_FACE_LAST;

    pub const PINE_SIDE: u16 = SIGN_FACE_LAST + 1;
    pub const PALM_SIDE: u16 = PINE_SIDE + 1;
    pub const PALM_FRONDS: u16 = PINE_SIDE + 2;
    pub const PALM_SAPLING: u16 = PINE_SIDE + 3;
    pub const CHERRY_SIDE: u16 = PINE_SIDE + 4;
    pub const CHERRY_BLOSSOM: u16 = PINE_SIDE + 5;
    pub const CHERRY_SAPLING: u16 = PINE_SIDE + 6;
    pub const BAOBAB_SIDE: u16 = PINE_SIDE + 7;
    pub const BAOBAB_LEAVES: u16 = PINE_SIDE + 8;
    pub const BAOBAB_SAPLING: u16 = PINE_SIDE + 9;
    pub const ACACIA_SIDE: u16 = PINE_SIDE + 10;
    pub const ACACIA_LEAVES: u16 = PINE_SIDE + 11;
    pub const ACACIA_SAPLING: u16 = PINE_SIDE + 12;
    pub const REDWOOD_SIDE: u16 = PINE_SIDE + 13;
    pub const REDWOOD_NEEDLES: u16 = PINE_SIDE + 14;
    pub const REDWOOD_SAPLING: u16 = PINE_SIDE + 15;
    pub const EUCALYPTUS_SIDE: u16 = PINE_SIDE + 16;
    pub const EUCALYPTUS_LEAVES: u16 = PINE_SIDE + 17;
    pub const EUCALYPTUS_SAPLING: u16 = PINE_SIDE + 18;
    pub const BAMBOO_SIDE: u16 = PINE_SIDE + 19;
    pub const BAMBOO_LEAVES: u16 = PINE_SIDE + 20;
    pub const BAMBOO_SHOOT: u16 = PINE_SIDE + 21;
    pub const TREE_LAYERS_END: u16 = PINE_SIDE + 22;
}

/// A generated log of one species: bark round it, the oak's rings on its ends; drops the oak's log.
const fn bark(name: &'static str, side: u16) -> BlockDef {
    BlockDef { placeable: false, ..cube(name, 0.9, pillar(side, tex::LOG_TOP, tex::LOG_TOP), LOG, sound::WOOD) }
}

/// The rows of `DEFS` from block 160 on, in id order.
pub(super) const TREES_DEFS: [BlockDef; TREES_COUNT] = [
    bark("Pine Log", layers::PINE_SIDE),
    bark("Palm Log", layers::PALM_SIDE),
    leaves("Palm Fronds", layers::PALM_FRONDS, LEAVES),
    sapling("Palm Sapling", layers::PALM_SAPLING, PALM_SAPLING),
    bark("Cherry Log", layers::CHERRY_SIDE),
    leaves("Cherry Blossom", layers::CHERRY_BLOSSOM, LEAVES),
    sapling("Cherry Sapling", layers::CHERRY_SAPLING, CHERRY_SAPLING),
    bark("Baobab Log", layers::BAOBAB_SIDE),
    leaves("Baobab Leaves", layers::BAOBAB_LEAVES, LEAVES),
    sapling("Baobab Sapling", layers::BAOBAB_SAPLING, BAOBAB_SAPLING),
    bark("Acacia Log", layers::ACACIA_SIDE),
    leaves("Acacia Leaves", layers::ACACIA_LEAVES, LEAVES),
    sapling("Acacia Sapling", layers::ACACIA_SAPLING, ACACIA_SAPLING),
    bark("Redwood Log", layers::REDWOOD_SIDE),
    leaves("Redwood Needles", layers::REDWOOD_NEEDLES, LEAVES),
    sapling("Redwood Sapling", layers::REDWOOD_SAPLING, REDWOOD_SAPLING),
    bark("Eucalyptus Log", layers::EUCALYPTUS_SIDE),
    leaves("Eucalyptus Leaves", layers::EUCALYPTUS_LEAVES, LEAVES),
    sapling("Eucalyptus Sapling", layers::EUCALYPTUS_SAPLING, EUCALYPTUS_SAPLING),
    bark("Bamboo", layers::BAMBOO_SIDE),
    leaves("Bamboo Leaves", layers::BAMBOO_LEAVES, LEAVES),
    sapling("Bamboo Shoot", layers::BAMBOO_SHOOT, BAMBOO_SHOOT),
];
