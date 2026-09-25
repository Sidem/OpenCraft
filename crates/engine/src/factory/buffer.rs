//! Item buffers: the stacks a machine holds (a box's slots, a miner's output, a processing
//! machine's inputs and outputs). The slot count comes from the machine table (`MACHINES`) and never
//! changes. Stacks follow the item's stack size, and filling tops up matching stacks first.

use crate::bytes::{ByteReader, ByteWriter};
use crate::inventory::{add_to_slots, Stack};
use crate::item::{stack_size, ItemId};

use super::belt::Belt;

pub struct Buffer {
    pub slots: Vec<Stack>,
}

impl Buffer {
    pub fn new(slots: usize) -> Buffer {
        Buffer { slots: vec![Stack::default(); slots] }
    }

    /// Adds up to `n` of `item`; returns what didn't fit.
    pub fn add(&mut self, item: ItemId, n: u32) -> u32 {
        add_to_slots(&mut self.slots, item, n)
    }

    /// Whether one more `item` fits.
    pub fn can_accept(&self, item: ItemId) -> bool {
        self.slots.iter().any(|s| s.is_empty() || (s.item == item && s.count < stack_size(item)))
    }

    /// How many of `item` it holds.
    #[cfg(test)]
    pub fn count(&self, item: ItemId) -> u32 {
        self.slots.iter().filter(|s| !s.is_empty() && s.item == item).map(|s| s.count).sum()
    }

    /// How many items it holds in all.
    pub fn total(&self) -> u32 {
        self.slots.iter().map(|s| s.count).sum()
    }

    /// Room left for `item` across all slots.
    pub fn space_for(&self, item: ItemId) -> u32 {
        let max = stack_size(item);
        let room = |s: &Stack| {
            if s.is_empty() {
                max
            } else if s.item == item {
                max.saturating_sub(s.count)
            } else {
                0
            }
        };
        self.slots.iter().map(room).sum()
    }

    /// Removes `n` items from `slot` (which must hold at least that many).
    pub fn take(&mut self, slot: usize, n: u32) {
        let s = &mut self.slots[slot];
        s.count -= n;
        if s.count == 0 {
            *s = Stack::default();
        }
    }

    /// Offers every stack to `take(item, count)`, which returns how many it accepted.
    pub fn drain(&mut self, mut take: impl FnMut(ItemId, u32) -> u32) {
        for s in self.slots.iter_mut().filter(|s| !s.is_empty()) {
            s.count -= take(s.item, s.count).min(s.count);
            if s.count == 0 {
                *s = Stack::default();
            }
        }
    }

    /// The non-empty stacks (what it drops when the machine is removed).
    pub fn contents(&self) -> Vec<Stack> {
        self.slots.iter().filter(|s| !s.is_empty()).copied().collect()
    }

    /// Pushes one item from its last non-empty slot into the next belt in `outs` that accepts it,
    /// round-robin from `next_out`. How boxes and processing machines feed belts leading away.
    pub fn feed(&mut self, outs: &[u32], next_out: &mut usize, belts: &mut [Belt]) {
        let Some(src) = self.slots.iter().rposition(|st| !st.is_empty()) else { return };
        let item = self.slots[src].item;
        let n = outs.len();
        for i in 0..n {
            let slot = (*next_out + i) % n;
            if belts[outs[slot] as usize].accept(item, false, 0.0) {
                self.take(src, 1);
                *next_out = (slot + 1) % n;
                return;
            }
        }
    }

    /// Every slot in order.
    pub fn write_state(&self, w: &mut ByteWriter) {
        self.slots.iter().for_each(|s| s.write_state(w));
    }

    pub fn read_state(r: &mut ByteReader, slots: usize) -> Option<Buffer> {
        let mut b = Buffer::new(slots);
        for s in &mut b.slots {
            *s = Stack::read_state(r)?;
        }
        Some(b)
    }
}
