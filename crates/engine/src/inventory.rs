//! Player inventory: a 9-slot hotbar (slots 0..9) plus a 27-slot backpack, and the stack held by
//! the mouse cursor while the inventory screen is open. `version` increments on every change so
//! the UI redraws only when needed. `add_to_slots`, `click_stack` and `sort_stacks` are shared with
//! storage boxes.

use crate::bytes::{ByteReader, ByteWriter};
use crate::item::{stack_size, ItemId};
use crate::math::sort_small_by_key;
use crate::tools;

pub const HOTBAR_SLOTS: usize = 9;
pub const INVENTORY_SLOTS: usize = 36;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Stack {
    pub item: ItemId,
    pub count: u32,
}

impl Stack {
    pub fn is_empty(&self) -> bool {
        self.count == 0
    }

    /// Item and count; every empty stack writes as (`ItemId::NONE`, 0), whatever item it last held.
    pub fn write_state(&self, w: &mut ByteWriter) {
        w.item(if self.is_empty() { ItemId::NONE } else { self.item });
        w.u32(self.count);
    }

    pub fn read_state(r: &mut ByteReader) -> Option<Stack> {
        let (item, count) = (r.item()?, r.u32()?);
        match count {
            0 => Some(Stack::default()),
            n if item != ItemId::NONE && n <= stack_size(item) => Some(Stack { item, count }),
            _ => None,
        }
    }
}

/// Adds up to `count` of `item` into `slots`, topping up matching stacks before filling empty
/// slots (in slice order). Returns what didn't fit.
pub fn add_to_slots(slots: &mut [Stack], item: ItemId, mut count: u32) -> u32 {
    if item == ItemId::NONE {
        return count;
    }
    let max = stack_size(item);
    for s in slots.iter_mut().filter(|s| !s.is_empty() && s.item == item) {
        let n = count.min(max.saturating_sub(s.count));
        s.count += n;
        count -= n;
    }
    for s in slots.iter_mut().filter(|s| s.is_empty()) {
        if count == 0 {
            break;
        }
        let n = count.min(max);
        *s = Stack { item, count: n };
        count -= n;
    }
    count
}

/// Moves every stack of `item` from `from` into `to` until `to` has no room for more; what doesn't fit
/// stays in `from`. Returns whether anything moved.
pub fn move_all_of(from: &mut [Stack], to: &mut [Stack], item: ItemId) -> bool {
    let mut moved = false;
    for s in from.iter_mut().filter(|s| s.item == item && !s.is_empty()) {
        let left = add_to_slots(to, item, s.count);
        moved |= left != s.count;
        *s = if left == 0 { Stack::default() } else { Stack { item, count: left } };
        if left > 0 {
            break; // the target is full of this item
        }
    }
    moved
}

/// Sorts `slots` in place: like items merged into full stacks (a tool's count is its wear, so tools
/// never merge), ordered by item id (blocks first) with the fullest stack first, empty slots last.
pub fn sort_stacks(slots: &mut [Stack]) {
    let mut sorted: Vec<Stack> = Vec::with_capacity(slots.len());
    for s in slots.iter().filter(|s| !s.is_empty()) {
        let mut left = s.count;
        if tools::tool(s.item).is_none() {
            for t in sorted.iter_mut().filter(|t| t.item == s.item) {
                let n = left.min(stack_size(s.item).saturating_sub(t.count));
                t.count += n;
                left -= n;
            }
        }
        if left > 0 {
            sorted.push(Stack { item: s.item, count: left });
        }
    }
    sort_small_by_key(&mut sorted, |s| (s.item.0, std::cmp::Reverse(s.count)));
    for (i, slot) in slots.iter_mut().enumerate() {
        *slot = sorted.get(i).copied().unwrap_or_default();
    }
}

/// A click on slot `s` holding the cursor stack `c` (inventory or box screen): pick up, put down,
/// merge or swap. Returns whether anything changed.
pub fn click_stack(s: &mut Stack, c: &mut Stack) -> bool {
    if c.is_empty() {
        if s.is_empty() {
            return false;
        }
        *c = std::mem::take(s);
    } else if s.is_empty() {
        *s = std::mem::take(c);
    } else if s.item == c.item {
        let n = c.count.min(stack_size(s.item).saturating_sub(s.count));
        s.count += n;
        c.count -= n;
        if c.count == 0 {
            *c = Stack::default();
        }
    } else {
        std::mem::swap(s, c);
    }
    true
}

#[derive(Clone)]
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
        Self::EMPTY
    }
}

impl Inventory {
    pub const EMPTY: Inventory = {
        let none = Stack { item: ItemId::NONE, count: 0 };
        Inventory { slots: [none; INVENTORY_SLOTS], selected: 0, cursor: none, version: 0 }
    };

    /// Core state: slots, cursor and selection (not `version`, which only serves the UI).
    pub fn write_state(&self, w: &mut ByteWriter) {
        for s in &self.slots {
            s.write_state(w);
        }
        self.cursor.write_state(w);
        w.u8(self.selected as u8);
    }

    pub fn read_state(r: &mut ByteReader) -> Option<Inventory> {
        let mut inv = Inventory::default();
        for s in &mut inv.slots {
            *s = Stack::read_state(r)?;
        }
        inv.cursor = Stack::read_state(r)?;
        inv.selected = r.u8()? as usize;
        (inv.selected < HOTBAR_SLOTS).then_some(inv)
    }

    /// Room left for `item` across all slots.
    pub fn space_for(&self, item: ItemId) -> u32 {
        let max = stack_size(item);
        self.slots
            .iter()
            .map(|s| match s {
                s if s.is_empty() => max,
                s if s.item == item => max.saturating_sub(s.count),
                _ => 0,
            })
            .sum()
    }

    /// Adds items: tops up existing stacks anywhere, then fills empty hotbar slots before the
    /// backpack. Returns what didn't fit.
    pub fn add(&mut self, item: ItemId, count: u32) -> u32 {
        if item == ItemId::NONE || count == 0 {
            return count;
        }
        let left = add_to_slots(&mut self.slots, item, count);
        if left != count {
            self.version += 1;
        }
        left
    }

    /// Total number of `item` held.
    pub fn count(&self, item: ItemId) -> u32 {
        self.slots.iter().filter(|s| s.item == item).map(|s| s.count).sum()
    }

    /// Removes `n` of `item`, backpack first so the hotbar keeps its layout. All or nothing.
    pub fn remove(&mut self, item: ItemId, mut n: u32) -> bool {
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

    /// Removes up to `n` items from `slot`, returning the item id and amount taken.
    pub fn take_slot(&mut self, slot: usize, n: u32) -> Option<(ItemId, u32)> {
        let s = self.slots.get_mut(slot)?;
        if s.is_empty() || n == 0 {
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
        if click_stack(s, &mut self.cursor) {
            self.version += 1;
        }
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

    /// Shift-right-click: moves every stack of the item in `slot` between the hotbar and the backpack.
    pub fn quick_move_all(&mut self, slot: usize) {
        let Some(item) = self.slots.get(slot).map(|s| s.item) else { return };
        let (hotbar, backpack) = self.slots.split_at_mut(HOTBAR_SLOTS);
        let moved =
            if slot < HOTBAR_SLOTS { move_all_of(hotbar, backpack, item) } else { move_all_of(backpack, hotbar, item) };
        if moved {
            self.version += 1;
        }
    }

    /// Sorts the backpack (the hotbar keeps the layout the player made).
    pub fn sort(&mut self) {
        let before = self.slots;
        sort_stacks(&mut self.slots[HOTBAR_SLOTS..]);
        if self.slots != before {
            self.version += 1;
        }
    }

    /// Puts the cursor stack back into the inventory. Returns whatever didn't fit.
    pub fn return_cursor(&mut self) -> Stack {
        let c = std::mem::take(&mut self.cursor);
        if c.is_empty() {
            return c;
        }
        self.version += 1;
        let left = add_to_slots(&mut self.slots, c.item, c.count);
        Stack { item: if left > 0 { c.item } else { ItemId::NONE }, count: left }
    }
}

#[cfg(test)]
mod tests;
