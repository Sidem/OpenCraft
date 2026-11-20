//! Storage boxes: a buffer of item stacks, `BOX_SLOTS[tier]` of them. Belts and miners deliver into
//! them; each step a box pushes one item from its last non-empty slot into the next belt leading away
//! (round-robin). The box is an ordinary meshed cube; its model is its Mk studs (`tier_mark.rs`) and its label
//! (`box_label.rs`: what it holds most of, or a category the player picked). Tiers add slots (an upgrade keeps every
//! stack where it is).

use crate::bytes::{ByteReader, ByteWriter};
use crate::inventory::Stack;
use crate::item::ItemId;
use crate::math::{sort_small_by_key, IVec3, Vec3};

use super::belt::Belt;
use super::buffer::Buffer;
use super::{box_label, tier_mark, Factory, Machine};

/// Slots of each tier, Mk1 first.
pub const BOX_SLOTS: [usize; 5] = [24, 36, 48, 60, 72];

pub struct Storage {
    pub pos: IVec3,
    pub tier: u8,
    pub buf: Buffer,
    /// Belt indices leading away from this box.
    pub outs: Vec<u32>,
    pub next_out: usize,
    /// A `box_label::LABELS` index: what its sides show.
    pub label: u8,
}

impl Storage {
    pub fn new(pos: IVec3) -> Storage {
        Storage { pos, tier: 0, buf: Buffer::new(BOX_SLOTS[0]), outs: Vec::new(), next_out: 0, label: box_label::AUTO }
    }

    /// Makes it tier `tier`, adding empty slots.
    pub fn set_tier(&mut self, tier: u8) {
        self.tier = tier;
        self.buf.slots.resize(BOX_SLOTS[tier as usize], Stack::default());
    }

    /// Pushes one item into the next belt leading away that accepts it.
    pub fn step(&mut self, belts: &mut [Belt]) {
        self.buf.feed(&self.outs, &mut self.next_out, belts, self.pos);
    }
}

impl Machine for Storage {
    fn pos(&self) -> IVec3 {
        self.pos
    }

    /// Core state: position, slots, the round-robin position and the label (`outs` is rebuilt by `relink`).
    fn write_state(&self, w: &mut ByteWriter) {
        w.ivec3(self.pos);
        w.u8(self.tier);
        self.buf.write_state(w);
        w.u32(self.next_out as u32);
        w.u8(self.label);
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
        // Older boxes show what they hold.
        s.label = if r.version >= 69 { r.u8()? } else { box_label::AUTO };
        if s.label as usize >= box_label::LABELS.len() {
            return None;
        }
        Some(s)
    }

    fn contents(&self) -> Vec<Stack> {
        self.buf.contents()
    }

    /// Capacity bar and top held stacks as icon chips.
    fn describe(&self, _: &Factory) -> String {
        use super::describe::{hud_bar, hud_items};
        let used = self.buf.slots.iter().filter(|st| !st.is_empty()).count();
        let cap = self.buf.slots.len();
        let mut totals: Vec<(ItemId, u32)> = Vec::new();
        for st in self.buf.contents() {
            match totals.iter_mut().find(|(item, _)| *item == st.item) {
                Some((_, n)) => *n += st.count,
                None => totals.push((st.item, st.count)),
            }
        }
        sort_small_by_key(&mut totals, |&(_, n)| u32::MAX - n);
        let tone = if used == cap { "warn" } else { "accent" };
        let mut lines = vec![hud_bar("Slots", used as u64, cap as u64, &format!("{used} / {cap}"), tone)];
        if !totals.is_empty() {
            let shown: Vec<(ItemId, u32)> = totals.iter().take(6).copied().collect();
            lines.extend(hud_items("Stored", &shown));
            if totals.len() > 6 {
                lines.push(format!("+{} more kinds", totals.len() - 6));
            }
            lines.push("\tRMB=Take all".to_string());
        }
        lines.join("\n")
    }

    /// Its Mk studs, low on its four sides, and its label on the sides facing the camera (the cube itself is meshed).
    fn model(&self, out: &mut Vec<f32>, rel: Vec3, _: f64) {
        tier_mark::ring(out, rel - Vec3::new(0.0, 0.32, 0.0), 0.0, [0.5, 0.5], self.tier);
        if let Some(item) = box_label::shown(&self.buf.slots, self.label) {
            box_label::draw(out, item, rel);
        }
    }
}
