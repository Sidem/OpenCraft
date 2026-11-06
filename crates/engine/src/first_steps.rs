//! The first-steps guide around the tips: where the maps' goal mark stands (the starter patch the opening, or the Go
//! electric tip, sends the player to), which recipe the build menu makes glow, and the copper patch while the Go
//! electric tip shows (a floating "Copper" prompt and its distance on the card). Presentation: lives on `Game`, reads
//! the core, never saved.
//!
//! Invariants: the copper guide shows only in a normal world, after the opening, while Go electric is the next tip, and
//! until a coal miner stands on copper or an electric miner does. To send the player somewhere new: a case in
//! `guide_goal`, and what to craft for it in `guide_craft_item`.

use crate::block::{BlockId, COAL_MINER, COAL_ORE, COPPER_ORE, IRON_ORE};
use crate::hints::GO_ELECTRIC;
use crate::item::ItemId;
use crate::math::IVec3;
use crate::opening::{miners_on, Step};
use crate::worldgen::compass;
use crate::Game;

impl Game {
    /// The patch the guide sends the player to now: its top block and its ore.
    pub(crate) fn guide_goal(&self) -> Option<(IVec3, BlockId)> {
        let o = self.sim.opening;
        match self.opening_now() {
            Step::PlaceCoal => o.and_then(|o| o.coal).map(|p| (p, COAL_ORE)),
            Step::PlaceIron => o.and_then(|o| o.iron).map(|p| (p, IRON_ORE)),
            Step::Done => self.copper_goal().map(|p| (p, COPPER_ORE)),
            _ => None,
        }
    }

    /// The item the build menu points at: a coal miner while the guide asks for one and none is held.
    pub(crate) fn guide_craft_item(&self) -> Option<ItemId> {
        let wanted = match self.opening_now() {
            Step::Craft | Step::CraftIron => true,
            Step::Done => self.copper_goal().is_some(),
            _ => false,
        };
        (wanted && self.inventory().count(COAL_MINER.into()) == 0).then_some(COAL_MINER.into())
    }

    /// The copper patch for the Go electric tip, while it is the next tip and no miner digs copper yet.
    pub(crate) fn copper_goal(&self) -> Option<IVec3> {
        let f = &self.sim.factory;
        let wanted = !self.is_creative()
            && self.opening_now() == Step::Done
            && self.hint_progress() as usize == GO_ELECTRIC
            && miners_on(f, COPPER_ORE).is_none()
            && f.electric_miners() == 0;
        if !wanted {
            return None;
        }
        *self.copper_patch.get_or_init(|| self.sim.world.generator().starter_patch(COPPER_ORE))
    }

    /// " It lies about 40 blocks north-east of you." for a cell to walk to.
    pub(crate) fn bearing(&self, cell: IVec3) -> String {
        let eye = self.body().eye();
        let (dx, dz) = (cell.x - eye.x.floor() as i32, cell.z - eye.z.floor() as i32);
        let far = ((dx * dx + dz * dz) as f64).sqrt() as i32;
        format!(" It lies about {far} blocks {} of you.", compass(dx, dz))
    }
}

#[cfg(test)]
mod tests;
