//! Storage boxes: [`STORAGE_SLOTS`] item stacks. Belts and miners deliver into them; each step a
//! box pushes one item from its last non-empty slot into the next belt leading away (round-robin).

use crate::block::BlockId;
use crate::bytes::{ByteReader, ByteWriter};
use crate::inventory::{Stack, MAX_STACK};
use crate::math::IVec3;

use super::belt::Belt;

pub const STORAGE_SLOTS: usize = 24;

pub struct Storage {
    pub pos: IVec3,
    pub slots: [Stack; STORAGE_SLOTS],
    /// Belt indices leading away from this box.
    pub outs: Vec<u32>,
    pub next_out: usize,
}

impl Storage {
    pub fn new(pos: IVec3) -> Storage {
        Storage { pos, slots: [Stack::default(); STORAGE_SLOTS], outs: Vec::new(), next_out: 0 }
    }

    /// Core state: position, slots and the round-robin position (`outs` is rebuilt by `relink`).
    pub fn write_state(&self, w: &mut ByteWriter) {
        w.ivec3(self.pos);
        for s in &self.slots {
            s.write_state(w);
        }
        w.u32(self.next_out as u32);
    }

    pub fn read_state(r: &mut ByteReader) -> Option<Storage> {
        let mut s = Storage::new(r.ivec3()?);
        for slot in &mut s.slots {
            *slot = Stack::read_state(r)?;
        }
        s.next_out = r.u32()? as usize;
        Some(s)
    }

    pub fn can_accept(&self, item: BlockId) -> bool {
        self.slots.iter().any(|s| s.is_empty() || (s.item == item && s.count < MAX_STACK))
    }

    /// Pushes one item into the next belt leading away that accepts it.
    pub fn output_step(&mut self, belts: &mut [Belt]) {
        if self.outs.is_empty() {
            return;
        }
        let Some(src) = self.slots.iter().rposition(|st| !st.is_empty()) else { return };
        let item = self.slots[src].item;
        let n = self.outs.len();
        for i in 0..n {
            let slot = (self.next_out + i) % n;
            if belts[self.outs[slot] as usize].accept(item, false, 0.0) {
                let st = &mut self.slots[src];
                st.count -= 1;
                if st.count == 0 {
                    *st = Stack::default();
                }
                self.next_out = (slot + 1) % n;
                break;
            }
        }
    }
}
