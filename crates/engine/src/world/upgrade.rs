//! Upgrading a world's `deposits` generator layer in place (`World::upgrade_deposits`): updates the
//! generator, clears unedited caches, and regenerates any loaded unedited chunks whose blocks differ
//! under the new deposit rules while leaving every edited chunk untouched.
//!
//! Invariant: core determinism is preserved because edited chunks stay untouched and unedited chunks
//! (loaded or unloaded) both regenerate from the same upgraded `WorldGen`.

use crate::math::IVec3;

use super::World;

impl World {
    /// Upgrades the generator's `deposits` layer to `to` and refreshes unedited loaded chunks.
    pub fn upgrade_deposits(&mut self, to: u32) {
        if self.generator.layers().deposits == to {
            return;
        }
        self.generator.set_deposits_version(to);
        self.generated.clear();
        let mut changed = Vec::new();
        let positions: Vec<IVec3> = self.chunks.iter().filter(|(_, e)| !e.chunk.modified).map(|(&p, _)| p).collect();
        for p in positions {
            let fresh = self.generator.generate(p);
            if let Some(e) = self.chunks.get_mut(&p) {
                if !fresh.same_blocks(&e.chunk) {
                    e.chunk = fresh;
                    changed.push(p);
                }
            }
        }
        if !changed.is_empty() {
            self.light_cache.clear();
            self.light_version = self.light_version.wrapping_add(1);
            self.dirty_around_chunks(&changed);
            self.mesh_queue_stale = true;
        }
    }

    /// Marks every loaded chunk in the 3×3 neighbourhood at or below each `changed` chunk dirty so
    /// AO, face culling and sky shadows rebuild cleanly.
    pub(super) fn dirty_around_chunks(&mut self, changed: &[IVec3]) {
        for &p in changed {
            for dz in -1..=1 {
                for y in 0..=p.y + 1 {
                    for dx in -1..=1 {
                        let q = IVec3::new(p.x + dx, y, p.z + dz);
                        if self.chunks.contains_key(&q) {
                            self.dirty.insert(q);
                        }
                    }
                }
            }
        }
    }
}
