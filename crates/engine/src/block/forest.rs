//! The forest's blocks (generator version 9, `worldgen/trees.rs`) and what groups them: logs, leaves and saplings
//! of every tree species. Birch, snag and pine blocks are generated, never held: they drop the oak's log and leaves,
//! so every log still saws into the same planks. Cactus and the two new saplings are items like any block.
//! To add a species: its blocks here (a row each in `DEFS`), a `Species` row in `worldgen/trees.rs`, and its
//! blocks in `is_log` / `is_leaves` / `is_sapling` / `sapling_of`.

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
pub fn is_log(b: BlockId) -> bool {
    matches!(b, LOG | BIRCH_LOG | SNAG_LOG)
}

/// A leaf block of any species.
#[inline]
pub fn is_leaves(b: BlockId) -> bool {
    matches!(b, LEAVES | BIRCH_LEAVES | PINE_LEAVES)
}

/// Any block a tree is made of, a cactus included: what terraforming and towns treat as growth to clear.
#[inline]
pub fn is_tree(b: BlockId) -> bool {
    is_log(b) || is_leaves(b) || b == CACTUS
}

/// A sapling of any species (plants, grows into that species' tree).
#[inline]
pub fn is_sapling(b: BlockId) -> bool {
    matches!(b, SAPLING | BIRCH_SAPLING | PINE_SAPLING)
}

/// The sapling a leaf block leaves behind.
#[inline]
pub fn sapling_of(leaves: BlockId) -> BlockId {
    match leaves {
        BIRCH_LEAVES => BIRCH_SAPLING,
        PINE_LEAVES => PINE_SAPLING,
        _ => SAPLING,
    }
}
