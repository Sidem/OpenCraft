//! The local player's hands for the personal helpers (`helpers/`; presentation only): holding jump in the air
//! with a jetpack in the pack and fuel to burn sends `Action::Jetpack` on changes, and with a charged hover pack
//! `Action::Hover` (which wins over the jetpack and lasts until the body lands); standing near a power pole
//! with a hover pack sends `Action::Charge` (a long-distance pole from much further away), and the power-up and
//! power-down sounds follow whether the pack is tied to a pole. Y sends the personal drone for the held item (`fetch_held`), and the
//! HUD line says how much fuel or charge is left or what the drone is on.

use wasm_bindgen::prelude::*;

use crate::action::Action;
use crate::helpers::{self, CHARGE_HOLD, CHARGE_RANGE, CHARGE_RANGE_LONG, HOVER_CAP, JET_TICKS_PER_COAL};
use crate::item;
use crate::math::{IVec3, Vec3};
use crate::sound;
use crate::{Game, TICK_RATE};

impl Game {
    /// Once a tick, before the bodies move: tell the core when the thrust or the hover should start or stop, and
    /// which pole the hover pack charges from.
    pub(crate) fn update_jetpack(&mut self) {
        if !self.hands_synced {
            // A world saved mid-flight loads still thrusting: start from what the core says, so it gets switched off.
            if let Some(p) = self.sim.player(self.local) {
                (self.jet_sent, self.hover_sent, self.charge_sent) =
                    (p.helpers.thrusting, p.helpers.hover, p.helpers.pole);
                self.charge_heard = p.helpers.pole.is_some();
                self.hands_synced = true;
            }
        }
        self.charge_sounds();
        let body = self.body();
        let in_air = !body.on_ground && !body.flying && !body.in_water;
        let (jump, feet) = (body.input.jump, body.pos);
        let inv = self.inventory();
        let (jet, charge) = self.sim.player(self.local).map_or((0, 0), |p| (p.helpers.jet, p.helpers.charge));
        let packed = inv.has_hover_pack();
        let hover = packed && charge > 0 && in_air && (self.hover_sent || jump);
        let fuel = jet > 0 || inv.count(helpers::coal()) > 0;
        let thrust = jump && in_air && fuel && inv.has_jetpack() && !hover;
        let pole = self.charging_pole(feet, packed, charge);
        if hover != self.hover_sent {
            self.hover_sent = hover;
            self.act(Action::Hover { on: hover });
        }
        if thrust != self.jet_sent {
            self.jet_sent = thrust;
            self.act(Action::Jetpack { on: thrust });
        }
        if pole != self.charge_sent {
            if let Some(p) = pole.or(self.charge_sent) {
                self.act(Action::Charge { pole: p, on: pole.is_some() });
            }
            self.charge_sent = pole;
        }
    }

    /// The nearest pole within reach of `feet` for a hover pack to charge from: none with no pack, and a pack not
    /// tied to a pole only starts when it is not full (a tied one stays tied, so a full pack does not flicker).
    /// A long-distance pole reaches much further; the pole the pack is tied to reaches a little further still.
    fn charging_pole(&self, feet: Vec3, packed: bool, charge: u32) -> Option<IVec3> {
        if !packed || (self.charge_sent.is_none() && charge >= HOVER_CAP) {
            return None;
        }
        // Squared distance from `feet` to the pole's middle, and the squared range it must be within.
        let near = |(c, mid, long): (IVec3, Vec3, bool)| {
            let d = mid - feet;
            let hold = if self.charge_sent == Some(c) { CHARGE_HOLD } else { 0 };
            let range = f64::from(if long { CHARGE_RANGE_LONG } else { CHARGE_RANGE } + hold);
            let d2 = d.x * d.x + d.y * d.y + d.z * d.z;
            (d2 <= range * range).then_some((d2, c))
        };
        let best = self.sim.factory.charge_poles().filter_map(near).min_by(|a, b| a.0.total_cmp(&b.0));
        best.map(|b| b.1)
    }

    /// Plays the power-up sound when the local hover pack starts charging and the power-down sound when it stops
    /// (it walked out of range, the pole was removed, the pack was taken off).
    fn charge_sounds(&mut self) {
        let tied = self.sim.player(self.local).is_some_and(|p| p.helpers.pole.is_some());
        if tied != self.charge_heard {
            self.charge_heard = tied;
            let kind = if tied { sound::POWER_UP } else { sound::POWER_DOWN };
            self.sounds.push(kind, 0, Vec3::new(0.0, -0.4, 0.0), 1.0);
        }
    }

    /// The status lines of the helpers (one per line): hover charge, the errand under way, jetpack fuel while it
    /// burns or has fuel in the tank. Own HUD line, so it never hides what the player aims at.
    pub(crate) fn helper_label(&self) -> String {
        let Some(p) = self.sim.player(self.local) else { return String::new() };
        let mut lines = Vec::new();
        if p.inventory.has_hover_pack() && (p.helpers.hover || p.helpers.pole.is_some() || p.helpers.charge > 0) {
            let secs = p.helpers.charge.div_ceil(TICK_RATE);
            let state = if p.helpers.pole.is_some() { ", charging" } else { "" };
            lines.push(format!("Hover pack · {secs} s of charge{state}"));
        }
        if let Some(f) = p.helpers.fetch {
            lines.push(format!(
                "Personal drone fetching {} · back in {} s",
                item::name(f.item),
                f.left.div_ceil(TICK_RATE)
            ));
        }
        if p.helpers.thrusting || (p.helpers.jet > 0 && p.inventory.has_jetpack()) {
            let coal = p.inventory.count(helpers::coal());
            let secs = (p.helpers.jet + JET_TICKS_PER_COAL * coal).div_ceil(TICK_RATE);
            lines.push(format!("Jetpack · {secs} s of fuel ({coal} coal)"));
        }
        lines.join("\n")
    }
}

#[wasm_bindgen]
impl Game {
    /// The helpers' status text for the HUD's own line ("" when there is nothing to say).
    pub fn helper_status(&self) -> String {
        self.helper_label()
    }

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
