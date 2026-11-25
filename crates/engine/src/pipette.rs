//! First-person pipette ("pick block", `Middle-click` or `V`; presentation only, acts only by queuing
//! existing slot/give `Action`s): selects the item that places the aimed block, machine or ghost, moving it
//! from the backpack into the hotbar if needed, and copies its orientation and slab/stair/slope shape mode.
//! With a Paint Sprayer in hand, eyedrops the aimed surface's paint colour instead.

use crate::action::Action;
use crate::block::{self, BlockId};
use crate::factory::{self, tiers};
use crate::inventory::HOTBAR_SLOTS;
use crate::item::{self, stack_size, ItemId};
use crate::mode::Mode;
use crate::shape::{self, Kind};
use crate::sound;
use crate::Game;

impl Game {
    /// Picks the aimed block, machine or ghost into the local player's hand, copying its facing and shape.
    /// Returns a short notice when the item is not in the player's pack (empty on success or when nothing is aimed).
    pub(crate) fn pipette_aimed(&mut self) -> String {
        if self.holds_paint_sprayer() && self.eyedrop_paint_colour() {
            return String::new();
        }
        let Some((want, family_block, facing, shape_byte)) = self.pipette_resolve() else {
            return String::new();
        };
        self.apply_pipette_pose(facing, shape_byte);
        let chosen = self.find_pipette_item(want, family_block);
        if let Some(slot) = chosen {
            self.bring_slot_to_hand(slot);
            self.play(sound::PLACE, block::sound::METAL, self.body().eye(), 0.35);
            return String::new();
        }
        if self.sim.mode == Mode::Creative {
            let n = stack_size(want).min(64);
            let dest = self.creative_pipette_slot(want);
            self.act(Action::Give { item: want, count: n });
            if let Some(h) = dest {
                self.act(Action::SelectSlot { slot: h as u8 });
            }
            self.play(sound::PLACE, block::sound::METAL, self.body().eye(), 0.35);
            return String::new();
        }
        self.sounds.push(sound::DENIED, 0, crate::math::Vec3::new(0.0, -0.2, 0.0), 0.5);
        format!("No {} in your pack", item::name(want))
    }

    /// Resolves `(wanted_item, optional_family_block, optional_facing, shape_byte)` under the crosshair.
    fn pipette_resolve(&self) -> Option<(ItemId, Option<BlockId>, Option<u8>, u8)> {
        if let Some(anchor) = self.aimed_ghost() {
            if let Some(g) = self.sim.ghosts.covering(anchor) {
                let it = g.item();
                if it.places().is_some() {
                    return Some((it, Some(g.block), Some(g.facing), shape::FULL));
                }
            }
        }
        let hit = self.target?;
        let anchor = self.sim.factory.footprint_at(hit.block).map_or(hit.block, |f| f.0);
        let placed = self.sim.factory.placed_as(anchor);
        if let Some((blk, tier)) = self.sim.factory.tiered_at(anchor) {
            let it = tiers::item_of(blk, tier).unwrap_or(ItemId::block(blk));
            return Some((it, Some(blk), placed.map(|p| p.0), shape::FULL));
        }
        let direct = ItemId::block(hit.id);
        let it = if direct.places().is_some() {
            direct
        } else {
            let drop = ItemId::block(block::def(hit.id).drop);
            drop.places().is_some().then_some(drop)?
        };
        let s = if shape::is_shapeable(hit.id) { self.sim.world.get_shape(hit.block) } else { shape::FULL };
        Some((it, None, placed.map(|p| p.0), s))
    }

    /// Matches `held_shape_mode` and `place_turn` to the aimed block or machine.
    fn apply_pipette_pose(&mut self, facing: Option<u8>, s: u8) {
        let (mode, shape_dir) = match shape::kind(s) {
            Kind::Full => (shape::MODE_CUBE, None),
            Kind::Slab { quarters: 1, .. } => (shape::MODE_SLAB_QUARTER, None),
            Kind::Slab { quarters: 3, .. } => (shape::MODE_SLAB_THREE_QUARTER, None),
            Kind::Slab { .. } => (shape::MODE_SLAB_HALF, None),
            Kind::VSlab { quarters: 1, dir } => (shape::MODE_VSLAB_QUARTER, Some(dir)),
            Kind::VSlab { dir, .. } => (shape::MODE_VSLAB_HALF, Some(dir)),
            Kind::Stair { dir, .. } => (shape::MODE_STAIR, Some(dir)),
            Kind::Slope { dir, .. } => (shape::MODE_SLOPE, Some(dir)),
        };
        self.held_shape_mode = mode;
        if let Some(dir) = shape_dir.or(facing) {
            let yaw_dir = factory::dir_from_yaw(self.body().yaw);
            self.place_turn = (dir + 4 - yaw_dir) & 3;
        }
    }

    /// Finds the inventory slot holding `want`, falling back to another Mk of the same tiered family when `want` is absent.
    fn find_pipette_item(&self, want: ItemId, family_block: Option<BlockId>) -> Option<usize> {
        let open = self.inventory().open();
        let slot_of = |it: ItemId| {
            let sel = self.inventory().selected;
            if open.get(sel).is_some_and(|s| !s.is_empty() && s.item == it) {
                return Some(sel);
            }
            open.iter().position(|s| !s.is_empty() && s.item == it)
        };
        if let Some(s) = slot_of(want) {
            return Some(s);
        }
        let blk = family_block?;
        let fam = tiers::FAMILIES.iter().find(|f| f.block == blk)?;
        fam.items.iter().rev().find_map(|&it| slot_of(it))
    }

    /// Selects `slot` when on the hotbar, else moves/swaps it from the backpack into the hotbar.
    fn bring_slot_to_hand(&mut self, slot: usize) {
        let inv = self.inventory();
        if slot == inv.selected {
            return;
        }
        if slot < HOTBAR_SLOTS {
            self.act(Action::SelectSlot { slot: slot as u8 });
            return;
        }
        if let Some(empty) = inv.slots[..HOTBAR_SLOTS].iter().position(|s| s.is_empty()) {
            self.act(Action::ClickSlot { slot: slot as u8, shift: true });
            self.act(Action::SelectSlot { slot: empty as u8 });
        } else if inv.cursor.is_empty() {
            let sel = inv.selected as u8;
            self.act(Action::ClickSlot { slot: slot as u8, shift: false });
            self.act(Action::ClickSlot { slot: sel, shift: false });
            self.act(Action::ClickSlot { slot: slot as u8, shift: false });
        }
    }

    /// Where `Inventory::add` will put a freshly given stack in Creative mode (if on the hotbar).
    fn creative_pipette_slot(&self, want: ItemId) -> Option<usize> {
        let hot = &self.inventory().slots[..HOTBAR_SLOTS];
        hot.iter().position(|s| !s.is_empty() && s.item == want).or_else(|| hot.iter().position(|s| s.is_empty()))
    }
}

#[cfg(test)]
mod tests;
