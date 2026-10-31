//! Docks as pieces of rail in the core (`factory/rail/pieces.rs`): a dock is built onto a straight rail node, taking
//! its place and keeping its tracks, or laid in line after a rail piece like a new node, joined to it in the same
//! action; breaking a dock that has track puts the rail node back. Every way a block is placed (hands, drones,
//! blueprints) goes through `put_block`, so no dock stands off the rails.

use crate::block::{self, BlockId, AIR, RAIL};
use crate::factory::{self, Curve, DockFit, Fit};
use crate::inventory::Stack;
use crate::math::IVec3;
use crate::sim::{PlayerId, Sim, SimEvent};

/// Heading steps between grid axes.
const AXIS: u8 = 64;

impl Sim {
    /// `put_block`'s path for a dock: onto the straight rail node at `pos`; or, where no rail piece is, laid in line
    /// after the rail piece at `against` with the heading `facing` (along a grid axis) and joined to it at once, if
    /// that track fits and lies on the ground. Nothing (false) otherwise.
    pub(super) fn put_dock(
        &mut self,
        player: PlayerId,
        pos: IVec3,
        placed: BlockId,
        facing: u8,
        against: IVec3,
    ) -> bool {
        match self.factory.dock_fit(pos) {
            DockFit::Ok => {
                self.factory.swap_rail_piece(&mut self.world, pos, placed);
            }
            DockFit::NoNode if self.dock_in_line_fits(pos, facing, against) => {
                self.factory.place(&mut self.world, placed, pos, facing / AXIS, pos, 0);
                self.factory.lay_track(against, pos);
            }
            _ => return false,
        }
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

    /// Whether a dock at the free cell `pos` heading `facing` can be joined to the rail piece at `from`: a heading
    /// along a grid axis, ground under it, and a track that fits and lies on the ground (as `connect` checks).
    fn dock_in_line_fits(&mut self, pos: IVec3, facing: u8, from: IVec3) -> bool {
        let Some(from_yaw) = self.factory.rail_yaw(from) else { return false };
        let world = &mut self.world;
        facing.is_multiple_of(AXIS)
            && self.factory.lay_fit(from, pos, facing) == Some(Fit::Ok)
            && block::replaceable(world.block_anywhere_or_generate(pos))
            && factory::rail_node_stands(|p| world.block_anywhere_or_generate(p), pos)
            && factory::rail_faults(&Curve::new(from, from_yaw, pos, facing), |p| world.block_anywhere_or_generate(p))
                .is_empty()
    }
}
