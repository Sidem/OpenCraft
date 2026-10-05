//! Setting cargo routes with a cargo drone in hand, part of the local player's hands (`interaction.rs`;
//! presentation plus ordinary actions, the routes themselves are `drones/cargo/`).
//! - Click a drone port to start a route there (blue outline), then click another to join them
//!   (`Action::SetRoute`): the first port sends its cargo drones to the second. Clicking the first again restarts.
//! - Crouch-click a port to clear its route.
//!
//! The outline is green where a click would do something and red where the second port is out of the first one's
//! range; the first port is blue and the aimed port's destination amber. The label says what a click does and shows
//! the trip's length and battery cost. The press is tracked with `Game::cargo_down`, the first port is
//! `Game::cargo_from` (cleared when the cargo drone leaves the hand).

#[cfg(test)]
mod tests;

use crate::action::Action;
use crate::block;
use crate::drones::cargo::trip_batteries;
use crate::item::CARGO_DRONE;
use crate::math::{IVec3, Vec3};
use crate::sound;
use crate::Game;

const GHOST_GREEN: i32 = 0x4ade80;
const BLOCKED_RED: i32 = 0xff4a3d;
const SELECTED_BLUE: i32 = 0x7dd3fc;
const ROUTE_AMBER: i32 = 0xffc247;

impl Game {
    fn cargo_in_hand(&self) -> bool {
        let stack = self.inventory().selected_stack();
        !stack.is_empty() && stack.item == CARGO_DRONE
    }

    /// The drone port the hand is aimed at (its anchor).
    fn cargo_aim(&self) -> Option<IVec3> {
        self.cargo_in_hand().then_some(())?;
        self.sim.factory.port_at(self.target?.block)
    }

    /// The distance between two ports' pads, and whether `from`'s cargo range reaches it.
    fn cargo_leg(&self, from: IVec3, to: IVec3) -> Option<(f64, bool)> {
        let ports = self.sim.factory.ports();
        let (a, b) = (ports.iter().find(|p| p.anchor == from)?, ports.iter().find(|p| p.anchor == to)?);
        let dist = (b.centre - a.centre).length();
        Some((dist, dist <= a.cargo_range))
    }

    /// Runs the cargo hand for one tick. True while a cargo drone is in hand (the press is its own).
    pub(crate) fn update_cargo_tools(&mut self) -> bool {
        if !self.cargo_in_hand() {
            self.cargo_from = None;
            return false;
        }
        let edge = self.using && !self.cargo_down;
        self.cargo_down = self.using;
        let Some(port) = self.cargo_aim().filter(|_| edge) else { return true };
        match self.cargo_from {
            _ if self.body().input.crouch => self.act(Action::SetRoute { from: port, to: port, clear: true }),
            Some(from) if from != port => {
                self.act(Action::SetRoute { from, to: port, clear: false });
                self.cargo_from = None;
            }
            _ => self.cargo_from = Some(port),
        }
        let at = self.sim.factory.port_centre(port).unwrap_or(Vec3::ZERO);
        self.play(sound::PLACE, block::sound::METAL, at, 0.5);
        true
    }

    /// Outlines (7 numbers each: lowest and highest cell, colour) of the aimed port, the first port of the route being
    /// set and the aimed port's destination.
    pub(crate) fn cargo_boxes(&self) -> Vec<i32> {
        let f = &self.sim.factory;
        let mut out = Vec::new();
        let mut outline = |port: IVec3, colour: i32| {
            if let Some((lo, hi)) = f.port_bounds(port) {
                out.extend([lo.x, lo.y, lo.z, hi.x, hi.y, hi.z, colour]);
            }
        };
        if !self.cargo_in_hand() {
            return Vec::new();
        }
        let aim = self.cargo_aim();
        if let Some(p) = aim {
            let ok = self.cargo_from.is_none_or(|from| from == p || self.cargo_leg(from, p).is_some_and(|l| l.1));
            outline(p, if ok { GHOST_GREEN } else { BLOCKED_RED });
            if let Some(dest) = self.sim.cargo.route_from(p) {
                outline(dest, ROUTE_AMBER);
            }
        }
        if let Some(from) = self.cargo_from {
            outline(from, SELECTED_BLUE);
        }
        out
    }

    /// The HUD lines while a cargo drone is in hand: a title, what a click does, then the aimed port's route.
    pub(crate) fn cargo_label(&self) -> String {
        if !self.cargo_in_hand() {
            return String::new();
        }
        let Some(port) = self.cargo_aim() else {
            return "Cargo Drone\naim at a drone port: click it, then another port, to send cargo from the first to the \
                    second"
                .to_string();
        };
        let text = match self.cargo_from {
            Some(from) if from != port => match self.cargo_leg(from, port) {
                Some((dist, true)) => {
                    format!(
                        "click to send cargo here · {} blocks, {} batteries a trip",
                        dist as u32,
                        trip_batteries(dist)
                    )
                }
                Some((dist, false)) => format!("{} blocks is too far for the first port's range", dist as u32),
                None => "the first port is gone".to_string(),
            },
            Some(_) => "click to start the route again · crouch-click to clear its route".to_string(),
            None => "click to start a route from this port · crouch-click to clear its route".to_string(),
        };
        let route = match self.sim.cargo.route_from(port).and_then(|d| self.cargo_leg(port, d).map(|l| (d, l.0))) {
            Some((dest, dist)) => format!(
                "Route: to the port at {}, {} · {} blocks, {} batteries a trip",
                dest.x,
                dest.z,
                dist as u32,
                trip_batteries(dist)
            ),
            None => "This port has no route".to_string(),
        };
        format!("Cargo Drone\n{text}\n{route}")
    }
}
