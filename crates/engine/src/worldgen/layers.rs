//! World generator layers: each of the four independent aspects of terrain generation keeps its own
//! version number (`1..=WORLDGEN_VERSION`), so an existing world can adopt newer underground deposits
//! without shifting its ground, water or surface cover.
//!
//! - `ground`: landforms (`landform.rs`, v7/v8/v11), biomes & rock provinces (`biome.rs`, v2), caves (`caves.rs`, v6).
//! - `water`: sea & ponds (`water.rs`, v3/v8), rivers & lakes (`rivers.rs`, v7/v8), waterfalls (`column.rs`, v8).
//! - `cover`: trees (`trees.rs`, v9; `grow/`, v12; forest biomes `local_biome_at`, v12) and ruins (`ruins.rs`, v9/v10).
//! - `deposits`: ore placement (`ore.rs`, v2/v3; `geology.rs`, v5/v6/v8; `strata.rs`, v4/v6; `bearing.rs`) and future
//!   additions (such as gas fields) placed on their own grid without disturbing existing ores.
//!
//! Invariants:
//! - Every layer version is in `1..=WORLDGEN_VERSION`.
//! - A save written before `LAYERS_SAVED` (save version 81) reads all four layers as its single `worldgen` version
//!   (`Layers::uniform(v)`), producing byte-identical chunks to before.
//! - From `DEPOSITS_SAFE_FROM` (v6) on, upgrading `deposits` never alters existing `ore_at` weights or `DepositKey`s.
//!   To bump deposits in a future release: raise `WORLDGEN_VERSION` and `DEPOSITS_LATEST` together.

use crate::bytes::ByteWriter;

use super::WORLDGEN_VERSION;

/// The save format version that began recording the four generator layers in the header.
pub const LAYERS_SAVED: u32 = 81;
/// The first deposits version whose existing ore deposits remain identical under all later upgrades
/// (v6 froze `ore_at` / `EXTRAS` / `strata`; later deposit additions use their own independent placement).
pub const DEPOSITS_SAFE_FROM: u32 = 6;
/// The newest generator version that changed the `deposits` layer (v13: independent natural gas fields in
/// `worldgen/gas.rs`). Bump alongside `WORLDGEN_VERSION` whenever new deposits ship.
pub const DEPOSITS_LATEST: u32 = 13;

/// The four independent generator layer versions of a world.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Layers {
    pub ground: u32,
    pub water: u32,
    pub cover: u32,
    pub deposits: u32,
}

impl Layers {
    /// Every layer at the newest generator version (for new worlds).
    pub const LATEST: Self = Self::uniform(WORLDGEN_VERSION);

    /// All four layers set to the same generator version (legacy saves before `LAYERS_SAVED`).
    pub const fn uniform(version: u32) -> Self {
        Self { ground: version, water: version, cover: version, deposits: version }
    }

    /// A copy of `self` with only the `deposits` layer changed.
    pub const fn with_deposits(self, deposits: u32) -> Self {
        Self { deposits, ..self }
    }

    /// Whether every layer version is at least 1 and at most `WORLDGEN_VERSION`.
    pub const fn all(self) -> [u32; 4] {
        [self.ground, self.water, self.cover, self.deposits]
    }

    /// Whether any layer is newer than this build supports.
    pub fn any_newer(self) -> bool {
        self.all().iter().any(|&v| v > WORLDGEN_VERSION)
    }

    /// Whether any layer is zero (invalid).
    pub fn any_zero(self) -> bool {
        self.all().contains(&0)
    }

    /// Canonical bytes for `Sim::state_hash`.
    pub fn write_state(&self, w: &mut ByteWriter) {
        for v in self.all() {
            w.u32(v);
        }
    }
}

#[cfg(test)]
mod tests;
