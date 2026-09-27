//! Torches: a torch stands in an empty cell on top of a solid block, and drops as an item when that
//! block goes. Core rules through `Sim::block`/`set_block_anywhere`, so they hold in unloaded chunks and
//! on every peer. How much light a torch gives is `light.rs`'s business.

use crate::block::{AIR, SOLID, TORCH};
use crate::math::{IVec3, Vec3};
use crate::sim::{Sim, SimEvent};

const UP: IVec3 = IVec3::new(0, 1, 0);

impl Sim {
    /// Whether a torch can go at `pos`: an empty cell (not water) on a solid block.
    pub(crate) fn torch_fits(&mut self, pos: IVec3) -> bool {
        self.block(pos) == AIR && SOLID[self.block(pos - UP) as usize]
    }

    /// After the block at `pos` changed: a torch standing on it drops if it no longer holds it up.
    pub(super) fn torch_support_changed(&mut self, pos: IVec3) {
        let top = pos + UP;
        if SOLID[self.block(pos) as usize] || self.block(top) != TORCH || !self.world.set_block_anywhere(top, AIR) {
            return;
        }
        let center = top.as_vec3() + Vec3::new(0.5, 0.3, 0.5);
        self.events.push(SimEvent::Dropped {
            pos: center,
            vel: Vec3::new(0.0, 1.0, 0.0),
            item: TORCH.into(),
            count: 1,
        });
        self.block_changed(top, TORCH);
    }
}

#[cfg(test)]
mod tests;
