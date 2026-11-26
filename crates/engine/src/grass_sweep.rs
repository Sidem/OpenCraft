//! The authority's grass sweep. Grass spreads only through block timers (`sim/timers.rs`), which start from block
//! changes, so dirt the terrain left bare on an open face (the step of a hillside) would never green on its own. Each
//! tick the authority looks down one column near each body in the loaded world and, where dirt could green
//! (`timers::can_green`) and no growth timer waits there yet, queues `Action::GrowGrass`. The core checks again and
//! starts the timer; the spread goes on from there through block changes.
//!
//! Invariants: never edits the core (an action does), never touches `Sim.rng` (the column is a hash of tick and
//! body), and a co-op client doesn't sweep. To tune: the constants.

use crate::action::Action;
use crate::block::{BlockId, DIRT, STONE};
use crate::math::IVec3;
use crate::net::Role;
use crate::sim::{can_green, TimerKind};
use crate::Game;

/// Columns within this many blocks of a body, on each axis.
const REACH: i32 = 48;
/// Cells looked at above and below a body's feet.
const DEPTH: i32 = 24;

impl Game {
    /// One column near each body: queues growth for the first dirt in it that could green.
    pub(crate) fn sweep_grass(&mut self) {
        if matches!(self.role, Role::Client(_)) {
            return;
        }
        let world = &self.sim.world;
        let mut block = |q: IVec3| world.get_block(q).unwrap_or(STONE);
        let span = (2 * REACH + 1) as u64;
        let mut found = Vec::new();
        for (slot, body) in self.bodies.iter().enumerate() {
            let Some(body) = body else { continue };
            let h = mix(self.sim.tick ^ ((slot as u64) << 56));
            let (dx, dz) = ((h % span) as i32 - REACH, ((h >> 20) % span) as i32 - REACH);
            let top = body.pos.floor() + IVec3::new(dx, DEPTH, dz);
            found
                .extend(seed_in_column(&mut block, top).filter(|&p| !self.sim.timers.pending(p, TimerKind::GrassGrow)));
        }
        for pos in found {
            self.act(Action::GrowGrass { pos });
        }
    }
}

/// The first dirt from `top` down `2 * DEPTH + 1` cells that could green.
fn seed_in_column(block: &mut impl FnMut(IVec3) -> BlockId, top: IVec3) -> Option<IVec3> {
    (0..=2 * DEPTH).map(|d| top - IVec3::new(0, d, 0)).find(|&p| block(p) == DIRT && can_green(&mut *block, p))
}

/// A well-mixed 64-bit hash (splitmix64's finaliser).
fn mix(mut x: u64) -> u64 {
    x = (x ^ (x >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    x = (x ^ (x >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    x ^ (x >> 31)
}

#[cfg(test)]
mod tests;
