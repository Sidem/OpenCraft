//! Doors (`block/house.rs`): `Action::ToggleDoor` swaps the door block at a position for its open or closed twin, and the
//! doors of the same state stacked right above and below it with it, so a two-high doorway moves as one.
//! Invariants: only the world changes (the core's block store, `block_changed`), plus one `SimEvent::DoorMoved` per
//! cell for the sound. A closed door is solid, so the view refuses to close one on a body (`interaction.rs`).
//! To add a door: a closed and an open block in `house.rs` and the pair in `block::toggled`.

use crate::action::Action;
use crate::block::{self, BlockId};
use crate::math::{IVec3, Vec3};
use crate::physics::Aabb;
use crate::sim::{Sim, SimEvent};
use crate::Game;

/// How many doors above and below the clicked one swing with it.
const STACK_REACH: i32 = 3;

impl Sim {
    /// Swings the door at `pos` and the doors of its state above and below it; nothing if `pos` is not a door.
    pub(crate) fn toggle_door(&mut self, pos: IVec3) {
        let id = self.world.block_anywhere_or_generate(pos);
        let Some(to) = block::toggled(id) else { return };
        if !self.swing(pos, id, to) {
            return;
        }
        for step in [1, -1] {
            for k in 1..=STACK_REACH {
                let at = pos + IVec3::new(0, k * step, 0);
                if self.world.block_anywhere_or_generate(at) != id || !self.swing(at, id, to) {
                    break;
                }
            }
        }
    }

    /// Replaces the door `from` at `at` with `to`; false if the world refused.
    fn swing(&mut self, at: IVec3, from: BlockId, to: BlockId) -> bool {
        if !self.world.set_block_anywhere(at, to) {
            return false;
        }
        self.events.push(SimEvent::DoorMoved { pos: at, block: to });
        self.block_changed(at, from);
        true
    }
}

impl Game {
    /// The use button on a door (crouching places against it instead): swings it, unless closing it would shut it on
    /// the player's own body. Returns whether the click was used up.
    pub(crate) fn use_door(&mut self) -> bool {
        if !self.using || self.body().input.crouch {
            return false;
        }
        let Some(hit) = self.target.filter(|h| block::toggled(h.id).is_some()) else { return false };
        self.using = false;
        let body = self.body().aabb();
        let shuts_on_body = block::is_open_door(hit.id)
            && (-STACK_REACH..=STACK_REACH).any(|k| {
                let at = hit.block + IVec3::new(0, k, 0);
                let cell = Aabb { min: at.as_vec3(), max: at.as_vec3() + Vec3::new(1.0, 1.0, 1.0) };
                self.sim.world.get_block(at) == Some(hit.id) && cell.intersects(&body)
            });
        if !shuts_on_body {
            self.act(Action::ToggleDoor { pos: hit.block });
        }
        true
    }
}

#[cfg(test)]
mod tests;
