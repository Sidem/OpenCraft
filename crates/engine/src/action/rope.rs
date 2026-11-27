//! Rope (GEAR-ROPE, block `block::ROPE`): `Action::HangRope` unrolls one coil from a slot down from a free cell that
//! hangs from something (a block above or beside it), at most [`PER_COIL`] cells and only through free cells, so it
//! reaches the floor below like a cable drop (`power_tools/cables.rs`); aimed at a strand, it carries on from that
//! strand's bottom. Breaking any cell of a strand takes it up whole (`take_rope`, from `dismantle`) and gives back one
//! coil per [`PER_COIL`] cells, rounded up: never more coils than went in, since rounding up never gains across strands.
//!
//! Invariants: core state, edited only here and only through actions; reads `block_anywhere_or_generate`, so it never
//! depends on loaded chunks. Climbing is the body's (`player::climbable`), outside the core.

use crate::block::{self, AIR, ROPE};
use crate::inventory::Stack;
use crate::item::ROPE_ITEM;
use crate::math::IVec3;
use crate::sim::{PlayerId, Sim, SimEvent};

/// Cells one coil of rope hangs.
pub const PER_COIL: usize = 8;
/// The longest strand followed up or down (a bound on the scan).
const MAX_STRAND: i32 = 256;
const UP: IVec3 = IVec3::new(0, 1, 0);

impl Sim {
    /// `Action::HangRope`: one coil from `slot` hung from `pos` (or below the strand at `pos`). Nothing when the slot
    /// holds no rope, the first cell is taken or nothing holds it up.
    pub(super) fn hang_rope(&mut self, player: PlayerId, pos: IVec3, slot: u8) {
        let held = self.player(player).and_then(|c| c.inventory.slots.get(slot as usize)).copied();
        if !held.is_some_and(|s| !s.is_empty() && s.item == ROPE_ITEM) {
            return;
        }
        let on_strand = self.world.block_anywhere_or_generate(pos) == ROPE;
        let mut at = if on_strand { self.strand_end(pos, -1) - UP } else { pos };
        let free = |sim: &mut Sim, p: IVec3| block::replaceable(sim.world.block_anywhere_or_generate(p));
        let held_up = [UP, IVec3::new(1, 0, 0), IVec3::new(-1, 0, 0), IVec3::new(0, 0, 1), IVec3::new(0, 0, -1)]
            .into_iter()
            .any(|d| !free(self, at + d));
        if !free(self, at) || !held_up {
            return;
        }
        let mut hung = 0;
        while hung < PER_COIL && free(self, at) {
            let old = self.world.block_anywhere_or_generate(at);
            if !self.world.set_block_anywhere(at, ROPE) {
                break;
            }
            self.block_changed(at, old);
            hung += 1;
            at = at - UP;
        }
        if hung > 0 {
            self.events.push(SimEvent::BlockPlaced { player, pos, block: ROPE });
            if let Some(Some(core)) = self.players.get_mut(player.0 as usize) {
                core.inventory.consume_slot(slot as usize, 1);
            }
        }
    }

    /// Takes up the whole strand through `pos` (a rope cell) and returns its coils, for `dismantle`.
    pub(crate) fn take_rope(&mut self, player: PlayerId, pos: IVec3) -> Option<(IVec3, Vec<Stack>)> {
        let (top, bottom) = (self.strand_end(pos, 1), self.strand_end(pos, -1));
        let mut cells: usize = 0;
        for y in bottom.y..=top.y {
            let p = IVec3::new(pos.x, y, pos.z);
            if self.world.set_block_anywhere(p, AIR) {
                self.block_changed(p, ROPE);
                cells += 1;
            }
        }
        if cells == 0 {
            return None;
        }
        self.events.push(SimEvent::BlockBroken { player, pos, block: ROPE });
        let coils = cells.div_ceil(PER_COIL) as u32;
        Some((pos, vec![Stack { item: ROPE_ITEM, count: coils }]))
    }

    /// The last rope cell of the strand through `pos`, going up (`dir` 1) or down (-1); `pos` itself when it ends there.
    fn strand_end(&mut self, pos: IVec3, dir: i32) -> IVec3 {
        let mut end = pos;
        for _ in 0..MAX_STRAND {
            let next = end + IVec3::new(0, dir, 0);
            if self.world.block_anywhere_or_generate(next) != ROPE {
                break;
            }
            end = next;
        }
        end
    }
}

#[cfg(test)]
mod tests;
