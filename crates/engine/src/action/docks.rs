//! Docks as pieces of rail in the core (`factory/rail/pieces.rs`): a dock is built only onto a straight rail node,
//! taking its place and keeping its tracks, and breaking a dock that has track puts the rail node back. Every way a
//! block is placed (hands, drones, blueprints) goes through `put_block`, so no dock stands off the rails.

use crate::block::{BlockId, AIR, RAIL};
use crate::factory::DockFit;
use crate::inventory::Stack;
use crate::math::IVec3;
use crate::sim::{PlayerId, Sim, SimEvent};

impl Sim {
    /// `put_block`'s path for a dock: onto the straight rail node at `pos`, or nothing (false).
    pub(super) fn put_dock(&mut self, player: PlayerId, pos: IVec3, placed: BlockId) -> bool {
        if self.factory.dock_fit(pos) != DockFit::Ok {
            return false;
        }
        self.factory.swap_rail_piece(&mut self.world, pos, placed);
        let old = self.world.block_anywhere_or_generate(pos);
        self.world.set_block_anywhere(pos, placed);
        self.block_changed(pos, old);
        self.events.push(SimEvent::BlockPlaced { player, pos, block: placed });
        true
    }

    /// Takes the dock at `pos` out of the factory once its block is broken: a dock with track becomes the rail node
    /// it stood on again (its block is put back); one without is removed. Returns what it held.
    pub(super) fn take_dock(&mut self, pos: IVec3) -> Vec<Stack> {
        if self.factory.track_count(pos) == 0 {
            return self.factory.remove(pos);
        }
        let held = self.factory.swap_rail_piece(&mut self.world, pos, RAIL);
        self.world.set_block_anywhere(pos, RAIL);
        self.block_changed(pos, AIR);
        held
    }
}
