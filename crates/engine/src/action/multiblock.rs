//! Placing and breaking multi-block machines in the core (`factory/footprint/`). Placing needs every
//! cell free (air or water); the anchor gets the machine's block and the other cells `MACHINE_PART`.
//! Breaking any cell breaks the machine: `break_block` works on its anchor (drops, contents) and
//! `clear_parts` empties the other cells.

use crate::block::{BlockId, AIR, MACHINE_PART};
use crate::factory::footprint::{self, Footprint};
use crate::factory::tiers;
use crate::math::IVec3;
use crate::sim::{PlayerId, Sim, SimEvent};

impl Sim {
    /// Places `placed`, a machine with footprint `fp`, anchored at `pos` and facing `facing`, from
    /// inventory `slot`, if every cell is free.
    pub(super) fn place_footprint(
        &mut self,
        player: PlayerId,
        pos: IVec3,
        slot: u8,
        (placed, fp): (BlockId, &Footprint),
        facing: u8,
    ) {
        let cells = fp.cells(pos, facing % 4);
        let world = &mut self.world;
        if footprint::blocked(&cells, |c| Some(world.block_anywhere_or_generate(c))).contains(&true) {
            return;
        }
        let Some(Some(core)) = self.players.get_mut(player.0 as usize) else { return };
        let item = core.inventory.slots[slot as usize].item;
        core.inventory.take_slot(slot as usize, 1);
        for (i, &c) in cells.iter().enumerate() {
            let old = self.world.block_anywhere_or_generate(c);
            self.world.set_block_anywhere(c, if i == 0 { placed } else { MACHINE_PART });
            self.block_changed(c, old);
        }
        let tier = tiers::placed_by(item).map_or(0, |(_, t)| t);
        self.factory.place(&mut self.world, placed, pos, facing, pos, tier);
        self.events.push(SimEvent::BlockPlaced { player, pos, block: placed });
    }

    /// Empties every cell but the anchor of the multi-block machine anchored at `anchor`.
    pub(super) fn clear_parts(&mut self, anchor: IVec3) {
        let Some((_, _, cells)) = self.factory.footprint_at(anchor) else { return };
        for c in cells.into_iter().skip(1) {
            if self.world.block_anywhere_or_generate(c) == MACHINE_PART && self.world.set_block_anywhere(c, AIR) {
                self.block_changed(c, MACHINE_PART);
            }
        }
    }
}
