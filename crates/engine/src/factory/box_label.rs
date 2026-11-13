//! Storage box labels: what a box shows on its lid, so a row of boxes reads at a glance. A light plate lies on the lid
//! with an item standing on it, turned to the camera: by default (`AUTO`) the item the box holds most of (nothing
//! when it is empty), or the icon of a category the player picked in the box screen (`Action::LabelBox`, saved with
//! the box: `storage.rs`). Drawn as far as the box is (beyond `DETAIL` as one plain box); the picking is core state,
//! the drawing presentation.
//!
//! To add a category: a [`LABELS`] row (its name and the item that stands for it). Rows are saved by index: append.

use crate::block::{self, tex};
use crate::inventory::Stack;
use crate::item::{self, ItemId};
use crate::math::{IVec3, Vec3};

use super::process::draw_item;
use super::render::push_box;
use super::{Factory, Slot};

/// The label that shows the most common item inside.
pub const AUTO: u8 = 0;

/// The label that makes a box the personal drone's catch-all: it fills it, last, with anything (`stock.rs`).
pub const MISC: u8 = 10;

/// Every label a player can pick: its name and the item drawn for it (`ItemId::NONE` for [`AUTO`]).
pub const LABELS: [(&str, ItemId); 11] = [
    ("Contents", ItemId::NONE),
    ("Ores", ItemId::block(block::IRON_ORE)),
    ("Metals", item::IRON_INGOT),
    ("Parts", item::GEAR),
    ("Electronics", item::CIRCUIT),
    ("Tools", item::IRON_PICKAXE),
    ("Wood", ItemId::block(block::LOG)),
    ("Stone", ItemId::block(block::STONE_BRICKS)),
    ("Fuel", ItemId::block(block::COAL_ORE)),
    ("Science", item::RED_PACK),
    ("Misc (drone fills)", item::PERSONAL_DRONE),
];

/// Within this many blocks a label is the item's model; farther, one plain box in its colours (it is a few pixels
/// there, and a wall of boxes stays cheap).
pub const DETAIL: f64 = 48.0;
/// The plate on the lid (side, thickness) and the item's slot on it (width, height), in blocks. `draw_item` fits an
/// item's diagonal to the width (it is made for displays that turn), so the slot is wider than it looks.
const PLATE: [f32; 2] = [0.7, 0.03];
const SLOT: [f32; 2] = [0.66, 0.5];

/// The item a box with `slots` and `label` shows: the label's icon, or for [`AUTO`] the item it holds the most of
/// (the first such in slot order on a tie), `None` when there is nothing to show.
pub fn shown(slots: &[Stack], label: u8) -> Option<ItemId> {
    if let Some(&(_, icon)) = LABELS.get(label as usize).filter(|_| label != AUTO) {
        return Some(icon);
    }
    let mut best: Option<(ItemId, u32)> = None;
    for (i, s) in slots.iter().enumerate().filter(|(_, s)| !s.is_empty()) {
        if slots[..i].iter().any(|t| t.item == s.item) {
            continue; // counted with its first stack
        }
        let total: u32 = slots[i..].iter().filter(|t| t.item == s.item).map(|t| t.count).sum();
        if best.is_none_or(|(_, n)| total > n) {
            best = Some((s.item, total));
        }
    }
    best.map(|(item, _)| item)
}

/// Draws the label `item` on the lid of the box centred at `rel` (relative to the camera): a light plate with the
/// item standing on it, turned to face the camera so it reads from every side (a flat item stands on its edge).
pub fn draw(out: &mut Vec<f32>, item: ItemId, rel: Vec3) {
    // A box's own +z looks along (-sin yaw, 0, cos yaw): this yaw points it at the camera, which is at -rel.
    let yaw = rel.x.atan2(-rel.z) as f32;
    let lid = rel + Vec3::new(0.0, 0.5, 0.0);
    let plate = [tex::SIGN, tex::FRAME, tex::FRAME];
    push_box(
        out,
        lid + Vec3::new(0.0, f64::from(PLATE[1]) / 2.0, 0.0),
        0.0,
        [PLATE[0], PLATE[1], PLATE[0]],
        0.0,
        plate,
        false,
    );
    let floor = lid + Vec3::new(0.0, f64::from(PLATE[1]), 0.0);
    if rel.length() <= DETAIL {
        draw_item(out, item, floor, yaw, SLOT, true);
    } else if let Some(def) = item::def(item) {
        let k = SLOT[1] * 0.8 / def.size[1].max(def.size[0]).max(def.size[2]);
        let size = def.size.map(|s| s * k);
        push_box(out, floor + Vec3::new(0.0, f64::from(size[1]) / 2.0, 0.0), yaw, size, 0.0, def.tex, false);
    }
}

impl Factory {
    /// The label of the storage box at `pos` (`None` where there is none).
    pub fn box_label(&self, pos: IVec3) -> Option<u8> {
        match self.at.get(&pos) {
            Some(&Slot::Storage(i)) => Some(self.storages[i as usize].label),
            _ => None,
        }
    }

    /// Labels the storage box at `pos` (nothing changes for another machine or an unknown label).
    pub fn set_box_label(&mut self, pos: IVec3, label: u8) {
        if let (Some(&Slot::Storage(i)), true) = (self.at.get(&pos), (label as usize) < LABELS.len()) {
            self.storages[i as usize].label = label;
        }
    }
}

#[cfg(test)]
mod tests;
