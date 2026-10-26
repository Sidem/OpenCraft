//! Cable drops, part of the power tools (`power_tools.rs`): a click with cables in hand hangs the aimed cell and
//! every free cell below it, a few cells per tick, so a long run needs one click. Everything goes out as ordinary
//! `PlaceBlock` actions.

use super::UP;
use crate::action::Action;
use crate::block::{self, CABLE};
use crate::math::IVec3;
use crate::Game;

/// The most cables one click hangs.
const MAX_DROP: usize = 64;
/// Cables placed per tick while a drop is built.
const DROP_PER_TICK: usize = 3;

impl Game {
    pub(super) fn holds_cable(&self) -> bool {
        let stack = self.inventory().selected_stack();
        !stack.is_empty() && stack.item.places() == Some(CABLE)
    }

    /// The cells a click with cables would fill, top first: the aimed cell and the free ones below it.
    pub(super) fn cable_drop(&self) -> Vec<IVec3> {
        let Some(hit) = self.target.filter(|h| h.normal != IVec3::ZERO) else { return Vec::new() };
        let world = &self.sim.world;
        let free = |p: IVec3| world.get_block(p).is_some_and(block::replaceable);
        let max = (self.inventory().selected_stack().count as usize).min(MAX_DROP);
        let mut cells = Vec::new();
        let mut c = hit.block + hit.normal;
        while cells.len() < max && free(c) {
            cells.push(c);
            c = c - UP;
        }
        cells
    }

    /// Sends the next few cables of a drop. False when there are none; stops if the slot changed.
    pub(super) fn send_cable_drop(&mut self) -> bool {
        if self.tools.drop.is_empty() {
            return false;
        }
        let (slot, item) = self.tools.drop_from;
        if self.inventory().slots[slot as usize].item.0 != item {
            self.tools.drop.clear();
            return false;
        }
        for _ in 0..DROP_PER_TICK {
            let Some(pos) = self.tools.drop.pop() else { break };
            self.act(Action::PlaceBlock { pos, slot, facing: 0, against: pos - UP });
        }
        true
    }
}
