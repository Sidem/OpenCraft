//! Player inventory: a 9-slot hotbar (slots 0..9) plus a 27-slot backpack, and the stack held by
//! the mouse cursor while the inventory screen is open. `version` increments on every change so
//! the UI redraws only when needed. `add_to_slots` is shared with storage boxes.

use crate::block::{BlockId, AIR};

pub const HOTBAR_SLOTS: usize = 9;
pub const INVENTORY_SLOTS: usize = 36;
pub const MAX_STACK: u32 = 64;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Stack {
    pub item: BlockId,
    pub count: u32,
}

impl Stack {
    pub fn is_empty(&self) -> bool {
        self.count == 0
    }
}

/// Adds up to `count` of `item` into `slots`, topping up matching stacks before filling empty
/// slots (in slice order). Returns what didn't fit.
pub fn add_to_slots(slots: &mut [Stack], item: BlockId, mut count: u32) -> u32 {
    if item == AIR {
        return count;
    }
    for s in slots.iter_mut().filter(|s| !s.is_empty() && s.item == item) {
        let n = count.min(MAX_STACK - s.count);
        s.count += n;
        count -= n;
    }
    for s in slots.iter_mut().filter(|s| s.is_empty()) {
        if count == 0 {
            break;
        }
        let n = count.min(MAX_STACK);
        *s = Stack { item, count: n };
        count -= n;
    }
    count
}

pub struct Inventory {
    pub slots: [Stack; INVENTORY_SLOTS],
    pub selected: usize,
    /// Held by the mouse in the inventory screen.
    pub cursor: Stack,
    /// Bumped on every change so the UI can skip redundant redraws.
    pub version: u32,
}

impl Default for Inventory {
    fn default() -> Self {
        Self { slots: [Stack::default(); INVENTORY_SLOTS], selected: 0, cursor: Stack::default(), version: 0 }
    }
}

impl Inventory {
    /// Room left for `item` across all slots.
    pub fn space_for(&self, item: BlockId) -> u32 {
        self.slots
            .iter()
            .map(|s| match s {
                s if s.is_empty() => MAX_STACK,
                s if s.item == item => MAX_STACK - s.count,
                _ => 0,
            })
            .sum()
    }

    /// Adds items: tops up existing stacks anywhere, then fills empty hotbar slots before the
    /// backpack. Returns what didn't fit.
    pub fn add(&mut self, item: BlockId, count: u32) -> u32 {
        if item == AIR || count == 0 {
            return count;
        }
        let left = add_to_slots(&mut self.slots, item, count);
        if left != count {
            self.version += 1;
        }
        left
    }

    /// Total number of `item` held.
    pub fn count(&self, item: BlockId) -> u32 {
        self.slots.iter().filter(|s| s.item == item).map(|s| s.count).sum()
    }

    /// Removes `n` of `item`, backpack first so the hotbar keeps its layout. All or nothing.
    pub fn remove(&mut self, item: BlockId, mut n: u32) -> bool {
        if self.count(item) < n {
            return false;
        }
        for s in self.slots.iter_mut().rev().filter(|s| s.item == item && !s.is_empty()) {
            let k = n.min(s.count);
            s.count -= k;
            n -= k;
            if s.count == 0 {
                *s = Stack::default();
            }
            if n == 0 {
                break;
            }
        }
        self.version += 1;
        true
    }

    pub fn selected_stack(&self) -> Stack {
        self.slots[self.selected]
    }

    /// Removes up to `n` items from the selected slot, returning the item id and amount taken.
    pub fn take_selected(&mut self, n: u32) -> Option<(BlockId, u32)> {
        let s = &mut self.slots[self.selected];
        if s.is_empty() {
            return None;
        }
        let taken = n.min(s.count);
        let item = s.item;
        s.count -= taken;
        if s.count == 0 {
            *s = Stack::default();
        }
        self.version += 1;
        Some((item, taken))
    }

    pub fn select(&mut self, slot: usize) {
        if slot < HOTBAR_SLOTS && slot != self.selected {
            self.selected = slot;
            self.version += 1;
        }
    }

    pub fn scroll(&mut self, delta: i32) {
        let n = HOTBAR_SLOTS as i32;
        self.select((self.selected as i32 + delta).rem_euclid(n) as usize);
    }

    /// Inventory-screen click: pick up, put down, merge or swap with the cursor stack.
    pub fn click(&mut self, slot: usize) {
        let Some(s) = self.slots.get_mut(slot) else { return };
        let c = &mut self.cursor;
        if c.is_empty() {
            if s.is_empty() {
                return;
            }
            *c = std::mem::take(s);
        } else if s.is_empty() {
            *s = std::mem::take(c);
        } else if s.item == c.item {
            let n = c.count.min(MAX_STACK - s.count);
            s.count += n;
            c.count -= n;
            if c.count == 0 {
                *c = Stack::default();
            }
        } else {
            std::mem::swap(s, c);
        }
        self.version += 1;
    }

    /// Shift-click: moves a stack between the hotbar and the backpack.
    pub fn quick_move(&mut self, slot: usize) {
        if slot >= INVENTORY_SLOTS || self.slots[slot].is_empty() {
            return;
        }
        let s = std::mem::take(&mut self.slots[slot]);
        let target = if slot < HOTBAR_SLOTS { HOTBAR_SLOTS..INVENTORY_SLOTS } else { 0..HOTBAR_SLOTS };
        let left = add_to_slots(&mut self.slots[target], s.item, s.count);
        if left > 0 {
            self.slots[slot] = Stack { item: s.item, count: left };
        }
        self.version += 1;
    }

    /// Puts the cursor stack back into the inventory. Returns whatever didn't fit.
    pub fn return_cursor(&mut self) -> Stack {
        let c = std::mem::take(&mut self.cursor);
        if c.is_empty() {
            return c;
        }
        self.version += 1;
        let left = add_to_slots(&mut self.slots, c.item, c.count);
        Stack { item: if left > 0 { c.item } else { AIR }, count: left }
    }
}

#[cfg(test)]
mod tests;
