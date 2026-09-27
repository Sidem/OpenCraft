//! Placing a quarry (the local player's, presentation only): while one is held and aimed at a free
//! cell, `quarry_preview` is the box it would dig there (outlined by the host), `quarry_label` says
//! how big it is and what it would yield, and R turns it a quarter (`turn_quarry`, from
//! `rotate_target`). The turn goes into the placing action's facing (`interaction.rs`). Reads loaded
//! chunks only; never touches the core.

use crate::block::{self, QUARRY};
use crate::factory::{self, survey, DigBox, DEFAULT_DEPTH, DEFAULT_WIDTH, DEPTHS};
use crate::item;
use crate::math::IVec3;
use crate::Game;

impl Game {
    /// Whether the selected slot holds a quarry.
    pub(crate) fn holds_quarry(&self) -> bool {
        let stack = self.inventory().selected_stack();
        !stack.is_empty() && stack.item.places() == Some(QUARRY)
    }

    /// The way a quarry placed now would face: the player's facing, turned by R.
    pub(crate) fn quarry_facing(&self) -> u8 {
        (factory::dir_from_yaw(self.body().yaw) + self.quarry_turn) % 4
    }

    pub(crate) fn turn_quarry(&mut self) {
        self.quarry_turn = (self.quarry_turn + 1) % 4;
    }

    /// The box a held quarry would dig if placed now, if it can be placed.
    pub(crate) fn quarry_preview(&self) -> Option<DigBox> {
        let hit = self.target.filter(|h| h.normal != IVec3::ZERO && self.holds_quarry())?;
        let pos = hit.block + hit.normal;
        self.sim.world.get_block(pos).filter(|&b| block::replaceable(b))?;
        Some(DigBox::new(pos, self.quarry_facing(), DEFAULT_WIDTH, DEFAULT_DEPTH))
    }

    /// "Quarry" and "7×7, 16 deep · about 600 blocks: stone, dirt · R turns it" on two lines ("" when
    /// not placing one).
    pub(crate) fn quarry_label(&self) -> String {
        let Some(dig) = self.quarry_preview() else { return String::new() };
        let (n, drops) = survey(&dig, 0, &self.sim.world);
        let about = if n >= 100 { (n + 5) / 10 * 10 } else { n };
        let names: Vec<String> = drops.iter().map(|&d| item::name(d).to_ascii_lowercase()).collect();
        let what = if names.is_empty() { String::new() } else { format!(": {}", names.join(", ")) };
        let (w, depth) = (dig.width, DEPTHS[DEFAULT_DEPTH as usize].label());
        format!("Quarry\n{w}×{w}, {depth} · about {about} blocks{what} · R turns it")
    }
}
