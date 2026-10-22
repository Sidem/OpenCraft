//! Cleared ground cover (U3, the sickle): the columns where the player cut the plants away. Ground cover itself is
//! drawn from a hash by the mesher (`mesher/cover.rs`) and never saved, so a cut needs state of its own, and it is core
//! state: `Action::ClearCover` sets it at a tick, `World::write_state` saves and hashes it, so a save and every co-op
//! peer agree on which columns are bare.
//!
//! One bit per world column, kept per chunk column that was ever touched: 32 rows of 32 bits (row = local z, bit =
//! local x), 128 bytes, in a `Vec` sorted by chunk column so the bytes are canonical without sorting. A chunk column
//! never cleared has no entry and meshes exactly as before (`rows` answers an empty slice). The mesher reads the rows
//! through `Cover::cleared`, one bit test per candidate column.
//!
//! Invariants: a bit is never cleared again. Cut plants do not drop anything and do not grow back; the bit belongs to the
//! column, not to its ground, so it also stays when the ground block is broken or replaced (a new floor there stays
//! bare, which is what someone who cleared the spot wants). The set holds columns whether or not they bore cover:
//! the core cannot see the mesher's light or biome rules, and a bit on a bare column changes nothing.
//! To widen the patch: `PATCH_RADIUS`.

use crate::block::AIR;
use crate::bytes::{ByteReader, ByteWriter};
use crate::chunk::{CHUNK_MASK, CHUNK_SHIFT, CHUNK_SIZE};
use crate::math::IVec3;
use crate::mesher::bears_cover;
use crate::worldgen::{WORLD_HEIGHT, WORLD_HEIGHT_CHUNKS};

use super::World;

/// Columns a clear reaches each way from its centre: 1 is a 3 × 3 patch.
pub const PATCH_RADIUS: i32 = 1;
/// The save version that added the cleared set after the edited chunks (older saves have none).
pub const COVER_SAVED: u32 = 55;
const ROWS: usize = CHUNK_SIZE as usize;

/// Which columns are cleared, by chunk column.
#[derive(Clone, Default, PartialEq)]
pub struct ClearedCover {
    columns: Vec<Column>,
}

#[derive(Clone, PartialEq)]
struct Column {
    cx: i32,
    cz: i32,
    rows: [u32; ROWS],
}

impl ClearedCover {
    /// The rows of chunk column (`cx`, `cz`): empty when nothing there was ever cleared.
    #[inline]
    pub fn rows(&self, cx: i32, cz: i32) -> &[u32] {
        match self.find(cx, cz) {
            Ok(i) => &self.columns[i].rows,
            Err(_) => &[],
        }
    }

    /// Whether world column (`x`, `z`) is cleared.
    pub fn is_cleared(&self, x: i32, z: i32) -> bool {
        let row = self.rows(x >> CHUNK_SHIFT, z >> CHUNK_SHIFT).get((z & CHUNK_MASK) as usize);
        row.is_some_and(|row| row >> (x & CHUNK_MASK) & 1 != 0)
    }

    /// Clears world column (`x`, `z`); false when it already was.
    fn set(&mut self, x: i32, z: i32) -> bool {
        let (cx, cz) = (x >> CHUNK_SHIFT, z >> CHUNK_SHIFT);
        let i = self.find(cx, cz).unwrap_or_else(|at| {
            self.columns.insert(at, Column { cx, cz, rows: [0; ROWS] });
            at
        });
        let row = &mut self.columns[i].rows[(z & CHUNK_MASK) as usize];
        let bit = 1u32 << (x & CHUNK_MASK);
        let fresh = *row & bit == 0;
        *row |= bit;
        fresh
    }

    fn find(&self, cx: i32, cz: i32) -> Result<usize, usize> {
        self.columns.binary_search_by_key(&(cx, cz), |c| (c.cx, c.cz))
    }

    /// Core state: the touched chunk columns in order, each with its rows.
    pub fn write_state(&self, w: &mut ByteWriter) {
        w.count(self.columns.len());
        for c in &self.columns {
            w.i32(c.cx);
            w.i32(c.cz);
            for row in c.rows {
                w.u32(row);
            }
        }
    }

    /// Reads what `write_state` wrote. Refuses an unsorted list or an empty column, which `write_state` never makes.
    pub fn read_state(r: &mut ByteReader) -> Option<Self> {
        let mut columns: Vec<Column> = Vec::new();
        for _ in 0..r.count()? {
            let (cx, cz) = (r.i32()?, r.i32()?);
            let mut rows = [0; ROWS];
            for row in &mut rows {
                *row = r.u32()?;
            }
            let ordered = columns.last().is_none_or(|l| (l.cx, l.cz) < (cx, cz));
            if !ordered || rows.iter().all(|&row| row == 0) {
                return None;
            }
            columns.push(Column { cx, cz, rows });
        }
        Some(ClearedCover { columns })
    }
}

impl World {
    /// Whether world column (`x`, `z`) is cleared.
    #[cfg(test)]
    pub fn is_cover_cleared(&self, x: i32, z: i32) -> bool {
        self.cleared.is_cleared(x, z)
    }

    /// Whether every column of the patch around `centre`'s column is cleared already (the hands skip such a swing).
    pub fn patch_cleared(&self, centre: IVec3) -> bool {
        (centre.z - PATCH_RADIUS..=centre.z + PATCH_RADIUS)
            .all(|z| (centre.x - PATCH_RADIUS..=centre.x + PATCH_RADIUS).all(|x| self.cleared.is_cleared(x, z)))
    }

    /// Clears the plants of the patch of columns around `centre`'s column (core state: only `Sim` actions call this).
    /// Returns whether any column was newly cleared. Loaded chunks that draw cover in a newly cleared column are
    /// remeshed within the streaming budget; the rest draw it cleared when they load.
    pub fn clear_cover(&mut self, centre: IVec3) -> bool {
        if centre.y < 0 || centre.y >= WORLD_HEIGHT {
            return false;
        }
        let mut any = false;
        for z in centre.z - PATCH_RADIUS..=centre.z + PATCH_RADIUS {
            for x in centre.x - PATCH_RADIUS..=centre.x + PATCH_RADIUS {
                if self.cleared.set(x, z) {
                    any = true;
                    self.dirty_cover_chunks(x, z);
                }
            }
        }
        if any {
            self.mesh_queue_stale = true;
        }
        any
    }

    /// Marks the loaded chunks of column (`x`, `z`) that hold a cell cover may stand in (air over soil, sand or rock).
    fn dirty_cover_chunks(&mut self, x: i32, z: i32) {
        for cy in 0..WORLD_HEIGHT_CHUNKS {
            let c = IVec3::new(x >> CHUNK_SHIFT, cy, z >> CHUNK_SHIFT);
            if !self.chunks.contains_key(&c) || self.dirty.contains(&c) {
                continue;
            }
            let (lo, hi) = (cy << CHUNK_SHIFT, (cy + 1) << CHUNK_SHIFT);
            let bears = |y: i32| {
                self.get_block(IVec3::new(x, y, z)) == Some(AIR)
                    && self.get_block(IVec3::new(x, y - 1, z)).is_some_and(bears_cover)
            };
            if (lo.max(1)..hi).any(bears) {
                self.dirty.insert(c);
            }
        }
    }
}

#[cfg(test)]
mod tests;
