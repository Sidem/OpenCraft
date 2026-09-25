//! The inventory screen and hotbar readouts: slots, the cursor stack, and pickup notifications.

use wasm_bindgen::prelude::*;

use crate::action::Action;
use crate::block::AIR;
use crate::inventory::{HOTBAR_SLOTS, INVENTORY_SLOTS};
use crate::{Game, LOCAL};

#[wasm_bindgen]
impl Game {
    pub fn inventory_version(&self) -> u32 {
        self.sim.player(LOCAL).inventory.version
    }

    pub fn hotbar_size(&self) -> u32 {
        HOTBAR_SLOTS as u32
    }

    /// All slots: the hotbar (0..9) followed by the backpack.
    pub fn inventory_size(&self) -> u32 {
        INVENTORY_SLOTS as u32
    }

    pub fn slot_item(&self, slot: u32) -> u8 {
        self.sim.player(LOCAL).inventory.slots.get(slot as usize).map_or(AIR, |s| s.item)
    }

    pub fn slot_count(&self, slot: u32) -> u32 {
        self.sim.player(LOCAL).inventory.slots.get(slot as usize).map_or(0, |s| s.count)
    }

    pub fn selected_slot(&self) -> u32 {
        self.sim.player(LOCAL).inventory.selected as u32
    }

    /// Inventory screen click (applied at the next tick; watch `inventory_version`). `shift` moves the
    /// stack between hotbar and backpack.
    pub fn click_slot(&mut self, slot: u32, shift: bool) {
        self.act(Action::ClickSlot { slot: slot.min(u8::MAX as u32) as u8, shift });
    }

    pub fn cursor_item(&self) -> u8 {
        self.sim.player(LOCAL).inventory.cursor.item
    }

    pub fn cursor_count(&self) -> u32 {
        self.sim.player(LOCAL).inventory.cursor.count
    }

    /// Closing the inventory screen: the stack on the cursor goes back (or is thrown if full).
    pub fn close_inventory(&mut self) {
        self.act(Action::CloseInventory);
    }

    pub fn item_total(&self, item: u8) -> u32 {
        self.sim.player(LOCAL).inventory.count(item)
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

    pub fn pickup_item(&self) -> u8 {
        self.cur_pickup.0
    }

    pub fn pickup_count(&self) -> u32 {
        self.cur_pickup.1
    }
}
