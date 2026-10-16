//! Where towns fit, for the world map while a town depot is held (presentation only): each explored tile (a chunk
//! column) gets the kind of town most of its dry columns fit by their place (`settlements::kind_at` on a `GRID`² grid), so
//! the player can see where to go before setting out. The generator never changes, so a tile is worked out once and
//! kept; at most `PER_CALL` new tiles a call keep the frame cheap. Spawn and the other towns are left to the caller
//! (they change).

use rustc_hash::FxHashMap;

use super::Atlas;
use crate::chunk::{CHUNK_SHIFT, CHUNK_SIZE};
use crate::settlements::{kind_at, KINDS};
use crate::worldgen::WorldGen;

/// Samples a side per tile.
const GRID: i32 = 4;
/// New tiles worked out per call.
const PER_CALL: usize = 24;
/// A tile no town fits.
pub(super) const NONE: u8 = u8::MAX;

/// The kind each tile fits, as worked out so far.
#[derive(Default)]
pub struct Sites {
    kinds: FxHashMap<(i32, i32), u8>,
}

impl Sites {
    /// The explored tiles in chunk columns `lo..=hi` that a town fits, as (chunk x, chunk z, kind), working out up to
    /// `PER_CALL` new ones (the rest come on later calls).
    pub fn tiles_in(&mut self, atlas: &Atlas, gen: &WorldGen, lo: (i32, i32), hi: (i32, i32)) -> Vec<(i32, i32, u8)> {
        let mut out = Vec::new();
        let mut budget = PER_CALL;
        atlas.each_tile_in(lo, hi, |cx, cz, _| {
            let kind = match self.kinds.get(&(cx, cz)) {
                Some(&k) => k,
                None if budget > 0 => {
                    budget -= 1;
                    *self.kinds.entry((cx, cz)).or_insert_with(|| tile_kind(gen, cx, cz))
                }
                None => return,
            };
            if kind != NONE {
                out.push((cx, cz, kind));
            }
        });
        out
    }
}

/// The kind most of tile (`cx`, `cz`)'s samples fit (the first kind on a tie), or `NONE`.
pub(super) fn tile_kind(gen: &WorldGen, cx: i32, cz: i32) -> u8 {
    let step = CHUNK_SIZE / GRID;
    let mut votes = [0u8; KINDS.len()];
    for j in 0..GRID {
        for i in 0..GRID {
            let (x, z) = ((cx << CHUNK_SHIFT) + i * step + step / 2, (cz << CHUNK_SHIFT) + j * step + step / 2);
            if gen.open_water(x, z).is_some() {
                continue; // no depot stands on the water
            }
            if let Some(k) = kind_at(gen, x, z) {
                votes[k as usize] += 1;
            }
        }
    }
    let best = (0..votes.len()).max_by_key(|&k| (votes[k], std::cmp::Reverse(k))).unwrap_or(0);
    if votes[best] == 0 {
        NONE
    } else {
        best as u8
    }
}
