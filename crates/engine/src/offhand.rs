//! The left hand: one more stack beside the hotbar (`Inventory::offhand`, saved from version 60, never counted
//! as backpack room). The selected hotbar slot is the right hand; the key swaps the two stacks (`Action::Offhand`),
//! and the inventory screen clicks the left hand like any slot.
//!
//! What the left hand does: a light in it shines (`held_light.rs`) and the avatar holds it; a tool in it breaks the
//! blocks it is made for when the right hand's tool is not (`Inventory::tool_hand`), wearing as it would there.
//! Everything else works from the right hand only. To give the left hand another job: read `Inventory::offhand`
//! where it matters, as `tool_hand` does.

use crate::block::BlockId;
use crate::inventory::{click_stack, Inventory, Stack};
use crate::tools;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Hand {
    /// The selected hotbar slot.
    Right,
    Left,
}

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

    pub fn in_hand(&self, hand: Hand) -> Stack {
        match hand {
            Hand::Right => self.selected_stack(),
            Hand::Left => self.offhand,
        }
    }

    /// The hand whose tool breaks `block`: the right hand's when it holds the tool for it, else the left's, else the right.
    pub fn tool_hand(&self, block: BlockId) -> Hand {
        let made_for = |h| tools::tool_for(self.in_hand(h).item, block).is_some();
        if !made_for(Hand::Right) && made_for(Hand::Left) {
            Hand::Left
        } else {
            Hand::Right
        }
    }

    /// Takes one use from the tool in `hand`; true when that was its last.
    pub fn wear_hand(&mut self, hand: Hand) -> bool {
        match hand {
            Hand::Right => {
                self.take_slot(self.selected, 1);
            }
            Hand::Left => {
                self.offhand.count = self.offhand.count.saturating_sub(1);
                if self.offhand.is_empty() {
                    self.offhand = Stack::default();
                }
                self.version += 1;
            }
        }
        self.in_hand(hand).is_empty()
    }
}

/// Test helper: the bytes of `sim` as a save before version 60 wrote them, with the (empty) left-hand stack taken out
/// after each player's worn gear. For tests that build an older save by cutting a current one.
#[cfg(test)]
pub(crate) fn before_offhand(sim: &crate::sim::Sim, stream: &[u8]) -> Vec<u8> {
    use crate::bytes::ByteWriter;
    let mut out = stream.to_vec();
    let mut empty = ByteWriter::default();
    Stack::default().write_state(&mut empty);
    for p in sim.players.iter().flatten() {
        assert!(p.inventory.offhand.is_empty(), "a save before 60 had no left hand");
        let mut w = ByteWriter::default();
        p.inventory.write_state(&mut w);
        let at = out.windows(w.bytes.len()).position(|s| s == w.bytes).expect("the inventory's bytes");
        let end = at + w.bytes.len();
        out.drain(end - empty.bytes.len()..end);
    }
    out
}

#[cfg(test)]
mod tests;
