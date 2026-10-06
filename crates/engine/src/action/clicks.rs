//! The inventory-screen and box-screen clicks (`Action::ClickSlot`, `ClickBox`, `ClickGear`, the shift-clicks, the
//! right-clicks that split a stack or place one item, and the sorts): what each does to the player's inventory and the box they touch. To add one: its arm below, and a
//! line naming it in `Sim::apply_to_player`.

use crate::inventory::{add_to_slots, click_stack, move_all_of, place_one, sort_stacks, split_stack, Stack};
use crate::item::ItemId;
use crate::sim::{PlayerId, Sim, SimEvent};

use super::Action;

impl Sim {
    /// Applies one of the click actions listed in `apply_to_player` (any other does nothing).
    pub(super) fn apply_click(&mut self, player: PlayerId, action: Action) {
        let Some(Some(core)) = self.players.get_mut(player.0 as usize) else { return };
        let inv = &mut core.inventory;
        match action {
            Action::ClickGear { slot, shift } => inv.click_gear(slot as usize, shift),
            Action::ClickSlot { slot, shift: true } => {
                if !inv.wear_from(slot as usize) {
                    inv.quick_move(slot as usize);
                }
            }
            Action::ClickSlot { slot, shift: false } => inv.click(slot as usize),
            Action::ClickBox { pos, slot, shift } => {
                let Some(s) = self.factory.box_slots_mut(pos).and_then(|b| b.get_mut(slot as usize)) else { return };
                if shift && !s.is_empty() {
                    s.count = inv.add(s.item, s.count);
                    if s.is_empty() {
                        *s = Stack::default();
                    }
                } else if !shift && click_stack(s, &mut inv.cursor) {
                    inv.version += 1;
                }
            }
            Action::RightClickSlot { slot, shift } => inv.right_click(slot as usize, shift),
            Action::RightClickBox { pos, slot, shift } => {
                let Some(s) = self.factory.box_slots_mut(pos).and_then(|b| b.get_mut(slot as usize)) else { return };
                let changed = if shift { place_one(s, &mut inv.cursor) } else { split_stack(s, &mut inv.cursor) };
                if changed {
                    inv.version += 1;
                }
            }
            Action::ThrowCursor => {
                if let Some(Stack { item, count }) = inv.take_cursor_one() {
                    self.events.push(SimEvent::Thrown { player, item, count });
                }
            }
            Action::StoreSlot { pos, slot } => {
                let Some(b) = self.factory.box_slots_mut(pos) else { return };
                let Some((item, n)) = inv.take_slot(slot as usize, u32::MAX) else { return };
                let left = add_to_slots(b, item, n);
                if left > 0 {
                    inv.slots[slot as usize] = Stack { item, count: left };
                }
            }
            Action::QuickMoveAll { slot } => inv.quick_move_all(slot as usize),
            Action::StoreAll { pos, slot } => {
                let item = inv.open().get(slot as usize).map_or(ItemId::NONE, |s| s.item);
                if self.factory.box_slots_mut(pos).is_some_and(|b| move_all_of(inv.open_mut(), b, item)) {
                    inv.version += 1;
                }
            }
            Action::TakeAll { pos, slot } => {
                let Some(b) = self.factory.box_slots_mut(pos) else { return };
                let item = b.get(slot as usize).map_or(ItemId::NONE, |s| s.item);
                if move_all_of(b, inv.open_mut(), item) {
                    inv.version += 1;
                }
            }
            Action::SortInventory => inv.sort(),
            Action::SortBox { pos } => {
                if let Some(b) = self.factory.box_slots_mut(pos) {
                    sort_stacks(b);
                }
            }
            _ => {}
        }
    }
}
