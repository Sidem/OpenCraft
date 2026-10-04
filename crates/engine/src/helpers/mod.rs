//! Personal helpers (Milestone 7): what a player carries in the pack to get around and to fetch, kept in the
//! core per player ([`Helpers`], in `PlayerCore`, saved since version 28).
//!
//! - **Coal jetpack** (`item::JETPACK`, anywhere in the pack): `Action::Jetpack { on }` starts and stops the
//!   thrust (the player holds jump in the air; the hands send it on changes). While it is on, the core burns
//!   [`JET_TICKS_PER_COAL`] ticks of fuel per coal taken from the pack and stops by itself when the pack has no
//!   coal left. The body lifts while `thrusting` (`player.rs`, set from here by `authority.rs`).
//! - **Personal drone** (`item::PERSONAL_DRONE`, in the pack): `Action::Fetch { item, at }` sends it to the
//!   nearest storage box within [`FETCH_RANGE`] blocks of the player's cell that holds the item; after the
//!   round trip a stack of it lands in the pack (what does not fit goes back in the box). One errand at a time.
//!
//! Invariants: `thrusting` implies `jet > 0` and a jetpack in the pack; an errand needs the drone in the pack
//! when it starts and when it ends. Both are deterministic: integer counters only (the trip time uses one
//! `sqrt`). To change a number: the constants below.

#[cfg(test)]
mod tests;

use crate::block;
use crate::bytes::{ByteReader, ByteWriter};
use crate::inventory::{Inventory, Stack};
use crate::item::{self, ItemId, JETPACK, PERSONAL_DRONE};
use crate::math::IVec3;
use crate::sim::{PlayerId, Sim};
use crate::TICK_RATE;

/// Ticks of thrust one coal gives.
pub const JET_TICKS_PER_COAL: u32 = 10 * TICK_RATE;
/// How far from the player the drone looks for a box, in blocks.
pub const FETCH_RANGE: i32 = 32;
/// Blocks a second the personal drone flies (it is quicker than a construction drone).
const FETCH_SPEED: f64 = 12.0;
/// The shortest errand, in ticks.
const FETCH_MIN_TICKS: u32 = TICK_RATE;

/// An errand in progress.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Fetch {
    pub item: ItemId,
    /// The box it flies to.
    pub from: IVec3,
    /// Ticks until it is back, and the whole trip's ticks (for drawing the flight).
    pub left: u32,
    pub total: u32,
}

#[derive(Default, Clone, PartialEq, Eq, Debug)]
pub struct Helpers {
    /// The jetpack is thrusting now.
    pub thrusting: bool,
    /// Ticks of thrust left in the fuel already burnt.
    pub jet: u32,
    pub fetch: Option<Fetch>,
}

impl Helpers {
    pub fn write_state(&self, w: &mut ByteWriter) {
        w.bool(self.thrusting);
        w.u32(self.jet);
        w.bool(self.fetch.is_some());
        if let Some(f) = &self.fetch {
            w.item(f.item);
            w.ivec3(f.from);
            w.u32(f.left);
            w.u32(f.total);
        }
    }

    pub fn read_state(r: &mut ByteReader) -> Option<Helpers> {
        let (thrusting, jet) = (r.bool()?, r.u32()?);
        let fetch = if r.bool()? {
            Some(Fetch { item: r.item()?, from: r.ivec3()?, left: r.u32()?, total: r.u32()? })
        } else {
            None
        };
        Some(Helpers { thrusting, jet, fetch })
    }

    /// One tick of the jetpack: burns a tick of fuel, taking the next coal from `inv` when it runs dry.
    fn step_jet(&mut self, inv: &mut Inventory) {
        if !self.thrusting {
            return;
        }
        if inv.count(JETPACK) == 0 {
            self.thrusting = false;
            return;
        }
        self.jet = self.jet.saturating_sub(1);
        if self.jet == 0 {
            self.thrusting = self.refuel(inv);
        }
    }

    /// Burns a coal from `inv` into the tank if it is empty; whether there is fuel.
    fn refuel(&mut self, inv: &mut Inventory) -> bool {
        if self.jet == 0 && inv.remove(coal(), 1) {
            self.jet = JET_TICKS_PER_COAL;
        }
        self.jet > 0
    }
}

/// The jetpack's fuel.
pub fn coal() -> ItemId {
    ItemId::block(block::COAL_ORE)
}

impl Sim {
    /// `Action::Jetpack`: thrust on needs a jetpack and fuel (a coal is burnt if the tank is empty).
    pub(crate) fn set_thrust(&mut self, player: PlayerId, on: bool) {
        let Some(Some(core)) = self.players.get_mut(player.0 as usize) else { return };
        core.helpers.thrusting = on && core.inventory.count(JETPACK) > 0 && core.helpers.refuel(&mut core.inventory);
    }

    /// `Action::Fetch`: sends the drone for `item` if the player has one, is not on an errand and a box holds it.
    pub(crate) fn start_fetch(&mut self, player: PlayerId, item: ItemId, at: IVec3) {
        let Some(Some(core)) = self.players.get_mut(player.0 as usize) else { return };
        if core.helpers.fetch.is_some() || core.inventory.count(PERSONAL_DRONE) == 0 || item::def(item).is_none() {
            return;
        }
        let Some((from, d2)) = self.factory.nearest_box_with(at, item, FETCH_RANGE) else { return };
        let trip = 2.0 * (d2 as f64).sqrt() / FETCH_SPEED;
        let total = ((trip * f64::from(TICK_RATE)) as u32).max(FETCH_MIN_TICKS);
        core.helpers.fetch = Some(Fetch { item, from, left: total, total });
    }

    /// One tick of every player's jetpack and errand.
    pub(crate) fn run_helpers(&mut self) {
        for slot in &mut self.players {
            let Some(core) = slot else { continue };
            core.helpers.step_jet(&mut core.inventory);
            let Some(f) = core.helpers.fetch.as_mut() else { continue };
            f.left = f.left.saturating_sub(1);
            if f.left > 0 {
                continue;
            }
            let (item, from) = (f.item, f.from);
            core.helpers.fetch = None;
            if core.inventory.count(PERSONAL_DRONE) == 0 {
                continue;
            }
            let took = self.factory.box_take_up_to(from, item, item::stack_size(item));
            let left = core.inventory.add(item, took);
            if left > 0 {
                self.factory.store_in_boxes(&[from], Stack { item, count: left });
            }
        }
    }
}
