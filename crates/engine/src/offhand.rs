//! The left hand: one more stack beside the hotbar (`Inventory::offhand`, saved from version 60, never counted
//! as backpack room). The selected hotbar slot is the right hand; the key swaps the two stacks (`Action::Offhand`),
//! and the inventory screen clicks the left hand like any slot.
//!
//! What the left hand does: a light in it shines (`held_light.rs`), the avatar holds it, and the scanner, core drill
//! and telescope work from it on `G` (`left_hand.rs`: it informs, never queues a core `Action`). Everything else works
//! from the right hand only (mining is the worn Tool Belt's: `tool_belt/`).

use crate::inventory::{click_stack, Inventory};

impl Inventory {
    /// Swaps the selected hotbar stack with the left hand's.
    pub fn swap_offhand(&mut self) {
        let selected = self.selected;
        if self.slots[selected].is_empty() && self.offhand.is_empty() {
            return;
        }
        std::mem::swap(&mut self.slots[selected], &mut self.offhand);
        self.version += 1;
    }

    /// An inventory-screen click on the left hand: pick up, put down, merge or swap with the cursor stack.
    pub fn click_offhand(&mut self) {
        if click_stack(&mut self.offhand, &mut self.cursor) {
            self.version += 1;
        }
    }
}

/// Test helper: the bytes of `sim` as a save before version 60 wrote them, with the (empty) left-hand stack taken out
/// after each player's worn gear. For tests that build an older save by cutting a current one.
#[cfg(test)]
pub(crate) fn before_offhand(sim: &crate::sim::Sim, stream: &[u8]) -> Vec<u8> {
    use crate::bytes::ByteWriter;
    let mut out = crate::helpers::before_drone_orders(sim, stream);
    let mut empty = ByteWriter::default();
    crate::inventory::Stack::default().write_state(&mut empty);
    for p in sim.players.iter().flatten() {
        assert!(p.inventory.offhand.is_empty(), "a save before 60 had no left hand");
        // `before_drone_orders` already cut the head slot out of the inventory.
        let w = p.inventory.state_before_head_slot();
        let at = out.windows(w.len()).position(|s| s == w).expect("the inventory's bytes");
        let end = at + w.len();
        out.drain(end - empty.bytes.len()..end);
    }
    out
}

#[cfg(test)]
mod tests;
