//! The local hands' side of the rope (GEAR-ROPE): with a coil in the selected slot, a click hangs it from the cell in
//! front of the aimed face, or on below the aimed strand, by queuing `Action::HangRope`; the core decides how far it
//! reaches (`action/rope.rs`). Climbing it is the body's (`player::climbable`). Presentation only: one action a click.

use crate::action::{Action, PER_COIL};
use crate::block::ROPE;
use crate::factory::hud_keys;
use crate::item::ROPE_ITEM;
use crate::Game;

impl Game {
    fn rope_held(&self) -> bool {
        self.inventory().selected_stack().item == ROPE_ITEM
    }

    /// Runs the use button for a held coil for one tick. True when it owns the click.
    pub(crate) fn update_rope(&mut self) -> bool {
        if !self.using || !self.rope_held() {
            return false;
        }
        if let Some(hit) = self.target {
            let pos = if hit.id == ROPE { hit.block } else { hit.block + hit.normal };
            let slot = self.inventory().selected as u8;
            self.act(Action::HangRope { pos, slot });
        }
        self.using = false;
        true
    }

    /// The HUD label with a coil in hand.
    pub(crate) fn rope_label(&self) -> Option<String> {
        self.rope_held().then(|| {
            let keys = hud_keys(&[("RMB", "Hang"), ("Space / Shift", "Climb")]);
            format!("Rope\nHangs {PER_COIL} blocks down from a ledge's side · aim at a rope to add more\n{keys}")
        })
    }
}
