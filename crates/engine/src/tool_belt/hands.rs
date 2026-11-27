//! The local hands' side of the Tool Belt: whether the swing being made is a wide one (crouched, a Mk3+ belt, cells to
//! take), read from loaded chunks only to time the swing and to pick `Action::MineWide`; the core works the cells out
//! again from its own world when it applies it. And what the swinging hand shows: the belt's head for the block.

use super::{face_of, head, reach, wide_cells, Reach};
use crate::block::{self, AIR};
use crate::inventory::Inventory;
use crate::item::ItemId;
use crate::player::Player;
use crate::raycast::{raycast, RayHit};
use crate::world::World;
use crate::Game;

/// What the right hand of `body` shows: while it swings at a block its worn belt has a head for (stone, wood or soil
/// within reach of `world`'s loaded chunks), that head (an old tool's model); otherwise `held`. Presentation only.
pub(crate) fn hand_item(world: &World, body: &Player, inv: &Inventory, held: ItemId) -> ItemId {
    if body.gesture & 1 == 0 || inv.belt() == ItemId::NONE {
        return held;
    }
    let solid = |p| world.get_block(p).filter(|&b| b != AIR && block::def(b).break_time >= 0.0);
    let hit = raycast(body.eye(), body.look_dir(), inv.reach(), solid);
    match hit.map(|h| head(inv.belt(), h.id)) {
        Some(item) if item != ItemId::NONE => item,
        _ => held,
    }
}

impl Game {
    /// The face and the number of blocks (the hit included) a swing at `hit` takes, when it takes more than one.
    pub(crate) fn wide_swing(&self, hit: RayHit) -> Option<(u8, usize)> {
        let reach = reach(self.inventory().belt());
        if reach == Reach::One || !self.body().input.crouch {
            return None;
        }
        let face = face_of(hit.normal)?;
        let (factory, towns) = (&self.sim.factory, &self.sim.settlements);
        let free = |p| !factory.occupies(p) && !towns.in_plot(p);
        if !free(hit.block) {
            return None;
        }
        let world = &self.sim.world;
        let cells = wide_cells(reach, hit.block, face, |p| world.get_block(p).unwrap_or(AIR), free);
        (!cells.is_empty()).then_some((face, cells.len() + 1))
    }
}
