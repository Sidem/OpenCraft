//! Keeping one belt item selected: a click on an item (the mining button) selects it, a click on anything else lets go,
//! and while it is selected its readout stays up and follows it along the belts, so a moving item can be read at leisure.
//! Presentation only: the selection never feeds the core. The one thing it changes is what the pick-up key takes
//! (`Action::TakeBeltItem`), and that only when the item is within reach.
//!
//! The item is found again each frame by its `ItemKey` (`factory/belt_items.rs`); if nothing fits any more (it was taken,
//! delivered into a machine or the belt was removed) the selection ends. What is shown is the item the crosshair is on,
//! else the selected one. To show more about an item: `Factory::belt_item_report`.

use crate::action::Action;
use crate::factory::{ItemAim, ItemKey};
use crate::math::Vec3;
use crate::Game;

/// The selected item and where it was last seen.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Selection {
    key: ItemKey,
    seen: ItemAim,
}

/// What the pick-up key can do with the item the readout shows (`Game::item_state`, the host reads the number).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ItemState {
    /// No item shown.
    None = 0,
    /// Pointed at, and within reach.
    Pointed = 1,
    /// Selected and within reach.
    Selected = 2,
    /// Selected but too far to pick up.
    OutOfReach = 3,
    /// Selected but inside a splitter or filter, where it cannot be taken.
    InRouter = 4,
}

impl Game {
    /// A press of the mining button: selects the item under the crosshair, or lets go of the selection if there is none.
    pub(crate) fn click_item(&mut self) {
        let tick = self.sim.tick;
        self.selected_item = self.target_item.map(|seen| Selection { key: seen.key(tick), seen });
    }

    /// Follows the selected item to where it is now (after the frame's targeting).
    pub(crate) fn track_selected_item(&mut self) {
        let Some(sel) = self.selected_item else { return };
        self.selected_item = self
            .sim
            .factory
            .find_item(sel.key, self.sim.tick, sel.seen.cell)
            .map(|seen| Selection { key: seen.key(self.sim.tick), seen });
    }

    /// The item the readout is about: the one pointed at, else the selected one.
    pub(crate) fn shown_item(&self) -> Option<ItemAim> {
        self.target_item.or(self.selected_item.map(|s| s.seen))
    }

    /// Whether the shown item is the selected one.
    fn shown_is_selected(&self) -> bool {
        match (self.target_item, self.selected_item) {
            (Some(aim), Some(sel)) => (aim.cell, aim.index) == (sel.seen.cell, sel.seen.index),
            (None, Some(_)) => true,
            _ => false,
        }
    }

    pub(crate) fn item_state(&self) -> ItemState {
        let Some(aim) = self.shown_item() else { return ItemState::None };
        let b = aim.bounds;
        let middle = aim.cell.as_vec3()
            + Vec3::new(b[0] as f64, b[1] as f64, b[2] as f64)
            + Vec3::new(b[3] as f64, b[4] as f64, b[5] as f64) * 0.5;
        match (self.shown_is_selected(), aim.in_router()) {
            (false, _) => ItemState::Pointed,
            (true, true) => ItemState::InRouter,
            (true, false) if (middle - self.render_eye).length() > self.reach() => ItemState::OutOfReach,
            (true, false) => ItemState::Selected,
        }
    }

    /// Takes the shown item if the state allows.
    pub(crate) fn take_shown_item(&mut self) {
        if let (Some(aim), ItemState::Pointed | ItemState::Selected) = (self.shown_item(), self.item_state()) {
            self.act(Action::TakeBeltItem { pos: aim.cell, item: aim.item, at: aim.at });
        }
    }

    /// The selected item's box in the world: lowest corner, then size; empty when none is selected.
    pub(crate) fn selected_box(&self) -> Vec<f32> {
        let Some(sel) = self.selected_item else { return Vec::new() };
        let (c, b) = (sel.seen.cell, sel.seen.bounds);
        vec![c.x as f32 + b[0], c.y as f32 + b[1], c.z as f32 + b[2], b[3], b[4], b[5]]
    }
}

#[cfg(test)]
mod tests;
