//! The forest's blocks (generator version 9, `worldgen/trees.rs`) and what groups them: logs, leaves and saplings
//! of every tree species. Birch, snag and pine blocks are generated, never held: they drop the oak's log and leaves,
//! so every log still saws into the same planks. Cactus and the two new saplings are items like any block.
//! Generator 12's species add their blocks in `trees.rs`; their groups are here too. To add a species: its blocks in`n//! `trees.rs`, then here in `is_log` / `is_leaves` / `is_sapling` / `sapling_of`.

use super::trees::*;
use super::{BlockId, LEAVES, LOG, SAPLING};

pub const BIRCH_LOG: BlockId = 117;
pub const BIRCH_LEAVES: BlockId = 118;
pub const PINE_LEAVES: BlockId = 119;
pub const CACTUS: BlockId = 120;
/// A dead tree's trunk on basalt fields.
pub const SNAG_LOG: BlockId = 121;
pub const BIRCH_SAPLING: BlockId = 122;
pub const PINE_SAPLING: BlockId = 123;

/// A trunk block of any species: leaves hang on it and decay without one.
#[inline]
pub const fn is_log(b: BlockId) -> bool {
    matches!(b, LOG | BIRCH_LOG | SNAG_LOG)
        || matches!(
            b,
            PINE_LOG | PALM_LOG | CHERRY_LOG | BAOBAB_LOG | ACACIA_LOG | REDWOOD_LOG | EUCALYPTUS_LOG | BAMBOO
        )
}

/// A leaf block of any species.
#[inline]
pub const fn is_leaves(b: BlockId) -> bool {
    matches!(b, LEAVES | BIRCH_LEAVES | PINE_LEAVES | PALM_FRONDS | CHERRY_BLOSSOM | BAOBAB_LEAVES | ACACIA_LEAVES)
        || matches!(b, REDWOOD_NEEDLES | EUCALYPTUS_LEAVES | BAMBOO_LEAVES)
}

/// Any block a tree is made of, a cactus included: what terraforming and towns treat as growth to clear.
#[inline]
pub fn is_tree(b: BlockId) -> bool {
    is_log(b) || is_leaves(b) || b == CACTUS
}

/// A sapling of any species (plants, grows into that species' tree).
#[inline]
pub fn is_sapling(b: BlockId) -> bool {
    matches!(b, SAPLING | BIRCH_SAPLING | PINE_SAPLING | PALM_SAPLING | CHERRY_SAPLING | BAOBAB_SAPLING)
        || matches!(b, ACACIA_SAPLING | REDWOOD_SAPLING | EUCALYPTUS_SAPLING | BAMBOO_SHOOT)
}

/// The sapling a leaf block leaves behind.
#[inline]
pub fn sapling_of(leaves: BlockId) -> BlockId {
    match leaves {
        BIRCH_LEAVES => BIRCH_SAPLING,
        PINE_LEAVES => PINE_SAPLING,
        PALM_FRONDS => PALM_SAPLING,
        CHERRY_BLOSSOM => CHERRY_SAPLING,
        BAOBAB_LEAVES => BAOBAB_SAPLING,
        ACACIA_LEAVES => ACACIA_SAPLING,
        REDWOOD_NEEDLES => REDWOOD_SAPLING,
        EUCALYPTUS_LEAVES => EUCALYPTUS_SAPLING,
        BAMBOO_LEAVES => BAMBOO_SHOOT,
        _ => SAPLING,
    }
}
