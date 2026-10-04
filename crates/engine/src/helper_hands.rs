//! The local player's hands for the personal helpers (`helpers/`; presentation only): holding jump in the air
//! with a jetpack in the pack and fuel to burn sends `Action::Jetpack` on changes, Y sends the personal drone
//! for the held item (`fetch_held`), and the HUD line says how much fuel is left or what the drone is on.

use wasm_bindgen::prelude::*;

use crate::action::Action;
use crate::helpers::{self, JET_TICKS_PER_COAL};
use crate::item::{self, JETPACK};
use crate::{Game, TICK_RATE};

impl Game {
    /// Once a tick, before the bodies move: tell the core when the thrust should start or stop.
    pub(crate) fn update_jetpack(&mut self) {
        let body = self.body();
        let in_air = !body.on_ground && !body.flying && !body.in_water;
        let inv = self.inventory();
        let jet = self.sim.player(self.local).map_or(0, |p| p.helpers.jet);
        let fuel = jet > 0 || inv.count(helpers::coal()) > 0;
        let want = body.input.jump && in_air && fuel && inv.count(JETPACK) > 0;
        if want != self.jet_sent {
            self.jet_sent = want;
            self.act(Action::Jetpack { on: want });
        }
    }

    /// The HUD line of the helpers: jetpack fuel while it burns or has fuel in the tank, or the errand under way.
    pub(crate) fn helper_label(&self) -> String {
        let Some(p) = self.sim.player(self.local) else { return String::new() };
        if let Some(f) = p.helpers.fetch {
            return format!(
                "Personal drone fetching {} · back in {} s",
                item::name(f.item),
                f.left.div_ceil(TICK_RATE)
            );
        }
        if p.helpers.thrusting || (p.helpers.jet > 0 && p.inventory.count(JETPACK) > 0) {
            let coal = p.inventory.count(helpers::coal());
            let secs = (p.helpers.jet + JET_TICKS_PER_COAL * coal).div_ceil(TICK_RATE);
            return format!("Jetpack · {secs} s of fuel ({coal} coal)");
        }
        String::new()
    }
}

#[wasm_bindgen]
impl Game {
    /// Y: sends the personal drone for more of the item in hand. False when nothing is sent (no drone in the
    /// pack, an empty hand, or no box within reach holds it).
    pub fn fetch_held(&mut self) -> bool {
        let stack = self.inventory().selected_stack();
        let at = self.body().pos.floor();
        let Some(p) = self.sim.player(self.local) else { return false };
        let ready = !stack.is_empty()
            && p.helpers.fetch.is_none()
            && p.inventory.count(item::PERSONAL_DRONE) > 0
            && self.sim.factory.nearest_box_with(at, stack.item, helpers::FETCH_RANGE).is_some();
        if ready {
            self.act(Action::Fetch { item: stack.item, at });
        }
        ready
    }
}
