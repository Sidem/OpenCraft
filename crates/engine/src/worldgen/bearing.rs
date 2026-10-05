//! Bearings to far ground (Milestone 9): which way to travel to reach a biome that holds an ore, for an ore that
//! is not within a scanner's range (bauxite lies only far from spawn, in deserts and basalt fields).
//!
//! Invariants: a query, a pure function of the seed, the version and the two coordinates (it reads the generator's
//! height and biome fields, never chunks, so unexplored ground answers like explored ground and the state hash
//! never moves). It samples a coarse grid in square rings round the start, nearest ring first, so the answer is
//! the nearest sampled column (to within `STEP`) whose biome holds the ore there. Worlds before version 2 had no
//! biome ores, and the extra ores exist from their own versions (`WorldGen::has_ore`): those answer `None`. To tune: `STEP` and `MAX_RING`.

use crate::block::BlockId;
use crate::math::IVec3;

use super::WorldGen;

/// Blocks between sampled columns.
const STEP: i32 = 48;
/// Rings searched: 64 × 48 = 3,072 blocks out.
const MAX_RING: i32 = 64;

/// The nearest ground that can hold an ore: a column (x, z) and its distance from where the search began.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Bearing {
    pub x: i32,
    pub z: i32,
    /// Blocks, horizontally. 0 when the starting column itself qualifies.
    pub distance: i32,
}

impl WorldGen {
    /// The nearest sampled ground at or round (x, z) whose biome can hold deposits of `ore`; `None` for a world
    /// that has no such ground or no biome ores, or within `MAX_RING` rings.
    pub fn bearing_to(&self, ore: BlockId, x: i32, z: i32) -> Option<Bearing> {
        if self.version < 2 || !self.has_ore(ore) {
            return None;
        }
        let holds = |px: i32, pz: i32| {
            let biome = self.biome_at(px, pz, self.height_at(px, pz));
            self.holds_ore(ore, biome, IVec3::new(px, 0, pz))
        };
        let mut best: Option<(i64, i32, i32)> = None;
        let mut last = MAX_RING;
        let mut ring = 0;
        while ring <= last {
            for (i, j) in ring_cells(ring) {
                let (px, pz) = (x + i * STEP, z + j * STEP);
                let d2 = (i as i64 * STEP as i64).pow(2) + (j as i64 * STEP as i64).pow(2);
                if best.is_none_or(|b| d2 < b.0) && holds(px, pz) {
                    best = Some((d2, px, pz));
                }
            }
            // A hit in ring r lies within r·√2 steps, so rings past 1.5·r cannot be nearer.
            if best.is_some() && last == MAX_RING {
                last = (ring + ring / 2 + 1).min(MAX_RING);
            }
            ring += 1;
        }
        best.map(|(d2, x, z)| Bearing { x, z, distance: (d2 as f64).sqrt().round() as i32 })
    }
}

/// The cells at Chebyshev distance `ring` from the origin (just the origin for 0).
fn ring_cells(ring: i32) -> impl Iterator<Item = (i32, i32)> {
    let side = 2 * ring + 1;
    (0..side * side).map(move |k| (k % side - ring, k / side - ring)).filter(move |c| c.0.abs().max(c.1.abs()) == ring)
}

#[cfg(test)]
mod tests;
