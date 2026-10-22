//! The sickle (the local player's hands, presentation only): with one in the selected slot, the use button (right-click,
//! like the scanner and the telescope) on natural ground cuts the plants of the 3 × 3 columns around the aimed ground
//! block by queuing `Action::ClearCover`; holding it sweeps, one swing every `SWING_SECONDS`. The sickle breaks and
//! places nothing, and the left button mines as with any item in hand. Aimed at anything that bears no cover (a
//! machine, a wall, stone), the click falls through to what it always did, so a sickle in the hotbar never blocks a
//! panel.
//!
//! Invariants: the plants themselves are cleared in the core (`world/cover_clear.rs`); this only decides *where*. A patch
//! that is already cleared sends nothing (a held button must not flood a co-op host). To change the patch size:
//! `cover_clear::PATCH_RADIUS`; the aim box below follows it.

use crate::action::Action;
use crate::block::sound::GRASS;
use crate::item::SICKLE;
use crate::math::Vec3;
use crate::mesher::bears_cover;
use crate::sound;
use crate::world::PATCH_RADIUS;
use crate::Game;

/// Seconds between swings while the button is held.
const SWING_SECONDS: f32 = 0.25;
/// The aim box's colour: a leaf green.
const PATCH_GREEN: i32 = 0x6BCB6B;

impl Game {
    /// Whether a sickle is in the selected slot.
    fn sickle_held(&self) -> bool {
        self.inventory().selected_stack().item == SICKLE
    }

    /// Runs the use button for a held sickle for one tick. Returns whether it swung (the click is then used up).
    pub(crate) fn update_sickle(&mut self, dt: f32) -> bool {
        if !self.using || !self.sickle_held() {
            return false;
        }
        let Some(hit) = self.target.filter(|h| bears_cover(h.id)) else { return false };
        if self.sim.world.patch_cleared(hit.block) {
            return true;
        }
        self.use_cooldown -= dt;
        if self.use_cooldown <= 0.0 {
            self.use_cooldown = SWING_SECONDS;
            self.act(Action::ClearCover { pos: hit.block });
            self.play(sound::DIG, GRASS, hit.block.as_vec3() + Vec3::new(0.5, 1.0, 0.5), 0.5);
        }
        true
    }

    /// The HUD label with a sickle in hand; `None` otherwise.
    pub(crate) fn sickle_label(&self) -> Option<String> {
        if !self.sickle_held() {
            return None;
        }
        Some(match self.target.filter(|h| bears_cover(h.id)) {
            Some(h) if self.sim.world.patch_cleared(h.block) => "Sickle\nNothing left to cut here".into(),
            Some(_) => "Sickle\nRight-click cuts the small plants of the patch (hold to sweep) · trees stay".into(),
            None => "Sickle\nAim at the ground and right-click to cut the plants around it".into(),
        })
    }

    /// The patch the sickle would cut, as one outline box on top of the ground (7 numbers; empty when it cuts nothing).
    pub(crate) fn sickle_box(&self) -> Vec<i32> {
        let Some(hit) = self.target.filter(|h| self.sickle_held() && bears_cover(h.id)) else { return Vec::new() };
        if self.sim.world.patch_cleared(hit.block) {
            return Vec::new();
        }
        let (c, r) = (hit.block, PATCH_RADIUS);
        vec![c.x - r, c.y + 1, c.z - r, c.x + r, c.y + 1, c.z + r, PATCH_GREEN]
    }
}

#[cfg(test)]
mod tests;
