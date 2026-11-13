//! The personal drone's panel and keys (`drone_hands.rs`, `helpers/drone/`): orders, its queue, what lies in reach.

use wasm_bindgen::prelude::*;

use crate::helpers::drone::{Job, ANY_SLOT, RANGE};
use crate::item::{self, ItemId, PERSONAL_DRONE};
use crate::Game;

#[wasm_bindgen]
impl Game {
    /// Whether the local player carries a personal drone.
    pub fn drone_ready(&self) -> bool {
        self.inventory().count(PERSONAL_DRONE) > 0
    }

    /// The drone's own item id (it never carries itself off).
    pub fn drone_item(&self) -> u16 {
        PERSONAL_DRONE.0
    }

    /// How far it reaches from the player, in blocks.
    pub fn drone_range(&self) -> u32 {
        RANGE as u32
    }

    /// Sends it for `count` of `item`.
    pub fn drone_fetch(&mut self, item: u16, count: u32) {
        self.order(Job::Fetch { item: ItemId(item), count, slot: ANY_SLOT });
    }

    /// Shift+Y: a stack more of the item in hand, topping up the hand's slot first. False with an empty hand or no
    /// drone.
    pub fn drone_fetch_held(&mut self) -> bool {
        let held = self.inventory().selected_stack();
        let ready = !held.is_empty() && self.drone_ready();
        if ready {
            let slot = self.inventory().selected as u8;
            self.order(Job::Fetch { item: held.item, count: item::stack_size(held.item), slot });
        }
        ready
    }

    /// Takes the stack in pack slot `slot` to the box holding its item, else an empty box off the belt lines, else a "Misc" box.
    pub fn drone_store_slot(&mut self, slot: u32) {
        let Some(s) = self.inventory().open().get(slot as usize).copied().filter(|s| !s.is_empty()) else { return };
        self.order(Job::Store { item: s.item, count: s.count, slot: slot.min(254) as u8 });
    }

    /// Fills pack slot `slot` with a stack of `item` from the boxes and machines in reach (tops it up when it holds
    /// some already; nothing when it holds something else).
    pub fn drone_fill_slot(&mut self, slot: u32, item: u16) {
        let item = ItemId(item);
        let Some(s) = self.inventory().open().get(slot as usize).copied() else { return };
        let count = item::stack_size(item).saturating_sub(s.count);
        if (s.is_empty() || s.item == item) && count > 0 {
            self.order(Job::Fetch { item, count, slot: slot.min(254) as u8 });
        }
    }

    /// The pack slots orders are tied to, six numbers each (`drone_hands::slot_rows`): slot, stores from it, item,
    /// on its way, progress in thousandths, seconds left.
    pub fn drone_slots(&self) -> Vec<u32> {
        self.slot_rows()
    }

    /// Takes every stack outside the hotbar whose item has a box of its own in reach to that box.
    pub fn drone_put_away(&mut self) {
        self.order(Job::PutAway);
    }

    /// How many kinds a put-away would take now.
    pub fn drone_put_away_kinds(&self) -> u32 {
        self.put_away_kinds()
    }

    /// Fetches what `times` of hand recipe `r` lacks, then crafts it.
    pub fn drone_craft(&mut self, r: u32, times: u32) {
        self.order(Job::Craft { recipe: r as u16, times: times.min(u32::from(u16::MAX)) as u16 });
    }

    /// What the drone would fetch for `times` of hand recipe `r`: empty when it can't gather it, else 1 followed by
    /// item, count pairs (just 1 when the pack holds everything).
    pub fn drone_craft_fetches(&self, r: u32, times: u32) -> Vec<u32> {
        let Some(list) = self.drone_craft_fetches_for(r as u16, times) else { return Vec::new() };
        std::iter::once(1).chain(list.into_iter().flat_map(|(i, n)| [u32::from(i.0), n])).collect()
    }

    /// Drops waiting job `index` (as `drone_jobs` lists them, after the trip).
    pub fn drone_cancel(&mut self, index: u32) {
        self.cancel_drone_job(index.min(255) as u8);
    }

    /// Item, in boxes, in machines, its stack size: everything in reach, four numbers each.
    pub fn drone_stock(&self) -> Vec<u32> {
        let row =
            |s: &crate::factory::stock::Stocked| [u32::from(s.item.0), s.boxes, s.machines, item::stack_size(s.item)];
        self.stock_in_reach().iter().flat_map(row).collect()
    }

    /// The trip in flight, then the waiting jobs, four numbers each (`drone_hands::job_rows`).
    pub fn drone_jobs(&self) -> Vec<u32> {
        self.job_rows()
    }

    /// Whether it restocks the hand stack as it runs low.
    pub fn drone_restock(&self) -> bool {
        self.drone_hands.restock
    }

    pub fn set_drone_restock(&mut self, on: bool) {
        self.drone_hands.restock = on;
    }
}
