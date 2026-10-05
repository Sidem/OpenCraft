//! The local player's hands for the personal helpers (`helpers/`; presentation only): holding jump in the air
//! with a jetpack in the pack and fuel to burn sends `Action::Jetpack` on changes, and with a charged hover pack
//! `Action::Hover` (which wins over the jetpack and lasts until the body lands); standing near a power pole
//! with a hover pack sends `Action::Charge`. Y sends the personal drone for the held item (`fetch_held`), and the
//! HUD line says how much fuel or charge is left or what the drone is on.

use wasm_bindgen::prelude::*;

use crate::action::Action;
use crate::helpers::{self, CHARGE_RANGE, HOVER_CAP, JET_TICKS_PER_COAL};
use crate::item::{self, HOVER_PACK, JETPACK};
use crate::math::{IVec3, Vec3};
use crate::{Game, TICK_RATE};

impl Game {
    /// Once a tick, before the bodies move: tell the core when the thrust or the hover should start or stop, and
    /// which pole the hover pack charges from.
    pub(crate) fn update_jetpack(&mut self) {
        let body = self.body();
        let in_air = !body.on_ground && !body.flying && !body.in_water;
        let (jump, feet) = (body.input.jump, body.pos);
        let inv = self.inventory();
        let (jet, charge) = self.sim.player(self.local).map_or((0, 0), |p| (p.helpers.jet, p.helpers.charge));
        let packed = inv.count(HOVER_PACK) > 0;
        let hover = packed && charge > 0 && in_air && (self.hover_sent || jump);
        let fuel = jet > 0 || inv.count(helpers::coal()) > 0;
        let thrust = jump && in_air && fuel && inv.count(JETPACK) > 0 && !hover;
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
    fn charging_pole(&self, feet: Vec3, packed: bool, charge: u32) -> Option<IVec3> {
        if !packed || (self.charge_sent.is_none() && charge >= HOVER_CAP) {
            return None;
        }
        let reach = f64::from(CHARGE_RANGE * CHARGE_RANGE);
        let d2 = |c: IVec3| {
            let d = Vec3::new(f64::from(c.x) + 0.5, f64::from(c.y) + 0.5, f64::from(c.z) + 0.5) - feet;
            d.x * d.x + d.y * d.y + d.z * d.z
        };
        self.sim.factory.poles().map(|(c, _)| c).filter(|&c| d2(c) <= reach).min_by(|&a, &b| d2(a).total_cmp(&d2(b)))
    }

    /// The HUD line of the helpers: hover charge, jetpack fuel while it burns or has fuel in the tank, or the errand
    /// under way.
    pub(crate) fn helper_label(&self) -> String {
        let Some(p) = self.sim.player(self.local) else { return String::new() };
        if p.inventory.count(HOVER_PACK) > 0 && (p.helpers.hover || p.helpers.pole.is_some() || p.helpers.charge > 0) {
            let secs = p.helpers.charge.div_ceil(TICK_RATE);
            let state = if p.helpers.pole.is_some() { ", charging" } else { "" };
            return format!("Hover pack · {secs} s of charge{state}");
        }
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
