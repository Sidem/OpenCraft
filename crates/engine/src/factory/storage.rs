//! Storage boxes: a buffer of item stacks, `BOX_SLOTS[tier]` of them. Belts and miners deliver into
//! them; each step a box pushes one item from its last non-empty slot into the next belt leading away
//! (round-robin). The box is an ordinary meshed cube, so it has no model of its own. Tiers add slots
//! (an upgrade keeps every stack where it is).

use crate::bytes::{ByteReader, ByteWriter};
use crate::inventory::Stack;
use crate::item::{self, ItemId};
use crate::math::{sort_small_by_key, IVec3, Vec3};

use super::belt::Belt;
use super::buffer::Buffer;
use super::describe::fmt_int;
use super::{Factory, Machine};

/// Slots of each tier, Mk1 first.
pub const BOX_SLOTS: [usize; 3] = [24, 36, 48];

pub struct Storage {
    pub pos: IVec3,
    pub tier: u8,
    pub buf: Buffer,
    /// Belt indices leading away from this box.
    pub outs: Vec<u32>,
    pub next_out: usize,
}

impl Storage {
    pub fn new(pos: IVec3) -> Storage {
        Storage { pos, tier: 0, buf: Buffer::new(BOX_SLOTS[0]), outs: Vec::new(), next_out: 0 }
    }

    /// Makes it tier `tier`, adding empty slots.
    pub fn set_tier(&mut self, tier: u8) {
        self.tier = tier;
        self.buf.slots.resize(BOX_SLOTS[tier as usize], Stack::default());
    }

    /// Pushes one item into the next belt leading away that accepts it.
    pub fn step(&mut self, belts: &mut [Belt]) {
        self.buf.feed(&self.outs, &mut self.next_out, belts);
    }
}

impl Machine for Storage {
    fn pos(&self) -> IVec3 {
        self.pos
    }

    /// Core state: position, slots and the round-robin position (`outs` is rebuilt by `relink`).
    fn write_state(&self, w: &mut ByteWriter) {
        w.ivec3(self.pos);
        w.u8(self.tier);
        self.buf.write_state(w);
        w.u32(self.next_out as u32);
    }

    fn read_state(r: &mut ByteReader) -> Option<Storage> {
        let mut s = Storage::new(r.ivec3()?);
        let tier = if r.version >= 21 { r.u8()? } else { 0 };
        if tier as usize >= BOX_SLOTS.len() {
            return None;
        }
        s.set_tier(tier);
        s.buf = Buffer::read_state(r, s.buf.slots.len())?;
        s.next_out = r.u32()? as usize;
        Some(s)
    }

    fn contents(&self) -> Vec<Stack> {
        self.buf.contents()
    }

    /// Slots used and the three largest stocks.
    fn describe(&self, _: &Factory) -> String {
        let used = self.buf.slots.iter().filter(|st| !st.is_empty()).count();
        let mut totals: Vec<(ItemId, u32)> = Vec::new();
        for st in self.buf.contents() {
            match totals.iter_mut().find(|(item, _)| *item == st.item) {
                Some((_, n)) => *n += st.count,
                None => totals.push((st.item, st.count)),
            }
        }
        sort_small_by_key(&mut totals, |&(_, n)| u32::MAX - n);
        let mut lines = vec![format!("{used} of {} slots used", self.buf.slots.len())];
        if !totals.is_empty() {
            let list: Vec<String> = totals
                .iter()
                .take(3)
                .map(|(item, n)| format!("{} {}", fmt_int(*n as u64), item::name(*item)))
                .collect();
            lines.push(list.join(", ") + if totals.len() > 3 { ", ..." } else { "" });
            lines.push("Right-click to take everything".to_string());
        }
        lines.join("\n")
    }

    fn model(&self, _: &mut Vec<f32>, _: Vec3, _: f64) {}
}
