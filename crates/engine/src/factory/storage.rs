//! Storage boxes: a buffer of item stacks, `BOX_SLOTS[tier]` of them. Belts and miners deliver into
//! them; each step a box pushes one item from its last non-empty slot into the next belt leading away
//! (round-robin). Drawn as a reinforced timber-and-iron supply chest cached in per-chunk static GPU buffers
//! (`write_static`), with near-camera Mk studs (`tier_mark.rs`) and top lid label (`box_label.rs`) drawn dynamically.
//! Tiers add slots (an upgrade keeps every stack where it is).

use crate::block::tex;
use crate::bytes::{ByteReader, ByteWriter};
use crate::inventory::Stack;
use crate::item::ItemId;
use crate::math::{sort_small_by_key, IVec3, Vec3};

use super::belt::Belt;
use super::buffer::Buffer;
use super::render::push_box;
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

    /// Static 3D chest geometry at offset-relative `rel` (cached per chunk in `static_boxes/`): cast-iron skid base
    /// with tier collar, recessed oak plank body bound by four forged corner posts, brass side handles & front/back
    /// padlock clasps, and an overhanging iron-rimmed lid flush at `y = +0.50` for stackability and top labels.
    pub(in crate::factory) fn write_static(&self, out: &mut Vec<f32>, rel: Vec3) {
        let wood = [tex::BOX_TOP, tex::BOX_SIDE, tex::FRAME];
        let iron = [tex::MAT_GUNMETAL; 3];
        let frame = [tex::FRAME; 3];
        let brass = [tex::MAT_BRASS; 3];
        // Recessed cast-iron base skid plinth (y = -0.50 .. -0.38).
        push_box(out, rel + Vec3::new(0.0, -0.44, 0.0), 0.0, [0.92, 0.12, 0.92], 0.0, frame, false);
        // Subtle tier-coloured trim collar on upgraded boxes (y = -0.38 .. -0.33); dark iron on Mk1.
        let collar = if self.tier > 0 { [tex::FRAME, tex::stripe(self.tier), tex::FRAME] } else { iron };
        push_box(out, rel + Vec3::new(0.0, -0.355, 0.0), 0.0, [0.94, 0.05, 0.94], 0.0, collar, false);
        // Recessed timber crate body (y = -0.33 .. +0.28).
        push_box(out, rel + Vec3::new(0.0, -0.025, 0.0), 0.0, [0.86, 0.61, 0.86], 0.0, wood, false);
        // Four forged dark-iron corner angle posts wrapping the timber body (non-coplanar with body & lid).
        for (sx, sz) in [(-1.0, -1.0), (1.0, -1.0), (-1.0, 1.0), (1.0, 1.0)] {
            push_box(out, rel + Vec3::new(sx * 0.425, -0.025, sz * 0.425), 0.0, [0.09, 0.59, 0.09], 0.0, iron, false);
        }
        // Forged brass carry handles on the left/right flanks (±X).
        for sx in [-1.0, 1.0] {
            push_box(out, rel + Vec3::new(sx * 0.45, -0.02, 0.0), 0.0, [0.04, 0.06, 0.26], 0.0, brass, false);
        }
        // Front and rear forged brass latch plates with dark iron keyhole escutcheons (±Z).
        for sz in [-1.0, 1.0] {
            push_box(out, rel + Vec3::new(0.0, 0.25, sz * 0.46), 0.0, [0.16, 0.15, 0.045], 0.0, brass, false);
            push_box(
                out,
                rel + Vec3::new(0.0, 0.24, sz * 0.486),
                0.0,
                [0.055, 0.055, 0.015],
                0.0,
                [tex::SOOT; 3],
                false,
            );
        }
        // Overhanging iron lid collar (y = +0.28 .. +0.36) and hardwood lid deck flush at y = +0.50.
        push_box(out, rel + Vec3::new(0.0, 0.32, 0.0), 0.0, [0.95, 0.08, 0.95], 0.0, frame, false);
        push_box(out, rel + Vec3::new(0.0, 0.43, 0.0), 0.0, [0.90, 0.14, 0.90], 0.0, wood, false);
        // Four brass lid corner caps (outside the 0.70 × 0.70 top label plate).
        for (sx, sz) in [(-1.0, -1.0), (1.0, -1.0), (-1.0, 1.0), (1.0, 1.0)] {
            push_box(out, rel + Vec3::new(sx * 0.415, 0.435, sz * 0.415), 0.0, [0.11, 0.125, 0.11], 0.0, brass, false);
        }
    }

    /// Near-camera Mk studs and top lid label at camera-relative `rel`.
    pub(in crate::factory) fn write_dynamic(&self, out: &mut Vec<f32>, rel: Vec3) {
        tier_mark::ring(out, rel - Vec3::new(0.0, 0.43, 0.0), 0.0, [0.46, 0.46], self.tier);
        if let Some(item) = box_label::shown(&self.buf.slots, self.label) {
            box_label::draw(out, item, rel);
        }
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

    /// Full 3D model (static crate body plus near Mk studs and top label), used by `write_instances` and icon baking.
    fn model(&self, out: &mut Vec<f32>, rel: Vec3, _: f64) {
        self.write_static(out, rel);
        self.write_dynamic(out, rel);
    }
}
