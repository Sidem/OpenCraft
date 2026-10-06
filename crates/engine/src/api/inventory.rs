//! The inventory screen and hotbar readouts: slots, the cursor stack, worn gear, and pickup notifications.

use wasm_bindgen::prelude::*;

use crate::action::Action;
use crate::equipment::{self, SLOTS, SLOT_NAMES};
use crate::inventory::{HOTBAR_SLOTS, MAX_SLOTS};
use crate::item::ItemId;
use crate::Game;

#[wasm_bindgen]
impl Game {
    pub fn inventory_version(&self) -> u32 {
        self.inventory().version
    }

    pub fn hotbar_size(&self) -> u32 {
        HOTBAR_SLOTS as u32
    }

    /// All slots in use: the hotbar (0..9) followed by the backpack (more with a hauler pack on).
    pub fn inventory_size(&self) -> u32 {
        self.inventory().capacity() as u32
    }

    /// The most slots there can be (the screen makes this many and hides those a pack has not opened).
    pub fn inventory_max_size(&self) -> u32 {
        MAX_SLOTS as u32
    }

    /// Equipment slots (back, boots, torso, tool belt), and the item worn in each (0 for nothing).
    pub fn gear_slots(&self) -> u32 {
        SLOTS as u32
    }

    pub fn gear_slot_name(&self, slot: u32) -> String {
        SLOT_NAMES.get(slot as usize).copied().unwrap_or_default().to_string()
    }

    pub fn worn_item(&self, slot: u32) -> u16 {
        self.inventory().worn.get(slot as usize).map_or(0, |i| i.0)
    }

    /// What an item does when worn (empty for anything that is not gear), and the slot it goes in (-1 for none).
    pub fn gear_blurb(&self, item: u16) -> String {
        equipment::gear(ItemId(item)).map_or_else(String::new, |g| g.blurb.to_string())
    }

    pub fn gear_slot_of(&self, item: u16) -> i32 {
        equipment::gear(ItemId(item)).map_or(-1, |g| g.slot as i32)
    }

    /// Equipment-panel click (next tick): swaps the cursor stack with the gear in `slot`; `shift` takes it off.
    pub fn click_gear(&mut self, slot: u32, shift: bool) {
        self.act(Action::ClickGear { slot: slot.min(u8::MAX as u32) as u8, shift });
    }

    pub fn slot_item(&self, slot: u32) -> u16 {
        self.inventory().slots.get(slot as usize).map_or(0, |s| s.item.0)
    }

    pub fn slot_count(&self, slot: u32) -> u32 {
        self.inventory().slots.get(slot as usize).map_or(0, |s| s.count)
    }

    pub fn selected_slot(&self) -> u32 {
        self.inventory().selected as u32
    }

    /// Inventory screen click (applied at the next tick; watch `inventory_version`). `shift` moves the
    /// stack between hotbar and backpack.
    pub fn click_slot(&mut self, slot: u32, shift: bool) {
        self.act(Action::ClickSlot { slot: slot.min(u8::MAX as u32) as u8, shift });
    }

    /// Right-click on an inventory slot (next tick): takes half of it onto the cursor, or half of what is left onto a
    /// cursor holding the same item. With `place`, the shift-right-click while holding a stack: puts one item in.
    pub fn right_click_slot(&mut self, slot: u32, place: bool) {
        self.act(Action::RightClickSlot { slot: slot.min(u8::MAX as u32) as u8, shift: place });
    }

    /// Shift-right-click outside the inventory screen while holding a stack: throws one item out.
    pub fn throw_cursor(&mut self) {
        self.act(Action::ThrowCursor);
    }

    /// Shift-right-click on an inventory slot: moves every stack of its item between hotbar and backpack.
    pub fn quick_move_all(&mut self, slot: u32) {
        self.act(Action::QuickMoveAll { slot: slot.min(u8::MAX as u32) as u8 });
    }

    /// Sorts the backpack (next tick; the hotbar keeps its layout).
    pub fn sort_inventory(&mut self) {
        self.act(Action::SortInventory);
    }

    pub fn cursor_item(&self) -> u16 {
        self.inventory().cursor.item.0
    }

    pub fn cursor_count(&self) -> u32 {
        self.inventory().cursor.count
    }

    /// Closing the inventory screen: the stack on the cursor goes back (or is thrown if full).
    pub fn close_inventory(&mut self) {
        self.act(Action::CloseInventory);
    }

    pub fn item_total(&self, item: u16) -> u32 {
        self.inventory().count(ItemId(item))
    }

    /// Pops the next pickup notification; read it with `pickup_item` / `pickup_count`.
    pub fn next_pickup(&mut self) -> bool {
        match self.pickups.pop_front() {
            Some(p) => {
                self.cur_pickup = p;
                true
            }
            None => false,
        }
    }

    pub fn pickup_item(&self) -> u16 {
        self.cur_pickup.0 .0
    }

    pub fn pickup_count(&self) -> u32 {
        self.cur_pickup.1
    }
}
