//! Player inventory (hotbar only for the MVP).

use crate::block::{BlockId, AIR};

pub const HOTBAR_SLOTS: usize = 9;
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

#[derive(Default)]
pub struct Inventory {
    pub slots: [Stack; HOTBAR_SLOTS],
    pub selected: usize,
    /// Bumped on every change so the UI can skip redundant redraws.
    pub version: u32,
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

    /// Adds items, topping up existing stacks before using empty slots. Returns what didn't fit.
    pub fn add(&mut self, item: BlockId, mut count: u32) -> u32 {
        if item == AIR || count == 0 {
            return count;
        }
        let before = count;
        for s in self.slots.iter_mut().filter(|s| !s.is_empty() && s.item == item) {
            let n = count.min(MAX_STACK - s.count);
            s.count += n;
            count -= n;
        }
        for s in self.slots.iter_mut().filter(|s| s.is_empty()) {
            if count == 0 {
                break;
            }
            let n = count.min(MAX_STACK);
            *s = Stack { item, count: n };
            count -= n;
        }
        if count != before {
            self.version += 1;
        }
        count
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stacks_then_fills_empty_slots() {
        let mut inv = Inventory::default();
        assert_eq!(inv.add(1, 10), 0);
        assert_eq!(inv.add(1, 60), 0);
        assert_eq!(inv.slots[0], Stack { item: 1, count: 64 });
        assert_eq!(inv.slots[1], Stack { item: 1, count: 6 });
        assert_eq!(inv.add(2, 3), 0);
        assert_eq!(inv.slots[2], Stack { item: 2, count: 3 });
    }

    #[test]
    fn overflow_is_returned() {
        let mut inv = Inventory::default();
        assert_eq!(inv.add(1, 64 * 9 + 5), 5);
        assert_eq!(inv.space_for(1), 0);
        assert_eq!(inv.space_for(2), 0);
    }

    #[test]
    fn take_clears_empty_slot() {
        let mut inv = Inventory::default();
        inv.add(3, 1);
        assert_eq!(inv.take_selected(1), Some((3, 1)));
        assert!(inv.slots[0].is_empty());
        assert_eq!(inv.take_selected(1), None);
    }

    #[test]
    fn scroll_wraps() {
        let mut inv = Inventory::default();
        inv.scroll(-1);
        assert_eq!(inv.selected, HOTBAR_SLOTS - 1);
        inv.scroll(1);
        assert_eq!(inv.selected, 0);
    }
}
