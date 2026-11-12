//! Storage box labels: what a box shows on its sides, so a wall of boxes reads at a glance. Each side facing the
//! camera gets a light plate with an item standing on a small ledge in front of it: by default (`AUTO`) the item the
//! box holds most of (nothing when it is empty), or the icon of a category the player picked in the box screen
//! (`Action::LabelBox`, saved with the box: `storage.rs`). Drawn within `NEAR` only; the picking is core state, the
//! drawing presentation.
//!
//! To add a category: a [`LABELS`] row (its name and the item that stands for it). Rows are saved by index: append.

use std::f32::consts::FRAC_PI_2;

use crate::block::{self, tex};
use crate::inventory::Stack;
use crate::item::{self, ItemId};
use crate::math::{IVec3, Vec3};

use super::process::draw_item;
use super::render::push_box;
use super::{Factory, Slot, DIRS};

/// The label that shows the most common item inside.
pub const AUTO: u8 = 0;

/// Every label a player can pick: its name and the item drawn for it (`ItemId::NONE` for [`AUTO`]).
pub const LABELS: [(&str, ItemId); 10] = [
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
];

/// Labels are drawn within this many blocks of the camera.
pub const NEAR: f64 = 24.0;
/// The plate (width and height, thickness) and the item's slot on its ledge (width, height), in blocks. `draw_item`
/// fits an item's diagonal to the width (it is made for displays that turn), so the slot is wider than it looks.
const PLATE: [f32; 2] = [0.66, 0.025];
const SLOT: [f32; 2] = [0.62, 0.46];

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

/// Draws the label `item` on every side of the box centred at `rel` (relative to the camera) that faces the camera.
pub fn draw(out: &mut Vec<f32>, item: ItemId, rel: Vec3) {
    if rel.length() > NEAR {
        return;
    }
    for (s, d) in DIRS.iter().enumerate() {
        let n = d.as_vec3();
        // The camera stands at -rel from the box's centre: draw only the sides it sees.
        if -(rel.x * n.x + rel.z * n.z) <= 0.5 {
            continue;
        }
        // A box's own +z looks along (-sin yaw, 0, cos yaw): this yaw turns it out of side `s`.
        let yaw = ((s + 2) % 4) as f32 * FRAC_PI_2;
        let face = rel + n * 0.5;
        let plate = face + n * f64::from(PLATE[1] / 2.0) + Vec3::new(0.0, 0.04, 0.0);
        push_box(out, plate, yaw, [PLATE[0], PLATE[0], PLATE[1]], 0.0, [tex::FRAME, tex::SIGN, tex::FRAME], false);
        // The ledge under the item, and the item standing on it in front of the plate, a flat one on its edge.
        let floor = face + n * 0.13 + Vec3::new(0.0, -0.24, 0.0);
        push_box(out, floor - Vec3::new(0.0, 0.015, 0.0), yaw, [PLATE[0], 0.03, 0.2], 0.0, [tex::FRAME; 3], false);
        draw_item(out, item, floor, yaw, SLOT, true);
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
