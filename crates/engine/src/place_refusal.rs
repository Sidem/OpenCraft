//! Telling the player why a placement did not happen (the local player's hands, presentation only; the core checks
//! every placement again). A block in hand is refused when the player's own body stands in the cell (`crowded_cell`),
//! and a multi-block machine when any cell of its footprint is taken (`footprint_preview.rs` marks those red). Both
//! show red on the screen (`crowded_box`, `crowded_label`) and, when the right button tries, play `sound::DENIED` once
//! per click and then every `REFUSE_REPEAT_SECONDS` while the button stays down (`refuse_placing`).
//! To extend: another thing that stops a click is one more arm in `refused_at`.

use crate::block::{self, BlockId};
use crate::math::{IVec3, Vec3};
use crate::physics::Aabb;
use crate::sound;
use crate::upgrade_aim::BLOCKED_RED;
use crate::Game;

/// A held right button repeats its "no" this often.
pub(crate) const REFUSE_REPEAT_SECONDS: f32 = 0.6;

impl Game {
    /// Whether a solid `placed` block at `pos` would trap the local player's body.
    pub(crate) fn body_blocks(&self, placed: BlockId, pos: IVec3) -> bool {
        let cell = Aabb { min: pos.as_vec3(), max: pos.as_vec3() + Vec3::new(1.0, 1.0, 1.0) };
        block::SOLID[placed as usize] && cell.intersects(&self.body().aabb())
    }

    /// The cell a held single block would fill, when only the player's own body stops it.
    pub(crate) fn crowded_cell(&self) -> Option<IVec3> {
        let hit = self.target.filter(|h| h.normal != IVec3::ZERO)?;
        let stack = self.inventory().selected_stack();
        let placed = stack.item.places().filter(|_| !stack.is_empty() && self.held_footprint().is_none())?;
        let pos = hit.block + hit.normal;
        (self.sim.world.get_block(pos).is_some_and(block::replaceable) && self.body_blocks(placed, pos)).then_some(pos)
    }

    /// A red box on the cell `crowded_cell` names (the placing outline's format: `Game::placement_box`).
    pub(crate) fn crowded_box(&self) -> Vec<i32> {
        self.crowded_cell().map_or_else(Vec::new, |c| vec![c.x, c.y, c.z, c.x, c.y, c.z, BLOCKED_RED])
    }

    /// "Step aside" and why, while the body is what stops the held block ("" otherwise).
    pub(crate) fn crowded_label(&self) -> String {
        match self.crowded_cell() {
            Some(_) => format!("{}\nYou are standing in the way: step aside", self.held_name()),
            None => String::new(),
        }
    }

    /// Called when a right-click found nothing to place: plays the "no" if something stopped it.
    pub(crate) fn refuse_placing(&mut self) {
        if self.refuse_cooldown > 0.0 {
            return;
        }
        let Some(at) = self.refused_at() else { return };
        self.refuse_cooldown = REFUSE_REPEAT_SECONDS;
        self.play(sound::DENIED, 0, at.as_vec3() + Vec3::new(0.5, 0.5, 0.5), 0.8);
    }

    /// Where the held block was refused: its footprint's anchor when a cell is taken, or the crowded cell.
    fn refused_at(&self) -> Option<IVec3> {
        if self.held_footprint().is_some() {
            let (_, anchor, _, cells) = self.footprint_ghost()?;
            return cells.iter().any(|c| c.1).then_some(anchor);
        }
        self.crowded_cell()
    }

    /// The name of the item in hand.
    fn held_name(&self) -> &'static str {
        let stack = self.inventory().selected_stack();
        stack.item.places().map_or("Block", |b| block::def(b).name)
    }
}

#[cfg(test)]
mod tests;
