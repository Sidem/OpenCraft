//! Personal helpers (Milestone 7): what a player carries in the pack to get around and to fetch, kept in the
//! core per player ([`Helpers`], in `PlayerCore`, saved since version 28).
//!
//! - **Coal jetpack** (`item::JETPACK`, worn in the Jetpack slot, `equipment.rs`): `Action::Jetpack { on }` starts and stops the
//!   thrust (the player holds jump in the air; the hands send it on changes). While it is on, the core burns
//!   [`JET_TICKS_PER_COAL`] ticks of fuel per coal taken from the pack and stops by itself when the pack has no
//!   coal left. The body lifts while `thrusting` (`player.rs`, set from here by `authority.rs`).
//! - **Personal drone** (`item::PERSONAL_DRONE`, in the pack): fetches, stores, puts away and gathers the parts of
//!   a craft between the player and their boxes and machines (`drone/`, its own module and state).
//!
//! - **Hover pack** (`item::HOVER_PACK`, worn in the jetpack's slot (so not with a jetpack), `equipment.rs`, Milestone 9): `Action::Hover { on }` starts and stops the
//!   hover (the player holds jump in the air; the body holds its height and moves faster while `hover`, `player.rs`).
//!   It runs on `charge` (ticks of hover, at most [`HOVER_CAP`]), one a tick. `Action::Charge { pole, on }` ties the
//!   pack to a power pole (the hands send it as the player walks within the distance that pole wires to another of
//!   its kind: `Factory::charge_poles`, `pole::span`) and the charge fills [`CHARGE_RATE`] a tick while that pole
//!   stands and the pack is held. The hands play the power-up and power-down sounds as the tie starts and ends.
//!
//! Invariants: `thrusting` implies `jet > 0` and a jetpack in the pack; `hover` implies `charge > 0` and a hover pack
//! worn. All deterministic: integer counters only. To change a number: the constants below.

pub mod drone;
#[cfg(test)]
mod tests;

use crate::block;
use crate::bytes::{ByteReader, ByteWriter};
use crate::inventory::Inventory;
use crate::item::ItemId;
use crate::math::IVec3;
use crate::sim::{PlayerId, Sim};
use crate::TICK_RATE;

/// Ticks of thrust one coal gives.
pub const JET_TICKS_PER_COAL: u32 = 10 * TICK_RATE;
/// The most charge the hover pack holds, in ticks of hover.
pub const HOVER_CAP: u32 = 90 * TICK_RATE;
/// Charge gained a tick while tied to a pole (the pack drains one a tick while hovering).
pub const CHARGE_RATE: u32 = 2;
/// A pack already tied to a pole stays tied this much further out, so walking along the edge does not flicker.
pub const CHARGE_HOLD: i32 = 2;

#[derive(Default, Clone, PartialEq, Eq, Debug)]
pub struct Helpers {
    /// The jetpack is thrusting now.
    pub thrusting: bool,
    /// Ticks of thrust left in the fuel already burnt.
    pub jet: u32,
    /// The personal drone's errands (`drone/`).
    pub drone: drone::Drone,
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
        self.drone.write_state(w);
        w.bool(self.hover);
        w.u32(self.charge);
        w.bool(self.pole.is_some());
        if let Some(p) = self.pole {
            w.ivec3(p);
        }
    }

    pub fn read_state(r: &mut ByteReader) -> Option<Helpers> {
        let (thrusting, jet) = (r.bool()?, r.u32()?);
        let drone = if r.version >= 71 {
            drone::Drone::read_state(r)?
        } else if r.bool()? {
            // One errand: item, box, ticks left, ticks in all.
            let (item, from) = (r.item()?, r.ivec3()?);
            r.u32()?;
            r.u32()?;
            drone::Drone::from_errand(item, from)
        } else {
            drone::Drone::default()
        };
        let mut h = Helpers { thrusting, jet, drone, ..Helpers::default() };
        if r.version >= 36 {
            h.hover = r.bool()?;
            h.charge = r.u32()?;
            h.pole = if r.bool()? { Some(r.ivec3()?) } else { None };
        }
        Some(h)
    }

    /// One tick of the hover pack: hovering costs a tick of charge; a pole in reach fills it.
    fn step_hover(&mut self, inv: &Inventory, pole_stands: impl Fn(IVec3) -> bool) {
        let packed = inv.has_hover_pack();
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

/// Test helper: the core bytes `stream` of `sim` as saves before version 71 wrote each player's helpers, with the
/// drone's state cut down to one `false` (no errand). Every drone must be idle. `offhand::before_offhand` calls it.
#[cfg(test)]
pub(crate) fn before_drone_orders(sim: &Sim, stream: &[u8]) -> Vec<u8> {
    let mut out = crate::sim::before_felling(sim, stream);
    for p in sim.players.iter().flatten() {
        assert!(p.helpers.drone.idle(), "a save before 71 had no drone orders");
        let mut lead = ByteWriter::default();
        p.inventory.write_state(&mut lead);
        lead.u64(p.key);
        p.crafts.write_state(&mut lead);
        let at = out.windows(lead.bytes.len()).position(|s| s == lead.bytes).expect("the player's bytes");
        let start = at + lead.bytes.len() + 5; // after thrusting and jet
        let mut drone = ByteWriter::default();
        p.helpers.drone.write_state(&mut drone);
        out.splice(start..start + drone.bytes.len(), [0]);
    }
    out
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
        core.helpers.hover = on && core.inventory.has_hover_pack() && core.helpers.charge > 0;
    }

    /// `Action::Charge`: ties the hover pack to the pole at `pole` (which must stand) or lets it go.
    pub(crate) fn set_charge(&mut self, player: PlayerId, pole: IVec3, on: bool) {
        let Some(Some(core)) = self.players.get_mut(player.0 as usize) else { return };
        let tied = on && core.inventory.has_hover_pack() && self.factory.charge_poles().any(|(c, ..)| c == pole);
        core.helpers.pole = tied.then_some(pole);
    }

    /// One tick of every player's jetpack, hover pack and personal drone.
    pub(crate) fn run_helpers(&mut self) {
        for slot in &mut self.players {
            let Some(core) = slot else { continue };
            core.helpers.step_jet(&mut core.inventory);
            core.helpers.step_hover(&core.inventory, |p| self.factory.charge_poles().any(|(c, ..)| c == p));
        }
        self.run_drones();
    }
}
