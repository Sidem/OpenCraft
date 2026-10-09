//! Personal helpers (Milestone 7): what a player carries in the pack to get around and to fetch, kept in the
//! core per player ([`Helpers`], in `PlayerCore`, saved since version 28).
//!
//! - **Coal jetpack** (`item::JETPACK`, worn in the Jetpack slot, `equipment.rs`): `Action::Jetpack { on }` starts and stops the
//!   thrust (the player holds jump in the air; the hands send it on changes). While it is on, the core burns
//!   [`JET_TICKS_PER_COAL`] ticks of fuel per coal taken from the pack and stops by itself when the pack has no
//!   coal left. The body lifts while `thrusting` (`player.rs`, set from here by `authority.rs`).
//! - **Personal drone** (`item::PERSONAL_DRONE`, in the pack): `Action::Fetch { item, at }` sends it to the
//!   nearest storage box within [`FETCH_RANGE`] blocks of the player's cell that holds the item; after the
//!   round trip a stack of it lands in the pack (what does not fit goes back in the box). One errand at a time.
//!
//! - **Hover pack** (`item::HOVER_PACK`, in the pack, Milestone 9): `Action::Hover { on }` starts and stops the
//!   hover (the player holds jump in the air; the body holds its height and moves faster while `hover`, `player.rs`).
//!   It runs on `charge` (ticks of hover, at most [`HOVER_CAP`]), one a tick. `Action::Charge { pole, on }` ties the
//!   pack to a power pole (the hands send it as the player walks within [`CHARGE_RANGE`] of one) and the charge
//!   fills [`CHARGE_RATE`] a tick while that pole stands and the pack is held.
//!
//! Invariants: `thrusting` implies `jet > 0` and a jetpack in the pack; `hover` implies `charge > 0` and a hover pack
//! in the pack; an errand needs the drone in the pack when it starts and when it ends. All deterministic: integer
//! counters only (the trip time uses one `sqrt`). To change a number: the constants below.

#[cfg(test)]
mod tests;

use crate::block;
use crate::bytes::{ByteReader, ByteWriter};
use crate::inventory::{Inventory, Stack};
use crate::item::{self, ItemId, HOVER_PACK, PERSONAL_DRONE};
use crate::math::IVec3;
use crate::sim::{PlayerId, Sim};
use crate::TICK_RATE;

/// Ticks of thrust one coal gives.
pub const JET_TICKS_PER_COAL: u32 = 10 * TICK_RATE;
/// The most charge the hover pack holds, in ticks of hover.
pub const HOVER_CAP: u32 = 90 * TICK_RATE;
/// Charge gained a tick while tied to a pole (the pack drains one a tick while hovering).
pub const CHARGE_RATE: u32 = 2;
/// How close to a pole the player must stand to charge, in blocks (the hands' check; the core wants the pole to exist).
pub const CHARGE_RANGE: i32 = 6;
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
    /// The hover pack is hovering now.
    pub hover: bool,
    /// Ticks of hover left in the hover pack.
    pub charge: u32,
    /// The pole the hover pack is charging from.
    pub pole: Option<IVec3>,
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
        w.bool(self.hover);
        w.u32(self.charge);
        w.bool(self.pole.is_some());
        if let Some(p) = self.pole {
            w.ivec3(p);
        }
    }

    pub fn read_state(r: &mut ByteReader) -> Option<Helpers> {
        let (thrusting, jet) = (r.bool()?, r.u32()?);
        let fetch = if r.bool()? {
            Some(Fetch { item: r.item()?, from: r.ivec3()?, left: r.u32()?, total: r.u32()? })
        } else {
            None
        };
        let mut h = Helpers { thrusting, jet, fetch, ..Helpers::default() };
        if r.version >= 36 {
            h.hover = r.bool()?;
            h.charge = r.u32()?;
            h.pole = if r.bool()? { Some(r.ivec3()?) } else { None };
        }
        Some(h)
    }

    /// One tick of the hover pack: hovering costs a tick of charge; a pole in reach fills it.
    fn step_hover(&mut self, inv: &Inventory, pole_stands: impl Fn(IVec3) -> bool) {
        let packed = inv.count(HOVER_PACK) > 0;
        if self.hover {
            self.charge = self.charge.saturating_sub(1);
            self.hover = packed && self.charge > 0;
        }
        match self.pole {
            Some(p) if packed && pole_stands(p) => self.charge = (self.charge + CHARGE_RATE).min(HOVER_CAP),
            _ => self.pole = None,
        }
    }

    /// One tick of the jetpack: burns a tick of fuel, taking the next coal from `inv` when it runs dry.
    fn step_jet(&mut self, inv: &mut Inventory) {
        if !self.thrusting {
            return;
        }
        if !inv.has_jetpack() {
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
        core.helpers.thrusting = on && core.inventory.has_jetpack() && core.helpers.refuel(&mut core.inventory);
    }

    /// `Action::Hover`: hover on needs a hover pack and some charge.
    pub(crate) fn set_hover(&mut self, player: PlayerId, on: bool) {
        let Some(Some(core)) = self.players.get_mut(player.0 as usize) else { return };
        core.helpers.hover = on && core.inventory.count(HOVER_PACK) > 0 && core.helpers.charge > 0;
    }

    /// `Action::Charge`: ties the hover pack to the pole at `pole` (which must stand) or lets it go.
    pub(crate) fn set_charge(&mut self, player: PlayerId, pole: IVec3, on: bool) {
        let Some(Some(core)) = self.players.get_mut(player.0 as usize) else { return };
        let tied = on && core.inventory.count(HOVER_PACK) > 0 && self.factory.poles().any(|(c, _)| c == pole);
        core.helpers.pole = tied.then_some(pole);
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

    /// One tick of every player's jetpack, hover pack and errand.
    pub(crate) fn run_helpers(&mut self) {
        for slot in &mut self.players {
            let Some(core) = slot else { continue };
            core.helpers.step_jet(&mut core.inventory);
            core.helpers.step_hover(&core.inventory, |p| self.factory.poles().any(|(c, _)| c == p));
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
