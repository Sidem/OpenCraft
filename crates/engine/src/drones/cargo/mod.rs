//! Cargo drones (Milestone 9): port-to-port hauling, the long-distance partner of the trains. A route joins a source
//! drone port to a destination port (`Action::SetRoute`; one route per source, many sources may share a destination).
//! A port that keeps cargo drones (item `CARGO_DRONE`; a port keeps construction or cargo drones, never both) with a
//! route sends one out every [`LAUNCH_EVERY`] ticks while it is powered, a drone is home, and:
//!
//! - the destination is a port within the source tier's cargo range (`HangarTier::cargo_range`), checked when the
//!   route is set and again at launch;
//! - the boxes touching the source pad hold something to carry (the first stack that is not a battery or a drone) and
//!   the boxes at the destination have room for some of it;
//! - the source's boxes hold the trip's batteries: one for each [`BLOCKS_PER_BATTERY`] blocks of the round trip, at
//!   least one. They are used up at launch, so a far route is paid for in aluminium.
//!
//! The drone takes up to [`LOAD`] of the item (as much as the destination has room for), flies to the destination pad
//! at [`SPEED`], puts the load into the boxes there, and flies home. What does not fit comes back and goes into the
//! source boxes. If the destination is gone it turns round with the load; if the source is gone the drone and its
//! load fall as items. Breaking a port brings its cargo drones back as items (`Sim::recall_cargo`).
//!
//! Core state ([`Cargo`], saved since version 37): the routes and the drones in flight. Positions are `f64` moved by
//! `+ - * / sqrt` only. Invariants: a drone is in the list or on a pad, never both; `Courier::load` is empty on the
//! way home unless the destination could not take it all. To change a number: the constants below.

#[cfg(test)]
mod tests;

use crate::bytes::{ByteReader, ByteWriter};
use crate::inventory::Stack;
use crate::item::{self, BATTERY, CARGO_DRONE, DRONE};
use crate::math::{IVec3, Vec3};
use crate::sim::Sim;
use crate::{TICK, TICK_RATE};

use super::approach;
use crate::factory::PortInfo;

/// Blocks a second a cargo drone flies.
pub const SPEED: f64 = 12.0;
/// Ticks between a port's launches.
const LAUNCH_EVERY: u64 = 2 * TICK_RATE as u64;
/// Items a trip carries at most.
pub const LOAD: u32 = 64;
/// Blocks of flight (there and back) one battery pays for.
pub const BLOCKS_PER_BATTERY: f64 = 150.0;
/// How high above the pad a cargo drone flies when it lands or unloads.
const HOVER: f64 = 1.2;
/// Most routes a world keeps.
const MAX_ROUTES: usize = 64;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Leg {
    /// Flying to the destination.
    Out,
    /// Flying back to the source.
    Home,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Courier {
    /// The source port's anchor.
    pub port: IVec3,
    /// The destination port's anchor.
    pub dest: IVec3,
    pub pos: Vec3,
    /// Where it was a tick ago (derived, for drawing between ticks; not saved).
    pub prev: Vec3,
    pub leg: Leg,
    pub load: Stack,
}

#[derive(Default, Clone, PartialEq, Debug)]
pub struct Cargo {
    /// `(source, destination)` port anchors, at most one per source.
    pub routes: Vec<(IVec3, IVec3)>,
    pub list: Vec<Courier>,
}

impl Cargo {
    /// How many cargo drones of the port at `port` are out.
    pub fn out_of(&self, port: IVec3) -> u32 {
        self.list.iter().filter(|c| c.port == port).count() as u32
    }

    /// Where the port at `from` sends its drones.
    pub fn route_from(&self, from: IVec3) -> Option<IVec3> {
        self.routes.iter().find(|r| r.0 == from).map(|r| r.1)
    }

    pub fn write_state(&self, w: &mut ByteWriter) {
        w.count(self.routes.len());
        for &(a, b) in &self.routes {
            w.ivec3(a);
            w.ivec3(b);
        }
        w.count(self.list.len());
        for c in &self.list {
            w.ivec3(c.port);
            w.ivec3(c.dest);
            w.vec3(c.pos);
            w.u8(c.leg as u8);
            w.item(c.load.item);
            w.u32(c.load.count);
        }
    }

    pub fn read_state(r: &mut ByteReader) -> Option<Cargo> {
        let mut cargo = Cargo::default();
        for _ in 0..r.count()? {
            cargo.routes.push((r.ivec3()?, r.ivec3()?));
        }
        for _ in 0..r.count()? {
            let (port, dest, pos) = (r.ivec3()?, r.ivec3()?, r.vec3()?);
            let leg = match r.u8()? {
                0 => Leg::Out,
                1 => Leg::Home,
                _ => return None,
            };
            let load = Stack { item: r.item()?, count: r.u32()? };
            if load.count > LOAD || !pos.x.is_finite() || !pos.y.is_finite() || !pos.z.is_finite() {
                return None;
            }
            cargo.list.push(Courier { port, dest, pos, prev: pos, leg, load });
        }
        Some(cargo)
    }
}

/// Batteries a trip over `dist` blocks (one way) costs.
pub fn trip_batteries(dist: f64) -> u32 {
    ((2.0 * dist / BLOCKS_PER_BATTERY).ceil() as u32).max(1)
}

impl Sim {
    /// `Action::SetRoute`: joins the ports at `from` and `to` (any cell of either), or with `clear` removes `from`'s
    /// route. Refused quietly unless both are drone ports, distinct, and `to` is within `from`'s cargo range.
    pub(crate) fn set_route(&mut self, from: IVec3, to: IVec3, clear: bool) {
        let Some(from) = self.factory.port_at(from) else { return };
        if clear {
            self.cargo.routes.retain(|r| r.0 != from);
            return;
        }
        let Some(to) = self.factory.port_at(to).filter(|&t| t != from) else { return };
        let ports = self.factory.ports();
        let centre = |a: IVec3| ports.iter().find(|p| p.anchor == a);
        let (Some(a), Some(b)) = (centre(from), centre(to)) else { return };
        if (b.centre - a.centre).length() > a.cargo_range {
            return;
        }
        if let Some(r) = self.cargo.routes.iter_mut().find(|r| r.0 == from) {
            r.1 = to;
        } else if self.cargo.routes.len() < MAX_ROUTES {
            self.cargo.routes.push((from, to));
        }
    }

    /// Brings back every cargo drone of the port at `port` that is out, as a stack of cargo drone items (breaking the
    /// port), and drops the routes to and from it.
    pub(crate) fn recall_cargo(&mut self, port: IVec3) -> Option<Stack> {
        self.cargo.routes.retain(|r| r.0 != port && r.1 != port);
        let before = self.cargo.list.len();
        self.cargo.list.retain(|c| c.port != port);
        let back = (before - self.cargo.list.len()) as u32;
        (back > 0).then_some(Stack { item: CARGO_DRONE, count: back })
    }

    /// One tick of every cargo drone: flights and unloading, landings, then launches from the ports.
    pub(crate) fn step_cargo(&mut self) {
        if self.cargo.routes.is_empty() && self.cargo.list.is_empty() {
            return;
        }
        let flying = std::mem::take(&mut self.cargo.list);
        let mut kept = Vec::with_capacity(flying.len());
        for mut c in flying {
            c.prev = c.pos;
            if self.fly_courier(&mut c) {
                kept.push(c);
            }
        }
        self.cargo.list = kept;
        let ports = self.factory.ports();
        for p in &ports {
            let stagger =
                (p.anchor.x.wrapping_mul(31).wrapping_add(p.anchor.z.wrapping_mul(17))).rem_euclid(LAUNCH_EVERY as i32);
            if p.couriers > 0 && p.powered && (self.tick + stagger as u64).is_multiple_of(LAUNCH_EVERY) {
                self.launch_courier(p, &ports);
            }
        }
    }

    /// Advances `c` one tick; false when it has landed or been lost.
    fn fly_courier(&mut self, c: &mut Courier) -> bool {
        let step = SPEED * TICK;
        let up = Vec3::new(0.0, HOVER, 0.0);
        match c.leg {
            Leg::Out => {
                let Some(goal) = self.factory.port_centre(c.dest) else {
                    c.leg = Leg::Home;
                    return true;
                };
                let (pos, arrived) = approach(c.pos, goal + up, step);
                c.pos = pos;
                if arrived {
                    let boxes = self.factory.port_boxes(c.dest);
                    c.load = self.factory.store_in_boxes(&boxes, c.load);
                    c.leg = Leg::Home;
                }
                true
            }
            Leg::Home => {
                let Some(centre) = self.factory.port_centre(c.port) else {
                    self.lose_courier(c);
                    return false;
                };
                let (pos, arrived) = approach(c.pos, centre + up, step);
                c.pos = pos;
                if arrived {
                    self.land_courier(c);
                }
                !arrived
            }
        }
    }

    /// A cargo drone reaches its pad: what it still carries goes back in the boxes, and it lands.
    fn land_courier(&mut self, c: &Courier) {
        if c.load.count > 0 {
            let boxes = self.factory.port_boxes(c.port);
            let left = self.factory.store_in_boxes(&boxes, c.load);
            if left.count > 0 {
                self.drop_stacks(c.pos.floor(), vec![left]);
            }
        }
        if !self.factory.port_land(c.port, CARGO_DRONE) {
            self.lose_courier(c);
        }
    }

    /// A cargo drone that cannot come home (its port is gone or full): it and its load fall as items.
    fn lose_courier(&mut self, c: &Courier) {
        let mut fallen = vec![Stack { item: CARGO_DRONE, count: 1 }];
        fallen.extend(Some(c.load).filter(|s| s.count > 0));
        self.drop_stacks(c.pos.floor(), fallen);
    }

    /// Sends a cargo drone from `port` along its route if there is something to carry, room for it and batteries.
    fn launch_courier(&mut self, port: &PortInfo, ports: &[PortInfo]) {
        let Some(dest) = self.cargo.route_from(port.anchor) else { return };
        let Some(to) = ports.iter().find(|p| p.anchor == dest) else { return };
        let dist = (to.centre - port.centre).length();
        if dist > port.cargo_range {
            return;
        }
        let (from_boxes, to_boxes) = (self.factory.port_boxes(port.anchor), self.factory.port_boxes(dest));
        let Some((item, held)) = self.factory.box_first_stack(&from_boxes, &[BATTERY, DRONE, CARGO_DRONE]) else {
            return;
        };
        let n = held.min(LOAD).min(item::stack_size(item)).min(self.factory.boxes_room(&to_boxes, item));
        let batteries = trip_batteries(dist);
        if n == 0 || self.factory.boxes_count(&from_boxes, BATTERY) < batteries {
            return;
        }
        if !self.factory.port_take(port.anchor, CARGO_DRONE) {
            return;
        }
        self.factory.boxes_take(&from_boxes, BATTERY, batteries);
        let took = self.factory.boxes_take(&from_boxes, item, n);
        let pos = port.centre + Vec3::new(0.0, 0.4, 0.0);
        self.cargo.list.push(Courier {
            port: port.anchor,
            dest,
            pos,
            prev: pos,
            leg: Leg::Out,
            load: Stack { item, count: took },
        });
    }
}
