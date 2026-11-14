//! Liquid edits that undo themselves. A pump lifts a source out of the sea and the sea fills the hole a few ticks later
//! (`sim/water.rs`), again and again; each edit would mark every chunk in light's reach for lighting and meshing
//! (water and air light differently: 27 chunks), and nothing would show in the end.
//!
//! An edit made `later` that touches liquid waits here for [`SETTLE`] seconds of frame time (`World::advance_clock`,
//! from `Game::update`). If the block is back to what the meshes show by then, nothing is marked; otherwise the edit
//! is passed on as one change from what the meshes show. Render cache only, never core state: the block itself changes
//! at once, and a remesh of a chunk near a waiting edit passes it on first so a mesh never keeps a half-way state.
//! To defer another kind of edit: add it to the test in `World::edit_loaded`.

use rustc_hash::FxHashMap;

use crate::block::BlockId;
use crate::math::IVec3;

use super::{chunk_of, streaming, World};

/// Seconds of frame time an edit waits: more than the 5 ticks (0.083 s) the water rules take to fill a pumped hole.
const SETTLE: f64 = 0.15;

/// A waiting edit: the block the meshes show at the cell, and when the cell last changed.
struct Waiting {
    shown: BlockId,
    at: f64,
}

#[derive(Default)]
pub(super) struct Settling {
    clock: f64,
    waiting: FxHashMap<IVec3, Waiting>,
}

impl World {
    /// Advances the clock the waiting edits are timed by (presentation time: no core state reads it).
    pub fn advance_clock(&mut self, dt: f64) {
        self.settling.clock += dt.max(0.0);
    }

    /// Whether any edit is still waiting to reach the meshes.
    #[cfg(test)]
    pub(super) fn settling_count(&self) -> usize {
        self.settling.waiting.len()
    }

    /// The block at `p` went from `old` to something else; `old` is what the meshes show unless an edit is already waiting.
    pub(super) fn settle_edit(&mut self, p: IVec3, old: BlockId) {
        let at = self.settling.clock;
        self.settling.waiting.entry(p).or_insert(Waiting { shown: old, at }).at = at;
    }

    /// Passes on the waiting edits that have waited long enough (`begin_work`, every frame).
    pub(super) fn flush_settled(&mut self) {
        if self.settling.waiting.is_empty() {
            return;
        }
        let due = self.settling.clock - SETTLE;
        let ready: Vec<IVec3> = self.settling.waiting.iter().filter(|(_, w)| w.at <= due).map(|(&p, _)| p).collect();
        self.flush(ready);
    }

    /// Passes on the waiting edits in or next to chunk `c`, which is about to be meshed from the blocks as they are.
    pub(super) fn flush_near(&mut self, c: IVec3) {
        let near = |p: &IVec3| {
            let d = chunk_of(*p) - c;
            d.x.abs() <= 1 && d.y.abs() <= 1 && d.z.abs() <= 1
        };
        let ready: Vec<IVec3> = self.settling.waiting.keys().filter(|p| near(p)).copied().collect();
        self.flush(ready);
    }

    /// Marks what the edit `old` to `b` at `p` changes for the render cache: the chunks to mesh again (at once for
    /// those its faces and corners touch if `remesh_now`; the rest of a canopy's reach (`mesher/foliage.rs`) with
    /// the queue) and the box light it can reach (`light_changed`).
    pub(super) fn invalidate(&mut self, p: IVec3, old: BlockId, b: BlockId, remesh_now: bool) {
        let near = streaming::chunks_around(p, 1);
        for c in streaming::chunks_around(p, streaming::mesh_reach(old, b)) {
            if self.chunks.contains_key(&c) {
                self.dirty.insert(c);
                if remesh_now && near.contains(&c) {
                    self.remesh(c);
                }
            }
        }
        self.light_changed(p, old, b);
        self.mesh_queue_stale = true;
    }

    fn flush(&mut self, cells: Vec<IVec3>) {
        for p in cells {
            let Some(w) = self.settling.waiting.remove(&p) else { continue };
            if let Some(now) = self.get_block(p).filter(|&b| b != w.shown) {
                self.invalidate(p, w.shown, now, false);
            }
        }
    }
}

#[cfg(test)]
mod tests;
